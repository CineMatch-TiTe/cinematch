use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::{Modify, OpenApi};

pub fn extract_user_id(
    identity: impl Into<Option<actix_identity::Identity>>,
) -> Result<uuid::Uuid, ApiError> {
    let identity_opt: Option<actix_identity::Identity> = identity.into();
    match identity_opt {
        Some(id) => match id.id() {
            Ok(id_str) => match uuid::Uuid::parse_str(&id_str) {
                Ok(uuid) => Ok(uuid),
                Err(_) => Err(ApiError::InternalServerError("Invalid user ID".to_string())),
            },
            Err(_) => Err(ApiError::Unauthorized("No user ID found".to_string())),
        },
        None => Err(ApiError::Unauthorized("No identity provided".to_string())),
    }
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
mod movie;
mod party;
mod recommendation;
mod user;
mod websocket;

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
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    components(schemas(
        crate::api_error::ApiError,
        cinematch_common::models::RecommendationMethod,
        cinematch_common::models::VectorType,
        cinematch_common::models::ErrorResponse,
        crate::movie::MovieResponse,
        crate::movie::CastMemberResponse,
        crate::movie::RecommendedMoviesResponse,
    )),
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
