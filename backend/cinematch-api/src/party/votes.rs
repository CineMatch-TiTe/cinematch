use super::{
    AppState, DbError, ErrorResponse, GetVoteResponse, PartyState, VoteMovieRequest,
    VoteMovieResponse, VoteTotals, extract_user_id,
};
use actix_identity::Identity;
use actix_web::{HttpResponse, get, post, web};
use log::{error, trace};
use std::collections::HashMap;
use uuid::Uuid;

use crate::WsStoreData;
use crate::handler::party::{EndVotingTransition, try_auto_end_voting};
use crate::websocket::broadcast_party_timeout;
use crate::websocket::models::{MovieVotes, ServerMessage, VotingRoundStarted};

#[utoipa::path(
    responses(
        (status = 200, description = "Ballot and vote totals", body = GetVoteResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Not a party member", body = ErrorResponse),
        (status = 404, description = "Party not found or not in Voting", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(("party_id" = Uuid, Path, description = "Party ID")),
    tags = ["party"],
    security(("cookie_auth" = [])),
    operation_id = "get_vote"
)]
#[get("/{party_id}/vote")]
pub async fn get_vote(db: AppState, user: Identity, party_id: web::Path<Uuid>) -> HttpResponse {
    let party_id = party_id.into_inner();
    let user_id = extract_user_id!(user);

    trace!("GET /{}/vote - user_id={}", party_id, user_id);

    if let Ok(false) | Err(_) = db.is_party_member(party_id, user_id).await {
        return HttpResponse::Forbidden().json(ErrorResponse::new("Not a member of this party"));
    }

    let party = match db.get_party(party_id).await {
        Ok(p) => p,
        Err(DbError::PartyNotFound(_)) => {
            return HttpResponse::NotFound().json(ErrorResponse::new("Party not found"));
        }
        Err(e) => {
            error!("get_vote: get_party failed: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to load party"));
        }
    };

    if party.state != PartyState::Voting {
        return HttpResponse::NotFound().json(ErrorResponse::new("Party is not in Voting"));
    }

    let movie_ids = match db.get_user_ballot(party_id, user_id).await {
        Ok(ids) => ids,
        Err(e) => {
            error!("get_vote: get_user_ballot failed: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to load ballot"));
        }
    };

    let vote_totals: HashMap<i64, VoteTotals> =
        match db.get_party_votes(party_id, Some(user_id)).await {
            Ok(m) => m
                .into_iter()
                .map(|(mid, (likes, dislikes))| (mid, VoteTotals { likes, dislikes }))
                .collect(),
            Err(e) => {
                error!("get_vote: get_party_votes failed: {}", e);
                return HttpResponse::InternalServerError()
                    .json(ErrorResponse::new("Failed to load vote totals"));
            }
        };

    let response = GetVoteResponse {
        movie_ids,
        voting_round: party.voting_round,
        can_vote: party.can_vote,
        vote_totals,
    };
    HttpResponse::Ok().json(response)
}

#[utoipa::path(
    request_body = VoteMovieRequest,
    responses(
        (status = 200, description = "Vote cast", body = VoteMovieResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Not a member or cannot vote for this movie", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("party_id" = Uuid, Path, description = "Party ID"),
        ("movie_id" = i64, Path, description = "TMDB movie ID")
    ),
    tags = ["party"],
    security(("cookie_auth" = [])),
    operation_id = "vote_movie"
)]
#[post("/{party_id}/vote/{movie_id}")]
pub async fn vote_movie(
    db: AppState,
    store: WsStoreData,
    user: Identity,
    path: web::Path<(Uuid, i64)>,
    vote: web::Json<VoteMovieRequest>,
) -> HttpResponse {
    let (party_id, movie_id) = path.into_inner();
    let vote = vote.into_inner();
    let user_id = extract_user_id!(user);

    // 1. Verify user is a member
    if let Ok(false) | Err(_) = db.is_party_member(party_id, user_id).await {
        return HttpResponse::Forbidden().json(ErrorResponse::new("Not a member of this party"));
    }

    // 2. Parse vote_value from query
    let vote_value = vote.like;

    // 3. Check user can vote for this movie
    match db.can_vote(party_id, user_id, movie_id).await {
        Ok(false) => {
            return HttpResponse::Forbidden().json(ErrorResponse::new(
                "You cannot vote for this movie. Or voting is disabled.",
            ));
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new(format!("DB error: {e}")));
        }
        Ok(true) => {}
    }

    // 4. Cast vote
    match db.cast_vote(party_id, user_id, movie_id, vote_value).await {
        Ok(_) => {
            let (likes, dislikes) = match db.get_vote_totals(movie_id, Some(party_id)).await {
                Ok((likes, dislikes)) => (likes as u32, dislikes as u32),
                Err(e) => {
                    return HttpResponse::InternalServerError()
                        .json(ErrorResponse::new(format!("DB error: {e}")));
                }
            };
            let ws_message = ServerMessage::MovieVoteUpdate(MovieVotes {
                movie_id,
                likes,
                dislikes,
            });
            let _ = store
                .send_message_to_party(party_id.to_string(), &ws_message, Some(&[user_id]))
                .await;

            if let Ok(Some(trans)) = try_auto_end_voting(db.as_ref(), party_id).await {
                let msg = match trans {
                    EndVotingTransition::Round2Started => {
                        ServerMessage::VotingRoundStarted(VotingRoundStarted { round: 2 })
                    }
                    EndVotingTransition::PhaseChanged(s) => ServerMessage::PartyStateChanged(s),
                };
                let _ = store
                    .send_message_to_party(party_id.to_string(), &msg, None)
                    .await;
                let _ = broadcast_party_timeout(db.as_ref(), store.as_ref(), party_id).await;
            }

            HttpResponse::Ok().json(VoteMovieResponse { likes, dislikes })
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(ErrorResponse::new(format!("Failed to cast vote: {e}"))),
    }
}
