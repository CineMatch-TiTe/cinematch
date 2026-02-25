//! Party database operations (CRUD + state machine).

use chrono::Utc;
use diesel::prelude::*;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use rand::RngExt;
use uuid::Uuid;

use super::models::{
    NewParty, NewPartyCode, NewPartyMember, Party, PartyCode, PartyMember, PartyState,
};

use crate::schema;
use crate::{Database, DbError, DbResult};

/// Characters allowed in party codes: A-Z, 0-9
const CODE_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const CODE_LENGTH: usize = 4;
const MAX_CODE_ATTEMPTS: usize = 100;

/// Generate a random 4-character party code
fn generate_code() -> String {
    let mut rng = rand::rng();
    (0..CODE_LENGTH)
        .map(|_| {
            let idx = rng.random_range(0..CODE_CHARS.len());
            CODE_CHARS[idx] as char
        })
        .collect()
}

// ============================================================================
// Party Operations
// ============================================================================

impl Database {
    /// Create a new party with a leader and generate a join code
    /// Returns the party and its join code
    pub(crate) async fn create_party(&self, leader_id: Uuid) -> DbResult<(Party, PartyCode)> {
        use schema::{parties, party_members};

        let new_party = NewParty {
            party_leader_id: leader_id,
            state: PartyState::Created,
            can_vote: false,
        };

        let mut conn = self.conn().await?;

        conn.transaction::<(Party, PartyCode), DbError, _>(|conn| {
            async move {
                // Insert the party
                let party: Party = diesel::insert_into(parties::table)
                    .values(&new_party)
                    .returning(Party::as_returning())
                    .get_result(conn)
                    .await?;

                // Generate a unique code with retries
                let code = self.generate_party_code_internal(conn, party.id).await?;

                // Add the leader as the first member
                let new_member = NewPartyMember {
                    user_id: leader_id,
                    party_id: party.id,
                };
                diesel::insert_into(party_members::table)
                    .values(&new_member)
                    .execute(conn)
                    .await?;

                Ok((party, code))
            }
            .scope_boxed()
        })
        .await
    }

