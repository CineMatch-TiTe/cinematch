use crate::movie::SearchResponse;
use cinematch_common::SearchFilter;

use super::{
    AppState, ErrorResponse, GenreResponse, MovieResponse, RecommendedMoviesResponse, SearchQuery,
    extract_user_id,
};
use actix_identity::Identity;
use actix_web::{HttpResponse, get, post, web};
use log::error;

#[utoipa::path(
    responses(
        (status = 200, description = "Movie details", body = MovieResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Movie not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(("movie_id" = i64, Path, description = "TMDB movie ID")),
    tags = ["movie"],
    security(("cookie_auth" = [])),
    operation_id = "movie_get_info"
)]
#[get("/info/{movie_id}")]
pub async fn get_movie(
    db: AppState,
    user: Option<Identity>,
    movie_id: web::Path<i64>,
) -> HttpResponse {
    let _ = extract_user_id!(user);

    let movie_id = movie_id.into_inner();

    match db.get_movie_by_id(movie_id).await {
        Ok(movie) => match movie {
            Some(movie) => HttpResponse::Ok().json(Into::<MovieResponse>::into(movie)),
            None => HttpResponse::NotFound().json(ErrorResponse::new("Movie not found")),
        },
        Err(e) => {
            error!("Failed to retrieve movie with ID {}: {}", movie_id, e);
            HttpResponse::InternalServerError().json(ErrorResponse::new("Failed to retrieve movie"))
        }
    }
}

#[utoipa::path(
    responses(
        (status = 200, description = "Genre names", body = GenreResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "No genres available"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tags = ["movie"],
    security(("cookie_auth" = [])),
    operation_id = "get_genres"
)]
#[get("/genres")]
pub async fn get_genres(db: AppState, user: Option<Identity>) -> HttpResponse {
    let _ = extract_user_id!(user);

    match db.get_genres().await {
        Ok(genre_map) => {
            let mut names: Vec<String> = genre_map.keys().cloned().collect();
            names.sort();
            if names.is_empty() {
                return HttpResponse::NotFound().finish();
            }
            HttpResponse::Ok().json(GenreResponse { genres: names })
        }
        Err(e) => {
            error!("Failed to retrieve genres: {}", e);
            HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to retrieve genres"))
        }
    }
}

#[utoipa::path(
    responses(
        (status = 200, description = "Recommended movies (Qdrant-based)", body = RecommendedMoviesResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "No recommendations"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tags = ["movie"],
    security(("cookie_auth" = [])),
    operation_id = "get_recommendations"
)]
#[get("/recommend")]
pub async fn get_recommendations(db: AppState, user: Option<Identity>) -> HttpResponse {
    let user_id = extract_user_id!(user);

    // 3 is good

    let ids = match cinematch_recommendation_engine::recommend_movies(&db, user_id, 3).await {
        Ok(movies) => {
            if movies.is_empty() {
                return HttpResponse::NotFound().finish();
            }
            movies
        }
        Err(e) => {
            error!("Failed to retrieve recommended movies: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to retrieve recommended movies"));
        }
    };

    // build a list of movie responses

    let mut responses = Vec::with_capacity(ids.len());

    for movie_id in ids.iter() {
        match db.get_movie_by_id(*movie_id).await {
            Ok(movie) => match movie {
                Some(movie) => {
                    responses.push(Into::<MovieResponse>::into(movie));
                }
                None => {
                    error!("Recommended movie with ID {} not found", movie_id);
                }
            },
            Err(e) => {
                error!("Failed to retrieve movie with ID {}: {}", movie_id, e);
                return HttpResponse::InternalServerError()
                    .json(ErrorResponse::new("Failed to retrieve movie"));
            }
        }
    }

    if responses.is_empty() {
        return HttpResponse::NotFound().finish();
    }

    HttpResponse::Ok().json(RecommendedMoviesResponse {
        recommended_movies: responses,
    })
}

#[utoipa::path(
    request_body(content = SearchFilter, description = "Search filter"),
    params(
        ("title" = String, Query, description = "Movie title"),
        ("page" = Option<i64>, Query, description = "Page number")
    ),
    responses(
        (status = 200, description = "Matching movies", body = SearchResponse),
        (status = 400, description = "Empty query", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "No movies found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tags = ["movie"],
    security(("cookie_auth" = [])),
    operation_id = "search_movies"
)]
#[post("/search")]
pub async fn search(
    db: AppState,
    user: Option<Identity>,
    query: web::Query<SearchQuery>,
    body: Option<web::Json<SearchFilter>>,
) -> HttpResponse {
    let _ = extract_user_id!(user);

    let query = query.into_inner();
    let filter = body.map(|b| b.into_inner());
    let title = query.title;
    let page = query.page.unwrap_or(1);

    let movies = match db.search_movies(&title, page, filter).await {
        Ok(movies) => movies,
        Err(e) => {
            error!("Failed to search movies with query '{}': {}", title, e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to search movies"));
        }
    };

    if movies.is_empty() {
        return HttpResponse::NotFound().json(ErrorResponse::new("No movies found"));
    }

    let responses: Vec<MovieResponse> = movies
        .into_iter()
        .map(Into::<MovieResponse>::into)
        .collect();

    HttpResponse::Ok().json(SearchResponse { movies: responses })
}
