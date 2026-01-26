use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use serde_json::json;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchFilter {
    #[schema(example = json!(["Action", "Drama"]))]
    pub exclude_genres: Vec<String>,
    #[schema(example = json!(["Horror", "Comedy"]))]
    pub include_genres: Vec<String>,
    #[schema(example = json!(2020))]
    pub min_year: Option<i32>,
    #[schema(example = json!(2025))]
    pub max_year: Option<i32>,
    #[schema(example = json!(30))]
    pub min_runtime: Option<i32>,
    #[schema(example = json!(180))]
    pub max_runtime: Option<i32>,
}

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
#[schema(example = json!({"error": "Not a member of this party"}))]
pub struct ErrorResponse {
    /// Human-readable error message
    pub error: String,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}
