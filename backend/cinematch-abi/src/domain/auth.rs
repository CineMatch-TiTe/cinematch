use super::DomainError;
use async_trait::async_trait;
use cinematch_common::Config;
use cinematch_db::domain::User;
use cinematch_db::models::AuthProvider;
use octocrab::Octocrab;
use secrecy::ExposeSecret;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct GithubUserInfo {
    pub provider_user_id: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
}

#[async_trait]
pub trait ExternalAuthLogic {
    /// Exchanges a GitHub OAuth code for user information.
    async fn exchange_github(code: &str) -> Result<GithubUserInfo, DomainError>;

    /// Handles the OAuth callback logic.
    /// Either logs in an existing user or links the account to the current session user.
    async fn handle_callback(
        ctx: &impl cinematch_db::AppContext,
        provider: AuthProvider,
        provider_user_id: String,
        email: Option<String>,
        display_name: Option<String>,
        session_user_id: Option<Uuid>,
    ) -> Result<User, DomainError>;
}

#[async_trait]
impl ExternalAuthLogic for User {
    async fn exchange_github(code: &str) -> Result<GithubUserInfo, DomainError> {
        let config = Config::get();
        let github = config
            .github
            .as_ref()
            .ok_or_else(|| DomainError::Internal("GitHub OAuth is not configured".to_string()))?;

        // 1. Exchange code for access token using Octocrab with explicit Accept header
        // Using unwrap() for "accept" parsing as it's a known valid header name, matching the user's example.
        let oauth_client = Octocrab::builder()
            .base_uri("https://github.com")
            .unwrap()
            .add_header("accept".parse().unwrap(), "application/json".to_string()) // unwrap safe, "accept" is a valid header name
            .build()
            .unwrap();

        let token_res = oauth_client
            .post::<_, serde_json::Value>(
                "/login/oauth/access_token",
                Some(&serde_json::json!({
                    "code": code,
                    "client_id": github.client_id,
                    "client_secret": github.client_secret.expose_secret(),
                })),
            )
            .await
            .map_err(|e| DomainError::Internal(format!("Token exchange failed: {}", e)))?;

        let oauth =
            serde_json::from_value::<octocrab::auth::OAuth>(token_res.clone()).map_err(|_| {
                DomainError::Internal(format!("Failed to parse OAuth response: {:?}", token_res))
            })?;

        // 2. Fetch user info from GitHub
        let crab = Octocrab::builder()
            .user_access_token(oauth.access_token.expose_secret())
            .build()
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let gh_user =
            crab.current().user().await.map_err(|e| {
                DomainError::Internal(format!("Failed to fetch GitHub user: {}", e))
            })?;

        Ok(GithubUserInfo {
            provider_user_id: gh_user.id.to_string(),
            display_name: Some(gh_user.login), // Fallback to login
            email: None, // Author model doesn't have email directly in this version
        })
    }

    async fn handle_callback(
        ctx: &impl cinematch_db::AppContext,
        provider: AuthProvider,
        provider_user_id: String,
        email: Option<String>,
        display_name: Option<String>,
        session_user_id: Option<Uuid>,
    ) -> Result<User, DomainError> {
        // 1. Try to find existing user by this external record
        let external_user = User::from_external_id(ctx, provider, &provider_user_id).await?;

        match (session_user_id, external_user) {
            // Case A: User is logged in AND account is already linked
            (Some(sid), Some(ext_user)) => {
                if sid == ext_user.id {
                    // Already linked to THIS user
                    Ok(ext_user)
                } else {
                    // Conflict: Already linked to a DIFFERENT user
                    Err(DomainError::Conflict(format!(
                        "This {:?} account is already linked to another CineMatch user.",
                        provider
                    )))
                }
            }

            // Case B: User is NOT logged in, but account is already linked
            (None, Some(ext_user)) => {
                // Log in as the existing user
                Ok(ext_user)
            }

            // Case C: User is logged in, but account is NOT linked yet
            (Some(sid), None) => {
                // Verify session user exists
                let user_obj = User::from_id(ctx, sid)
                    .await
                    .map_err(|_| DomainError::NotFound("Session user not found".to_string()))?;

                // Link it
                user_obj
                    .link_account(
                        ctx,
                        provider,
                        &provider_user_id,
                        email.as_deref(),
                        display_name.as_deref(),
                    )
                    .await?;

                // If it was a guest account, make it persistent
                if user_obj.is_oneshot(ctx).await? {
                    user_obj.make_persistent(ctx).await?;
                }

                Ok(user_obj)
            }

            // Case D: Neither logged in nor account linked
            (None, None) => {
                // Create a new user (start as guest to satisfy DB trigger)
                let name = display_name
                    .clone()
                    .unwrap_or_else(|| format!("user_{}", &provider_user_id[..6]));
                let user_obj = User::create_guest(ctx, &name).await?;

                // Link it
                user_obj
                    .link_account(
                        ctx,
                        provider,
                        &provider_user_id,
                        email.as_deref(),
                        display_name.as_deref(),
                    )
                    .await?;

                // Finalize as persistent
                user_obj.make_persistent(ctx).await?;

                Ok(user_obj)
            }
        }
    }
}
