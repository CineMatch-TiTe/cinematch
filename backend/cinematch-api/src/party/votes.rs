use super::{
    AppState, CreatePartyResponse, DbError, ErrorResponse, PartyCode, PartyResponse, PartyState,
    extract_user_id, VoteMovieRequest, VoteMovieResponse,
};
use actix_identity::Identity;
use actix_web::{HttpResponse, get, post, web};
use log::{debug, error, trace};
use uuid::Uuid;


use crate::WsBroadcaster;
use crate::websocket::models::{MovieVotes, ServerMessage};

#[utoipa::path(
    request_body = VoteMovieRequest,
    responses(
        (status = 200, description = "Vote cast", body = VoteMovieResponse), // likes, dislikes
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Not a party member or cannot vote", body = ErrorResponse),
        (status = 404, description = "Party or movie not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("party_id" = Uuid, Path, description = "The party's unique ID"),
        ("movie_id" = i64, Path, description = "The movie's unique ID"),
        ("vote_value" = bool, Query, description = "true for like, false for dislike")
    ),
    tags = ["party"],
    security(("bearer_auth" = []))
)]
#[post("/{party_id}/vote/{movie_id}")]
pub async fn vote_movie(
    db: AppState,
    rooms: WsBroadcaster,
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
            return HttpResponse::Forbidden().json(ErrorResponse::new("You cannot vote for this movie. Or voting is disabled."));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse::new(format!("DB error: {e}")));
        }
        Ok(true) => {}
    }

    // 4. Cast vote
    match db.cast_vote(party_id, user_id, movie_id, vote_value).await {
        Ok(_) => {
            let (likes, dislikes) = match db.get_vote_totals(movie_id, Some(party_id)).await {
                Ok((likes, dislikes)) => (likes as u32, dislikes as u32),
                Err(e) => return HttpResponse::InternalServerError().json(ErrorResponse::new(format!("DB error: {e}"))),
            };
            // Notify via WebSocket
            // send websocket update to party members
            let ws_message = ServerMessage::MovieVoteUpdate(
                MovieVotes {
                    movie_id,
                    likes,
                    dislikes,
                }
            );
        
            crate::websocket::send_message_to_party(&rooms, party_id.to_string(), &ws_message, Some(&[user_id])).await;

            HttpResponse::Ok().json(VoteMovieResponse { likes, dislikes })


        }
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse::new(format!("Failed to cast vote: {e}"))),
    }
}