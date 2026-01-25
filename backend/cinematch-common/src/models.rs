
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FullUserPreferences {
    pub user_id: Uuid,
    pub preferred_year: Option<i32>,
    pub year_flexibility: i32,
    pub included_genres: Vec<Uuid>,
    pub excluded_genres: Vec<Uuid>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub is_tite: bool,
}

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