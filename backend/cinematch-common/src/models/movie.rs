use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MovieData {
    pub movie_id: i64,
    pub title: String,
    pub runtime: i64,
    pub popularity: f32,
    pub imdb_id: Option<String>,
    pub mediawiki_id: Option<String>,
    pub rating: Option<String>,
    pub release_date: i64,
    pub original_language: Option<String>,
    pub poster_url: Option<String>,
    pub overview: Option<String>,
    pub tagline: Option<String>,
    pub director: Vec<String>,
    pub genres: Vec<String>,
    pub keywords: Vec<String>,
    pub cast: Vec<CastMember>,
    pub production_countries: Vec<String>,
    pub reviews: Vec<String>,
    pub video_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq, Hash)]
pub struct CastMember {
    pub name: String,
    pub profile_url: Option<String>,
}
