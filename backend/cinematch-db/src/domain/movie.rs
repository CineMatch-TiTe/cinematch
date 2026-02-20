use crate::conn::qdrant::models::MovieData;
use crate::{AppContext, DbResult};
use cinematch_common::{FullUserPreferences, SearchFilter};

use uuid::Uuid;

/// Movie domain object with lazy-loading.
pub struct Movie {
    pub id: i64,
}

impl Movie {
    pub fn new(id: i64) -> Self {
        Self { id }
    }

    /// Fetch full movie data.
    pub async fn data(&self, ctx: &impl AppContext) -> DbResult<Option<MovieData>> {
        ctx.db().get_movie_by_id(self.id).await
    }

    /// Check if this movie matches user preferences.
    pub async fn matches_prefs(
        &self,
        ctx: &impl AppContext,
        prefs: &FullUserPreferences,
        excluded_ids: Option<&[i64]>,
    ) -> DbResult<bool> {
        ctx.db()
            .filter_check_movie(self.id, prefs, excluded_ids)
            .await
    }

    // --- Static methods for general movie queries ---

    /// Get all genres as a map: name -> id.
    pub async fn all_genres(
        ctx: &impl AppContext,
    ) -> DbResult<std::collections::HashMap<String, Uuid>> {
        ctx.db().get_genres().await
    }

    /// Get popular movie IDs.
    pub async fn popular_ids(ctx: &impl AppContext, limit: i64) -> DbResult<Vec<i64>> {
        ctx.db().get_popular_movie_ids(limit).await
    }

    /// Get popular movies with full data.
    pub async fn popular(ctx: &impl AppContext, limit: i64) -> DbResult<Vec<MovieData>> {
        ctx.db().get_popular_movies(limit).await
    }

    /// Search movies by name.
    pub async fn search(
        ctx: &impl AppContext,
        name: &str,
        page: i64,
        filter: Option<SearchFilter>,
    ) -> DbResult<Vec<MovieData>> {
        ctx.db().search_movies(name, page, filter).await
    }
}
