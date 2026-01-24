use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::{external_accounts, parties, party_codes, party_members, users};

// ============================================================================
// Enums (mapped to PostgreSQL ENUMs)
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
    Created,   // Initial state, people can join and start picking
    Picking, // people can pick movies (taste), people cant join anymore (code is freed, and party is identified by uuid)
    Voting, // people can vote on picked movies, until a movie is decided, we can go to picking or watching from here
    Watching, // movie is being watched, people can start review after 15 minutes, and can update their review
    Review,   // 90% of runtime passed start showin results, leader can also skip
    Disbanded, // dead party, keep for history
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::AuthProvider"]
pub enum AuthProvider {
    Google,
    Github,
    Discord,
}

// ============================================================================
// User Models
// ============================================================================

/// Queryable User from database
#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub oneshot: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// For inserting a new user
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub oneshot: bool,
}

/// For updating a user
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = users)]
pub struct UpdateUser<'a> {
    pub username: Option<&'a str>,
    pub oneshot: Option<bool>,
}

// ============================================================================
// External Account Models
// ============================================================================

/// Queryable ExternalAccount from database
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = external_accounts)]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ExternalAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: AuthProvider,
    pub provider_user_id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// For inserting a new external account
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = external_accounts)]
pub struct NewExternalAccount<'a> {
    pub user_id: Uuid,
    pub provider: AuthProvider,
    pub provider_user_id: &'a str,
    pub email: Option<&'a str>,
    pub display_name: Option<&'a str>,
}

/// For updating an external account
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = external_accounts)]
pub struct UpdateExternalAccount<'a> {
    pub email: Option<&'a str>,
    pub display_name: Option<&'a str>,
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
}

/// For inserting a new party
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = parties)]
pub struct NewParty {
    pub party_leader_id: Uuid,
    pub state: PartyState,
}

/// For updating a party
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = parties)]
pub struct UpdateParty {
    pub party_leader_id: Option<Uuid>,
    pub state: Option<PartyState>,
    pub updated_at: Option<DateTime<Utc>>,
    pub disbanded_at: Option<DateTime<Utc>>,
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
