//! User API route handlers for Actix-web
//!
//! These handlers implement the user management endpoints.
//!

use crate::auth::guard::Auth;
use actix_web::{HttpResponse, get, post, put, web};

use super::{CurrentUserResponse, GetTasteQuery, UpdateTasteQuery};

use crate::AppState;
use crate::api_error::ApiError;

use cinematch_abi::domain::UserLogic;
use cinematch_common::models::{ErrorResponse, UserTasteResponse};
use cinematch_db::domain::User;

// ============================================================================
// User Management Endpoints
// ============================================================================

/// Rename the authenticated user.
///
/// Username must be 3–32 characters. Caller must match `user_id` (cookie).
#[utoipa::path(
    responses(
        (status = 200, description = "Renamed"),
        (status = 400, description = "Invalid username", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Cannot rename another user"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(crate::user::RenameQuery),
    tags = ["User"],
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    operation_id = "rename_user"
)]
#[post("/rename")]
pub async fn rename_user(
    ctx: AppState,
    #[allow(unused_variables)] _user_id: web::Query<crate::party::OptionalIdParam>, // Kept for future path-based admin overrides
    query: web::Query<crate::user::RenameQuery>,
    auth: Option<Auth>,
) -> Result<HttpResponse, ApiError> {
    let new_username = &query.name;
    let claims = auth
        .ok_or_else(|| ApiError::Unauthorized("No identity provided".to_string()))?
        .user_id();

    let user_obj = User::from_id(&ctx, claims).await?;

    user_obj.rename(&ctx, new_username).await?;

    Ok(HttpResponse::Ok().finish())
}
/// Current user profile (cookie-authenticated).
#[utoipa::path(
    responses(
        (status = 200, description = "User profile", body = CurrentUserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tags = ["User"],
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    operation_id = "get_current_user"
)]
#[get("")]
pub async fn get_current_user(
    auth: Option<Auth>,
    ctx: AppState,
) -> Result<web::Json<CurrentUserResponse>, ApiError> {
    let auth = auth.ok_or_else(|| ApiError::Unauthorized("No identity provided".to_string()))?;
    let user_id = auth.user_id();
    let user_obj = User::from_id(&ctx, user_id).await?;

    // Fetch details
    let username = user_obj.username(&ctx).await?;
    let is_guest = user_obj.is_oneshot(&ctx).await?;
    let record = user_obj.record(&ctx).await?;

    let now = chrono::Utc::now().timestamp();

    // derive expiration from the JWT if we have one
    let token_expires_at = auth.token_expires_at();
    let token_expires_in = match token_expires_at {
        Some(exp) if exp > now => Some(exp - now),
        _ => None,
    };

    let response = CurrentUserResponse {
        user_id: user_obj.id,
        username,
        is_guest,
        created_at: record.created_at,
        updated_at: record.updated_at,
        token_expires_at,
        token_expires_in,
    };
    Ok(web::Json(response))
}

/// Update movie taste (liked status and rating).
#[utoipa::path(
    responses(
        (status = 200, description = "Taste updated"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Movie not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(crate::user::UpdateTasteQuery),
    tags = ["User"],
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    operation_id = "update_taste"
)]
#[put("/taste")]
pub async fn update_taste(
    ctx: AppState,
    auth: Option<Auth>,
    query: web::Query<UpdateTasteQuery>,
) -> Result<HttpResponse, ApiError> {
    let _user_id = auth
        .ok_or_else(|| ApiError::Unauthorized("No identity provided".to_string()))?
        .user_id();
    let movie_id = query.movie_id;
    let liked = query.liked;
    let rating = query.rating;

    let user_obj = User::from_id(&ctx, _user_id).await?;
    user_obj
        .update_rating(&ctx, movie_id, liked, rating)
        .await?;

    Ok(HttpResponse::Ok().finish())
}

/// Retrieve user taste for a specific movie.
#[utoipa::path(
    responses(
        (status = 200, description = "User taste", body = UserTasteResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(crate::user::GetTasteQuery),
    tags = ["User"],
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    operation_id = "get_taste"
)]
#[get("/taste")]
pub async fn get_taste(
    ctx: AppState,
    auth: Option<Auth>,
    query: web::Query<GetTasteQuery>,
) -> Result<web::Json<UserTasteResponse>, ApiError> {
    let user_id = auth
        .ok_or_else(|| ApiError::Unauthorized("No identity provided".to_string()))?
        .user_id();
    let movie_id = query.movie_id;

    let user_obj = User::from_id(&ctx, user_id).await?;
    let taste = user_obj.get_rating(&ctx, movie_id).await?;

    match taste {
        Some((liked, rating, updated_at)) => Ok(web::Json(UserTasteResponse {
            movie_id,
            liked,
            rating,
            updated_at,
        })),
        None => Err(ApiError::NotFound(
            "No taste found for this movie".to_string(),
        )),
    }
}
