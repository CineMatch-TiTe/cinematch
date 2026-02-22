//! Domain error types.
//!
//! `DomainError` implements `ResponseError` for automatic HTTP response conversion.

use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use cinematch_common::models::ErrorResponse;
use cinematch_db::DbError;
use std::fmt;

/// Domain-level errors with automatic HTTP response mapping.
#[derive(Debug)]
pub enum DomainError {
    /// Resource not found (404)
    NotFound(String),
    /// User lacks permission (403)
    Forbidden(String),
    /// Invalid request data (400)
    BadRequest(String),
    /// Resource conflict (409)
    Conflict(String),
    /// Server error (500)
    Internal(String),
    /// Unauthorized (401)
    Unauthorized(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
            Self::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            Self::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            Self::Conflict(msg) => write!(f, "Conflict: {}", msg),
            Self::Internal(msg) => write!(f, "Internal error: {}", msg),
            Self::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
        }
    }
}

impl std::error::Error for DomainError {}

impl From<DbError> for DomainError {
    fn from(e: DbError) -> Self {
        match &e {
            DbError::UserNotFound(_) => DomainError::NotFound("User not found".into()),
            DbError::PartyNotFound(_) => DomainError::NotFound("Party not found".into()),
            DbError::NotPartyMember => DomainError::Forbidden("Not a party member".into()),
            DbError::UserNotInParty(_) => DomainError::Forbidden("User not in party".into()),
            DbError::Connection(msg) => DomainError::Internal(msg.clone()),
            DbError::Query(e) => DomainError::Internal(e.to_string()),
            DbError::Pool(e) => DomainError::Internal(e.to_string()),
            _ => DomainError::Internal(e.to_string()),
        }
    }
}

impl ResponseError for DomainError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let msg = match self {
            Self::NotFound(m)
            | Self::Forbidden(m)
            | Self::BadRequest(m)
            | Self::Conflict(m)
            | Self::Internal(m)
            | Self::Unauthorized(m) => m.clone(),
        };
        HttpResponse::build(self.status_code()).json(ErrorResponse::new(msg))
    }
}
