//! Connection backends for different databases.
//!
//! - `postgres` - PostgreSQL (via Diesel) - primary data store
//! - `redis` - Redis caching layer with TTL support
//! - `qdrant` - Qdrant vector database for semantic search

pub mod postgres;
pub mod qdrant;
pub mod redis;

pub use qdrant::QdrantService;
