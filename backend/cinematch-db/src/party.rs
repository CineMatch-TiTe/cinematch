//! Party database operations

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use rand::Rng;
use uuid::Uuid;

use crate::models::{
    NewParty, NewPartyCode, NewPartyMember, Party, PartyCode, PartyMember, PartyState, User,
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
    pub async fn create_party(&self, leader_id: Uuid) -> DbResult<(Party, PartyCode)> {
        use schema::parties;

        let new_party = NewParty {
            party_leader_id: leader_id,
            state: PartyState::Created,
            can_vote: false,
        };

        let mut conn = self.conn().await?;

        // Insert the party
        let party: Party = diesel::insert_into(parties::table)
            .values(&new_party)
            .returning(Party::as_returning())
            .get_result(&mut conn)
            .await?;

        // Generate a unique code with retries
        let code = self
            .generate_party_code_internal(&mut conn, party.id)
            .await?;

        Ok((party, code))
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
    pub async fn get_party(&self, party_id: Uuid) -> DbResult<Party> {
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
    pub async fn transfer_party_leadership(
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

    /// Delete (disband) a party
    /// # Safety
    /// This function is unsafe because it may leave orphaned records or inconsistent state if not all related data is cleaned up.
    /// Caller must ensure all necessary cleanup is performed after party deletion.
    pub async unsafe fn delete_party(&self, party_id: Uuid) -> DbResult<usize> {
        use schema::parties::dsl::*;

        let mut conn = self.conn().await?;
        diesel::delete(parties.find(party_id))
            .execute(&mut conn)
            .await
            .map_err(DbError::from)
    }

    /// Get the party code for a party (returns None if code released)
    pub async fn get_party_code(&self, pid: Uuid) -> DbResult<Option<PartyCode>> {
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
    pub async fn get_party_by_code(&self, join_code: &str) -> DbResult<Option<Party>> {
        use schema::{parties, party_codes};

        let mut conn = self.conn().await?;
        party_codes::table
            .inner_join(parties::table)
            .filter(party_codes::code.eq(join_code.to_uppercase()))
            .select(Party::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(DbError::from)
    }

    /// Delete a party's join code (e.g., when party moves past Created state)
    pub async fn release_party_code(&self, pid: Uuid) -> DbResult<usize> {
        use schema::party_codes::dsl::*;

        let mut conn = self.conn().await?;
        diesel::delete(party_codes.filter(party_id.eq(pid)))
            .execute(&mut conn)
            .await
            .map_err(DbError::from)
    }

    /// Generate a new party code (used when starting a new round from Review state)
    pub async fn regenerate_party_code(&self, pid: Uuid) -> DbResult<PartyCode> {
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
    /// Set the party's state and reset all members' ready states
    pub async fn set_phase(&self, party_id: Uuid, new_state: PartyState) -> DbResult<Party> {
        use schema::parties::dsl::*;

        // Update state
        let mut conn = self.conn().await?;
        let updated_party = diesel::update(parties.find(party_id))
            .set(state.eq(new_state))
            .returning(Party::as_returning())
            .get_result(&mut conn)
            .await?;

        // Reset all members' ready state
        self.reset_all_ready_states(party_id).await?;

        Ok(updated_party)
    }

    pub async fn get_state(&self, party_id: Uuid) -> DbResult<PartyState> {
        use schema::parties::dsl::*;

        let mut conn = self.conn().await?;
        let party_state = parties
            .filter(id.eq(party_id))
            .select(state)
            .first::<PartyState>(&mut conn)
            .await?;

        Ok(party_state)
    }

    /// Start a new movie round
    /// Resets to Created state with a new join code, keeps existing members
    pub async fn start_new_round(&self, party_id: Uuid) -> DbResult<PartyCode> {
        // Reset to Created state
        let mut conn = self.conn().await?;

        // Generate new join code
        let code = self
            .generate_party_code_internal(&mut conn, party_id)
            .await?;

        // Reset all members' ready state
        self.reset_all_ready_states(party_id).await?;

        Ok(code)
    }

    /// Disband a party (leader only)
    pub async fn disband_party(&self, party_id: Uuid) -> DbResult<Party> {
        use schema::parties::dsl::*;

        // Release code if exists
        let _ = self.release_party_code(party_id).await;

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
    pub async fn add_party_member(&self, party_id: Uuid, user_id: Uuid) -> DbResult<PartyMember> {
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
    pub async fn remove_party_member(&self, pid: Uuid, uid: Uuid) -> DbResult<()> {
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

    /// Get all party user records, for names etc.
    pub async fn get_party_users(&self, pid: Uuid) -> DbResult<Vec<User>> {
        use schema::party_members::dsl::*;

        let mut conn = self.conn().await?;
        party_members
            .inner_join(crate::schema::users::table)
            .filter(party_id.eq(pid))
            .select(User::as_select())
            .load(&mut conn)
            .await
            .map_err(DbError::from)
    }

    /// Check if a user is a member of a party
    pub async fn is_party_member(&self, pid: Uuid, uid: Uuid) -> DbResult<bool> {
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
    pub async fn get_party_member(&self, pid: Uuid, uid: Uuid) -> DbResult<Option<PartyMember>> {
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

    /// Get the oldest member of a party (by joined_at)
    pub async fn get_oldest_party_member(&self, pid: Uuid) -> DbResult<Option<PartyMember>> {
        use schema::party_members::dsl::*;

        let mut conn = self.conn().await?;
        party_members
            .filter(party_id.eq(pid))
            .order(joined_at.asc())
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
    pub async fn set_member_ready(&self, pid: Uuid, uid: Uuid, ready: bool) -> DbResult<()> {
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
    pub async fn reset_all_ready_states(&self, pid: Uuid) -> DbResult<usize> {
        use schema::party_members::dsl::*;

        let mut conn = self.conn().await?;
        diesel::update(party_members.filter(party_id.eq(pid)))
            .set(is_ready.eq(false))
            .execute(&mut conn)
            .await
            .map_err(DbError::from)
    }

    /// Check if all members are ready
    pub async fn are_all_members_ready(&self, pid: Uuid) -> DbResult<bool> {
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
    pub async fn get_ready_status(&self, pid: Uuid) -> DbResult<(i64, i64)> {
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
}
