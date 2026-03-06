use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use serde_json::json;
use utoipa::ToSchema;
use uuid::Uuid;

pub mod movie;
pub mod websocket;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
    Default,
)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationMethod {
    Reviews,
    Semantic,
    #[default]
    Default,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
    Default,
)]
#[serde(rename_all = "snake_case")]
pub enum VectorType {
    Plot,
    CastCrew,
    Reviews,
    #[default]
    Combined,
}

impl VectorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            VectorType::Plot => "plot_vector",
            VectorType::CastCrew => "cast_crew_vector",
            VectorType::Reviews => "reviews_vector",
            VectorType::Combined => "combined_vector",
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum PartyState {
    Created,   // Initial state, people can join and start picking
    Picking, // people can pick movies (taste), people cant join anymore (code is freed, and party is identified by uuid)
    Voting, // people can vote on picked movies, until a movie is decided, we can go to picking or watching from here
    Watching, // movie is being watched, people can start review after 15 minutes, and can update their review
    Review,   // 90% of runtime passed start showin results, leader can also skip
    Disbanded, // dead party, keep for history
}

/// Timeout/schedule event types
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, utoipa::ToSchema,
)]
pub enum TimeoutType {
    /// Voting phase is starting
    VotingStarting,
    /// Voting phase is ending (auto-tally)
    VotingEnding,
    /// Watching phase is ending (ready to review)
    WatchingEnding,
    /// All members ready timeout countdown
    ReadyTimeout,
}

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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserTasteResponse {
    pub movie_id: i64,
    pub liked: Option<bool>,
    pub rating: Option<i32>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
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

/// User action on a movie (e.g. during discovery or rating flow)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SwipeAction {
    Like,
    Dislike,
    Skip,
    SuperLike,
}
