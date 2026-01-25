use super::{
    AppState, ErrorResponse, MovieResponse, GenreResponse, RecommendedMoviesResponse,
    extract_user_id,
};
use actix_identity::Identity;
use actix_web::{HttpResponse, get, web};
use cinematch_recommendation_engine::recommend_movies;
use futures::SinkExt;
use log::{error};
use chrono::Utc;

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
pub async fn get_movie(db: AppState, user: Option<Identity>, movie_id: web::Path<i64>) -> HttpResponse {
    let _ = extract_user_id!(user);

    let movie_id = movie_id.into_inner();

    match db.get_movie_by_id(movie_id).await {
        Ok(movie) => match movie {
            Some(movie) => {
                HttpResponse::Ok().json(Into::<MovieResponse>::into(movie))
            }
            None => {
                HttpResponse::NotFound().json(ErrorResponse::new("Movie not found"))
            }
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
        Ok(genres) => {
            if genres.is_empty() {
                return HttpResponse::NotFound().finish();
            }
            HttpResponse::Ok().json(GenreResponse { genres })
        }
        Err(e) => {
            error!("Failed to retrieve genres: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse::new("Failed to retrieve genres"))
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
    let movies = recommend_movies(&db, user_id, 3).await;

    let ids =match movies {
        Ok(movies) => {
            if movies.is_empty() {
                return HttpResponse::NotFound().finish();
            }
            movies
        }
        Err(e) => {
            error!("Failed to retrieve recommended movies: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse::new("Failed to retrieve recommended movies"))
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
                return HttpResponse::InternalServerError().json(ErrorResponse::new("Failed to retrieve movie"))
            }
        }
    }

    if responses.is_empty() {
        return HttpResponse::NotFound().finish();
    }

    HttpResponse::Ok().json(RecommendedMoviesResponse { recommended_movies: responses })

}