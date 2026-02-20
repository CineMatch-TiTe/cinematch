//! Onboarding database models.

use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::{onboarding_clusters, onboarding_movies};

// ============================================================================
// Onboarding Clusters
// ============================================================================

/// Queryable cluster row.
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = onboarding_clusters)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OnboardingCluster {
    pub cluster_id: i16,
    pub centroid: serde_json::Value,
    pub user_count: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Insertable cluster row.
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = onboarding_clusters)]
pub struct NewOnboardingCluster {
    pub cluster_id: i16,
    pub centroid: serde_json::Value,
    pub user_count: i32,
}

// ============================================================================
// Onboarding Movies
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = onboarding_movies)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OnboardingMovie {
    pub movie_id: i64,
    pub info_gain: f32,
    pub rating_dist: serde_json::Value,
    pub rating_count: i32,
    pub genre_ids: Vec<Option<uuid::Uuid>>,
}

/// Insertable onboarding movie row.
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = onboarding_movies)]
pub struct NewOnboardingMovie {
    pub movie_id: i64,
    pub info_gain: f32,
    pub rating_dist: serde_json::Value,
    pub rating_count: i32,
    pub genre_ids: Vec<Option<uuid::Uuid>>,
}
