use crate::models::{UpdateUserPreferences, UserPreferences};

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{Database, DbError, DbResult};
use cinematch_common::FullUserPreferences;

impl Database {
    /// Get user preferences
    pub async fn get_user_preferences(&self, _user_id: Uuid) -> DbResult<FullUserPreferences> {
        use crate::schema::user_preferences::dsl::*;

        let mut conn = self.conn().await?;
        let prefs = user_preferences
            .find(_user_id)
            .first::<UserPreferences>(&mut conn)
            .await
            .map_err(DbError::from)?;

        // now add include exclude to this
        let include = self.get_user_include_genres(_user_id).await?;
        let exclude = self.get_user_exclude_genres(_user_id).await?;
        Ok(FullUserPreferences {
            user_id: _user_id,
            preferred_year: prefs.target_release_year,
            year_flexibility: prefs.release_year_flex,
            included_genres: include,
            excluded_genres: exclude,
            is_tite: prefs.is_tite,
            updated_at: prefs.updated_at,
            created_at: prefs.created_at,
        })
    }

    /// Update user preferences (now only handles year/flex)
    pub async fn update_user_preferences(
        &self,
        _user_id: Uuid,
        update: UpdateUserPreferences,
    ) -> DbResult<UserPreferences> {
        use crate::schema::user_preferences::dsl::*;
        let mut conn = self.conn().await?;
        diesel::update(user_preferences.find(_user_id))
            .set(&update)
            .returning(UserPreferences::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(DbError::from)
    }

    // --- Genre preference join table CRUD stubs ---

    /// Add an included genre for a user
    pub async fn add_user_include_genre(&self, _user_id: Uuid, _genre_id: Uuid) -> DbResult<()> {
        use crate::models::NewPrefsIncludeGenre;
        use crate::schema::prefs_include_genre;
        let mut conn = self.conn().await?;
        diesel::insert_into(prefs_include_genre::table)
            .values(&NewPrefsIncludeGenre {
                user_id: _user_id,
                genre_id: _genre_id,
            })
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    /// Remove an included genre for a user
    pub async fn remove_user_include_genre(&self, _user_id: Uuid, _genre_id: Uuid) -> DbResult<()> {
        use crate::schema::prefs_include_genre::dsl::*;
        let mut conn = self.conn().await?;
        diesel::delete(
            prefs_include_genre.filter(user_id.eq(_user_id).and(genre_id.eq(_genre_id))),
        )
        .execute(&mut conn)
        .await?;
        Ok(())
    }

    /// Add an excluded genre for a user
    pub async fn add_user_exclude_genre(&self, _user_id: Uuid, _genre_id: Uuid) -> DbResult<()> {
        use crate::models::NewPrefsExcludeGenre;
        use crate::schema::prefs_exclude_genre;
        let mut conn = self.conn().await?;
        diesel::insert_into(prefs_exclude_genre::table)
            .values(&NewPrefsExcludeGenre {
                user_id: _user_id,
                genre_id: _genre_id,
            })
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    /// Remove an excluded genre for a user
    pub async fn remove_user_exclude_genre(&self, _user_id: Uuid, _genre_id: Uuid) -> DbResult<()> {
        use crate::schema::prefs_exclude_genre::dsl::*;
        let mut conn = self.conn().await?;
        diesel::delete(
            prefs_exclude_genre.filter(user_id.eq(_user_id).and(genre_id.eq(_genre_id))),
        )
        .execute(&mut conn)
        .await?;
        Ok(())
    }

    /// Get included genres for a user
    pub async fn get_user_include_genres(&self, _user_id: Uuid) -> DbResult<Vec<Uuid>> {
        use crate::schema::prefs_include_genre::dsl::*;
        let mut conn = self.conn().await?;
        let gids = prefs_include_genre
            .filter(user_id.eq(_user_id))
            .select(genre_id)
            .load::<Uuid>(&mut conn)
            .await
            .map_err(DbError::from)?;
        Ok(gids)
    }
    /// Get excluded genres for a user
    pub async fn get_user_exclude_genres(&self, _user_id: Uuid) -> DbResult<Vec<Uuid>> {
        use crate::schema::prefs_exclude_genre::dsl::*;
        let mut conn = self.conn().await?;
        let gids = prefs_exclude_genre
            .filter(user_id.eq(_user_id))
            .select(genre_id)
            .load::<Uuid>(&mut conn)
            .await
            .map_err(DbError::from)?;
        Ok(gids)
    }
}
