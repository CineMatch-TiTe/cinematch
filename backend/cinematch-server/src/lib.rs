use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::{Modify, OpenApi};

pub fn extract_user_id(
    identity: impl Into<Option<actix_identity::Identity>>,
    jwt: Option<crate::Jwt>,
) -> Result<uuid::Uuid, ApiError> {
    let identity_opt: Option<actix_identity::Identity> = identity.into();
    crate::auth::guard::user_id_from(identity_opt, jwt)
}

pub fn validate_username(input: &str) -> Result<String, ApiError> {
    let name = input
        .trim()
        .chars()
        .filter(|c| !c.is_control())
        .collect::<String>();
    if name.len() < cinematch_common::NAME_MIN_LENGTH
        || name.len() > cinematch_common::NAME_MAX_LENGTH
    {
        return Err(ApiError::BadRequest(format!(
            "Username must be between {} and {} characters",
            cinematch_common::NAME_MIN_LENGTH,
            cinematch_common::NAME_MAX_LENGTH
        )));
    }
    Ok(name)
}

mod auth;
mod handlers;
mod movie;
mod party;
mod recommendation;
mod user;
mod websocket;

// convenient re-exports
pub use auth::guard::Jwt;

pub mod api_error;
pub mod routes;
pub use api_error::ApiError;

pub use cinematch_abi::AppState;
pub use cinematch_db::Database;

// Re-export domain from abi for external use
pub use cinematch_abi::domain;

// Re-export websocket models for external use
pub use websocket::{ServerMessage, broadcast_to_party};

/// Adds cookie-based auth security scheme. Actix-identity uses the `id` cookie
/// (httpOnly, path=/, samesite=Lax, secure). Obtain via `POST /api/user/login/guest`.
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "cookie_auth",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("id"))),
            );
            // bearer scheme with JWT format
            let mut http = utoipa::openapi::security::Http::new(
                utoipa::openapi::security::HttpAuthScheme::Bearer,
            );
            http.bearer_format = Some("JWT".to_string());
            components.add_security_scheme("bearer_auth", SecurityScheme::Http(http));
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    nest((
        path = "/api/recommend",
        api = crate::recommendation::handlers::RecommendationApi
    )),
    components(schemas(crate::api_error::ApiError)),
    tags(
        (name = "Auth", description = "Authentication and session management."),
        (name = "User", description = "User profile and global preferences."),
        (name = "Party", description = "Base party management (create, detail, join)."),
        (name = "Member Ops", description = "Member operations (ready state, listing)."),
        (name = "Picking", description = "Movie selection phase tools."),
        (name = "Voting", description = "Consensus and ballot management."),
        (name = "Leader Tools", description = "Administrative tools for party leaders."),
        (name = "Recommendation", description = "AI-powered movie suggestions."),
        (name = "Movie", description = "Global movie retrieval and search."),
        (name = "websocket", description = "Real-time communication.")
    )
)]
pub struct ApiDoc;
