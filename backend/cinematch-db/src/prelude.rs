//! Prelude for convenient imports.
//!
//! # Example
//! ```ignore
//! use cinematch_db::prelude::*;
//!
//! let party = Party::from_id(db.clone(), party_id).await?;
//! let members = party.members().await?;
//! ```

// Domain types (lazy-loading)
pub use crate::domain::{Member, Party, Preferences, User};

// Database connection
pub use crate::Database;

// Error types
pub use crate::{DbError, DbResult};

// Re-export common types for convenience
pub use std::sync::Arc;
pub use uuid::Uuid;

// Re-export party state enum (frequently needed)
pub use crate::repo::party::models::PartyState;
