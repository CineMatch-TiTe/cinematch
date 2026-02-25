//! Party domain type with lazy-loading.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use cinematch_common::HasId;
use uuid::Uuid;

use crate::repo::party::models::{PartyCode, PartyMember, PartyState};
use crate::{AppContext, Database, DbResult};

use super::Member;

/// A party with lazy-loading data access.
///
/// Only stores party ID.
/// All data is fetched fresh from the database on each method call, using the provided context.
#[derive(Clone, Copy, Debug)]
pub struct Party {
    pub id: Uuid,
}

impl HasId for Party {
    fn id(&self) -> Uuid {
        self.id
    }
}

impl Party {
    /// Create a new Party handle from an existing party ID.
    /// Verifies the party exists in the database.
    pub async fn from_id(ctx: &impl AppContext, id: Uuid) -> DbResult<Self> {
        // Verify party exists
        ctx.db().get_party(id).await?;
        Ok(Self { id })
    }

    /// Create a new Party handle without verifying existence.
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }

    /// Look up a party by its join code.
    pub async fn from_code(ctx: &impl AppContext, code: &str) -> DbResult<Option<Self>> {
        let party = ctx.db().get_party_by_code(code).await?;
        Ok(party.map(|p| Self { id: p.id }))
    }

    /// Create a new party with the given user as leader.
    /// Returns the new party and its join code.
    pub async fn create(ctx: &impl AppContext, leader_id: Uuid) -> DbResult<(Self, PartyCode)> {
        let (party, code) = ctx.db().create_party(leader_id).await?;
        Ok((Self { id: party.id }, code))
    }

    // ========================================================================
    // Lazy Getters - All fetch fresh from DB
    // ========================================================================

    /// Get the party's current state.
    pub async fn state(&self, ctx: &impl AppContext) -> DbResult<PartyState> {
        ctx.db().get_state(self.id).await
    }

    /// Get the party leader's user ID.
    pub async fn leader_id(&self, ctx: &impl AppContext) -> DbResult<Uuid> {
        let party = ctx.db().get_party(self.id).await?;
        Ok(party.party_leader_id)
    }

    /// Get the current voting round (None if not in voting).
    pub async fn voting_round(&self, ctx: &impl AppContext) -> DbResult<Option<i16>> {
        ctx.db().get_voting_round(self.id).await
    }

    /// Get the party's join code (None if code has been released).
    pub async fn join_code(&self, ctx: &impl AppContext) -> DbResult<Option<String>> {
        let code = ctx.db().get_party_code(self.id).await?;
        Ok(code.map(|c| c.code))
    }

    /// Get when the current phase was entered.
    pub async fn phase_entered_at(&self, ctx: &impl AppContext) -> DbResult<DateTime<Utc>> {
        let party = ctx.db().get_party(self.id).await?;
        Ok(party.phase_entered_at)
    }

    /// Get the selected movie ID (if one has been picked).
    pub async fn selected_movie_id(&self, ctx: &impl AppContext) -> DbResult<Option<i64>> {
        let party = ctx.db().get_party(self.id).await?;
        Ok(party.selected_movie_id)
    }

    /// Get whether voting is allowed.
    pub async fn can_vote(&self, ctx: &impl AppContext) -> DbResult<bool> {
        let party = ctx.db().get_party(self.id).await?;
        Ok(party.can_vote)
    }

    /// Get all party members as domain Member types.
    pub async fn members(&self, ctx: &impl AppContext) -> DbResult<Vec<Member>> {
        let members = ctx.db().get_party_members(self.id).await?;
        Ok(members
            .into_iter()
            .map(|m| Member::new(m.user_id, self.id))
            .collect())
    }

    /// Get all party member user IDs.
    pub async fn member_ids(&self, ctx: &impl AppContext) -> DbResult<Vec<Uuid>> {
        let members = ctx.db().get_party_members(self.id).await?;
        Ok(members.into_iter().map(|m| m.user_id).collect())
    }

    /// Get raw party member records (includes joined_at, is_ready).
    pub async fn member_records(&self, ctx: &impl AppContext) -> DbResult<Vec<PartyMember>> {
        ctx.db().get_party_members(self.id).await
    }

    /// Get the number of members in the party.
    pub async fn member_count(&self, ctx: &impl AppContext) -> DbResult<usize> {
        let members = ctx.db().get_party_members(self.id).await?;
        Ok(members.len())
    }

    /// Check if every member has voted in the current round.
    pub async fn have_all_members_voted(&self, ctx: &impl AppContext) -> DbResult<bool> {
        ctx.db().have_all_members_voted(self.id).await
    }

    /// Get number of unique members who have voted in this round.
    pub async fn get_voting_participation_count(&self, ctx: &impl AppContext) -> DbResult<usize> {
        ctx.db().get_voting_participation_count(self.id).await
    }

    /// Get all votes cast in the party.
    pub async fn get_votes(
        &self,
        ctx: &impl AppContext,
        user_id: Option<Uuid>,
    ) -> DbResult<std::collections::HashMap<i64, (u32, u32)>> {
        ctx.db().get_party_votes(self.id, user_id).await
    }

    /// Get votes cast by a specific user in this party.
    pub async fn get_user_votes(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
    ) -> DbResult<Vec<crate::repo::vote::models::Vote>> {
        ctx.db().get_user_votes(self.id, user_id).await
    }

    // ========================================================================
    // Mutations - All write directly to DB
    // ========================================================================

    /// Set the party's state and reset all ready states.
    pub async fn set_phase(&self, ctx: &impl AppContext, new_state: PartyState) -> DbResult<()> {
        ctx.db().set_phase(self.id, new_state).await?;
        Ok(())
    }

    /// Set the voting round number.
    pub async fn set_voting_round(
        &self,
        ctx: &impl AppContext,
        round: Option<i16>,
    ) -> DbResult<()> {
        ctx.db().set_voting_round(self.id, round).await
    }

    /// Set the selected movie ID.
    pub async fn set_selected_movie_id(
        &self,
        ctx: &impl AppContext,
        movie_id: Option<i64>,
    ) -> DbResult<()> {
        ctx.db().set_selected_movie_id(self.id, movie_id).await
    }

    /// Update phase_entered_at to now.
    pub async fn set_phase_entered_at_now(&self, ctx: &impl AppContext) -> DbResult<()> {
        ctx.db().set_phase_entered_at_now(self.id).await
    }

    /// Update phase_entered_at to now (alias for set_phase_entered_at_now).
    pub async fn reset_phase_timer(&self, ctx: &impl AppContext) -> DbResult<()> {
        self.set_phase_entered_at_now(ctx).await
    }

    /// Add a user to this party.
    pub async fn add_member(&self, ctx: &impl AppContext, user_id: Uuid) -> DbResult<Member> {
        ctx.db().add_party_member(self.id, user_id).await?;
        Ok(Member::new(user_id, self.id))
    }

    /// Remove a user from this party.
    pub async fn remove_member(&self, ctx: &impl AppContext, user_id: Uuid) -> DbResult<()> {
        ctx.db().remove_party_member(self.id, user_id).await
    }

    /// Transfer leadership to another user.
    pub async fn transfer_leadership(
        &self,
        ctx: &impl AppContext,
        new_leader_id: Uuid,
    ) -> DbResult<()> {
        ctx.db()
            .transfer_party_leadership(self.id, new_leader_id)
            .await?;
        Ok(())
    }

    /// Release the party's join code.
    pub async fn release_code(&self, ctx: &impl AppContext) -> DbResult<()> {
        ctx.db().release_party_code(self.id).await?;
        Ok(())
    }

    /// Regenerate the party's join code.
    pub async fn regenerate_code(&self, ctx: &impl AppContext) -> DbResult<PartyCode> {
        ctx.db().regenerate_party_code(self.id).await
    }

    /// Disband the party (removes all members, releases code, sets state to Disbanded).
    pub async fn disband(&self, ctx: &impl AppContext) -> DbResult<()> {
        ctx.db().disband_party(self.id).await?;
        Ok(())
    }

    /// Start a new round (clears votes, resets state to Created, generates new code).
    pub async fn start_new_round(&self, ctx: &impl AppContext) -> DbResult<PartyCode> {
        ctx.db().start_new_round(self.id).await
    }

    // ========================================================================
    // Query Helpers
    // ========================================================================

    /// Check if a user is a member of this party.
    pub async fn is_member(&self, ctx: &impl AppContext, user_id: Uuid) -> DbResult<bool> {
        ctx.db().is_party_member(self.id, user_id).await
    }

    /// Check if all members are ready.
    pub async fn are_all_ready(&self, ctx: &impl AppContext) -> DbResult<bool> {
        ctx.db().are_all_members_ready(self.id).await
    }

    /// Get ready status (ready_count, total_count).
    pub async fn ready_status(&self, ctx: &impl AppContext) -> DbResult<(i64, i64)> {
        ctx.db().get_ready_status(self.id).await
    }

    /// Reset all members' ready states to false.
    pub async fn reset_ready_states(&self, ctx: &impl AppContext) -> DbResult<()> {
        ctx.db().reset_all_ready_states(self.id).await?;
        Ok(())
    }

    /// Enable voting for the party.
    pub async fn enable_voting(&self, ctx: &impl AppContext) -> DbResult<()> {
        ctx.db().enable_voting(self.id).await
    }

    /// Disable voting for the party.
    pub async fn disable_voting(&self, ctx: &impl AppContext) -> DbResult<()> {
        ctx.db().disable_voting(self.id).await
    }

    /// Clear all shown movies and ballots for the party.
    pub async fn clear_ballots(&self, ctx: &impl AppContext) -> DbResult<()> {
        ctx.db().clear_shown_movies_for_party(self.id).await
    }

    /// Build initial voting ballots for all members.
    pub async fn build_voting_ballots(&self, ctx: &impl AppContext) -> DbResult<()> {
        ctx.db().build_voting_ballots(self.id).await
    }

    /// Build round 2 ballots for the top 3 movies.
    pub async fn build_round2_ballots(&self, ctx: &impl AppContext, top3: &[i64]) -> DbResult<()> {
        ctx.db().build_round2_ballots(self.id, top3).await
    }

    /// Record a movie pick for a user in this party session.
    pub async fn add_pick(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
        movie_id: i64,
        liked: Option<bool>,
    ) -> DbResult<()> {
        ctx.db()
            .add_party_pick(user_id, self.id, movie_id, liked)
            .await?;
        Ok(())
    }

    /// Get a user's current voting ballot (movie IDs).
    pub async fn get_ballot(&self, ctx: &impl AppContext, user_id: Uuid) -> DbResult<Vec<i64>> {
        ctx.db().get_user_ballot(self.id, user_id).await
    }

    /// Check if a user can vote for a specific movie in this party.
    pub async fn can_user_vote(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
        movie_id: i64,
    ) -> DbResult<bool> {
        ctx.db().can_vote(self.id, user_id, movie_id).await
    }

    /// Cast a vote for a movie in this party.
    pub async fn cast_vote(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
        movie_id: i64,
        like: bool,
    ) -> DbResult<()> {
        ctx.db()
            .cast_vote(self.id, user_id, movie_id, like)
            .await
            .map(|_| ())
    }

    /// Get total likes/dislikes for a movie within this party.
    pub async fn get_movie_vote_totals(
        &self,
        ctx: &impl AppContext,
        movie_id: i64,
    ) -> DbResult<(i64, i64)> {
        ctx.db().get_vote_totals(movie_id, Some(self.id)).await
    }

    /// Add movies to be shown to a user in this party (ballot building).
    pub async fn add_shown_movies(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
        movie_ids: &[i64],
    ) -> DbResult<()> {
        ctx.db().add_shown_movies(self.id, user_id, movie_ids).await
    }

    /// Get all movie picks made by all users in this party.
    /// Returns (user_id, movie_id, liked).
    pub async fn get_picks(
        &self,
        ctx: &impl AppContext,
    ) -> DbResult<Vec<(Uuid, i64, Option<bool>)>> {
        ctx.db().get_party_picks(self.id).await
    }

    /// Get movie picks for a specific user in this party.
    pub async fn get_user_picks(&self, ctx: &impl AppContext, user_id: Uuid) -> DbResult<Vec<i64>> {
        ctx.db().get_user_party_picks(self.id, user_id).await
    }

    /// Remove a movie pick for a user in this party.
    pub async fn remove_pick(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
        movie_id: i64,
    ) -> DbResult<()> {
        ctx.db().remove_party_pick(user_id, self.id, movie_id).await
    }

    // ========================================================================
    // Static Queries - Return Party handles or raw IDs for batch processing
    // ========================================================================

    /// Get all parties that are in timed phases (Voting, Watching) with their entry time.
    pub async fn get_in_timed_phases(
        db: Arc<Database>,
    ) -> DbResult<Vec<(Uuid, PartyState, DateTime<Utc>)>> {
        db.get_parties_in_timed_phases().await
    }

    /// Get all parties in specific phases where ALL members are ready.
    pub async fn get_all_ready_in_phases(
        db: Arc<Database>,
        phases: &[PartyState],
    ) -> DbResult<Vec<Uuid>> {
        db.get_parties_all_ready_in_phases(phases).await
    }
}
