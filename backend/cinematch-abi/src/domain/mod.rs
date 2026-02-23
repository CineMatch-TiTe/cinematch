//! Domain layer for CineMatch.
//!
//! Contains extension traits that add business logic to the slim domain types
//! from `cinematch_db`. These traits provide validation, state machine logic,
//! and authorization checks.
//!
//! # Architecture
//!
//! - `cinematch_db::domain` - Slim types with data access methods
//! - `cinematch_abi::domain` - Extension traits with business logic
//!
//! # Usage
//!
//! ```ignore
//! use cinematch_abi::prelude::*;
//!
//! let party = Party::from_id(db.clone(), party_id).await?;
//! party.require_leader(user_id).await?;  // From PartyLogic trait
//! party.advance_phase(user_id).await?;   // From PartyStateMachine trait
//! ```

mod auth;
mod error;
mod party;
mod recommendation;
mod user;

pub use auth::ExternalAuthLogic;
pub use cinematch_db::domain::User;
pub use error::DomainError;
pub use party::{
    EndVotingTransition, PartyAdvanceOutcome, PartyCrud, PartyJoin, PartyLogic, PartyStateMachine,
    PartyValidation,
};
pub use recommendation::Recommendation;
pub use user::{UserCreation, UserLogic};
