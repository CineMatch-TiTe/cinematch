pub mod handlers;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// Re-export types that are used in responses
pub use crate::AppState;
pub use cinematch_common::ErrorResponse;
pub use cinematch_common::extract_user_id;
pub use cinematch_db::DbError;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MovieResponse {
    pub movie_id: i64,
    pub title: String,
    pub director: Option<String>,
    pub genres: Vec<String>,
    pub overview: Option<String>,
    pub release_date: Option<DateTime<Utc>>,
    pub poster_url: Option<String>,
    pub runtime: Option<i32>,
    pub imdb_id: Option<String>,
    pub mediawiki_id: Option<String>,
    pub rating: Option<String>,
    pub tagline: Option<String>,
    pub popularity: Option<f32>,
    pub trailers: Vec<TrailerResponse>,
    pub cast: Vec<CastMemberResponse>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TrailerResponse {
    pub trailer_url: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CastMemberResponse {
    pub name: String,
    pub profile_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenreResponse {
    pub genres: Vec<String>,
}