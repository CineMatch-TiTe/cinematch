//! User API route handlers for Actix-web
//!
//! These handlers implement the user management endpoints.
//! 

use actix_web::{web, HttpResponse};
use uuid::Uuid;

use actix_jwt_auth_middleware::TokenSigner;
use jwt_compact::alg::Ed25519;

use crate::models::*;
use cinematch_db::{Database, DbError};

use cinematch_common::UserClaims;

/// Application state wrapper providing database access
pub type AppState = web::Data<Database>;
type Signer = web::Data<TokenSigner<UserClaims, Ed25519>>;

// ============================================================================
// User Management Endpoints
// ============================================================================

/// Create a guest user
///
/// Creates a new temporary (oneshot) user with an auto-generated username.
/// This user can join parties but will be ephemeral to that party's lifetime.
/// To become persistent, the user must link an external OAuth account.
///
/// This is the entry point for users without an account. Use the returned `user_id`
/// for subsequent API requests that require authentication.
///
/// **Auth**: No authentication required.
#[utoipa::path(
    post,
    path = "/api/user/login/guest",
    responses(
        (status = 201, description = "Guest user created successfully", body = GuestLoginResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tags = ["user"],
    operation_id = "login_guest"
)]

pub async fn login_guest(db: AppState, token_signer: Signer) -> HttpResponse {
    // Generate a random username for guest user
    let random_suffix = Uuid::new_v4()
        .to_string()
        .chars()
        .take(8)
        .collect::<String>();
    let username = format!("guest_{}", random_suffix);

    match db.create_guest_user(&username).await {
        Ok(user) => {

            let claims = UserClaims {
                user_id: user.id,
            };

            let response = GuestLoginResponse {
                user_id: user.id,
                username: user.username,
                is_guest: user.oneshot,
                created_at: user.created_at,
            };

            let access_cookie = match token_signer.create_access_cookie(&claims) {
                Ok(cookie) => cookie,
                Err(e) => {
                    log::error!("Failed to create access cookie: {}", e);
                    return HttpResponse::InternalServerError().json(ErrorResponse::new(format!(
                        "Failed to create access cookie: {}",
                        e
                    )));
                }
            };

            let refresh_cookie = match token_signer.create_refresh_cookie(&claims) {
                Ok(cookie) => cookie,
                Err(e) => {
                    log::error!("Failed to create refresh cookie: {}", e);
                    return HttpResponse::InternalServerError().json(ErrorResponse::new(format!(
                        "Failed to create refresh cookie: {}",
                        e
                    )));
                }
            };

            HttpResponse::Created()
            .cookie(access_cookie)
            .cookie(refresh_cookie)
            .json(response)
        }
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
    patch,
    path = "/api/user/rename/{user_id}",
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
    operation_id = "rename_user"
)]
pub async fn rename_user(
    claims: UserClaims,
    db: AppState,
    user_id: web::Path<Uuid>,
    body: web::Json<RenameUserRequest>,
) -> HttpResponse {
    let user_id = user_id.into_inner();
    let new_username = body.new_username.trim();

    // Validate username length
    if new_username.len() < 3 || new_username.len() > 32 {
        return HttpResponse::BadRequest().json(ErrorResponse::new(
            "Username must be between 3 and 32 characters",
        ));
    }

    if claims.user_id != user_id {
        return HttpResponse::Forbidden().finish();
    }

    match db.update_user_username(user_id, new_username).await {
        Ok(_) => {
            HttpResponse::Ok().finish()
        }
        Err(DbError::UserNotFound(_)) => {
            HttpResponse::NotFound().finish()
        }
        Err(e) => {
            log::error!("Failed to rename user: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse::new(format!(
                "Failed to rename user: {}",
                e
            )))
        }
    }
}
