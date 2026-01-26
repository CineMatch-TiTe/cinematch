pub mod handlers;
pub mod pref;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ============================================================================
// User Responses
// ============================================================================

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GuestUserRequest {
    /// Desired username for the guest user (optional)
    pub username: Option<String>,
}

/// Response when creating a guest user
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GuestLoginResponse {
    /// The newly created user ID
    pub user_id: Uuid,
    /// The username
    pub username: String,
}

/// Response with user details
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserResponse {
    /// The user's unique ID
    pub user_id: Uuid,
    /// The user's display name
    pub username: String,
    /// Whether this is a guest/oneshot user
    pub is_guest: bool,
    /// When the user was created
    pub created_at: DateTime<Utc>,
    /// When the user was last updated
    pub updated_at: DateTime<Utc>,
}

/// Response with current user info and token details
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CurrentUserResponse {
    /// The user's unique ID
    pub user_id: Uuid,
    /// The user's display name
    pub username: String,
    /// Whether this is a guest/oneshot user
    pub is_guest: bool,
    /// When the user was created
    pub created_at: DateTime<Utc>,
    /// When the user was last updated
    pub updated_at: DateTime<Utc>,
    /// Token expiry timestamp (Unix seconds)
    pub token_expires_at: i64,
    /// Seconds until token expiry
    pub token_expires_in: i64,
}

// ============================================================================
// User Request Models
// ============================================================================

/// Request to rename a user
#[derive(Debug, Deserialize, ToSchema)]
pub struct RenameUserRequest {
    /// The new username (max 32 characters)
    #[schema(example = "NewUsername")]
    pub new_username: String,
}

/// Request to update user taste for a movie
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTasteRequest {
    /// Whether the user liked the movie (true = like, false = dislike), if missing, the movie is skipped
    pub liked: Option<bool>,
}

/// User-facing update struct: genre names instead of UUIDs
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateUserPreferencesRequest {
    #[schema(example = json!(["Action", "Drama"]))]
    pub include_genres: Option<Vec<String>>,
    #[schema(example = json!(["Horror", "Comedy"]))]
    pub exclude_genres: Option<Vec<String>>,
    #[schema(example = json!(2020))]
    pub target_release_year: Option<Option<i32>>,
    #[schema(example = json!(2))]
    pub release_year_flex: Option<i32>,
    #[schema(example = json!(true))]
    pub is_tite: Option<bool>,
}

/// API-facing user preferences response (genre names, not IDs)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserPreferencesResponse {
    pub include_genres: Vec<String>,
    pub exclude_genres: Vec<String>,
    pub target_release_year: Option<i32>,
    pub release_year_flex: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
