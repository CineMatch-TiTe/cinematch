//! Onboarding data repository - pre-computed entropy data for adaptive onboarding.

pub mod models;

use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};

use self::models::*;
use crate::schema::{onboarding_clusters, onboarding_movies};
use crate::{Database, DbError, DbResult};

impl Database {
    /// Atomically replace all onboarding clusters (delete + insert).
    pub async fn store_onboarding_clusters(
        &self,
        clusters: &[NewOnboardingCluster],
    ) -> DbResult<usize> {
        let mut conn = self.pool.get().await.map_err(DbError::from)?;

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            Box::pin(async move {
                diesel::delete(onboarding_clusters::table)
                    .execute(conn)
                    .await?;
                diesel::insert_into(onboarding_clusters::table)
                    .values(clusters)
                    .execute(conn)
                    .await?;
                Ok(())
            })
        })
        .await
        .map_err(DbError::from)
        .map(|_| clusters.len())
    }

    /// Atomically replace all onboarding movies (delete + insert).
    pub async fn store_onboarding_movies(&self, movies: &[NewOnboardingMovie]) -> DbResult<usize> {
        let mut conn = self.pool.get().await.map_err(DbError::from)?;

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            Box::pin(async move {
                diesel::delete(onboarding_movies::table)
                    .execute(conn)
                    .await?;

                // Insert in batches
                let batch_size = 1000;
                for chunk in movies.chunks(batch_size) {
                    diesel::insert_into(onboarding_movies::table)
                        .values(chunk)
                        .execute(conn)
                        .await?;
                }
                Ok(())
            })
        })
        .await
        .map_err(DbError::from)
        .map(|_| movies.len())
    }

    /// Get all onboarding clusters.
    pub async fn get_onboarding_clusters(&self) -> DbResult<Vec<OnboardingCluster>> {
        let mut conn = self.pool.get().await.map_err(DbError::from)?;
        onboarding_clusters::table
            .order(onboarding_clusters::cluster_id.asc())
            .load(&mut conn)
            .await
            .map_err(DbError::from)
    }

    /// Get top onboarding candidate movies for a set of genre IDs, ordered by info_gain.
    /// Returns movies that have ANY of the requested genres (array overlap).
    pub async fn get_onboarding_candidates(
        &self,
        genre_ids: &[uuid::Uuid],
    ) -> DbResult<Vec<OnboardingMovie>> {
        let mut conn = self.pool.get().await.map_err(DbError::from)?;
        onboarding_movies::table
            .filter(onboarding_movies::genre_ids.overlaps_with(genre_ids))
            .order(onboarding_movies::info_gain.desc())
            .load(&mut conn)
            .await
            .map_err(DbError::from)
    }

    /// Get a specific onboarding movie by ID (to retrieve its rating distribution).
    pub async fn get_onboarding_movie(&self, movie_id: i64) -> DbResult<OnboardingMovie> {
        let mut conn = self.pool.get().await.map_err(DbError::from)?;
        onboarding_movies::table
            .filter(onboarding_movies::movie_id.eq(movie_id))
            .first(&mut conn)
            .await
            .map_err(DbError::from)
    }

    /// Get all onboarding candidates (across all genres).
    /// Used to re-evaluate info gain against current belief.
    pub async fn get_all_onboarding_candidates(&self) -> DbResult<Vec<OnboardingMovie>> {
        let mut conn = self.pool.get().await.map_err(DbError::from)?;
        onboarding_movies::table
            .load(&mut conn)
            .await
            .map_err(DbError::from)
    }

    /// Get the number of onboarding clusters.
    pub async fn get_onboarding_cluster_count(&self) -> DbResult<i64> {
        let mut conn = self.pool.get().await.map_err(DbError::from)?;
        onboarding_clusters::table
            .count()
            .get_result(&mut conn)
            .await
            .map_err(DbError::from)
    }

    /// Get all onboarding candidates with metadata (release_year, popularity).
    pub async fn get_all_onboarding_candidates_with_meta(
        &self,
    ) -> DbResult<Vec<(OnboardingMovie, Option<i32>, f32)>> {
        let mut conn = self.pool.get().await.map_err(DbError::from)?;
        use crate::schema::movies;

        onboarding_movies::table
            .inner_join(movies::table.on(movies::movie_id.eq(onboarding_movies::movie_id)))
            .select((
                onboarding_movies::all_columns,
                movies::release_year,
                movies::popularity,
            ))
            .order(onboarding_movies::movie_id.asc())
            .load::<(OnboardingMovie, Option<i32>, f32)>(&mut conn)
            .await
            .map_err(DbError::from)
    }

    /// Get multiple onboarding movies by ID (batch fetch).
    pub async fn get_onboarding_movies_by_ids(
        &self,
        movie_ids: &[i64],
    ) -> DbResult<Vec<OnboardingMovie>> {
        let mut conn = self.pool.get().await.map_err(DbError::from)?;
        onboarding_movies::table
            .filter(onboarding_movies::movie_id.eq_any(movie_ids))
            .load(&mut conn)
            .await
            .map_err(DbError::from)
    }
}
