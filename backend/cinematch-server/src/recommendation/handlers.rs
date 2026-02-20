use crate::AppState;
use crate::api_error::ApiError;
use crate::extract_user_id;
use crate::movie::MovieResponse;
use actix_identity::Identity;
use actix_web::{get, web};
use cinematch_abi::domain::{PartyValidation, Recommendation};
use cinematch_common::models::{ErrorResponse, RecommendationMethod, VectorType};
use cinematch_db::domain::{Movie, Party};
use rand::seq::SliceRandom;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct RecommendationQuery {
    /// Optional party ID to get recommendations for a party context.
    pub party_id: Option<Uuid>,
    /// Recommendation method:
    /// - `reviews`: Based on user reviews.
    /// - `semantic`: based on semantic similarity of plots/reviews.
    /// - `default`: Hybrid approach.
    #[param(default = "default")]
    pub method: Option<RecommendationMethod>,
    /// Vector type for semantic search:
    /// - `plot`: Movie plot embeddings.
    /// - `cast_crew`: Cast and crew embeddings.
    /// - `reviews`: Review embeddings.
    /// - `combined`: Combined embeddings.
    #[param(default = "combined")]
    pub vector: Option<VectorType>,
    /// Number of recommendations to return (default: 3).
    #[param(default = 3)]
    pub limit: Option<usize>,
    /// Whether to force onboarding recommendations.
    #[param(default = false)]
    pub onboard: Option<bool>,
}

use crate::movie::RecommendedMoviesResponse;

/// Get movie recommendations
///
/// Returns a list of recommended movies based on user preferences or party context.
#[utoipa::path(
    responses(
        (status = 200, description = "Recommended movies", body = RecommendedMoviesResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "No recommendations found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(RecommendationQuery),
    tags = ["Recommendation"],
    security(("cookie_auth" = [])),
    operation_id = "get_recommendations"
)]
#[get("")]
pub async fn get_recommendations(
    db: AppState,
    user: Identity,
    query: web::Query<RecommendationQuery>,
) -> Result<web::Json<RecommendedMoviesResponse>, ApiError> {
    let user_id = extract_user_id(user)?;
    let query = query.into_inner();

    let party_id = if let Some(pid) = query.party_id {
        // Verify user is in this party if party_id is provided
        let party = Party::from_id(&db, pid).await?;
        party.require_member(&db, user_id).await?;
        Some(pid)
    } else {
        None
    };

    let method = query.method.unwrap_or_default();
    let vector = query.vector.unwrap_or_default();
    let limit = query.limit.unwrap_or(3);

    let rec_handle = if let Some(pid) = party_id {
        Recommendation::for_party(std::sync::Arc::new(db.clone()), user_id, pid)
    } else {
        Recommendation::for_user(std::sync::Arc::new(db.clone()), user_id)
    };

    let movie_ids = match method {
        RecommendationMethod::Reviews => {
            rec_handle
                .get_from_reviews(vector, limit, query.onboard)
                .await?
        }
        RecommendationMethod::Semantic => {
            rec_handle
                .get_standard(vector, limit, query.onboard)
                .await?
        }
        RecommendationMethod::Default => {
            // Default logic: mix reviews and standard
            let reviews_ids = rec_handle
                .get_from_reviews(vector, 5, query.onboard)
                .await
                .unwrap_or_default();
            let standard_ids = rec_handle
                .get_standard(vector, 2, query.onboard)
                .await
                .unwrap_or_default();

            let mut combined = reviews_ids;
            for id in standard_ids {
                if !combined.contains(&id) {
                    combined.push(id);
                }
            }

            combined.shuffle(&mut rand::rng());
            combined.into_iter().take(limit).collect::<Vec<_>>()
        }
    };

    if movie_ids.is_empty() {
        return Err(ApiError::NotFound(
            "No recommendations available".to_string(),
        ));
    }

    let mut responses = Vec::with_capacity(movie_ids.len());
    for movie_id in movie_ids {
        if let Some(movie) = Movie::new(movie_id).data(&db).await? {
            responses.push(MovieResponse::from(movie));
        }
    }

    Ok(web::Json(RecommendedMoviesResponse {
        recommended_movies: responses,
    }))
}
