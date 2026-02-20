//! Party extension traits and business logic.
//!
//! Provides `PartyLogic` trait that extends `cinematch_db::domain::Party`
//! with validation, state machine, and authorization methods.

mod crud;
mod state_machine;
pub mod utils;
mod validation;

use cinematch_db::PartyState;
use cinematch_db::domain::Party;
use uuid::Uuid;

use super::DomainError;
use async_trait::async_trait;

/// Result of end-voting: either round 2 started (stay in Voting) or phase changed.
#[derive(Debug, Clone)]
pub enum EndVotingTransition {
    Round2Started,
    PhaseChanged(PartyState),
}

/// Result of advance_phase: phase change or voting-ended outcome.
#[derive(Debug, Clone)]
pub enum PartyAdvanceOutcome {
    PhaseChanged(PartyState),
    VotingEnded(EndVotingTransition),
}

pub use crud::PartyCrud;
pub use state_machine::PartyStateMachine;
pub use validation::PartyValidation;

/// Extension trait for Party business logic.
///
/// Combines validation, state machine, and CRUD operations.
/// Import this trait to use business logic methods on `Party`.
pub trait PartyLogic: PartyValidation + PartyStateMachine + PartyCrud + PartyJoin {}

// Blanket implementation for any type that implements all sub-traits
// Blanket implementation for any type that implements all sub-traits
#[async_trait]
impl<T: PartyValidation + PartyStateMachine + PartyCrud + PartyJoin + Sync> PartyLogic for T {}

/// Static-like operations for Party (creation/lookup).
#[async_trait]
pub trait PartyJoin {
    /// Join a party by code.
    /// Join a party by code.
    async fn join_by_code(
        ctx: &impl cinematch_db::AppContext,
        user_id: Uuid,
        code: &str,
    ) -> Result<Party, DomainError>;
}

#[async_trait]
impl PartyJoin for Party {
    async fn join_by_code(
        ctx: &impl cinematch_db::AppContext,
        user_id: Uuid,
        code: &str,
    ) -> Result<Party, DomainError> {
        // Find party by code
        let party = Party::from_code(ctx, code)
            .await
            .map_err(DomainError::from)?
            .ok_or_else(|| DomainError::NotFound(format!("Party with code {} not found", code)))?;

        // Add member checked (validates if already in)
        party.add_member_checked(ctx, user_id).await?;

        // Broadcast joined
        // Or just fetch it.
        // Server handler did this. Now we do it here.
        use cinematch_db::domain::User;
        let user_obj = User::from_id(ctx, user_id)
            .await
            .map_err(DomainError::from)?;
        let username = user_obj
            .username(ctx)
            .await
            .unwrap_or_else(|_| "Unknown".to_string());

        use cinematch_common::models::websocket::{MemberJoined, ServerMessage};
        let msg = ServerMessage::PartyMemberJoined(MemberJoined { user_id, username });

        if let Ok(members) = party.member_ids(ctx).await {
            ctx.send_users(&members, &msg);
        }

        Ok(party)
    }
}
