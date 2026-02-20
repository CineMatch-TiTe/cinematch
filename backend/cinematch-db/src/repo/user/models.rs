//! User-related database models.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::{external_accounts, users};

// ============================================================================
// Enums
// ============================================================================

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
// User Preferences Models
// ============================================================================

use crate::schema::{prefs_exclude_genre, prefs_include_genre, user_preferences};

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
