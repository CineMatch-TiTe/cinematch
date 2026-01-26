use super::{AppState, DbError, ErrorResponse, GetPicksResponse, PartyState, extract_user_id};
use actix_identity::Identity;
use actix_web::{HttpResponse, delete, get, put, web};
use log::{error, trace};
use uuid::Uuid;

#[utoipa::path(
    responses(
        (status = 200, description = "Your picks", body = GetPicksResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Not a party member", body = ErrorResponse),
        (status = 404, description = "Party not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(("party_id" = Uuid, Path, description = "The party ID")),
    tags = ["party"],
    security(("cookie_auth" = [])),
    operation_id = "get_picks"
)]
#[get("/{party_id}/picks")]
pub async fn get_picks(db: AppState, user: Identity, party_id: web::Path<Uuid>) -> HttpResponse {
    let party_id = party_id.into_inner();
    let user_id = extract_user_id!(user);

    trace!("GET /{}/picks - user_id={}", party_id, user_id);

    if let Ok(false) | Err(_) = db.is_party_member(party_id, user_id).await {
        return HttpResponse::Forbidden().json(ErrorResponse::new("Not a member of this party"));
    }

    match db.get_state(party_id).await {
        Ok(PartyState::Picking | PartyState::Created) => {}
        Ok(_) => {
            return HttpResponse::Ok().json(GetPicksResponse { movie_ids: vec![] });
        }
        Err(DbError::PartyNotFound(_)) => {
            return HttpResponse::NotFound().json(ErrorResponse::new("Party not found"));
        }
        Err(e) => {
            error!("get_picks: get_state failed: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to load party state"));
        }
    }

    let movie_ids = match db.get_user_party_picks(party_id, user_id).await {
        Ok(ids) => ids,
        Err(e) => {
            error!("get_picks: get_user_party_picks failed: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to load picks"));
        }
    };

    HttpResponse::Ok().json(GetPicksResponse { movie_ids })
}

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
    security(("cookie_auth" = []))
)]
#[put("/{party_id}/pick/{movie_id}")]
pub async fn pick_movie(
    db: AppState,
    user: Identity,
    path: web::Path<(Uuid, i64)>,
) -> HttpResponse {
    let (party_id, movie_id) = path.into_inner();
    let user_id = extract_user_id!(user);

    if let Ok(false) | Err(_) = db.is_party_member(party_id, user_id).await {
        return HttpResponse::Forbidden().json(ErrorResponse::new("Not a member of this party"));
    }

    match db.get_state(party_id).await {
        Ok(PartyState::Picking | PartyState::Created) => {}
        Ok(_) => {
            return HttpResponse::Forbidden()
                .json(ErrorResponse::new("Picking is not allowed at this time."));
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new(format!("DB error: {e}")));
        }
    }

    match db.add_party_taste(user_id, party_id, movie_id, true).await {
        Ok(_) => HttpResponse::Ok().json("Movie picked successfully"),
        Err(e) => {
            error!(
                "Failed to register pick for user {} in party {}: {}",
                user_id, party_id, e
            );
            HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to register movie pick"))
        }
    }
}

#[utoipa::path(
    responses(
        (status = 200, description = "Pick removed"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Not a party member or cannot pick", body = ErrorResponse),
        (status = 404, description = "Party not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("party_id" = Uuid, Path, description = "The party ID"),
        ("movie_id" = i64, Path, description = "The movie ID to remove from picks")
    ),
    tags = ["party"],
    security(("cookie_auth" = []))
)]
#[delete("/{party_id}/pick/{movie_id}")]
pub async fn delete_pick(
    db: AppState,
    user: Identity,
    path: web::Path<(Uuid, i64)>,
) -> HttpResponse {
    let (party_id, movie_id) = path.into_inner();
    let user_id = extract_user_id!(user);

    if let Ok(false) | Err(_) = db.is_party_member(party_id, user_id).await {
        return HttpResponse::Forbidden().json(ErrorResponse::new("Not a member of this party"));
    }

    match db.get_state(party_id).await {
        Ok(PartyState::Picking | PartyState::Created) => {}
        Ok(_) => {
            return HttpResponse::Forbidden()
                .json(ErrorResponse::new("Picking is not allowed at this time."));
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new(format!("DB error: {e}")));
        }
    }

    match db.remove_party_taste(user_id, party_id, movie_id).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            error!(
                "Failed to remove pick for user {} in party {}: {}",
                user_id, party_id, e
            );
            HttpResponse::InternalServerError().json(ErrorResponse::new("Failed to remove pick"))
        }
    }
}
