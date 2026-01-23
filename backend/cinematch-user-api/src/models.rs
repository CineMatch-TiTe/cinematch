//! API request/response models with utoipa OpenAPI documentation

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ============================================================================
// Error Response
// ============================================================================

/// Standard error response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}

// ============================================================================
// User Responses
// ============================================================================

/// Response when creating a guest user
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GuestLoginResponse {
    /// The newly created user ID
    pub user_id: Uuid,
    /// The auto-generated username
    pub username: String,
    /// Whether this is a guest/oneshot user
    pub is_guest: bool,
    /// When the user was created
    pub created_at: DateTime<Utc>,
}

/// Response with user details
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
    pub new_username: String,
}
