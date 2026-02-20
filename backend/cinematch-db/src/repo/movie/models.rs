//! Movie-related database models (PostgreSQL).

use diesel::prelude::*;

use crate::schema::{
    cast_members, directors, genres, keywords, movies, production_countries, trailers,
};

// ============================================================================
// Movie Models
// ============================================================================

/// Queryable Movie from database
#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = movies)]
#[diesel(primary_key(movie_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Movie {
    pub movie_id: i64,
    pub title: String,
    pub runtime: i32,
    pub popularity: f32,
    pub imdb_id: Option<String>,
    pub mediawiki_id: Option<String>,
    pub rating: Option<String>,
    pub release_date: chrono::NaiveDateTime,
    pub original_language: Option<String>,
    pub poster_url: Option<String>,
    pub overview: Option<String>,
    pub tagline: Option<String>,
    pub release_year: Option<i32>,
}

// ============================================================================
// Movie Metadata Models
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = directors)]
#[diesel(primary_key(director_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Director {
    pub director_id: uuid::Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = genres)]
#[diesel(primary_key(genre_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Genre {
    pub genre_id: uuid::Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = keywords)]
#[diesel(primary_key(keyword_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Keyword {
    pub keyword_id: uuid::Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = cast_members)]
#[diesel(primary_key(cast_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CastMemberRow {
    pub cast_id: uuid::Uuid,
    pub name: String,
    pub profile_url: Option<String>,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = production_countries)]
#[diesel(primary_key(country_code))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProductionCountry {
    pub country_code: String,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = trailers)]
#[diesel(primary_key(trailer_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Trailer {
    pub trailer_id: uuid::Uuid,
    pub video_key: String,
}
