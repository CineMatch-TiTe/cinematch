//! Recommendation domain model for movie suggestions.

use crate::domain::DomainError;
use crate::domain::onboarding::OnboardingService;
use cinematch_common::models::VectorType;
use cinematch_db::AppContext;
use std::sync::Arc;
use uuid::Uuid;

/// Domain model for movie recommendations.
pub struct Recommendation {
    ctx: Arc<dyn AppContext>,
    onboarding_service: Arc<OnboardingService>,
    user_id: Uuid,
    party_id: Option<Uuid>,
}

impl Recommendation {
    /// Create a recommendation handle for a user.
    pub fn for_user(
        ctx: Arc<dyn AppContext>,
        onboarding_service: Arc<OnboardingService>,
        user_id: Uuid,
    ) -> Self {
        Self {
            ctx,
            onboarding_service,
            user_id,
            party_id: None,
        }
    }

    /// Create a recommendation handle for a user within a party context.
    pub fn for_party(
        ctx: Arc<dyn AppContext>,
        onboarding_service: Arc<OnboardingService>,
        user_id: Uuid,
        party_id: Uuid,
    ) -> Self {
        Self {
            ctx,
            onboarding_service,
            user_id,
            party_id: Some(party_id),
        }
    }

    /// Fetch recommendations based on user reviews (Qdrant sparse query).
    pub async fn get_from_reviews(
        &self,
        vector_type: VectorType,
        limit: usize,
        onboard: Option<bool>,
    ) -> Result<Vec<i64>, DomainError> {
        // ... (check onboarding) ...
        if self.should_use_onboarding(onboard).await? {
            return self.get_onboarding_recommendations(limit).await;
        }

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
        onboard: Option<bool>,
    ) -> Result<Vec<i64>, DomainError> {
        if self.should_use_onboarding(onboard).await? {
            return self.get_onboarding_recommendations(limit).await;
        }

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

    /// Check if user should use onboarding recommendations.
    ///
    /// If `force_onboard` is Some, it overrides the automatic check.
    /// Otherwise, checks if rated < 10 movies.
    async fn should_use_onboarding(
        &self,
        force_onboard: Option<bool>,
    ) -> Result<bool, DomainError> {
        if let Some(force) = force_onboard {
            return Ok(force);
        }

        let user = cinematch_db::domain::User::new(self.user_id);
        let (pos, neg, _skipped) = user.get_ratings(&self.ctx).await?;
        let total_rated = pos.len() + neg.len();
        Ok(total_rated < 10)
    }

    /// Get valid onboarding candidates (top N by info gain).
    async fn get_onboarding_recommendations(&self, limit: usize) -> Result<Vec<i64>, DomainError> {
        use cinematch_common::models::SwipeAction;
        use cinematch_recommendation_engine::onboarding::{
            OnboardingCandidate, bayesian_update, pick_best_movies,
        };

        let cluster_count = self.ctx.db().get_onboarding_cluster_count().await?;
        if cluster_count == 0 {
            return Ok(vec![]);
        }

        let mut belief = vec![1.0 / cluster_count as f64; cluster_count as usize];
        let user = cinematch_db::domain::User::new(self.user_id);
        let (pos, neg, skipped) = user.get_ratings(&self.ctx).await?;

        // Replay ratings to build belief
        // Optimization: Fetch all rated movies in one batch
        let rated_movie_ids: Vec<i64> = pos.iter().chain(neg.iter()).cloned().collect();
        let rated_movies = self
            .ctx
            .db()
            .get_onboarding_movies_by_ids(&rated_movie_ids)
            .await?;
        let rated_movies_map: std::collections::HashMap<i64, _> =
            rated_movies.into_iter().map(|m| (m.movie_id, m)).collect();

        for movie_id in &pos {
            if let Some(movie) = rated_movies_map.get(movie_id) {
                if let Ok(dist) =
                    serde_json::from_value::<Vec<[f64; 10]>>(movie.rating_dist.clone())
                {
                    bayesian_update(&mut belief, &dist, SwipeAction::Like);
                }
            }
        }
        for movie_id in &neg {
            if let Some(movie) = rated_movies_map.get(movie_id) {
                if let Ok(dist) =
                    serde_json::from_value::<Vec<[f64; 10]>>(movie.rating_dist.clone())
                {
                    bayesian_update(&mut belief, &dist, SwipeAction::Dislike);
                }
            }
        }

        // Get user preferences
        let prefs = user.preferences(&self.ctx).await?.record(&self.ctx).await?;

        // Get candidates from Redis Cache
        let cached_candidates = self
            .onboarding_service
            .get_cached_candidates()
            .await
            .map_err(|e| DomainError::Internal(format!("Cache error: {}", e)))?;

        let rated_ids: std::collections::HashSet<i64> = pos
            .into_iter()
            .chain(neg.into_iter())
            .chain(skipped.into_iter())
            .collect();

        // Filter in-memory
        let candidates: Vec<OnboardingCandidate> = cached_candidates
            .into_iter()
            .filter(|c| !rated_ids.contains(&c.movie_id))
            .filter(|c| {
                // 2. Filter excluded genres
                if !prefs.excluded_genres.is_empty() {
                    let has_excluded = c
                        .genre_ids
                        .iter()
                        .flatten()
                        .any(|g| prefs.excluded_genres.contains(g));
                    if has_excluded {
                        return false;
                    }
                }

                // 3. Filter included genres
                if !prefs.included_genres.is_empty() {
                    let has_included = c
                        .genre_ids
                        .iter()
                        .flatten()
                        .any(|g| prefs.included_genres.contains(g));
                    if !has_included {
                        return false;
                    }
                }

                // 4. Filter release year
                if let Some(target_year) = prefs.preferred_year {
                    if let Some(year) = c.release_year {
                        let min_year = target_year - prefs.year_flexibility;
                        let max_year = target_year + prefs.year_flexibility;
                        if year < min_year || year > max_year {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }

                true
            })
            .collect();

        // 5. Pick best strictly by info gain (and popularity via tie-breaker)
        let best = pick_best_movies(&belief, &candidates, limit);
        let result_ids: Vec<i64> = best
            .into_iter()
            .map(|(idx, _)| candidates[idx].movie_id)
            .collect();

        Ok(result_ids)
    }
}
