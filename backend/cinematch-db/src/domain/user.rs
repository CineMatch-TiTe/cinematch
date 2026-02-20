//! User domain type with lazy-loading.

use chrono::{DateTime, Utc};
use cinematch_common::HasId;
use uuid::Uuid;

use crate::repo::user::models::User as DbUser;
use crate::{AppContext, DbResult};

use super::{Party, Preferences};

/// A user with lazy-loading data access.
///
/// Only stores user ID.
/// All data is fetched fresh from the database on each method call, using the provided context.
#[derive(Clone, Copy, Debug)]
pub struct User {
    pub id: Uuid,
}

impl HasId for User {
    fn id(&self) -> Uuid {
        self.id
    }
}

impl User {
    /// Create a new User handle from an existing user ID.
    /// Verifies the user exists in the database.
    /// Create a new User handle from an existing user ID.
    /// Verifies the user exists in the database.
    pub async fn from_id(ctx: &impl AppContext, id: Uuid) -> DbResult<Self> {
        // Verify user exists
        ctx.db().get_user(id).await?;
        Ok(Self { id })
    }

    /// Create a new User handle without verifying existence.
    /// Create a new User handle without verifying existence.
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }

    /// Create a new guest (oneshot) user.
    /// Create a new guest (oneshot) user.
    pub async fn create_guest(ctx: &impl AppContext, username: &str) -> DbResult<Self> {
        let user = ctx.db().create_guest_user(username).await?;
        Ok(Self { id: user.id })
    }

    /// Create a new persistent user.
    /// Create a new persistent user.
    pub async fn create_persistent(ctx: &impl AppContext, username: &str) -> DbResult<Self> {
        let user = ctx.db().create_persistent_user(username).await?;
        Ok(Self { id: user.id })
    }

    // ========================================================================
    // Lazy Getters - All fetch fresh from DB
    // ========================================================================

    /// Get the user's username.
    /// Get the user's username.
    pub async fn username(&self, ctx: &impl AppContext) -> DbResult<String> {
        let user = ctx.db().get_user(self.id).await?;
        Ok(user.username)
    }

    /// Get whether this is a oneshot (guest) user.
    /// Get whether this is a oneshot (guest) user.
    pub async fn is_oneshot(&self, ctx: &impl AppContext) -> DbResult<bool> {
        let user = ctx.db().get_user(self.id).await?;
        Ok(user.oneshot)
    }

    /// Get the full user record.
    /// Get the full user record.
    pub async fn record(&self, ctx: &impl AppContext) -> DbResult<DbUser> {
        ctx.db().get_user(self.id).await
    }

    /// Get the user's preferences as a Preferences domain type.
    /// Get the user's preferences as a Preferences domain type.
    pub async fn preferences(&self, ctx: &impl AppContext) -> DbResult<Preferences> {
        Preferences::from_user(ctx, self.id).await
    }

    /// Get the user's current active party (if any).
    pub async fn current_party(&self, ctx: &impl AppContext) -> DbResult<Option<Party>> {
        match ctx.db().get_user_active_party(self.id).await {
            Ok(party_id) => Ok(Some(Party::from_id(ctx, party_id).await?)),
            Err(crate::DbError::UserNotInParty(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    // ========================================================================
    // Mutations - All write directly to DB
    // ========================================================================

    /// Update the user's username.
    pub async fn set_username(&self, ctx: &impl AppContext, username: &str) -> DbResult<()> {
        use crate::repo::user::models::UpdateUser;
        ctx.db()
            .update_user(
                self.id,
                UpdateUser {
                    username: Some(username),
                    oneshot: None,
                },
            )
            .await?;
        Ok(())
    }

    /// Delete this user.
    pub async fn delete(&self, ctx: &impl AppContext) -> DbResult<()> {
        ctx.db().delete_user(self.id).await?;
        Ok(())
    }

    /// Get user ratings: (positive, negative, skipped).
    pub async fn get_ratings(
        &self,
        ctx: &impl AppContext,
    ) -> DbResult<(Vec<i64>, Vec<i64>, Vec<i64>)> {
        ctx.db().get_user_ratings(self.id).await
    }

    /// Add or update a global movie rating.
    pub async fn add_rating(
        &self,
        ctx: &impl AppContext,
        movie_id: i64,
        liked: Option<bool>,
        rating: Option<i32>,
    ) -> DbResult<()> {
        ctx.db().add_rating(self.id, movie_id, liked, rating).await
    }

    /// Get detailed rating for a specific movie.
    pub async fn get_movie_rating(
        &self,
        ctx: &impl AppContext,
        movie_id: i64,
    ) -> DbResult<Option<(Option<bool>, Option<i32>, DateTime<Utc>)>> {
        ctx.db().get_movie_rating(self.id, movie_id).await
    }

    /// Get the user's movie picks in a specific party.
    pub async fn get_party_picks(
        &self,
        ctx: &impl AppContext,
        party_id: Uuid,
    ) -> DbResult<Vec<i64>> {
        ctx.db().get_user_party_picks(party_id, self.id).await
    }

    /// Update user preferences.
    pub async fn update_preferences(
        &self,
        ctx: &impl AppContext,
        new_prefs: crate::repo::user::models::UpdateUserPreferences,
    ) -> DbResult<()> {
        ctx.db()
            .update_user_preferences(self.id, new_prefs)
            .await
            .map(|_| ())
    }

    /// Get included genres for user.
    pub async fn included_genres(&self, ctx: &impl AppContext) -> DbResult<Vec<Uuid>> {
        ctx.db().get_user_include_genres(self.id).await
    }

    /// Add an included genre.
    pub async fn add_included_genre(&self, ctx: &impl AppContext, genre_id: Uuid) -> DbResult<()> {
        ctx.db().add_user_include_genre(self.id, genre_id).await
    }

    /// Remove an included genre.
    pub async fn remove_included_genre(
        &self,
        ctx: &impl AppContext,
        genre_id: Uuid,
    ) -> DbResult<()> {
        ctx.db().remove_user_include_genre(self.id, genre_id).await
    }

    /// Get excluded genres for user.
    pub async fn excluded_genres(&self, ctx: &impl AppContext) -> DbResult<Vec<Uuid>> {
        ctx.db().get_user_exclude_genres(self.id).await
    }

    /// Add an excluded genre.
    pub async fn add_excluded_genre(&self, ctx: &impl AppContext, genre_id: Uuid) -> DbResult<()> {
        ctx.db().add_user_exclude_genre(self.id, genre_id).await
    }

    /// Remove an excluded genre.
    pub async fn remove_excluded_genre(
        &self,
        ctx: &impl AppContext,
        genre_id: Uuid,
    ) -> DbResult<()> {
        ctx.db().remove_user_exclude_genre(self.id, genre_id).await
    }
}
