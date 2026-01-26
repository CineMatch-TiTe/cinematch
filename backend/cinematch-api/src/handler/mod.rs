//! Server-side domain handlers. Party logic lives here; HTTP handlers are thin wrappers.

pub mod party;

pub use party::{
    EndVotingTransition, Party, PartyAdvanceOutcome, PartyError, run_timeouts_tick,
    try_auto_advance_on_ready, try_auto_end_voting,
};
