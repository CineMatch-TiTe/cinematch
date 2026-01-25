use super::{
    AppState, CreatePartyResponse, DbError, ErrorResponse, PartyCode, PartyResponse, PartyState,
    extract_user_id, VoteMovieRequest, VoteMovieResponse,
};
use actix_identity::Identity;
use actix_web::{HttpResponse, get, post, web, put};
use log::{debug, error, trace};
use utoipa::openapi::path;
use uuid::Uuid;


use crate::WsBroadcaster;
use crate::websocket::models::{MovieVotes, ServerMessage};

#[utoipa::path(
    responses(
        (status = 200, description = "Movie picked"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Not a party member or cannot pick", body = ErrorResponse),
        (status = 404, description = "Party or movie not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("party_id" = Uuid, Path, description = "The party's unique ID"),
        ("movie_id" = i64, Path, description = "The movie's unique ID")
    ),
    tags = ["party"],
    security(("bearer_auth" = []))
)]
#[put("/{party_id}/pick/{movie_id}")]
pub async fn pick_movie(
    db: AppState,
    user: Identity,
    path: web::Path<(Uuid, i64)>,
) -> HttpResponse {
    let (party_id, movie_id) = path.into_inner();
    let user_id = extract_user_id!(user);

    // 1. Verify user is a member
    if let Ok(false) | Err(_) = db.is_party_member(party_id, user_id).await {
        return HttpResponse::Forbidden().json(ErrorResponse::new("Not a member of this party"));
    }

    // 2. Check user can pick movies
    match db.get_state(party_id).await {
        Ok(PartyState::Picking | PartyState::Created) => {}
        Ok(_) => {
            return HttpResponse::Forbidden().json(ErrorResponse::new("Picking is not allowed at this time."));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse::new(format!("DB error: {e}")));
        }
    }
    // 3. Register the pick

    match db.add_party_taste(user_id, party_id, movie_id, true).await {
        Ok(_) => HttpResponse::Ok().json("Movie picked successfully"),
        Err(e) => {
            error!("Failed to register pick for user {} in party {}: {}", user_id, party_id, e);
            HttpResponse::InternalServerError().json(ErrorResponse::new("Failed to register movie pick"))
        }
    }
}