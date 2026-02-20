use cinematch_common::models::SwipeAction;
use cinematch_db::Database;
use cinematch_recommendation_engine::onboarding::{
    OnboardingCandidate, bayesian_update, pick_best_movie,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingState {
    pub belief: Vec<f64>,
    pub step: usize,
    pub max_steps: usize,
    // Add more state as needed, e.g., session ID, rated movies
}

#[derive(Clone)]
pub struct OnboardingCache(Arc<RwLock<Option<Vec<OnboardingCandidate>>>>);

impl OnboardingCache {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(None)))
    }

    pub async fn get(&self) -> Option<Vec<OnboardingCandidate>> {
        let lock = self.0.read().await;
        lock.clone()
    }

    pub async fn set(&self, candidates: Vec<OnboardingCandidate>) {
        let mut lock = self.0.write().await;
        *lock = Some(candidates);
    }
}

pub struct OnboardingService {
    db: Arc<Database>,
    onboarding_candidates: OnboardingCache,
}

impl OnboardingService {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            onboarding_candidates: OnboardingCache::new(),
        }
    }

    /// Start a new onboarding session, persist to Redis, and return session ID + first movie.
    pub async fn start_session(&self) -> anyhow::Result<(Uuid, Option<OnboardingCandidate>)> {
        let cluster_count = self.db.get_onboarding_cluster_count().await?;
        if cluster_count == 0 {
            anyhow::bail!(
                "No onboarding clusters found. Run 'cinematch-importer update-onboarding' first."
            );
        }

        let belief = vec![1.0 / cluster_count as f64; cluster_count as usize];

        let state = OnboardingState {
            belief,
            step: 0,
            max_steps: 10,
        };

        // Pick the first movie
        let candidate = self.select_next_movie(&state, &[]).await?;

        // Generate Session ID
        let session_id = Uuid::new_v4();
        self.save_session(session_id, &state).await?;

        Ok((session_id, candidate))
    }

    /// Process a user's swipe using session ID.
    pub async fn rate_movie(
        &self,
        session_id: Uuid,
        movie_id: i64,
        action: SwipeAction,
    ) -> anyhow::Result<Option<OnboardingCandidate>> {
        let mut state = self.load_session(session_id).await?;

        // TODO: Track rated movies in state to avoid repeats
        // For now, we rely on the client or just basic filtering?
        // Let's add rated_movies to state.
        // Assuming OnboardingState can be updated to include it.
        // But for now, let's just pass empty list or implement proper tracking.
        let rated_movies = vec![]; // functionality gap

        // 1. Update belief if not skip
        if action != SwipeAction::Skip {
            let movie = self.db.get_onboarding_movie(movie_id).await?;
            let dist: Vec<[f64; 10]> = serde_json::from_value(movie.rating_dist)?;
            bayesian_update(&mut state.belief, &dist, action);
        }

        state.step += 1;

        if state.step >= state.max_steps {
            // Clear session? Or keep for final result generation?
            // Keep it.
            self.save_session(session_id, &state).await?;
            return Ok(None);
        }

        // 2. Select next movie
        let next = self.select_next_movie(&state, &rated_movies).await?;

        // Save state
        self.save_session(session_id, &state).await?;

        Ok(next)
    }

    async fn save_session(&self, session_id: Uuid, state: &OnboardingState) -> anyhow::Result<()> {
        let mut conn = self.db.redis.get().await?;
        let key = format!("onboarding:{}", session_id);

        cinematch_db::conn::redis::cache::set(
            &mut conn, &key, state, 3600, // 1 hour
        )
        .await?;
        Ok(())
    }

    async fn load_session(&self, session_id: Uuid) -> anyhow::Result<OnboardingState> {
        let mut conn = self.db.redis.get().await?;
        let key = format!("onboarding:{}", session_id);

        match cinematch_db::conn::redis::cache::get::<OnboardingState>(&mut conn, &key).await? {
            Some(state) => Ok(state),
            None => anyhow::bail!("Session not found or expired"),
        }
    }

    /// Helper to select the next best movie based on current belief.
    async fn select_next_movie(
        &self,
        state: &OnboardingState,
        rated_movie_ids: &[i64],
    ) -> anyhow::Result<Option<OnboardingCandidate>> {
        // Fetch valid candidates from Redis cache (or DB if missing)
        let candidates = self.get_cached_candidates().await?;

        // Filter out already rated movies
        // Note: We might want to apply genre/year filtering here too if we passed preferences
        let mut filtered_candidates: Vec<OnboardingCandidate> = candidates
            .into_iter()
            .filter(|c| !rated_movie_ids.contains(&c.movie_id))
            .collect();

        // Use core engine logic to pick best
        if let Some((idx, _info_gain)) = pick_best_movie(&state.belief, &filtered_candidates) {
            Ok(Some(filtered_candidates.swap_remove(idx)))
        } else {
            Ok(None)
        }
    }

    /// Fetch all onboarding candidates, with L1 (RAM) and L2 (Redis) caching.
    pub async fn get_cached_candidates(&self) -> anyhow::Result<Vec<OnboardingCandidate>> {
        // 1. Check L1 Cache (RAM)
        if let Some(candidates) = self.onboarding_candidates.get().await {
            return Ok(candidates);
        }

        // 2. Check L2 Cache (Redis)
        let mut conn = self.db.redis.get().await?;
        let key = "onboarding:candidates:full";

        if let Some(cached) =
            cinematch_db::conn::redis::cache::get::<Vec<OnboardingCandidate>>(&mut conn, key)
                .await?
        {
            // Populate L1 and return
            self.onboarding_candidates.set(cached.clone()).await;
            return Ok(cached);
        }

        // 3. Cache Miss: Fetch from DB (L3)
        let all_movies_meta = self.db.get_all_onboarding_candidates_with_meta().await?;

        // Map to OnboardingCandidate
        let candidates: Vec<OnboardingCandidate> = all_movies_meta
            .into_iter()
            .map(|(m, release_year, popularity)| {
                let rating_dist: Vec<[f64; 10]> =
                    serde_json::from_value(m.rating_dist).unwrap_or_else(|_| vec![[0.0; 10]; 200]); // 200 clusters default

                OnboardingCandidate {
                    movie_id: m.movie_id,
                    rating_dist,
                    popularity,
                    release_year,
                    genre_ids: m.genre_ids,
                }
            })
            .collect();

        // Populate L2 (Redis) - 1 hour TTL
        cinematch_db::conn::redis::cache::set(&mut conn, key, &candidates, 3600).await?;

        // Populate L1 (RAM)
        self.onboarding_candidates.set(candidates.clone()).await;

        Ok(candidates)
    }
}
