//! Thin HTTP handlers for party leader actions.
//! Uses domain logic from `cinematch_abi::domain`.

use super::{AppState, KickQuery, OptionalIdParam, TransferQuery};
use crate::api_error::ApiError;
use crate::extract_user_id;

use actix_identity::Identity;
use actix_web::{HttpResponse, post, web};
use cinematch_common::models::ErrorResponse;
use log::debug;

// Import domain types and traits from cinematch_abi
use cinematch_abi::domain::{
    EndVotingTransition, PartyAdvanceOutcome, PartyCrud, PartyStateMachine,
};
use cinematch_db::domain::{Party, User};

#[utoipa::path(
    responses(
        (status = 200, description = "Phase advanced"),
        (status = 400, description = "Invalid state transition", body = ErrorResponse),
        (status = 403, description = "Leader only", body = ErrorResponse),
        (status = 404, description = "Party not found", body = ErrorResponse)
    ),
    params(super::OptionalIdParam),
    tags = ["Leader Tools"],
    security(("cookie_auth" = [])),
    operation_id = "advance_phase"
)]
#[post("/advance")]
pub async fn advance_phase(
    ctx: AppState,
    user: Identity,
    party_query: web::Query<OptionalIdParam>,
) -> Result<HttpResponse, ApiError> {
    let leader_id = extract_user_id(user)?;
    let party_id = match party_query.id {
        Some(id) => id,
        None => {
            let user_obj = User::from_id(&ctx, leader_id).await?;
            user_obj
                .current_party(&ctx)
                .await?
                .ok_or_else(|| ApiError::NotFound("No active party found".to_string()))?
                .id
        }
    };

    let party = Party::from_id(&ctx, party_id).await?;

    let outcome = party.advance_phase(&ctx, leader_id).await?;
    match &outcome {
        PartyAdvanceOutcome::PhaseChanged(s) => {
            debug!("Party {} phase advanced to {:?}", party_id, s);
            ctx.scheduler
                .enforce_phase_timeout_and_broadcast(party_id, ctx.clone())
                .await;
        }
        PartyAdvanceOutcome::VotingEnded(EndVotingTransition::Round2Started) => {
            debug!("Party {} voting round 2 started", party_id);
            ctx.scheduler
                .enforce_phase_timeout_and_broadcast(party_id, ctx.clone())
                .await;
        }
        PartyAdvanceOutcome::VotingEnded(EndVotingTransition::PhaseChanged(s)) => {
            debug!("Party {} voting ended -> {:?}", party_id, s);
            ctx.scheduler
                .enforce_phase_timeout_and_broadcast(party_id, ctx.clone())
                .await;
        }
    }
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    responses(
        (status = 200, description = "Party disbanded"),
        (status = 403, description = "Leader only", body = ErrorResponse),
        (status = 404, description = "Party not found")
    ),
    params(super::OptionalIdParam),
    tags = ["Leader Tools"],
    security(("cookie_auth" = [])),
    operation_id = "disband_party"
)]
#[post("/disband")]
pub async fn disband_party(
    ctx: AppState,
    user: Identity,
    party_query: web::Query<OptionalIdParam>,
) -> Result<HttpResponse, ApiError> {
    let user_id = extract_user_id(user)?;
    let party_id = match party_query.id {
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
    party_obj.disband_checked(&ctx, user_id).await?;

    debug!("Party {} disbanded successfully", party_id);
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    responses(
        (status = 200, description = "Member kicked"),
        (status = 403, description = "Leader only or not a member", body = ErrorResponse),
        (status = 404, description = "Party or member not found", body = ErrorResponse)
    ),
    params(
        super::KickQuery,
        super::OptionalIdParam
    ),
    tags = ["Leader Tools"],
    security(("cookie_auth" = [])),
    operation_id = "kick_member"
)]
#[post("/kick")]
pub async fn kick_member(
    ctx: AppState,
    user: Identity,
    kick_query: web::Query<KickQuery>,
    party_query: web::Query<OptionalIdParam>,
) -> Result<HttpResponse, ApiError> {
    let requester_id = extract_user_id(user)?;
    let target_user_id = kick_query.target_user_id;
    let party_id = match party_query.id {
        Some(id) => id,
        None => {
            let user_obj = User::from_id(&ctx, requester_id).await?;
            user_obj
                .current_party(&ctx)
                .await?
                .ok_or_else(|| ApiError::NotFound("No active party found".to_string()))?
                .id
        }
    };

    let party_obj = Party::from_id(&ctx, party_id).await?;
    party_obj.kick(&ctx, requester_id, target_user_id).await?;

    debug!("User {} kicked from party {}", target_user_id, party_id);
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    responses(
        (status = 200, description = "Leadership transferred"),
        (status = 400, description = "New leader not a member", body = ErrorResponse),
        (status = 403, description = "Leader only", body = ErrorResponse),
        (status = 404, description = "Party not found", body = ErrorResponse)
    ),
    params(
        super::TransferQuery,
        super::OptionalIdParam
    ),
    tags = ["Leader Tools"],
    security(("cookie_auth" = [])),
    operation_id = "transfer_leadership"
)]
#[post("/transfer-leadership")]
pub async fn transfer_leadership(
    ctx: AppState,
    user: Identity,
    transfer_query: web::Query<TransferQuery>,
    party_query: web::Query<OptionalIdParam>,
) -> Result<HttpResponse, ApiError> {
    let requester_id = extract_user_id(user)?;
    let new_leader_id = transfer_query.new_leader_id;
    let party_id = match party_query.id {
        Some(id) => id,
        None => {
            let user_obj = User::from_id(&ctx, requester_id).await?;
            user_obj
                .current_party(&ctx)
                .await?
                .ok_or_else(|| ApiError::NotFound("No active party found".to_string()))?
                .id
        }
    };

    let party_obj = Party::from_id(&ctx, party_id).await?;
    party_obj
        .transfer_leadership_checked(&ctx, requester_id, new_leader_id)
        .await?;
    Ok(HttpResponse::Ok().finish())
}
