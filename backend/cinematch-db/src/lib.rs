//! CineMatch Database Library
//!
//! This crate provides async database models and connection utilities for the CineMatch application.
//!
//! ## Module Organization
//!
//! - `conn/` - Connection backends (postgres, redis, qdrant)
//! - `repo/` - Repository layer with domain modules (user, party, movie, vote)
//! - `domain/` - Lazy-loading domain types (Party, User, Member, Preferences)
//! - `schema` - Diesel-generated PostgreSQL schema
//! - `models` - Re-exports all models for convenience
//! - `prelude` - Convenient imports for common types
//!
//! ## Usage
//!
//! ```ignore
//! use cinematch_db::prelude::*;
//!
//! let party = Party::from_id(db.clone(), party_id).await?;
//! let members = party.members().await?;  // Lazy - fetches fresh from DB
//! ```

use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::deadpool::Pool;
use thiserror::Error;
use uuid::Uuid;

// Connection backends
pub mod conn;
pub mod schema;

// Repository layer (raw database operations)
pub mod repo;

// Domain layer (lazy-loading types)
pub mod domain;

// Prelude for convenient imports
pub mod prelude;

// Re-export all models from domain modules for backwards compatibility
pub mod models;
pub use models::*;

// Batch size for bulk operations
pub const BATCH_SIZE: usize = 20;

use diesel::Connection;
use diesel::PgConnection;

use crate::conn::QdrantService;

// Re-export pool types for convenience
pub use deadpool_redis;
pub type RedisPool = deadpool_redis::Pool;

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

    #[error("Redis pool error: {0}")]
    RedisPool(#[from] deadpool_redis::PoolError),

    #[error("Redis error: {0}")]
    Redis(#[from] deadpool_redis::redis::RedisError),

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

    #[error("Cache miss")]
    CacheMiss,

    #[error("Other database error: {0}")]
    Other(String),
}

pub type DbResult<T> = Result<T, DbError>;

use cinematch_common::models::websocket::ServerMessage;
use std::sync::Arc;

/// Application context trait for dependency injection.
/// Allows standardized access to DB and broadcasting capabilities.
pub trait AppContext: Send + Sync {
    /// Get the database connection pool.
    fn db(&self) -> &Arc<Database>;

    /// Broadcast a message to a party, optionally excluding one user.
    fn broadcast_party(&self, party_id: Uuid, msg: &ServerMessage, exclude: Option<Uuid>);

    /// Send a message to specific users.
    fn send_users(&self, user_ids: &[Uuid], msg: &ServerMessage);
}

/// A simple context that provides database access but no-op broadcasting.
/// Useful for tests, recommendation engine, or non-interactive components.
pub struct SimpleContext(pub Arc<Database>);

impl AppContext for SimpleContext {
    fn db(&self) -> &Arc<Database> {
        &self.0
    }
    fn broadcast_party(&self, _party_id: Uuid, _msg: &ServerMessage, _exclude: Option<Uuid>) {}
    fn send_users(&self, _user_ids: &[Uuid], _msg: &ServerMessage) {}
}

// ============================================================================
// Database Connection Pool
// ============================================================================

#[derive(Clone)]
/// Async database connection pool with PostgreSQL, Redis, and Qdrant
pub struct Database {
    /// PostgreSQL connection pool (primary data store)
    pub pool: Pool<AsyncPgConnection>,
    /// Redis connection pool (caching layer)
    pub redis: RedisPool,
    /// Qdrant vector database service (semantic search)
    pub vector: QdrantService,
}

impl Database {
    /// Create a new database connection pool from database URLs
    ///
    /// # Arguments
    /// * `postgres_url` - PostgreSQL connection URL
    /// * `redis_url` - Redis connection URL (e.g., "redis://127.0.0.1:6379")
    /// * `qdrant_url` - Qdrant vector database URL
    pub fn new(postgres_url: &str, redis_url: &str, qdrant_url: &str) -> DbResult<Self> {
        // PostgreSQL pool
        let pg_config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(postgres_url);
        let pool = Pool::builder(pg_config)
            .build()
            .map_err(|e| DbError::Connection(format!("PostgreSQL: {}", e)))?;

        // Redis pool
        let redis_config = deadpool_redis::Config::from_url(redis_url);
        let redis = redis_config
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .map_err(|e| DbError::Connection(format!("Redis: {}", e)))?;

        // Qdrant vector service
        let vector = QdrantService::new(qdrant_url)
            .map_err(|e| DbError::Connection(format!("Qdrant: {}", e)))?;

        Ok(Self {
            pool,
            redis,
            vector,
        })
    }

    /// Get a PostgreSQL connection from the pool
    pub async fn conn(
        &self,
    ) -> DbResult<diesel_async::pooled_connection::deadpool::Object<AsyncPgConnection>> {
        self.pool.get().await.map_err(DbError::from)
    }

    /// Get a Redis connection from the pool
    pub async fn redis_conn(&self) -> DbResult<deadpool_redis::Connection> {
        self.redis.get().await.map_err(DbError::from)
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
