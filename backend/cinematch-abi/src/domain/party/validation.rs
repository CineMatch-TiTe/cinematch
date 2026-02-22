//! Validation helpers and permission checks for Party.

use cinematch_db::AppContext;
use cinematch_db::PartyState;
use cinematch_db::domain::Party;
use uuid::Uuid;

use super::super::DomainError;

/// Validation and authorization methods for Party.
pub trait PartyValidation {
    /// Check if a user is the party leader.
    fn is_leader(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
    ) -> impl std::future::Future<Output = Result<bool, DomainError>>;

    /// Check if a user is a member of the party.
    fn is_member_of(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
    ) -> impl std::future::Future<Output = Result<bool, DomainError>>;

    /// Require the user to be a member, return error if not.
    fn require_member(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
    ) -> impl std::future::Future<Output = Result<(), DomainError>>;

    /// Require the user to be the leader, return error if not.
    fn require_leader(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
    ) -> impl std::future::Future<Output = Result<(), DomainError>>;

    /// Require the party to be in a specific state.
    fn require_state(
        &self,
        ctx: &impl AppContext,
        expected: PartyState,
    ) -> impl std::future::Future<Output = Result<(), DomainError>>;
}

impl PartyValidation for Party {
    async fn is_leader(&self, ctx: &impl AppContext, user_id: Uuid) -> Result<bool, DomainError> {
        let leader_id = self.leader_id(ctx).await.map_err(DomainError::from)?;
        Ok(leader_id == user_id)
    }

    async fn is_member_of(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
    ) -> Result<bool, DomainError> {
        self.is_member(ctx, user_id)
            .await
            .map_err(DomainError::from)
    }

    async fn require_member(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
    ) -> Result<(), DomainError> {
        if !self.is_member_of(ctx, user_id).await? {
            return Err(DomainError::Forbidden("Not a party member".into()));
        }
        Ok(())
    }

    async fn require_leader(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
    ) -> Result<(), DomainError> {
        if !self.is_leader(ctx, user_id).await? {
            return Err(DomainError::Forbidden("Not the party leader".into()));
        }
        Ok(())
    }

    async fn require_state(
        &self,
        ctx: &impl AppContext,
        expected: PartyState,
    ) -> Result<(), DomainError> {
        let current = self.state(ctx).await.map_err(DomainError::from)?;
        if current != expected {
            return Err(DomainError::BadRequest(format!(
                "Party must be in {:?} state",
                expected
            )));
        }
        Ok(())
    }
}
