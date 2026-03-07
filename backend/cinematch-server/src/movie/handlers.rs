use super::{AppState, GenreResponse, MovieResponse, SearchQuery, SearchResponse};
use crate::api_error::ApiError;
use crate::auth::guard::Auth;
use crate::party::PartyState;
use actix_web::{get, post, web};
use cinematch_common::SearchFilter;
use cinematch_common::models::ErrorResponse;
use cinematch_db::AppContext;
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
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    operation_id = "movie_get_info"
)]
#[get("")]
pub async fn get_movie(
    ctx: AppState,
    auth: Option<Auth>,
    query: web::Query<crate::party::MovieIdQuery>,
) -> Result<web::Json<super::MovieResponse>, ApiError> {
    let _ = auth.ok_or_else(|| ApiError::Unauthorized("No identity provided".to_string()))?;
    let movie_id = query.movie_id;

    let movie = Movie::new(movie_id)
        .data(&ctx)
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
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    operation_id = "get_genres"
)]
#[get("/genres")]
pub async fn get_genres(
    ctx: AppState,
    auth: Option<Auth>,
) -> Result<web::Json<super::GenreResponse>, ApiError> {
    let _ = auth.ok_or_else(|| ApiError::Unauthorized("No identity provided".to_string()))?;

    let genre_map = Movie::all_genres(&ctx).await?;
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
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    operation_id = "search_movies"
)]
#[post("/search")]
pub async fn search(
    ctx: AppState,
    auth: Option<Auth>,
    query: web::Query<SearchQuery>,
    body: Option<web::Json<cinematch_common::SearchFilter>>,
) -> Result<web::Json<SearchResponse>, ApiError> {
    let _ = auth.ok_or_else(|| ApiError::Unauthorized("No identity provided".to_string()))?;

    let query_inner = query.into_inner();
    let filter = body.map(|b| b.into_inner());
    let title = query_inner.title;
    let page = query_inner.page.unwrap_or(1);

    let movies = Movie::search(&ctx, &title, page, filter).await?;

    if movies.is_empty() {
        return Err(ApiError::NotFound("No movies found".to_string()));
    }

    let responses: Vec<super::MovieResponse> = movies
        .into_iter()
        .map(Into::<super::MovieResponse>::into)
        .collect();

    Ok(web::Json(super::SearchResponse { movies: responses }))
}

#[utoipa::path(
    request_body(content = super::RateMovieRequest, description = "Rating data"),
    responses(
        (status = 200, description = "Rating saved"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tags = ["Movie"],
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    operation_id = "rate_movie"
)]
#[post("/rate")]
pub async fn rate_movie(
    ctx: AppState,
    auth: Option<Auth>,
    body: web::Json<super::RateMovieRequest>,
) -> Result<actix_web::HttpResponse, ApiError> {
    let auth = auth.ok_or_else(|| ApiError::Unauthorized("No identity provided".to_string()))?;
    let user_id = auth.user_id();
    let params = body.into_inner();

    let user = cinematch_db::domain::User::new(user_id);
    user.add_rating(&ctx, params.movie_id, params.liked, params.rating)
        .await?;

    // Early guard: if the user is not in a party, or the party is not in a state where ratings are collected, return.
    if let Ok(Some(party)) = user.current_party(&ctx).await {
        let state = party.state(&ctx).await?;
        // not in watching or review state, so no need to broadcast
        if !(state == PartyState::Watching || state == PartyState::Review) {
            return Ok(actix_web::HttpResponse::Ok().finish());
        }

        if let Ok(Some(selected_id)) = party.selected_movie_id(&ctx).await
            && selected_id == params.movie_id
        {
            let mut total_rating = 0;
            let mut rating_count = 0;
            let mut party_size = 0;

            if let Ok(members) = party.members(&ctx).await {
                party_size = members.len();
                for member in members {
                    let member_user = cinematch_db::domain::User::new(member.user_id);
                    if let Ok(Some((_, Some(r), _))) =
                        member_user.get_movie_rating(&ctx, params.movie_id).await
                    {
                        total_rating += r;
                        rating_count += 1;
                    }
                }
            }

            let party_average = if rating_count > 0 {
                (total_rating as f32) / (rating_count as f32)
            } else {
                0.0
            };

            // Broadcast
            ctx.broadcast_party(
                party.id,
                &cinematch_common::models::websocket::ServerMessage::PartyMemberRated(
                    cinematch_common::models::websocket::PartyMemberRated {
                        user_id,
                        rating: params.rating.unwrap_or(0),
                        party_average,
                    },
                ),
                None,
            );

            // If everyone has rated, trigger auto-advance countdown (15s cooldown)
            if state == PartyState::Review && party_size > 0 && rating_count == party_size {
                let delay = chrono::Duration::seconds(15);
                ctx.scheduler
                    .schedule_custom_countdown(party.id, delay, ctx.clone())
                    .await;

                let deadline = chrono::Utc::now() + delay;
                ctx.broadcast_party(
                    party.id,
                    &cinematch_common::models::websocket::ServerMessage::PartyStateChanged(
                        cinematch_common::models::websocket::PartyStateChanged {
                            state: PartyState::Review.into(),
                            deadline_at: Some(deadline),
                            timeout_reason: Some(
                                cinematch_common::models::websocket::TimeoutReason::PhaseTimeout,
                            ),
                            selected_movie_id: Some(selected_id),
                        },
                    ),
                    None,
                );
            }
        }
    }

    Ok(actix_web::HttpResponse::Ok().finish())
}
