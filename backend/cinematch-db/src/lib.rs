//! CineMatch Database Library
//!
//! This crate provides async database models and connection utilities for the CineMatch application.

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use thiserror::Error;
use uuid::Uuid;

pub mod models;
pub mod schema;

mod external_account;
mod party;
mod user;

pub use models::*;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database connection error: {0}")]
    Connection(String),

    #[error("Database query error: {0}")]
    Query(#[from] diesel::result::Error),

    #[error("Pool error: {0}")]
    Pool(#[from] diesel_async::pooled_connection::deadpool::PoolError),

    #[error("User not found: {0}")]
    UserNotFound(Uuid),

    #[error("Party not found: {0}")]
    PartyNotFound(Uuid),

    #[error("External account not found")]
    ExternalAccountNotFound,

    #[error("Cannot kick yourself from party")]
    CannotKickSelf,

    #[error("User is not a party member")]
    NotPartyMember,

    #[error("Only party leader can perform this action")]
    NotPartyLeader,

    #[error("Failed to generate unique party code after max attempts")]
    CodeGenerationFailed,

    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),

    #[error("Invalid username: {0}")]
    InvalidUsername(String),
}

pub type DbResult<T> = Result<T, DbError>;

// ============================================================================
// Validation
// ============================================================================

/// Username constraints
const USERNAME_MIN_LENGTH: usize = 3;
const USERNAME_MAX_LENGTH: usize = 32;

/// Validate a username: 3-32 chars, alphanumeric + underscore only
pub fn validate_username(username: &str) -> DbResult<()> {
    let len = username.len();

    if len < USERNAME_MIN_LENGTH {
        return Err(DbError::InvalidUsername(format!(
            "Username must be at least {} characters",
            USERNAME_MIN_LENGTH
        )));
    }

    if len > USERNAME_MAX_LENGTH {
        return Err(DbError::InvalidUsername(format!(
            "Username must be at most {} characters",
            USERNAME_MAX_LENGTH
        )));
    }

    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(DbError::InvalidUsername(
            "Username can only contain letters, numbers, and underscores".to_string(),
        ));
    }

    Ok(())
}

// ============================================================================
// Database Connection Pool
// ============================================================================

/// Async database connection pool
pub struct Database {
    pool: Pool<AsyncPgConnection>,
}

impl Database {
    /// Create a new database connection pool from a database URL
    pub fn new(database_url: &str) -> DbResult<Self> {
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
        let pool = Pool::builder(config)
            .build()
            .map_err(|e| DbError::Connection(e.to_string()))?;
        Ok(Self { pool })
    }

    /// Get a connection from the pool
    pub async fn conn(
        &self,
    ) -> DbResult<diesel_async::pooled_connection::deadpool::Object<AsyncPgConnection>> {
        self.pool.get().await.map_err(DbError::from)
    }
}
