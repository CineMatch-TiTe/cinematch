//! Re-exports all database models from domain modules.
//!
//! This module provides backwards-compatible access to all models.
//! Prefer importing from repo modules directly (e.g., `crate::repo::user::User`).

// Re-export from repo modules
pub use crate::repo::movie::models::*;
pub use crate::repo::party::models::*;
pub use crate::repo::schedules::models::*;
pub use crate::repo::user::models::*;
pub use crate::repo::vote::models::*;

// Re-export qdrant models for easy access
pub use crate::conn::qdrant::models::{CastMember, MovieData};
