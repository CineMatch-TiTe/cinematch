//! Recommendation domain model for movie suggestions.

use crate::domain::DomainError;
use cinematch_common::models::VectorType;
use cinematch_db::AppContext;
use std::sync::Arc;
use uuid::Uuid;

/// Domain model for movie recommendations.
pub struct Recommendation {
    ctx: Arc<dyn AppContext>,
    user_id: Uuid,
    party_id: Option<Uuid>,
}

impl Recommendation {
    /// Create a recommendation handle for a user.
    pub fn for_user(ctx: Arc<dyn AppContext>, user_id: Uuid) -> Self {
        Self {
            ctx,
            user_id,
            party_id: None,
        }
    }

    /// Create a recommendation handle for a user within a party context.
    pub fn for_party(ctx: Arc<dyn AppContext>, user_id: Uuid, party_id: Uuid) -> Self {
        Self {
            ctx,
            user_id,
            party_id: Some(party_id),
        }
    }

    /// Fetch recommendations based on user reviews (Qdrant sparse query).
    pub async fn get_from_reviews(
        &self,
        vector_type: VectorType,
        limit: usize,
        _onboard: Option<bool>,
    ) -> Result<Vec<i64>, DomainError> {
        cinematch_recommendation_engine::recommed_movies_from_reviews(
            &self.ctx,
            self.user_id,
            self.party_id,
            vector_type,
            limit,
        )
        .await
        .map_err(DomainError::from)
    }

    /// Fetch standard recommendations (Qdrant average vector query).
    pub async fn get_standard(
        &self,
        vector_type: VectorType,
        limit: usize,
        _onboard: Option<bool>,
    ) -> Result<Vec<i64>, DomainError> {
        cinematch_recommendation_engine::recommend_movies(
            &self.ctx,
            self.user_id,
            self.party_id,
            vector_type,
            limit,
        )
        .await
        .map_err(DomainError::from)
    }
}
