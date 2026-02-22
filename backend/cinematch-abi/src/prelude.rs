//! Prelude for convenient imports.
//!
//! # Example
//! ```ignore
//! use cinematch_abi::prelude::*;
//!
//! let party = Party::from_id(db.clone(), party_id).await?;
//! party.require_leader(user_id).await?;  // From PartyLogic trait
//! party.advance_phase(user_id).await?;   // From PartyStateMachine trait
//! ```

// Re-export db prelude (domain types, Database, errors, etc.)
pub use cinematch_db::prelude::*;

// Extension traits for business logic
pub use crate::domain::{
    DomainError, EndVotingTransition, PartyAdvanceOutcome, PartyCrud, PartyLogic,
    PartyStateMachine, PartyValidation, UserLogic, get_timeout_secs,
};
