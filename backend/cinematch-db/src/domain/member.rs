//! Party member domain type with lazy-loading.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::repo::party::models::PartyMember;
use crate::{AppContext, DbResult};

use super::{Party, User};

/// A party member with lazy-loading data access.
///
/// Represents the relationship between a user and a party.
/// Only stores references and IDs, fetching data fresh on each call.
#[derive(Clone, Copy, Debug)]
pub struct Member {
    pub user_id: Uuid,
    pub party_id: Uuid,
}

impl Member {
    /// Create a new Member handle (does not verify existence).
    pub(crate) fn new(user_id: Uuid, party_id: Uuid) -> Self {
        Self { user_id, party_id }
    }

    /// Create a Member handle, verifying the membership exists.
    pub async fn from_ids(ctx: &impl AppContext, user_id: Uuid, party_id: Uuid) -> DbResult<Self> {
        // Verify membership exists
        ctx.db()
            .get_party_member(party_id, user_id)
            .await?
            .ok_or(crate::DbError::NotPartyMember)?;
        Ok(Self { user_id, party_id })
    }

    // ========================================================================
    // Lazy Getters - All fetch fresh from DB
    // ========================================================================

    /// Get the user as a User domain type.
    pub async fn user(&self, ctx: &impl AppContext) -> DbResult<User> {
        User::from_id(ctx, self.user_id).await
    }

    /// Get the party as a Party domain type.
    pub async fn party(&self, ctx: &impl AppContext) -> DbResult<Party> {
        Party::from_id(ctx, self.party_id).await
    }

    /// Get whether this member is ready.
    pub async fn is_ready(&self, ctx: &impl AppContext) -> DbResult<bool> {
        let member = ctx
            .db()
            .get_party_member(self.party_id, self.user_id)
            .await?
            .ok_or(crate::DbError::NotPartyMember)?;
        Ok(member.is_ready)
    }

    /// Get when this member joined the party.
    pub async fn joined_at(&self, ctx: &impl AppContext) -> DbResult<DateTime<Utc>> {
        let member = ctx
            .db()
            .get_party_member(self.party_id, self.user_id)
            .await?
            .ok_or(crate::DbError::NotPartyMember)?;
        Ok(member.joined_at)
    }

    /// Get the raw member record.
    pub async fn record(&self, ctx: &impl AppContext) -> DbResult<PartyMember> {
        ctx.db()
            .get_party_member(self.party_id, self.user_id)
            .await?
            .ok_or(crate::DbError::NotPartyMember)
    }

    // ========================================================================
    // Mutations - All write directly to DB
    // ========================================================================

    /// Set this member's ready state.
    pub async fn set_ready(&self, ctx: &impl AppContext, ready: bool) -> DbResult<()> {
        ctx.db()
            .set_member_ready(self.party_id, self.user_id, ready)
            .await
    }

    /// Remove this member from the party.
    pub async fn leave(&self, ctx: &impl AppContext) -> DbResult<()> {
        ctx.db()
            .remove_party_member(self.party_id, self.user_id)
            .await
    }
}
