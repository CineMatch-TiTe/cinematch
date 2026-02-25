use super::{AppState, GetVoteResponse, OptionalIdParam, VoteMovieResponse, VoteQuery, VoteTotals};
use crate::api_error::ApiError;
use crate::extract_user_id;

use actix_identity::Identity;
use actix_web::{HttpResponse, get, post, web};
use cinematch_abi::domain::{PartyCrud, PartyValidation};
use cinematch_common::models::ErrorResponse;
use cinematch_db::PartyState;
use cinematch_db::domain::{Party, User};
use log::debug;
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

    // Participation-based timeout: check if >= 50% have voted
    let participation = party_obj.voting_participation_count(&ctx).await?;
    let total_members = party_obj.member_count(&ctx).await?;
    let has_fifty_pct = total_members > 0 && participation * 2 >= total_members;

    if has_fifty_pct && !ctx.scheduler.is_scheduled(party_id).await {
        debug!(
            "Voting participation reached {}/{} (>= 50%), triggering timeout",
            participation, total_members
        );
        ctx.scheduler
            .trigger_voting_timeout(party_id, ctx.clone())
            .await;
    }

    Ok(HttpResponse::Ok().json(VoteMovieResponse { likes, dislikes }))
}
