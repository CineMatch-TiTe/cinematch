use crate::models::{NewUserPreferences, UserPreferences};

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::models::{NewUser, UpdateUser, User};
use crate::{Database, DbError, DbResult};
use crate::schema;

use uuid::Uuid;

impl Database {
    /// Create user preferences (now only handles year/flex)
    pub async fn create_user_preferences(
        &self,
        new_prefs: NewUserPreferences,
    ) -> DbResult<UserPreferences> {
        use crate::schema::user_preferences;
        let mut conn = self.conn().await?;
        diesel::insert_into(user_preferences::table)
            .values(&new_prefs)
            .returning(UserPreferences::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(DbError::from)
    }

    /// Create a new oneshot (temporary) user
    pub async fn create_oneshot_user(&self, username: &str) -> DbResult<User> {
        use schema::users;

        let new_user = NewUser {
            username,
            oneshot: true,
        };

        let mut conn = self.conn().await?;
        let user = diesel::insert_into(users::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(DbError::from)?;

        // Create default user preferences

        let prefs = NewUserPreferences {
            user_id: user.id,
            target_release_year: None,
            release_year_flex: 0,
            is_tite: false,
        };
        self.create_user_preferences(prefs).await?;
        Ok(user)
    }

    /// Create a new guest user (alias for create_oneshot_user for API clarity)
    pub async fn create_guest_user(&self, username: &str) -> DbResult<User> {
        self.create_oneshot_user(username).await
    }

    /// Create a new persistent user (must link external account after)
    pub async fn create_persistent_user(&self, username: &str) -> DbResult<User> {
        use schema::users;

        let new_user = NewUser {
            username,
            oneshot: false,
        };

        let mut conn = self.conn().await?;
        diesel::insert_into(users::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(DbError::from)
    }

    /// Get a user by ID
    pub async fn get_user(&self, user_id: Uuid) -> DbResult<User> {
        use schema::users::dsl::*;

        let mut conn = self.conn().await?;
        users
            .find(user_id)
            .select(User::as_select())
            .first(&mut conn)
            .await
            .optional()?
            .ok_or(DbError::UserNotFound(user_id))
    }

    /// Get a users party which they are in (this can only return one ongoing party).
    /// Excludes Disbanded parties so WS and "my party" never use disbanded parties.
    pub async fn get_user_active_party(&self, user_id: Uuid) -> DbResult<Uuid> {
        use crate::models::PartyState;
        use schema::parties::dsl as p;
        use schema::party_members::dsl as pm;

        let mut conn = self.conn().await?;
        pm::party_members
            .inner_join(p::parties.on(p::id.eq(pm::party_id)))
            .filter(pm::user_id.eq(user_id))
            .filter(p::state.ne(PartyState::Disbanded))
            .select(p::id)
            .first::<Uuid>(&mut conn)
            .await
            .optional()?
            .ok_or(DbError::UserNotInParty(user_id))
    }

    /// Update a user
    pub async fn update_user(&self, user_id: Uuid, update: UpdateUser<'_>) -> DbResult<User> {
        use schema::users::dsl::*;

        let mut conn = self.conn().await?;
        diesel::update(users.find(user_id))
            .set(&update)
            .returning(User::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(DbError::from)
    }

    /// Delete a user
    pub async fn delete_user(&self, user_id: Uuid) -> DbResult<usize> {
        use schema::users::dsl::*;

        let mut conn = self.conn().await?;
        diesel::delete(users.find(user_id))
            .execute(&mut conn)
            .await
            .map_err(DbError::from)
    }
}
