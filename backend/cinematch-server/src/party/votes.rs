use super::{AppState, GetVoteResponse, OptionalIdParam, VoteMovieResponse, VoteQuery, VoteTotals};
use crate::api_error::ApiError;
use crate::extract_user_id;

use actix_identity::Identity;
use actix_web::{HttpResponse, get, post, web};
use cinematch_abi::domain::{PartyCrud, PartyStateMachine, PartyValidation};
use cinematch_common::models::ErrorResponse;
use cinematch_db::PartyState;
use cinematch_db::domain::{Party, User};
use std::collections::HashMap;

#[utoipa::path(
    responses(
        (status = 200, description = "Current vote for this user", body = GetVoteResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Not a party member", body = ErrorResponse),
        (status = 404, description = "Party not found or not in voting", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(super::OptionalIdParam),
    tags = ["Voting"],
    security(("cookie_auth" = [])),
    operation_id = "get_vote"
)]
#[get("/vote")]
pub async fn get_vote(
    ctx: AppState,
    user: Identity,
    party_query: web::Query<OptionalIdParam>,
) -> Result<HttpResponse, ApiError> {
    let user_id = extract_user_id(user)?;
    let party_id = match party_query.party_id {
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
    if state != PartyState::Voting {
        return Err(ApiError::NotFound("Party is not in Voting".to_string()));
    }

    let movie_ids = party_obj.get_ballot(&ctx, user_id).await?;

    let vote_totals: HashMap<i64, VoteTotals> = party_obj
        .get_votes(&ctx, Some(user_id))
        .await?
        .into_iter()
        .map(|(mid, (l, d))| {
            (
                mid,
                VoteTotals {
                    likes: l,
                    dislikes: d,
                },
            )
        })
        .collect();

    let response = GetVoteResponse {
        movie_ids,
        voting_round: party_obj.voting_round(&ctx).await?,
        can_vote: party_obj.can_vote(&ctx).await?,
        vote_totals,
    };
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    responses(
        (status = 200, description = "Vote registered", body = VoteMovieResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Not a member or cannot vote", body = ErrorResponse),
        (status = 404, description = "Party not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        super::VoteQuery,
        super::OptionalIdParam
    ),
    tags = ["Voting"],
    security(("cookie_auth" = [])),
    operation_id = "vote_movie"
)]
#[post("/vote")]
pub async fn vote_movie(
    ctx: AppState,
    user: Identity,
    vote_query: web::Query<VoteQuery>,
    party_query: web::Query<OptionalIdParam>,
) -> Result<HttpResponse, ApiError> {
    let user_id = extract_user_id(user)?;
    let movie_id = vote_query.movie_id;
    let vote_value = vote_query.like;
    let party_id = match party_query.party_id {
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

    // Cast vote with broadcast
    let (likes, dislikes) = party_obj
        .cast_vote_with_broadcast(&ctx, user_id, movie_id, vote_value)
        .await?;

    // Try auto-end voting if all members have voted (ONLY IN ROUND 2)
    let voting_round = party_obj.voting_round(&ctx).await?;
    if voting_round == Some(2) {
        if let Ok(Some(_)) = party_obj.try_auto_end_voting(&ctx).await {
            // Broadcast handled by ABI
            ctx.scheduler
                .enforce_phase_timeout_and_broadcast(party_id, ctx.clone())
                .await;
        }
    }

    Ok(HttpResponse::Ok().json(VoteMovieResponse { likes, dislikes }))
}
