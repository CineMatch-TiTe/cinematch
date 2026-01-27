use super::{AppState, DbError, ErrorResponse, GetPicksResponse, PartyState, extract_user_id};

use crate::user::UpdateTasteRequest;

use std::collections::HashSet;

use actix_identity::Identity;
use actix_web::{HttpResponse, delete, get, put, web};
use log::{error, trace};
use uuid::Uuid;

#[utoipa::path(
    responses(
        (status = 200, description = "Your picks (movie IDs)", body = GetPicksResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Not a party member", body = ErrorResponse),
        (status = 404, description = "Party not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(("party_id" = Uuid, Path, description = "Party ID")),
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
    request_body = UpdateTasteRequest,
    responses(
        (status = 200, description = "Movie action registered"),
        (status = 400, description = "Invalid action", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Not a member or picking not allowed", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("party_id" = Uuid, Path, description = "Party ID"),
        ("movie_id" = i64, Path, description = "TMDB movie ID")
    ),
    tags = ["party"],
    security(("cookie_auth" = [])),
    operation_id = "pick_movie"
)]
#[put("/{party_id}/pick/{movie_id}")]
pub async fn pick_movie(
    db: AppState,
    user: Identity,
    path: web::Path<(Uuid, i64)>,
    body: web::Json<UpdateTasteRequest>,
) -> HttpResponse {
    let (party_id, movie_id) = path.into_inner();
    let user_id = extract_user_id!(user);
    let liked = body.into_inner().liked;

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
    // Store party taste
    match db.add_party_taste(user_id, party_id, movie_id, liked).await {
        Ok(_) => {}
        Err(e) => {
            error!(
                "Failed to register pick for user {} in party {}: {}",
                user_id, party_id, e
            );
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to register movie pick"));
        }
    }

    HttpResponse::Ok().finish()
}

#[utoipa::path(
    responses(
        (status = 200, description = "Pick removed"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Not a member or picking not allowed", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("party_id" = Uuid, Path, description = "Party ID"),
        ("movie_id" = i64, Path, description = "TMDB movie ID to remove")
    ),
    tags = ["party"],
    security(("cookie_auth" = [])),
    operation_id = "delete_pick"
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

#[utoipa::path(
    responses(
        (status = 200, description = "Recommended movies for party (excluding already picked)", body = crate::movie::RecommendedMoviesResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Not a party member", body = ErrorResponse),
        (status = 404, description = "No recommendations", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(("party_id" = Uuid, Path, description = "Party ID")),
    tags = ["party"],
    security(("cookie_auth" = [])),
    operation_id = "get_party_recommendations"
)]
#[get("/{party_id}/recommend")]
pub async fn get_party_recommendations(
    db: AppState,
    user: Identity,
    party_id: web::Path<Uuid>,
) -> HttpResponse {
    let party_id = party_id.into_inner();
    let user_id = extract_user_id!(user);

    if let Ok(false) | Err(_) = db.is_party_member(party_id, user_id).await {
        return HttpResponse::Forbidden().json(ErrorResponse::new("Not a member of this party"));
    }

    let ids: Vec<i64> = match cinematch_recommendation_engine::recommed_movies_from_reviews(
        &db,
        user_id,
        Some(party_id),
        5,
    )
    .await
    {
        Ok(movies) => {
            if movies.is_empty() {
                return HttpResponse::NotFound()
                    .json(ErrorResponse::new("No recommendations available"));
            }
            movies
        }
        Err(e) => {
            error!("Failed to retrieve party recommendations: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to retrieve recommendations"));
        }
    };

    let other_ids: Vec<i64> =
        match cinematch_recommendation_engine::recommend_movies(&db, user_id, Some(party_id), 2)
            .await
        {
            Ok(movies) => movies,
            Err(e) => {
                error!("Failed to retrieve other recommendations: {}", e);
                return HttpResponse::InternalServerError()
                    .json(ErrorResponse::new("Failed to retrieve recommendations"));
            }
        };

    let all_ids = ids
        .into_iter()
        .chain(other_ids.into_iter())
        .collect::<HashSet<_>>();

    let mut all_ids = all_ids.into_iter().collect::<Vec<_>>();

    use rand::seq::SliceRandom;
    all_ids.shuffle(&mut rand::rng());

    // pick 3
    let selected_ids = all_ids.into_iter().take(3).collect::<Vec<_>>();

    let mut responses = Vec::with_capacity(selected_ids.len());
    for movie_id in selected_ids.iter() {
        match db.get_movie_by_id(*movie_id).await {
            Ok(Some(movie)) => {
                responses.push(Into::<crate::movie::MovieResponse>::into(movie));
            }
            Ok(None) => {
                error!("Recommended movie with ID {} not found", movie_id);
            }
            Err(e) => {
                error!("Failed to retrieve movie with ID {}: {}", movie_id, e);
                return HttpResponse::InternalServerError()
                    .json(ErrorResponse::new("Failed to retrieve movie"));
            }
        }
    }

    if responses.is_empty() {
        return HttpResponse::NotFound().json(ErrorResponse::new("No recommendations available"));
    }

    HttpResponse::Ok().json(crate::movie::RecommendedMoviesResponse {
        recommended_movies: responses,
    })
}
