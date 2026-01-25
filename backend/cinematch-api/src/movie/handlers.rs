use crate::movie::SearchResponse;

use super::{
    AppState, ErrorResponse, GenreResponse, MovieResponse, RecommendedMoviesResponse, SearchQuery,
    extract_user_id,
};
use actix_identity::Identity;
use actix_web::{HttpResponse, get, web};
use log::error;

#[utoipa::path(
    responses(
        (status = 200, description = "Get movie information", body = MovieResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Movie not found"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("movie_id" = i64, Path, description = "The movie's unique ID")
    ),
    tags = ["movie"],
    security(("bearer_auth" = [])),
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
        (status = 200, description = "Get list of genres", body = GenreResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Genres not found"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tags = ["movie"],
    security(("bearer_auth" = [])),
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
        (status = 200, description = "Get list of recommended movies", body = RecommendedMoviesResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Recommended movies not found"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tags = ["movie"],
    security(("bearer_auth" = [])),
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
    responses(
        (status = 200, description = "Get list of movies", body = SearchResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Movies not found"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("query" = String, Query, description = "The search query string"),
        ("page" = Option<i64>, Query, description = "The page number for pagination")
    ),
    tags = ["movie"],
    security(("bearer_auth" = [])),
    operation_id = "search_movies"
)]
#[get("/search")]
pub async fn search(
    db: AppState,
    user: Option<Identity>,
    params: web::Query<SearchQuery>,
) -> HttpResponse {
    let _ = extract_user_id!(user);

    let query = params.query.trim();
    let page = params.page.unwrap_or(1);
    if query.is_empty() {
        return HttpResponse::BadRequest().json(ErrorResponse::new("Query cannot be empty"));
    }

    let movies = match db.search_movies(query, page).await {
        Ok(movies) => movies,
        Err(e) => {
            error!("Failed to search movies with query '{}': {}", query, e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to search movies"));
        }
    };

    if movies.is_empty() {
        return HttpResponse::NotFound().json(ErrorResponse::new("No movies found"));
    }

    let responses: Vec<MovieResponse> = movies
        .into_iter()
        .map(|movie| Into::<MovieResponse>::into(movie))
        .collect();

    HttpResponse::Ok().json(SearchResponse { movies: responses })
}
