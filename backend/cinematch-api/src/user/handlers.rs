//! User API route handlers for Actix-web
//!
//! These handlers implement the user management endpoints.
//!

use actix_identity::Identity;
use actix_web::HttpMessage;
use actix_web::HttpRequest;
use actix_web::{HttpResponse, get, patch, post, put, web};
use log::{debug, trace};
use uuid::Uuid;

use log::error;

use super::{
    CurrentUserResponse, GuestLoginResponse, GuestUserRequest, RenameUserRequest,
    UpdateTasteRequest,
};

use crate::AppState;

use cinematch_db::DbError;

use cinematch_common::{ErrorResponse, extract_user_id};

// ============================================================================
// User Management Endpoints
// ============================================================================

/// Create a guest user (login as guest)
///
/// Creates a new temporary (oneshot) user with an auto-generated username.
/// This user can join parties but will be ephemeral to that party's lifetime.
/// To become persistent, the user must link an external OAuth account.
///
/// **Session**: On success, the server sets an `id` cookie (httpOnly, path=/, samesite=Lax, secure).
/// Send this cookie on all subsequent requests; protected endpoints use `cookie_auth`.
/// Use `GET /api/user/token` to obtain your `user_id` for clients that need it.
///
/// **Auth**: None required.
#[utoipa::path(
    request_body = GuestUserRequest,
    responses(
        (status = 201, description = "Guest user created; `id` cookie set", body = GuestLoginResponse),
        (status = 400, description = "Invalid username", body = ErrorResponse),
        (status = 409, description = "Already logged in", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tags = ["user"],
    operation_id = "login_guest"
)]
#[post("/login/guest")]

pub async fn login_guest(
    db: AppState,
    request: HttpRequest,
    body: web::Json<GuestUserRequest>,
    user: Option<Identity>,
) -> HttpResponse {
    if let Some(existing_user) = user {
        trace!("User already logged in with ID: {:?}", existing_user.id());
        return HttpResponse::Conflict().json(ErrorResponse::new("User already logged in"));
    }
    let username = body.into_inner();
    let username = match &username.username {
        Some(name) => {
            let name = cinematch_common::extract_and_validate_username!(name); // early return on invalid
            name
        }
        None => {
            // Generate a random username for guest user, this can also be empty
            let random_suffix = Uuid::new_v4()
                .to_string()
                .chars()
                .take(8)
                .collect::<String>();
            let username = format!("guest_{}", random_suffix);
            username
        }
    };
    debug!("Creating guest user with username: {}", username);
    match db.create_guest_user(&username).await {
        Ok(user) => match Identity::login(&request.extensions(), user.id.to_string()) {
            Ok(_) => {
                trace!("User identity set in session for user_id={}", user.id);
                HttpResponse::Created().json(GuestLoginResponse {
                    user_id: user.id,
                    username: user.username,
                })
            }
            Err(e) => {
                error!("Failed to set user identity in session: {}", e);
                HttpResponse::InternalServerError()
                    .json(ErrorResponse::new("Failed to set user identity in session"))
            }
        },
        Err(e) => {
            log::error!("Failed to create guest user: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse::new(format!(
                "Failed to create guest user: {}",
                e
            )))
        }
    }
}

