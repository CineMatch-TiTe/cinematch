//! User preferences domain type with lazy-loading.

use uuid::Uuid;

use crate::{AppContext, DbResult};
use cinematch_common::FullUserPreferences;

/// User preferences with lazy-loading data access.
///
/// Only stores the database reference and user ID.
/// All data is fetched fresh from the database on each method call.
#[derive(Clone, Copy, Debug)]
pub struct Preferences {
    pub user_id: Uuid,
}

impl Preferences {
    /// Create a Preferences handle for a user.
    pub async fn from_user(ctx: &impl AppContext, user_id: Uuid) -> DbResult<Self> {
        // Verify preferences exist (they're created with the user)
        ctx.db().get_user_preferences(user_id).await?;
        Ok(Self { user_id })
    }

    /// Create a new Preferences handle without verifying existence.
    pub fn new(user_id: Uuid) -> Self {
        Self { user_id }
    }

    // ========================================================================
    // Lazy Getters - All fetch fresh from DB
    // ========================================================================

    /// Get the target release year preference.
    pub async fn target_release_year(&self, ctx: &impl AppContext) -> DbResult<Option<i32>> {
        let prefs = ctx.db().get_user_preferences(self.user_id).await?;
        Ok(prefs.preferred_year)
    }

    /// Get the release year flexibility.
    pub async fn release_year_flex(&self, ctx: &impl AppContext) -> DbResult<i32> {
        let prefs = ctx.db().get_user_preferences(self.user_id).await?;
        Ok(prefs.year_flexibility)
    }

    /// Get whether the user prefers "tite" (tight) recommendations.
    pub async fn is_tite(&self, ctx: &impl AppContext) -> DbResult<bool> {
        let prefs = ctx.db().get_user_preferences(self.user_id).await?;
        Ok(prefs.is_tite)
    }

    /// Get the included genre IDs.
    pub async fn include_genre_ids(&self, ctx: &impl AppContext) -> DbResult<Vec<Uuid>> {
        ctx.db().get_user_include_genres(self.user_id).await
    }

    /// Get the excluded genre IDs.
    pub async fn exclude_genre_ids(&self, ctx: &impl AppContext) -> DbResult<Vec<Uuid>> {
        ctx.db().get_user_exclude_genres(self.user_id).await
    }

    /// Get the full preferences record.
    pub async fn record(&self, ctx: &impl AppContext) -> DbResult<FullUserPreferences> {
        ctx.db().get_user_preferences(self.user_id).await
    }

    // ========================================================================
    // Mutations - All write directly to DB
    // ========================================================================

    /// Set the target release year.
    pub async fn set_target_release_year(
        &self,
        ctx: &impl AppContext,
        year: Option<i32>,
    ) -> DbResult<()> {
        use crate::repo::user::models::UpdateUserPreferences;
        ctx.db()
            .update_user_preferences(
                self.user_id,
                UpdateUserPreferences {
                    target_release_year: Some(year),
                    release_year_flex: None,
                    is_tite: None,
                },
            )
            .await?;
        Ok(())
    }

    /// Set the release year flexibility.
    pub async fn set_release_year_flex(&self, ctx: &impl AppContext, flex: i32) -> DbResult<()> {
        use crate::repo::user::models::UpdateUserPreferences;
        ctx.db()
            .update_user_preferences(
                self.user_id,
                UpdateUserPreferences {
                    target_release_year: None,
                    release_year_flex: Some(flex),
                    is_tite: None,
                },
            )
            .await?;
        Ok(())
    }

    /// Set the "tite" preference.
    pub async fn set_is_tite(&self, ctx: &impl AppContext, is_tite: bool) -> DbResult<()> {
        use crate::repo::user::models::UpdateUserPreferences;
        ctx.db()
            .update_user_preferences(
                self.user_id,
                UpdateUserPreferences {
                    target_release_year: None,
                    release_year_flex: None,
                    is_tite: Some(is_tite),
                },
            )
            .await?;
        Ok(())
    }

    /// Add an included genre.
    pub async fn add_include_genre(&self, ctx: &impl AppContext, genre_id: Uuid) -> DbResult<()> {
        ctx.db()
            .add_user_include_genre(self.user_id, genre_id)
            .await?;
        Ok(())
    }

    /// Remove an included genre.
    pub async fn remove_include_genre(
        &self,
        ctx: &impl AppContext,
        genre_id: Uuid,
    ) -> DbResult<()> {
        ctx.db()
            .remove_user_include_genre(self.user_id, genre_id)
            .await?;
        Ok(())
    }

    /// Add an excluded genre.
    pub async fn add_exclude_genre(&self, ctx: &impl AppContext, genre_id: Uuid) -> DbResult<()> {
        ctx.db()
            .add_user_exclude_genre(self.user_id, genre_id)
            .await?;
        Ok(())
    }

    /// Remove an excluded genre.
    pub async fn remove_exclude_genre(
        &self,
        ctx: &impl AppContext,
        genre_id: Uuid,
    ) -> DbResult<()> {
        ctx.db()
            .remove_user_exclude_genre(self.user_id, genre_id)
            .await?;
        Ok(())
    }
}
