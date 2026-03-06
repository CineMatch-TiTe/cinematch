//! Authentication extractors for the server.
//!
//! Provides `JwtAuth` for bearer tokens and a small helper that allows
//! routes to accept either a cookie-based identity or a JWT when
//! extracting a user ID.

use actix_identity::Identity;
use actix_web::dev::Payload;
use actix_web::{Error, FromRequest, HttpRequest, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use futures::future::{Ready, ready};
use serde::Deserialize;
use uuid::Uuid;

use crate::api_error::ApiError;
use cinematch_abi::auth::jwt;

/// Simple extractor that validates a bearer JWT and returns the contained
/// user id plus the expiration timestamp.  If the header is missing or the
/// token is invalid the request will fail with `401 Unauthorized`.
#[derive(Debug)]
pub struct JwtAuth {
    pub user_id: Uuid,
    pub expires_at: i64,
}

/// Convenience alias exported by the server crate.  Handlers can import
/// `crate::Jwt` instead of the longer path.  The alias preserves the
/// structure with an expiration field.
pub type Jwt = JwtAuth;

/// Unified authentication extractor that combines cookie identity and bearer
/// JWT into one type.  Acts similarly to `AppState` in that you can declare
/// `auth: Option<Auth>` in handler signatures and let Actix wire it up.
#[derive(Debug)]
pub struct Auth {
    pub user_id: Uuid,
    /// If authentication was performed via a JWT, this contains the token's
    /// expiration timestamp (unix seconds).  Cookie-based identities do not
    /// populate this field.
    pub token_expires_at: Option<i64>,
}

impl FromRequest for Auth {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        // try cookie identity first
        let identity_opt = actix_identity::Identity::from_request(req, payload)
            .into_inner()
            .ok();
        // try bearer jwt next
        let jwt_opt = JwtAuth::from_request(req, payload).into_inner().ok();
        // determine expiration if we have a JWT
        let token_expires = jwt_opt.as_ref().map(|j| j.expires_at);
        match user_id_from(identity_opt, jwt_opt) {
            Ok(uid) => ready(Ok(Auth {
                user_id: uid,
                token_expires_at: token_expires,
            })),
            Err(e) => ready(Err(actix_web::error::ErrorUnauthorized(e.to_string()))),
        }
    }
}

impl Auth {
    /// convenient accessor
    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    /// Returns the optional token expiration (unix seconds) if available.
    pub fn token_expires_at(&self) -> Option<i64> {
        self.token_expires_at
    }
}

#[derive(Deserialize)]
struct TokenQuery {
    token: String,
}

impl FromRequest for JwtAuth {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let token = match BearerAuth::from_request(req, payload).into_inner() {
            Ok(bearer) => Some(bearer.token().to_string()),
            Err(_) => {
                // Fallback to query parameter for WebSockets
                if let Ok(query) = web::Query::<TokenQuery>::from_query(req.query_string()) {
                    Some(query.into_inner().token)
                } else {
                    None
                }
            }
        };

        match token {
            Some(t) => match jwt::decode_claims(&t) {
                Ok((uid, exp)) => ready(Ok(JwtAuth {
                    user_id: uid,
                    expires_at: exp,
                })),
                Err(e) => ready(Err(actix_web::error::ErrorUnauthorized(e.to_string()))),
            },
            None => ready(Err(actix_web::error::ErrorUnauthorized(
                "Bearer token or token query param required".to_string(),
            ))),
        }
    }
}

/// Helper that takes an optional `Identity` (cookie) and an optional
/// `JwtAuth` and chooses whichever is present.  Used in `extract_user_id`.
pub fn user_id_from(identity: Option<Identity>, jwt: Option<JwtAuth>) -> Result<Uuid, ApiError> {
    if let Some(id) = identity {
        match id.id() {
            Ok(s) => match Uuid::parse_str(&s) {
                Ok(uuid) => return Ok(uuid),
                Err(_) => return Err(ApiError::InternalServerError("Invalid user ID".to_string())),
            },
            Err(_) => return Err(ApiError::Unauthorized("No user ID found".to_string())),
        }
    }

    if let Some(jwt_auth) = jwt {
        return Ok(jwt_auth.user_id);
    }

    Err(ApiError::Unauthorized("No identity provided".to_string()))
}
