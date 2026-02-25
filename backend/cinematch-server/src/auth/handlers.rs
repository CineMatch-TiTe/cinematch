use actix_identity::Identity;
use actix_web::HttpMessage;
use actix_web::HttpRequest;
use actix_web::{HttpResponse, post, web};
use log::{debug, error, trace};

use crate::AppState;
use crate::api_error::ApiError;
use crate::auth::guard::Auth;
use crate::user::{GuestLoginResponse, GuestUserRequest};
use cinematch_abi::auth::jwt::sign_token;
use cinematch_common::{Config, models::ErrorResponse};

/// Response for renewing a JWT token
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct RenewJwtResponse {
    /// Newly issued JWT string
    pub jwt: String,
    /// Unix timestamp when the token expires
    pub token_expires_at: i64,
    /// Seconds until expiration
    pub token_expires_in: i64,
}

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
    ctx: AppState,
    request: HttpRequest,
    body: web::Json<GuestUserRequest>,
    auth: Option<Auth>,
) -> Result<HttpResponse, ApiError> {
    if auth.is_some() {
        return Err(ApiError::Conflict(
            "Already authenticated; logout first to create a new guest".to_string(),
        ));
    }

    let username_input = body.into_inner().username;
    let username = username_input.unwrap_or_else(|| {
        let suffix = uuid::Uuid::new_v4().to_string();
        format!("Guest-{}", &suffix[..8])
    });
    debug!("Creating guest user with username choice: {:?}", username);

    match cinematch_db::domain::User::create_guest(&ctx, &username).await {
        Ok(user) => {
            if let Err(e) = Identity::login(&request.extensions(), user.id.to_string()) {
                error!("Failed to set user identity in session: {e}");
                return Err(ApiError::InternalServerError(
                    "Failed to set user identity in session".to_string(),
                ));
            }
            trace!("User identity set in session for user_id={}", user.id);

            let username = user.username(&ctx).await.unwrap_or_default();
            // sign a token for the newly created guest
            let jwt = match sign_token(user.id) {
                Ok(tok) => tok,
                Err(e) => {
                    error!("Failed to sign JWT: {}", e);
                    return Err(ApiError::InternalServerError(
                        "Failed to generate token".to_string(),
                    ));
                }
            };

            let now = chrono::Utc::now().timestamp();
            let expires_at = now + Config::get().jwt_expiry_secs as i64;
            let expires_in = if expires_at > now {
                expires_at - now
            } else {
                0
            };

            let response = GuestLoginResponse {
                user_id: user.id,
                username,
                jwt,
                token_expires_at: expires_at,
                token_expires_in: expires_in,
            };
            Ok(HttpResponse::Created().json(response))
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

/// Renew the client's JWT by issuing a fresh token with a new expiry.
///
/// Requires existing authentication (cookie or bearer). WebSocket sessions
/// continue uninterrupted – the new token is only for future HTTP/API calls.
#[utoipa::path(
    responses(
        (status = 200, description = "New JWT issued", body = RenewJwtResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tags = ["Auth"],
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    operation_id = "renew_jwt"
)]
#[post("/renew")]
pub async fn renew_jwt(auth: Option<Auth>) -> Result<HttpResponse, ApiError> {
    let auth = auth.ok_or_else(|| ApiError::Unauthorized("No identity provided".to_string()))?;

    let user_id = auth.user_id();
    let jwt = sign_token(user_id).map_err(|e| {
        error!("Failed to sign new JWT: {}", e);
        ApiError::InternalServerError("Failed to generate token".to_string())
    })?;

    let now = chrono::Utc::now().timestamp();
    let expires_at = now + Config::get().jwt_expiry_secs as i64;
    let expires_in = if expires_at > now {
        expires_at - now
    } else {
        0
    };

    Ok(HttpResponse::Ok().json(RenewJwtResponse {
        jwt,
        token_expires_at: expires_at,
        token_expires_in: expires_in,
    }))
}
