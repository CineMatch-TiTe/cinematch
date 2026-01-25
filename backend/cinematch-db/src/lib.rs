//! CineMatch Database Library
//!
//! This crate provides async database models and connection utilities for the CineMatch application.

use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::deadpool::Pool;
use thiserror::Error;
use uuid::Uuid;

pub mod models;
pub mod schema;
pub mod vector;

// Batch size for bulk operations
pub const BATCH_SIZE: usize = 20;

mod external_account;
mod movie;
mod party;
mod taste;
mod user;

use diesel::Connection;
use diesel::PgConnection;
pub use models::*;

use crate::vector::qdrant::QdrantService;

// ============================================================================
// Error Types
// ============================================================================

use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../cinematch-db/migrations");

use std::error::Error;

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

    #[error("User is not a party member")]
    NotPartyMember,

    #[error("User is not in a party: {0}")]
    UserNotInParty(Uuid),

    #[error("Failed to generate unique party code after max attempts")]
    CodeGenerationFailed,

    #[error("Invalid genre id: {0}")]
    InvalidGenreId(Uuid),

    #[error("Invalid user preferences: {0}")]
    InvalidPreferences(String),

    #[error("Other database error: {0}")]
    Other(String),
}

pub type DbResult<T> = Result<T, DbError>;

// ============================================================================
// Database Connection Pool
// ============================================================================

/// Async database connection pool
pub struct Database {
    pub pool: Pool<AsyncPgConnection>,
    pub vector: QdrantService,
}

impl Database {
    /// Create a new database connection pool from a database URL
    pub fn new(postgres_url: &str, qdrant_url: &str) -> DbResult<Self> {
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(postgres_url);
        let pool = Pool::builder(config)
            .build()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let vector_service =
            QdrantService::new(qdrant_url).map_err(|e| DbError::Connection(e.to_string()))?;
        Ok(Self {
            pool,
            vector: vector_service,
        })
    }

    /// Get a connection from the pool
    pub async fn conn(
        &self,
    ) -> DbResult<diesel_async::pooled_connection::deadpool::Object<AsyncPgConnection>> {
        self.pool.get().await.map_err(DbError::from)
    }

    pub async fn run_migrations(
        &self,
        database_url: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        let mut conn = PgConnection::establish(database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

        conn.run_pending_migrations(MIGRATIONS)?;

        Ok(())
    }
}
