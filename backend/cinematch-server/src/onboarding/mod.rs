use actix_web::{HttpResponse, Responder, post, web};
use cinematch_abi::domain::onboarding::OnboardingService;
use cinematch_common::models::{ErrorResponse, SwipeAction};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// DTO for OnboardingCandidate to avoid dependency on recommendation engine types in API doc
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OnboardingCandidateDto {
    pub movie_id: i64,
    /// P(rating_bucket | cluster_k) — shape: [num_clusters][NUM_BUCKETS]
    #[schema(value_type = Vec<Vec<f64>>)]
    pub rating_dist: Vec<[f64; 10]>,
    /// Popularity score for tie-breaking
    pub popularity: f32,
    /// Release year for filtering
    pub release_year: Option<i32>,
    /// Genre IDs for filtering
    pub genre_ids: Vec<Option<uuid::Uuid>>,
}

impl From<cinematch_recommendation_engine::onboarding::OnboardingCandidate>
    for OnboardingCandidateDto
{
    fn from(c: cinematch_recommendation_engine::onboarding::OnboardingCandidate) -> Self {
        Self {
            movie_id: c.movie_id,
            rating_dist: c.rating_dist,
            popularity: c.popularity,
            release_year: c.release_year,
            genre_ids: c.genre_ids,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct StartSessionResponse {
    pub session_id: Uuid,
    pub next_movie: Option<OnboardingCandidateDto>,
}

#[derive(Deserialize, ToSchema)]
pub struct RateRequest {
    pub session_id: Uuid,
    pub movie_id: i64,
    pub action: SwipeAction,
}

#[derive(Serialize, ToSchema)]
pub struct RateResponse {
    pub next_movie: Option<OnboardingCandidateDto>,
}

/// Start a new onboarding session
#[utoipa::path(
    responses(
        (status = 200, description = "Session started", body = StartSessionResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tags = ["Onboarding"],
    operation_id = "start_onboarding_session"
)]
#[post("/start")]
async fn start_session(service: web::Data<OnboardingService>) -> actix_web::Result<impl Responder> {
    let (session_id, next_movie) = service
        .start_session()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(StartSessionResponse {
        session_id,
        next_movie: next_movie.map(Into::into),
    }))
}

/// Rate a movie in an onboarding session
#[utoipa::path(
    request_body = RateRequest,
    responses(
        (status = 200, description = "Rating recorded", body = RateResponse),
        (status = 404, description = "Session not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tags = ["Onboarding"],
    operation_id = "rate_onboarding_movie"
)]
#[post("/rate")]
async fn rate_movie(
    service: web::Data<OnboardingService>,
    req: web::Json<RateRequest>,
) -> actix_web::Result<impl Responder> {
    let next_movie = service
        .rate_movie(req.session_id, req.movie_id, req.action)
        .await
        .map_err(|e| {
            // Check if session not found, return 404 or 400
            if e.to_string().contains("not found") {
                actix_web::error::ErrorNotFound(e)
            } else {
                actix_web::error::ErrorInternalServerError(e)
            }
        })?;

    Ok(HttpResponse::Ok().json(RateResponse {
        next_movie: next_movie.map(Into::into),
    }))
}

pub fn configure() -> impl FnOnce(&mut utoipa_actix_web::service_config::ServiceConfig) {
    |cfg: &mut utoipa_actix_web::service_config::ServiceConfig| {
        cfg.service(start_session).service(rate_movie);
    }
}
