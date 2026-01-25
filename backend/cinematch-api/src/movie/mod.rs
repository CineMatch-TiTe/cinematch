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
    pub trailers: Vec<String>,
    pub cast: Vec<CastMemberResponse>,
}

impl Into<MovieResponse> for cinematch_db::MovieData {
    fn into(self) -> MovieResponse {
        MovieResponse {
            movie_id: self.movie_id,
            title: self.title,
            director: self.director.get(0).cloned(),
            genres: self.genres,
            overview: self.overview,
            release_date: if self.release_date > 0 {
                Some({
                    let naive = chrono::NaiveDateTime::from_timestamp(self.release_date, 0);
                    chrono::DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc)
                })
            } else {
                None
            },
            poster_url: self.poster_url,
            runtime: Some(self.runtime as i32),
            imdb_id: self.imdb_id,
            mediawiki_id: self.mediawiki_id,
            rating: self.rating,
            tagline: self.tagline,
            popularity: Some(self.popularity),
            trailers: self
                .video_keys
                .into_iter()
                .map(|video_id| format!("https://www.youtube.com/watch?v={}", video_id))
                .collect(),
            cast: self
                .cast
                .into_iter()
                .map(|member| CastMemberResponse {
                    name: member.name,
                    profile_url: member.profile_url,
                })
                .collect(),
        }
    }
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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RecommendedMoviesResponse {
    pub recommended_movies: Vec<MovieResponse>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SearchQuery {
    pub query: String,
    pub page: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SearchResponse {
    pub movies: Vec<MovieResponse>,
}
