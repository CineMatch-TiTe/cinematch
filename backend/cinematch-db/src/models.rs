// Minimal models for movie metadata tables for ergonomic queries
use crate::schema::{cast_members, directors, genres, keywords, production_countries, trailers, shown_movies, votes};

// Re-export movie/vector models for easy access from crate root
pub use crate::vector::models::{CastMember, MovieData};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::{
    external_accounts, movies, parties, party_codes, party_members, prefs_exclude_genre,
    prefs_include_genre, user_preferences, users,
};

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
    pub selected_movie_id: Option<i64>,
    pub can_vote: bool,
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

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = directors)]
#[diesel(primary_key(director_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Director {
    pub director_id: uuid::Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = genres)]
#[diesel(primary_key(genre_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Genre {
    pub genre_id: uuid::Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = keywords)]
#[diesel(primary_key(keyword_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Keyword {
    pub keyword_id: uuid::Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = cast_members)]
#[diesel(primary_key(cast_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CastMemberRow {
    pub cast_id: uuid::Uuid,
    pub name: String,
    pub profile_url: Option<String>,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = production_countries)]
#[diesel(primary_key(country_code))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProductionCountry {
    pub country_code: String,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = trailers)]
#[diesel(primary_key(trailer_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Trailer {
    pub trailer_id: uuid::Uuid,
    pub video_key: String,
}

// ============================================================================
// Movie Models
// ============================================================================

/// Queryable Movie from database
#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = movies)]
#[diesel(primary_key(movie_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Movie {
    pub movie_id: i64,
    pub title: String,
    pub runtime: i32,
    pub popularity: f32,
    pub imdb_id: Option<String>,
    pub mediawiki_id: Option<String>,
    pub rating: Option<String>,
    pub release_date: chrono::NaiveDateTime,
    pub original_language: Option<String>,
    pub poster_url: Option<String>,
    pub overview: Option<String>,
    pub tagline: Option<String>,
    pub release_year: Option<i32>,
}

// ============================================================================
// User Preferences Models
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = user_preferences)]
#[diesel(primary_key(user_id))]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserPreferences {
    pub user_id: Uuid,
    pub target_release_year: Option<i32>,
    pub release_year_flex: i32,
    pub is_tite: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = user_preferences)]
pub struct NewUserPreferences {
    pub user_id: Uuid,
    pub target_release_year: Option<i32>,
    pub release_year_flex: i32,
    pub is_tite: bool,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = user_preferences)]
pub struct UpdateUserPreferences {
    pub target_release_year: Option<Option<i32>>,
    pub release_year_flex: Option<i32>,
    pub is_tite: Option<bool>,
}

// Join table models for included genres
#[derive(Debug, Clone, Queryable, Identifiable, Associations)]
#[diesel(table_name = prefs_include_genre)]
#[diesel(primary_key(user_id, genre_id))]
#[diesel(belongs_to(User, foreign_key = user_id))]
#[diesel(belongs_to(Genre, foreign_key = genre_id))]
pub struct PrefsIncludeGenre {
    pub user_id: Uuid,
    pub genre_id: Uuid,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = prefs_include_genre)]
pub struct NewPrefsIncludeGenre {
    pub user_id: Uuid,
    pub genre_id: Uuid,
}

// Join table models for excluded genres
#[derive(Debug, Clone, Queryable, Identifiable, Associations)]
#[diesel(table_name = prefs_exclude_genre)]
#[diesel(primary_key(user_id, genre_id))]
#[diesel(belongs_to(User, foreign_key = user_id))]
#[diesel(belongs_to(Genre, foreign_key = genre_id))]
pub struct PrefsExcludeGenre {
    pub user_id: Uuid,
    pub genre_id: Uuid,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = prefs_exclude_genre)]
pub struct NewPrefsExcludeGenre {
    pub user_id: Uuid,
    pub genre_id: Uuid,
}

#[derive(Debug, Clone, Queryable, Identifiable, Associations)]
#[diesel(table_name = shown_movies)]
#[diesel(primary_key(party_id, user_id, movie_id))]
#[diesel(belongs_to(crate::models::Party, foreign_key = party_id))]
#[diesel(belongs_to(crate::models::User, foreign_key = user_id))]
#[diesel(belongs_to(crate::models::Movie, foreign_key = movie_id))]
pub struct ShownMovie {
    pub party_id: Uuid,
    pub user_id: Uuid,
    pub movie_id: i64,
    pub shown_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = shown_movies)]
pub struct NewShownMovie {
    pub party_id: Uuid,
    pub user_id: Uuid,
    pub movie_id: i64,
}

#[derive(Debug, Clone, Queryable, Identifiable, Associations)]
#[diesel(table_name = votes)]
#[diesel(primary_key(party_id, user_id, movie_id))]
#[diesel(belongs_to(crate::models::Party, foreign_key = party_id))]
#[diesel(belongs_to(crate::models::User, foreign_key = user_id))]
#[diesel(belongs_to(crate::models::Movie, foreign_key = movie_id))]
pub struct Vote {
    pub party_id: Uuid,
    pub user_id: Uuid,
    pub movie_id: i64,
    pub vote_value: bool,
    pub voted_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = votes)]
pub struct NewVote {
    pub party_id: Uuid,
    pub user_id: Uuid,
    pub movie_id: i64,
    pub vote_value: bool,
}
