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
    /// Short-lived JWT for API access (1h)
    pub jwt: String,
    /// Token expiry timestamp (Unix seconds)
    pub token_expires_at: i64,
    /// Seconds until token expiry
    pub token_expires_in: i64,
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
    /// This is optional because we can only get if user logged in via JWT; cookie-authenticated users won't have this field.
    pub token_expires_at: Option<i64>,
    /// Seconds until token expiry
    /// This is optional because we can only get if user logged in via JWT; cookie-authenticated users won't have this field.
    pub token_expires_in: Option<i64>,
}

// ============================================================================
// User Request Models
// ============================================================================

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

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct RenameQuery {
    /// New username (3–32 characters).
    pub name: String,
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct UpdateTasteQuery {
    /// Movie ID to update taste for.
    pub movie_id: i64,
    /// (Optional) Liked status. Defaults to true.
    #[param(default = true)]
    pub liked: Option<bool>,
    /// (Optional) Numeric rating for the movie.
    pub rating: Option<i32>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GetTasteQuery {
    /// Movie ID to retrieve taste for.
    pub movie_id: i64,
}