/// Rename a user
///
/// Updates the username for a user. Username can be 3-32 characters.
///
/// **Auth Required**: User can only rename their own account (user_id must match JWT token).
///
#[utoipa::path(
    request_body = RenameUserRequest,
    responses(
        (status = 200, description = "User renamed successfully"),
        (status = 400, description = "Invalid username", body = ErrorResponse),
        (status = 401, description = "Unauthorized - authentication required"),
        (status = 403, description = "Forbidden - cannot rename another user"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("user_id" = Uuid, Path, description = "The user ID")
    ),
    tags = ["user"],
    security(("cookie_auth" = [])),
    operation_id = "rename_user"
)]
#[patch("/rename/{user_id}")]
pub async fn rename_user(
    db: AppState,
    user_id: web::Path<Uuid>,
    body: web::Json<RenameUserRequest>,
    user: Identity,
) -> HttpResponse {
    let user_id = user_id.into_inner();

    let claims = extract_user_id!(user);

    if claims != user_id {
        trace!(
            "Unauthorized rename attempt: claims.user_id={} != target_user_id={}",
            claims, user_id
        );
        return HttpResponse::Forbidden().finish();
    }

    // Validate username length
    let new_username = cinematch_common::extract_and_validate_username!(&body.new_username);

    let update = cinematch_db::UpdateUser {
        username: Some(&new_username),
        oneshot: None,
    };

    match db.update_user(user_id, update).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(DbError::UserNotFound(_)) => HttpResponse::NotFound().finish(),
        Err(e) => {
            log::error!("Failed to rename user: {}", e);
            HttpResponse::InternalServerError()
                .json(ErrorResponse::new(format!("Failed to rename user: {}", e)))
        }
    }
}
/// Get current user info
///
/// Returns the currently authenticated user's profile information
/// along with JWT token validity details.
///
/// **Auth Required**: User must be authenticated with a valid JWT token.
#[utoipa::path(
    responses(
        (status = 200, description = "User info retrieved successfully", body = CurrentUserResponse),
        (status = 401, description = "Unauthorized - authentication required", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tags = ["user"],
    security(("cookie_auth" = [])),
    operation_id = "get_current_user"
)]
#[get("")]
pub async fn get_current_user(user: Identity, db: AppState) -> HttpResponse {
    let user_id = extract_user_id!(user);

    match db.get_user(user_id).await {
        Ok(user) => {
            debug!("Successfully fetched user profile for {}", user_id);
            // Token expiry: 24 hours from now
            // TODO make sure this matches main auth token expiry, it should be set to 24 hours, or just remove
            let now = chrono::Utc::now().timestamp();
            let token_expires_in = 24 * 60 * 60; // 24 hours in seconds
            let token_expires_at = now + token_expires_in;

            let response = CurrentUserResponse {
                user_id: user.id,
                username: user.username,
                is_guest: user.oneshot,
                created_at: user.created_at,
                updated_at: user.updated_at,
                token_expires_at,
                token_expires_in,
            };
            HttpResponse::Ok().json(response)
        }
        Err(DbError::UserNotFound(_)) => {
            HttpResponse::NotFound().json(ErrorResponse::new("User not found"))
        }
        Err(e) => {
            log::error!("Failed to get user: {}", e);
            HttpResponse::InternalServerError()
                .json(ErrorResponse::new(format!("Failed to get user: {}", e)))
        }
    }
}

/// Logout user
///
/// Clears authentication cookies by setting them to expire.
/// This effectively logs out the user on the client side.
///
/// **Auth**: No authentication required (any user can logout).
#[utoipa::path(
    responses(
        (status = 200, description = "Successfully logged out"),
        (status = 204, description = "No user was logged in"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tags = ["user"],
    operation_id = "logout_user"
)]
#[post("/logout")]
pub async fn logout_user(user: Option<Identity>) -> HttpResponse {
    if let Some(user) = user {
        user.logout();
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::NoContent().finish()
    }
}

#[utoipa::path(
    request_body = UpdateTasteRequest,
    responses(
        (status = 201, description = "Updated"),
        (status = 400, description = "Bad Request - invalid input", body = ErrorResponse),
        (status = 401, description = "Unauthorized - authentication required"),
        (status = 404, description = "Movie not found"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("movie_id" = i64, Path, description = "The movie ID")
    ),
    tags = ["user"],
    security(("cookie_auth" = [])),
    operation_id = "update_taste"
)]
#[put("/like/{movie_id}")]
pub async fn update_taste(
    db: AppState,
    user: Identity,
    movie_id: web::Path<i64>,
    body: web::Json<UpdateTasteRequest>,
) -> HttpResponse {
    let user_id = extract_user_id!(user);
    let movie_id = movie_id.into_inner();
    let like = body.into_inner();

    // Check if movie exists
    match db.get_movie_by_id(movie_id).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return HttpResponse::NotFound().json(ErrorResponse::new("Movie not found"));
        }
        Err(e) => {
            log::error!("DB error checking movie existence: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse::new("Database error"));
        }
    }

    match db.add_taste(user_id, movie_id, like.liked).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            log::error!("Failed to update taste for user {}: {}", user_id, e);
            HttpResponse::InternalServerError()
                .json(ErrorResponse::new(format!("Failed to update taste: {}", e)))
        }
    }
}
