//! Recommendation domain model for movie suggestions.

use crate::domain::DomainError;
use cinematch_common::models::VectorType;
use cinematch_db::AppContext;
use rand::seq::SliceRandom;
use uuid::Uuid;

/// Domain model for movie recommendations.
pub struct Recommendation<C: AppContext> {
    ctx: C,
    user_id: Uuid,
    party_id: Option<Uuid>,
}

impl<C: AppContext> Recommendation<C> {
    /// Create a recommendation handle for a user.
    pub fn for_user(ctx: C, user_id: Uuid) -> Self {
        Self {
            ctx,
            user_id,
            party_id: None,
        }
    }

    /// Create a recommendation handle for a user within a party context.
    pub fn for_party(ctx: C, user_id: Uuid, party_id: Uuid) -> Self {
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
    ) -> Result<Vec<i64>, DomainError> {
        cinematch_recommendation_engine::recommend_from_reviews(
            &self.ctx,
            self.user_id,
            self.party_id,
            vector_type,
            limit,
        )
        .await
        .map_err(DomainError::from)
    }

    /// Fetch semantic recommendations (Qdrant average vector query).
    pub async fn get_semantic(
        &self,
        vector_type: VectorType,
        limit: usize,
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

    /// Fetch hybrid recommendations (mix of reviews and semantic).
    pub async fn get_hybrid(
        &self,
        vector_type: VectorType,
        limit: usize,
    ) -> Result<Vec<i64>, DomainError> {
        let reviews_ids = self
            .get_from_reviews(vector_type, 5)
            .await
            .unwrap_or_default();
        let semantic_ids = self.get_semantic(vector_type, 2).await.unwrap_or_default();

        let mut combined = reviews_ids;
        for id in semantic_ids {
            if !combined.contains(&id) {
                combined.push(id);
            }
        }

        combined.shuffle(&mut rand::rng());
        Ok(combined.into_iter().take(limit).collect())
    }
}
