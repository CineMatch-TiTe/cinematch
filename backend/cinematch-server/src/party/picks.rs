use super::{AppState, GetPicksResponse, OptionalIdParam};
use crate::api_error::ApiError;
use crate::extract_user_id;
use cinematch_common::models::ErrorResponse;

use actix_identity::Identity;
use actix_web::{HttpResponse, delete, get, put, web};
use cinematch_abi::domain::PartyCrud;
use cinematch_db::domain::{Party, User};

#[utoipa::path(
    responses(
        (status = 200, description = "Your picks (movie IDs)", body = GetPicksResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Not a party member", body = ErrorResponse),
        (status = 404, description = "Party not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(super::OptionalIdParam),
    tags = ["Picking"],
    security(("cookie_auth" = [])),
    operation_id = "get_picks"
)]
#[get("/pick")]
pub async fn get_picks(
    ctx: AppState,
    user: Identity,
    party_query: web::Query<OptionalIdParam>,
) -> Result<web::Json<super::GetPicksResponse>, ApiError> {
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
    let movie_ids = party_obj.get_user_picks(&ctx, user_id).await?;

    Ok(web::Json(super::GetPicksResponse { movie_ids }))
}

#[utoipa::path(
    responses(
        (status = 200, description = "Movie action registered"),
        (status = 400, description = "Invalid action", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Not a member or picking not allowed", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        crate::user::UpdateTasteQuery,
        super::OptionalIdParam
    ),
    tags = ["Picking"],
    security(("cookie_auth" = [])),
    operation_id = "pick_movie"
)]
#[put("/pick")]
pub async fn pick_movie(
    ctx: AppState,
    user: Identity,
    query: web::Query<crate::user::UpdateTasteQuery>,
    party_query: web::Query<OptionalIdParam>,
) -> Result<HttpResponse, ApiError> {
    let user_id = extract_user_id(user)?;
    let movie_id = query.movie_id;
    let liked = query
        .liked
        .ok_or_else(|| ApiError::BadRequest("liked is required for party picks".to_string()))?;
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
    party_obj
        .set_user_pick(&ctx, user_id, movie_id, liked)
        .await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    responses(
        (status = 200, description = "Pick removed"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Not a member or picking not allowed", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        super::MovieIdQuery,
        super::OptionalIdParam
    ),
    tags = ["Picking"],
    security(("cookie_auth" = [])),
    operation_id = "delete_pick"
)]
#[delete("/pick")]
pub async fn delete_pick(
    ctx: AppState,
    user: Identity,
    query: web::Query<super::MovieIdQuery>,
    party_query: web::Query<OptionalIdParam>,
) -> Result<HttpResponse, ApiError> {
    let user_id = extract_user_id(user)?;
    let movie_id = query.movie_id;
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
    party_obj.remove_user_pick(&ctx, user_id, movie_id).await?;

    Ok(HttpResponse::Ok().finish())
}
