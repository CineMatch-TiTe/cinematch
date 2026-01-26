//! Party domain handler. In-memory Party model; load from DB on access, persist on change.

#[allow(unused_imports)]
pub use cinematch_api::handler::party::{
    Party, PartyError, try_auto_advance_on_ready, try_auto_end_voting,
};
