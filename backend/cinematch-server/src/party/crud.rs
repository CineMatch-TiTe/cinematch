use super::{CreatePartyResponse, PartyResponse};
use crate::AppState;
use crate::api_error::ApiError;
use crate::extract_user_id;
use cinematch_common::models::ErrorResponse;

use actix_identity::Identity;
use actix_web::{HttpResponse, get, post, web};
use cinematch_abi::domain::{PartyValidation, UserLogic};
use cinematch_db::domain::{Party, User};
use log::debug;

#[utoipa::path(
    responses(
        (status = 201, description = "Party created", body = CreatePartyResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "User already in a party", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tags = ["Party"],
    security(("cookie_auth" = [])),
    operation_id = "create_party"
)]
#[post("")]
pub async fn create_party(ctx: AppState, user: Identity) -> Result<HttpResponse, ApiError> {
    let user_id = extract_user_id(user)?;

    let user_obj = User::from_id(&ctx, user_id).await?;

    // Check if user is already in a party
    if user_obj.is_in_party(&ctx).await? {
        return Err(ApiError::Forbidden(
            "User is already in a party".to_string(),
        ));
    }

    debug!("Creating party for user {}", user_id);
    let (party, code) = Party::create(&ctx, user_id).await?;

    debug!(
        "Party created successfully: id={}, code={}",
        party.id, code.code
    );

    // Party::create now adds the leader as a member transactionally in the DB

    let response = super::CreatePartyResponse {
        party_id: party.id,
        code: code.code,
        created_at: party.phase_entered_at(&ctx).await?, // Use entered_at or created_at if exists
    };
    Ok(HttpResponse::Created().json(response))
}

#[utoipa::path(
    responses(
        (status = 200, description = "Party details", body = PartyResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Not a party member", body = ErrorResponse),
        (status = 404, description = "Party not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(super::OptionalIdParam),
    tags = ["Party"],
    security(("cookie_auth" = [])),
    operation_id = "get_party"
)]
#[get("")]
pub async fn get_party(
    ctx: AppState,
    user: Identity,
    query: web::Query<super::OptionalIdParam>,
) -> Result<web::Json<super::PartyResponse>, ApiError> {
    let user_id = extract_user_id(user)?;
    let party_id = match query.party_id {
        Some(id) => id,
        None => {
            let user_obj = User::from_id(&ctx, user_id).await?;
            user_obj
                .current_party(&ctx)
                .await?
                .ok_or_else(|| ApiError::NotFound("No active party found".to_string()))?
                .id
        }
    };

    let party_obj = Party::from_id(&ctx, party_id).await?;
    party_obj.require_member(&ctx, user_id).await?;

    let state = party_obj.state(&ctx).await?;
    let code = if state == cinematch_db::PartyState::Created {
        party_obj.join_code(&ctx).await?
    } else {
        None
    };

    let vote_status = if state == cinematch_db::PartyState::Voting {
        Some(party_obj.get_votes(&ctx, Some(user_id)).await?)
    } else {
        None
    };
    let timeouts = &cinematch_common::Config::get().timeouts;
    let watching_timeout_secs = timeouts.watching_timeout_secs;
    let mut voting_timeout_secs =
        if party_obj.voting_round(&ctx).await.unwrap_or(Some(1)) == Some(2) {
            timeouts.voting_r2_timeout_secs
        } else {
            timeouts.voting_r1_timeout_secs
        };

    if state == cinematch_db::PartyState::Voting && !ctx.scheduler.is_scheduled(party_id).await {
        voting_timeout_secs = 0;
    }

    let response = super::PartyResponse {
        id: party_obj.id,
        leader_id: party_obj.leader_id(&ctx).await?,
        state: state.into(),
        created_at: party_obj.phase_entered_at(&ctx).await?, // Use entered_at as approximation if created_at not lazy
        code,
        vote_status,
        selected_movie_id: party_obj.selected_movie_id(&ctx).await?,
        phase_entered_at: party_obj.phase_entered_at(&ctx).await?,
        voting_timeout_secs,
        watching_timeout_secs,
        ready_deadline_at: ctx.scheduler.get_deadline(party_id).await,
    };
    Ok(web::Json(response))
}
