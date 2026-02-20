use actix_identity::Identity;
use actix_web::HttpMessage;
use actix_web::HttpRequest;
use actix_web::{HttpResponse, post, web};
use log::{debug, error, trace};

use crate::AppState;
use crate::api_error::ApiError;
use crate::user::{GuestLoginResponse, GuestUserRequest};
use cinematch_common::models::ErrorResponse;

/// Create a guest user (login as guest).
///
/// Creates a temporary (oneshot) user. Optionally provide a username (3–32 chars);
/// otherwise one is generated. On success, the server sets the `id` cookie (httpOnly,
/// path=/, samesite=Lax, secure). Send it on subsequent requests; protected endpoints use `cookie_auth`.
///
/// **Auth**: None.
#[utoipa::path(
    request_body = GuestUserRequest,
    responses(
        (status = 201, description = "Guest created; `id` cookie set", body = GuestLoginResponse),
        (status = 400, description = "Invalid username", body = ErrorResponse),
        (status = 409, description = "Already logged in", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tags = ["Auth"],
    operation_id = "login_guest"
)]
#[post("/login/guest")]
pub async fn login_guest(
    db: AppState,
    request: HttpRequest,
    body: web::Json<GuestUserRequest>,
    user: Option<Identity>,
) -> Result<web::Json<GuestLoginResponse>, ApiError> {
    if let Some(existing_user) = user {
        trace!("User already logged in with ID: {:?}", existing_user.id());
        return Err(ApiError::Conflict("User already logged in".to_string()));
    }
    let username_input = body.into_inner().username;
    let username = username_input.unwrap_or_else(|| {
        let suffix = uuid::Uuid::new_v4().to_string();
        format!("Guest-{}", &suffix[..8])
    });
    debug!("Creating guest user with username choice: {:?}", username);

    match cinematch_db::domain::User::create_guest(&db, &username).await {
        Ok(user) => {
            if let Err(e) = Identity::login(&request.extensions(), user.id.to_string()) {
                error!("Failed to set user identity in session: {e}");
                return Err(ApiError::InternalServerError(
                    "Failed to set user identity in session".to_string(),
                ));
            }
            trace!("User identity set in session for user_id={}", user.id);

            let username = user.username(&db).await.unwrap_or_default();
            Ok(web::Json(GuestLoginResponse {
                user_id: user.id,
                username,
            }))
        }
        Err(e) => {
            error!("Failed to create guest user: {e}");
            Err(e.into())
        }
    }
}

/// Logout: clear `id` cookie. No auth required.
#[utoipa::path(
    responses(
        (status = 200, description = "Logged out"),
        (status = 204, description = "Not logged in")
    ),
    tags = ["Auth"],
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
