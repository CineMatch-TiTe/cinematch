use super::{AppState, GenreResponse, MovieResponse, SearchQuery, SearchResponse};
use crate::api_error::ApiError;
use crate::extract_user_id;
use actix_identity::Identity;
use actix_web::{get, post, web};
use cinematch_common::SearchFilter;
use cinematch_common::models::ErrorResponse;
use cinematch_db::domain::Movie;

#[utoipa::path(
    responses(
        (status = 200, description = "Movie details", body = MovieResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Movie not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(("movie_id" = i64, Query, description = "TMDB movie ID")),
    tags = ["Movie"],
    security(("cookie_auth" = [])),
    operation_id = "movie_get_info"
)]
#[get("")]
pub async fn get_movie(
    db: AppState,
    user: Option<Identity>,
    query: web::Query<crate::party::MovieIdQuery>,
) -> Result<web::Json<super::MovieResponse>, ApiError> {
    let _ = extract_user_id(user)?;
    let movie_id = query.movie_id;

    let movie = Movie::new(movie_id)
        .data(&db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Movie not found".to_string()))?;

    Ok(web::Json(Into::<super::MovieResponse>::into(movie)))
}

#[utoipa::path(
    responses(
        (status = 200, description = "Genre names", body = GenreResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "No genres available", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tags = ["Movie"],
    security(("cookie_auth" = [])),
    operation_id = "get_genres"
)]
#[get("/genres")]
pub async fn get_genres(
    db: AppState,
    user: Option<Identity>,
) -> Result<web::Json<super::GenreResponse>, ApiError> {
    let _ = extract_user_id(user)?;

    let genre_map = Movie::all_genres(&db).await?;
    let mut names: Vec<String> = genre_map.keys().cloned().collect();
    names.sort();

    if names.is_empty() {
        return Err(ApiError::NotFound("No genres available".to_string()));
    }

    Ok(web::Json(super::GenreResponse { genres: names }))
}

#[utoipa::path(
    request_body(content = SearchFilter, description = "Search filter"),
    params(SearchQuery),
    responses(
        (status = 200, description = "Matching movies", body = SearchResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "No movies found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tags = ["Movie"],
    security(("cookie_auth" = [])),
    operation_id = "search_movies"
)]
#[post("/search")]
pub async fn search(
    db: AppState,
    user: Option<Identity>,
    query: web::Query<SearchQuery>,
    body: Option<web::Json<cinematch_common::SearchFilter>>,
) -> Result<web::Json<SearchResponse>, ApiError> {
    let _ = extract_user_id(user)?;

    let query_inner = query.into_inner();
    let filter = body.map(|b| b.into_inner());
    let title = query_inner.title;
    let page = query_inner.page.unwrap_or(1);

    let movies = Movie::search(&db, &title, page, filter).await?;

    if movies.is_empty() {
        return Err(ApiError::NotFound("No movies found".to_string()));
    }

    let responses: Vec<super::MovieResponse> = movies
        .into_iter()
        .map(Into::<super::MovieResponse>::into)
        .collect();

    Ok(web::Json(super::SearchResponse { movies: responses }))
}