    /// Internal helper to generate a unique party code
    async fn generate_party_code_internal(
        &self,
        conn: &mut diesel_async::AsyncPgConnection,
        pid: Uuid,
    ) -> DbResult<PartyCode> {
        use schema::party_codes;

        let mut attempts = 0;
        loop {
            let candidate = generate_code();

            let new_code = NewPartyCode {
                code: &candidate,
                party_id: pid,
            };

            match diesel::insert_into(party_codes::table)
                .values(&new_code)
                .returning(PartyCode::as_returning())
                .get_result::<PartyCode>(conn)
                .await
            {
                Ok(code) => return Ok(code),
                Err(diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                )) => {
                    // Code collision, try again
                    attempts += 1;
                    if attempts >= MAX_CODE_ATTEMPTS {
                        return Err(DbError::CodeGenerationFailed);
                    }
                }
                Err(e) => return Err(DbError::from(e)),
            }
        }
    }

    /// Get a party by ID
    pub(crate) async fn get_party(&self, party_id: Uuid) -> DbResult<Party> {
        use schema::parties::dsl::*;

        let mut conn = self.conn().await?;
        parties
            .find(party_id)
            .select(Party::as_select())
            .first(&mut conn)
            .await
            .optional()?
            .ok_or(DbError::PartyNotFound(party_id))
    }

    /// Transfer party leadership
    pub(crate) async fn transfer_party_leadership(
        &self,
        party_id: Uuid,
        new_leader_id: Uuid,
    ) -> DbResult<Party> {
        use schema::parties::dsl::*;

        let mut conn = self.conn().await?;
        diesel::update(parties.find(party_id))
            .set(party_leader_id.eq(new_leader_id))
            .returning(Party::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(DbError::from)
    }

    /// Get the party code for a party (returns None if code released)
    pub(crate) async fn get_party_code(&self, pid: Uuid) -> DbResult<Option<PartyCode>> {
        use schema::party_codes::dsl::*;

        let mut conn = self.conn().await?;
        party_codes
            .filter(party_id.eq(pid))
            .select(PartyCode::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(DbError::from)
    }

    /// Get a party by its join code (for joining)
    pub(crate) async fn get_party_by_code(&self, join_code: &str) -> DbResult<Option<Party>> {
        use schema::{parties, party_codes};
        let mut conn = self.conn().await?;

        let result = parties::table
            .inner_join(party_codes::table)
            .filter(party_codes::code.eq(join_code))
            .filter(
                parties::state
                    .eq(PartyState::Created)
                    .or(parties::state.eq(PartyState::Picking)),
            )
            .select(Party::as_returning())
            .get_result(&mut conn)
            .await
            .optional()?;

        Ok(result)
    }

    /// Delete a party's join code (e.g., when party moves past Created state)
    pub(crate) async fn release_party_code(&self, pid: Uuid) -> DbResult<usize> {
        use schema::party_codes::dsl::*;

        let mut conn = self.conn().await?;
        diesel::delete(party_codes.filter(party_id.eq(pid)))
            .execute(&mut conn)
            .await
            .map_err(DbError::from)
    }

    /// Generate a new party code (used when starting a new round from Review state)
    pub(crate) async fn regenerate_party_code(&self, pid: Uuid) -> DbResult<PartyCode> {
        // First release any existing code
        let _ = self.release_party_code(pid).await;

        // Generate new code
        let mut conn = self.conn().await?;
        self.generate_party_code_internal(&mut conn, pid).await
    }
}

// ============================================================================
// Party State Transitions (Backend Logic)
// ============================================================================

impl Database {
    /// Set the party's state, phase_entered_at = now(), and reset all members' ready states.
    pub(crate) async fn set_phase(&self, party_id: Uuid, new_state: PartyState) -> DbResult<Party> {
        use schema::parties::dsl::*;

        let now = Utc::now();
        let mut conn = self.conn().await?;
        let updated_party = diesel::update(parties.find(party_id))
            .set((state.eq(new_state), phase_entered_at.eq(now)))
            .returning(Party::as_returning())
            .get_result(&mut conn)
            .await?;

        self.reset_all_ready_states(party_id).await?;

        Ok(updated_party)
    }

    /// Set phase_entered_at to now() (e.g. when starting round 2, still in Voting).
    pub(crate) async fn set_phase_entered_at_now(&self, party_id: Uuid) -> DbResult<()> {
        use schema::parties::dsl::*;

        let now = Utc::now();
        let mut conn = self.conn().await?;
        diesel::update(parties.find(party_id))
            .set(phase_entered_at.eq(now))
            .execute(&mut conn)
            .await
            .map_err(DbError::from)?;
        Ok(())
    }

    pub(crate) async fn get_state(&self, party_id: Uuid) -> DbResult<PartyState> {
        use schema::parties::dsl::*;

        let mut conn = self.conn().await?;
        let party_state = parties
            .filter(id.eq(party_id))
            .select(state)
            .first::<PartyState>(&mut conn)
            .await?;

        Ok(party_state)
    }

    pub(crate) async fn get_voting_round(&self, party_id: Uuid) -> DbResult<Option<i16>> {
        use schema::parties::dsl::*;

        let mut conn = self.conn().await?;
        let round = parties
            .filter(id.eq(party_id))
            .select(voting_round)
            .first::<Option<i16>>(&mut conn)
            .await?;
        Ok(round)
    }

    pub(crate) async fn set_voting_round(
        &self,
        party_id: Uuid,
        round: Option<i16>,
    ) -> DbResult<()> {
        use schema::parties::dsl::*;

        let mut conn = self.conn().await?;
        diesel::update(parties.find(party_id))
            .set(voting_round.eq(round))
            .execute(&mut conn)
            .await
            .map_err(DbError::from)?;
        Ok(())
    }

    pub(crate) async fn set_selected_movie_id(
        &self,
        party_id: Uuid,
        movie_id: Option<i64>,
    ) -> DbResult<()> {
        use schema::parties::dsl::*;

        let mut conn = self.conn().await?;
        diesel::update(parties.find(party_id))
            .set(selected_movie_id.eq(movie_id))
            .execute(&mut conn)
            .await
            .map_err(DbError::from)?;
        Ok(())
    }

    /// Start a new movie round from Review.
    /// Clears votes/shown_movies/selected_movie_id/voting_round, sets state to Created, new join code, resets ready. Keeps party tastes.
    pub(crate) async fn start_new_round(&self, party_id: Uuid) -> DbResult<PartyCode> {
        self.clear_shown_movies_for_party(party_id).await?;
        self.set_selected_movie_id(party_id, None).await?;
        self.set_voting_round(party_id, None).await?;
        self.set_phase(party_id, PartyState::Created).await?;
        let code = self.regenerate_party_code(party_id).await?;
        Ok(code)
    }

    /// Disband a party (leader only)
    pub(crate) async fn disband_party(&self, party_id: Uuid) -> DbResult<Party> {
        use schema::parties::dsl::*;

        // Release code if exists
        let _ = self.release_party_code(party_id).await;

        // kick all members
        let members = self.get_party_members(party_id).await?;
        for member in members {
            self.remove_party_member(party_id, member.user_id).await?;
        }

        // Set state to disbanded
        let mut conn = self.conn().await?;
        diesel::update(parties.find(party_id))
            .set(state.eq(PartyState::Disbanded))
            .returning(Party::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(DbError::from)
    }
}

// ============================================================================
// Party Member Operations
// ============================================================================

impl Database {
    /// Add a user to a party
    pub(crate) async fn add_party_member(
        &self,
        party_id: Uuid,
        user_id: Uuid,
    ) -> DbResult<PartyMember> {
        use schema::party_members;

        let new_member = NewPartyMember { user_id, party_id };

        let mut conn = self.conn().await?;
        diesel::insert_into(party_members::table)
            .values(&new_member)
            .returning(PartyMember::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(DbError::from)
    }

    /// Remove a user from a party
    pub(crate) async fn remove_party_member(&self, pid: Uuid, uid: Uuid) -> DbResult<()> {
        use schema::party_members::dsl::*;

        let mut conn = self.conn().await?;
        diesel::delete(
            party_members
                .filter(party_id.eq(pid))
                .filter(user_id.eq(uid)),
        )
        .execute(&mut conn)
        .await
        .map_err(DbError::from)
        .map(|_| ())
    }

    /// Get all party member records (includes joined_at and is_ready)
    pub async fn get_party_members(&self, pid: Uuid) -> DbResult<Vec<PartyMember>> {
        use schema::party_members::dsl::*;

        let mut conn = self.conn().await?;
        party_members
            .filter(party_id.eq(pid))
            .select(PartyMember::as_select())
            .load(&mut conn)
            .await
            .map_err(DbError::from)
    }

    /// Check if a user is a member of a party
    pub(crate) async fn is_party_member(&self, pid: Uuid, uid: Uuid) -> DbResult<bool> {
        use schema::party_members::dsl::*;

        let mut conn = self.conn().await?;
        let exists = party_members
            .filter(party_id.eq(pid))
            .filter(user_id.eq(uid))
            .select(PartyMember::as_select())
            .first(&mut conn)
            .await
            .optional()?;
        Ok(exists.is_some())
    }

    /// Get a specific party member record
    pub(crate) async fn get_party_member(
        &self,
        pid: Uuid,
        uid: Uuid,
    ) -> DbResult<Option<PartyMember>> {
        use schema::party_members::dsl::*;
        let mut conn = self.conn().await?;
        party_members
            .filter(party_id.eq(pid))
            .filter(user_id.eq(uid))
            .select(PartyMember::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(DbError::from)
    }
}

// ============================================================================
// Ready State Operations
// ============================================================================

impl Database {
    /// Set a member's ready state explicitly
    pub(crate) async fn set_member_ready(&self, pid: Uuid, uid: Uuid, ready: bool) -> DbResult<()> {
        use schema::party_members::dsl::*;

        let mut conn = self.conn().await?;
        let rows = diesel::update(
            party_members
                .filter(party_id.eq(pid))
                .filter(user_id.eq(uid)),
        )
        .set(is_ready.eq(ready))
        .execute(&mut conn)
        .await?;

        if rows == 0 {
            return Err(DbError::NotPartyMember);
        }

        Ok(())
    }

    /// Reset all members' ready state to false (called on state transitions)
    pub(crate) async fn reset_all_ready_states(&self, pid: Uuid) -> DbResult<usize> {
        use schema::party_members::dsl::*;

        let mut conn = self.conn().await?;
        diesel::update(party_members.filter(party_id.eq(pid)))
            .set(is_ready.eq(false))
            .execute(&mut conn)
            .await
            .map_err(DbError::from)
    }

    /// Check if all members are ready
    pub(crate) async fn are_all_members_ready(&self, pid: Uuid) -> DbResult<bool> {
        use schema::party_members::dsl::*;

        let mut conn = self.conn().await?;

        // Count total members
        let total: i64 = party_members
            .filter(party_id.eq(pid))
            .count()
            .get_result(&mut conn)
            .await?;

        if total == 0 {
            return Ok(false);
        }

        // Count ready members
        let ready_count: i64 = party_members
            .filter(party_id.eq(pid))
            .filter(is_ready.eq(true))
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(ready_count == total)
    }

    /// Get ready status summary for a party
    pub(crate) async fn get_ready_status(&self, pid: Uuid) -> DbResult<(i64, i64)> {
        use schema::party_members::dsl::*;

        let mut conn = self.conn().await?;

        let total: i64 = party_members
            .filter(party_id.eq(pid))
            .count()
            .get_result(&mut conn)
            .await?;

        let ready_count: i64 = party_members
            .filter(party_id.eq(pid))
            .filter(is_ready.eq(true))
            .count()
            .get_result(&mut conn)
            .await?;

        Ok((ready_count, total))
    }

    /// Get all parties in Voting or Watching phase with their phase_entered_at (for timeout reschedule on startup).
    pub(crate) async fn get_parties_in_timed_phases(
        &self,
    ) -> DbResult<Vec<(Uuid, PartyState, chrono::DateTime<Utc>)>> {
        use schema::parties::dsl::*;

        let mut conn = self.conn().await?;
        let results = parties
            .filter(state.eq_any(&[PartyState::Voting, PartyState::Watching]))
            .select((id, state, phase_entered_at))
            .load::<(Uuid, PartyState, chrono::DateTime<Utc>)>(&mut conn)
            .await?;
        Ok(results)
    }

    /// Get parties in specified phases where all members are ready (for ready countdown reschedule on startup).
    pub(crate) async fn get_parties_all_ready_in_phases(
        &self,
        phases: &[PartyState],
    ) -> DbResult<Vec<Uuid>> {
        use schema::{parties, party_members};

        let mut conn = self.conn().await?;

        // Get parties in the given phases
        let candidate_parties: Vec<Uuid> = parties::table
            .filter(parties::state.eq_any(phases))
            .select(parties::id)
            .load(&mut conn)
            .await?;

        let mut all_ready_parties = Vec::new();

        for pid in candidate_parties {
            // Check if this party has all members ready
            let total: i64 = party_members::table
                .filter(party_members::party_id.eq(pid))
                .count()
                .get_result(&mut conn)
                .await?;

            if total == 0 {
                continue; // Empty party, skip
            }

            let not_ready: i64 = party_members::table
                .filter(party_members::party_id.eq(pid))
                .filter(party_members::is_ready.eq(false))
                .count()
                .get_result(&mut conn)
                .await?;

            if not_ready == 0 {
                all_ready_parties.push(pid);
            }
        }

        Ok(all_ready_parties)
    }
}
