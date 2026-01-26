pub mod handlers;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Re-export types that are used in responses
pub use crate::AppState;
pub use cinematch_common::ErrorResponse;
pub use cinematch_common::extract_user_id;

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

impl From<cinematch_db::MovieData> for MovieResponse {
    fn from(val: cinematch_db::MovieData) -> Self {
        MovieResponse {
            movie_id: val.movie_id,
            title: val.title,
            director: val.director.first().cloned(),
            genres: val.genres,
            overview: val.overview,
            release_date: if val.release_date > 0 {
                chrono::DateTime::from_timestamp(val.release_date, 0)
            } else {
                None
            },
            poster_url: val.poster_url,
            runtime: Some(val.runtime as i32),
            imdb_id: val.imdb_id,
            mediawiki_id: val.mediawiki_id,
            rating: val.rating,
            tagline: val.tagline,
            popularity: Some(val.popularity),
            trailers: val
                .video_keys
                .into_iter()
                .map(|video_id| format!("https://www.youtube.com/watch?v={}", video_id))
                .collect(),
            cast: val
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
