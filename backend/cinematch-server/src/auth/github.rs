use crate::AppState;
use crate::api_error::ApiError;
use actix_identity::Identity;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, get, web};
use cinematch_abi::auth::jwt::sign_token;
use cinematch_abi::domain::{ExternalAuthLogic as _, User};
use cinematch_common::Config;
use cinematch_common::models::ErrorResponse;
use cinematch_db::models::AuthProvider;
use log::{debug, error};

#[derive(Debug, serde::Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: Option<String>,
}

// Response body returned by the OAuth callback when JWT is issued.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct OAuthCallbackResponse {
    /// short-lived JWT for API calls
    pub jwt: String,
    /// Unix timestamp when the token expires
    pub expires_at: i64,
    /// Seconds until expiration
    pub expires_in: i64,
    /// logged-in user ID
    pub user_id: uuid::Uuid,
    /// user display name (if available)
    pub username: Option<String>,
}

/// Redirect to GitHub OAuth login page.
///
/// **Auth**: None (or session user to link account).
#[utoipa::path(
    responses(
        (status = 302, description = "Redirect to GitHub login")
    ),
    tags = ["Auth"],
    operation_id = "login_github"
)]
#[get("/login/github")]
pub async fn login_github() -> Result<HttpResponse, ApiError> {
    let config = cinematch_common::Config::get();
    let github = match &config.github {
        Some(gh) => gh,
        None => {
            return Err(ApiError::InternalServerError(
                "GitHub OAuth is not configured".to_string(),
            ));
        }
    };

    let auth_url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&scope=user:email",
        github.client_id
    );

    Ok(HttpResponse::Found()
        .append_header(("Location", auth_url))
        .finish())
}

/// GitHub OAuth callback handler.
///
/// Exchanges the code for a token and handles user login or account linking.
///
/// **Auth**: None (or session user to link account).
#[utoipa::path(
    responses(
        (status = 200, description = "OAuth login payload", body = OAuthCallbackResponse),
        (status = 400, description = "OAuth state/code error", body = ErrorResponse),
        (status = 409, description = "Account already linked to another user", body = ErrorResponse),
        (status = 500, description = "OAuth exchange failed", body = ErrorResponse)
    ),
    tags = ["Auth"],
    operation_id = "callback_github"
)]
#[get("/callback/github")]
pub async fn callback_github(
    ctx: AppState,
    req: HttpRequest,
    params: web::Query<CallbackParams>,
    identity: Option<Identity>,
) -> Result<web::Json<OAuthCallbackResponse>, ApiError> {
    debug!(
        "GitHub callback triggered with code: {} and state: {:?}",
        &params.code, &params.state
    );

    // 1. Domain exchange logic (ABI level)
    let gh_user_info = User::exchange_github(&params.code).await?;

    debug!(
        "Processing GitHub login for user: {} (ID: {})",
        gh_user_info.display_name.as_deref().unwrap_or("unknown"),
        gh_user_info.provider_user_id
    );

    // 2. Handle ABI logic for linking/login
    let session_user_id = identity
        .as_ref()
        .and_then(|id| id.id().ok().and_then(|s| uuid::Uuid::parse_str(&s).ok()));

    let user = User::handle_callback(
        &ctx,
        AuthProvider::Github,
        gh_user_info.provider_user_id,
        gh_user_info.email,
        gh_user_info.display_name,
        session_user_id,
    )
    .await?;

    // 3. Log in the user (or refresh session); identity cookie remains.
    if let Err(e) = Identity::login(&req.extensions(), user.id.to_string()) {
        error!("Failed to set user identity in session: {e}");
        return Err(ApiError::InternalServerError(
            "Failed to set user identity".to_string(),
        ));
    }

    // generate JWT payload
    let jwt_token = sign_token(user.id).map_err(|e| {
        error!("Failed to sign JWT for github callback: {}", e);
        ApiError::InternalServerError("Failed to generate token".to_string())
    })?;

    let now = chrono::Utc::now().timestamp();
    let expires_at = now + Config::get().jwt_expiry_secs as i64;
    let expires_in = if expires_at > now {
        expires_at - now
    } else {
        0
    };

    let resp = OAuthCallbackResponse {
        jwt: jwt_token,
        expires_at,
        expires_in,
        user_id: user.id,
        username: user.username(&ctx).await.ok(),
    };

    Ok(web::Json(resp))
}
