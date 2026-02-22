//! Party-related database models.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::repo::user::models::User;
use crate::schema::{parties, party_codes, party_members};

// ============================================================================
// Enums
// ============================================================================

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    diesel_derive_enum::DbEnum,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
#[ExistingTypePath = "crate::schema::sql_types::PartyState"]
pub enum PartyState {
    Created,
    Picking,
    Voting,
    Watching,
    Review,
    Disbanded,
}

impl From<PartyState> for cinematch_common::models::PartyState {
    fn from(s: PartyState) -> Self {
        match s {
            PartyState::Created => cinematch_common::models::PartyState::Created,
            PartyState::Picking => cinematch_common::models::PartyState::Picking,
            PartyState::Voting => cinematch_common::models::PartyState::Voting,
            PartyState::Watching => cinematch_common::models::PartyState::Watching,
            PartyState::Review => cinematch_common::models::PartyState::Review,
            PartyState::Disbanded => cinematch_common::models::PartyState::Disbanded,
        }
    }
}

// ============================================================================
// Party Models
// ============================================================================

/// Queryable Party from database
#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = parties)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Party {
    pub id: Uuid,
    pub party_leader_id: Uuid,
    pub state: PartyState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub disbanded_at: Option<DateTime<Utc>>,
    pub selected_movie_id: Option<i64>,
    pub can_vote: bool,
    pub voting_round: Option<i16>,
    pub phase_entered_at: DateTime<Utc>,
}

/// For inserting a new party
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = parties)]
pub struct NewParty {
    pub party_leader_id: Uuid,
    pub state: PartyState,
    pub can_vote: bool,
}

/// For updating a party
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = parties)]
pub struct UpdateParty {
    pub party_leader_id: Option<Uuid>,
    pub state: Option<PartyState>,
    pub can_vote: Option<bool>,
    pub updated_at: Option<DateTime<Utc>>,
    pub disbanded_at: Option<DateTime<Utc>>,
    pub selected_movie_id: Option<Option<i64>>,
    pub voting_round: Option<Option<i16>>,
    pub phase_entered_at: Option<DateTime<Utc>>,
}

// ============================================================================
// Party Member Models (Junction Table)
// ============================================================================

/// Queryable PartyMember from database
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = party_members)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Party))]
#[diesel(primary_key(user_id, party_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PartyMember {
    pub user_id: Uuid,
    pub party_id: Uuid,
    pub joined_at: DateTime<Utc>,
    pub is_ready: bool,
}

/// For inserting a new party member
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = party_members)]
pub struct NewPartyMember {
    pub user_id: Uuid,
    pub party_id: Uuid,
}

// ============================================================================
// Party Code Models
// ============================================================================

/// Queryable PartyCode from database
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = party_codes)]
#[diesel(belongs_to(Party))]
#[diesel(primary_key(code))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PartyCode {
    pub code: String,
    pub party_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// For inserting a new party code
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = party_codes)]
pub struct NewPartyCode<'a> {
    pub code: &'a str,
    pub party_id: Uuid,
}
