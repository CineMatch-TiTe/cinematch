use super::{
    AppState, ErrorResponse, MovieResponse, GenreResponse, TrailerResponse, CastMemberResponse,
    extract_user_id,
};
use actix_identity::Identity;
use actix_web::{HttpResponse, get, web};
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
                HttpResponse::Ok().json(MovieResponse {
                    movie_id: movie.movie_id,
                    title: movie.title,
                    director: movie.director.get(0).cloned(),
                    genres: movie.genres,
                    overview: movie.overview,
                    release_date: if movie.release_date > 0 {
                        Some({
                            let naive = chrono::NaiveDateTime::from_timestamp(movie.release_date, 0);
                            chrono::DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc)
                        })
                    } else {
                        None
                    },
                    poster_url: movie.poster_url,
                    runtime: Some(movie.runtime as i32),
                    imdb_id: movie.imdb_id,
                    mediawiki_id: movie.mediawiki_id,
                    rating: movie.rating,
                    tagline: movie.tagline,
                    popularity: Some(movie.popularity),
                    trailers: movie.video_keys.into_iter().map(|video_id| TrailerResponse { trailer_url: format!("https://www.youtube.com/watch?v={}", video_id) }).collect(),
                    cast: movie.cast.into_iter().map(|member| CastMemberResponse {
                        name: member.name,
                        profile_url: member.profile_url,
                    }).collect(),
                })
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
