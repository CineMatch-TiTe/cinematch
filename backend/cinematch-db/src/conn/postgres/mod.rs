//! PostgreSQL-specific types and re-exports.
//!
//! This module provides organized access to PostgreSQL-related code:
//! - `schema` - Diesel-generated table definitions
//! - `models` - Queryable/Insertable structs for database rows

// Re-export from parent for organized access
pub use crate::models;
pub use crate::schema;
