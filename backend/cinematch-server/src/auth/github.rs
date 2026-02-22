use crate::AppState;
use crate::api_error::ApiError;
use actix_identity::Identity;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, get, web};
use cinematch_abi::domain::{ExternalAuthLogic as _, User};
use cinematch_common::models::ErrorResponse;
use cinematch_db::models::AuthProvider;
use log::{debug, error};

#[derive(Debug, serde::Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: Option<String>,
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
        (status = 302, description = "Login successful, redirect back to app"),
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
) -> Result<HttpResponse, ApiError> {
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

    // 3. Log in the user (or refresh session)
    if let Err(e) = Identity::login(&req.extensions(), user.id.to_string()) {
        error!("Failed to set user identity in session: {e}");
        return Err(ApiError::InternalServerError(
            "Failed to set user identity".to_string(),
        ));
    }

    // Success - redirect back to home (root)
    Ok(HttpResponse::Found()
        .append_header(("Location", "/"))
        .finish())
}
