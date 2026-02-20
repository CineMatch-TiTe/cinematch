//! Domain types with lazy-loading pattern.
//!
//! These types hold only an `Arc<Database>` reference and an ID,
//! fetching all data fresh from the database on each method call.
//! This eliminates stale data issues at the cost of more DB queries.
//!
//! # Example
//! ```ignore
//! use cinematch_db::prelude::*;
//!
//! let party = Party::from_id(db.clone(), party_id).await?;
//! let members = party.members().await?;  // Fetches fresh from DB
//! let state = party.state().await?;      // Fetches fresh from DB
//! ```

pub mod member;
pub mod movie;
pub mod party;
pub mod preferences;
pub mod user;

pub use member::Member;
pub use movie::Movie;
pub use party::Party;
pub use preferences::Preferences;
pub use user::User;
