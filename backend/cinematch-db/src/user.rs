//! User database operations

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::models::{NewUser, UpdateUser, User};
use crate::schema;
use crate::{validate_username, Database, DbError, DbResult};

impl Database {
    /// Create a new oneshot (temporary) user
    pub async fn create_oneshot_user(&self, username: &str) -> DbResult<User> {
        use schema::users;

        validate_username(username)?;

        let new_user = NewUser {
            username,
            oneshot: true,
        };

        let mut conn = self.conn().await?;
        diesel::insert_into(users::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(DbError::from)
    }

    /// Create a new guest user (alias for create_oneshot_user for API clarity)
    pub async fn create_guest_user(&self, username: &str) -> DbResult<User> {
        self.create_oneshot_user(username).await
    }

    /// Create a new persistent user (must link external account after)
    pub async fn create_persistent_user(&self, username: &str) -> DbResult<User> {
        use schema::users;

        validate_username(username)?;

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

    /// Get a user by username
    pub async fn get_user_by_username(&self, name: &str) -> DbResult<Option<User>> {
        use schema::users::dsl::*;

        let mut conn = self.conn().await?;
        users
            .filter(username.eq(name))
            .select(User::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(DbError::from)
    }

    /// Update a user
    pub async fn update_user(&self, user_id: Uuid, update: UpdateUser<'_>) -> DbResult<User> {
        use schema::users::dsl::*;

        // Validate username if it's being updated
        if let Some(new_username) = update.username {
            validate_username(new_username)?;
        }

        let mut conn = self.conn().await?;
        diesel::update(users.find(user_id))
            .set(&update)
            .returning(User::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(DbError::from)
    }

    /// Update a user's username
    pub async fn update_user_username(&self, user_id: Uuid, new_username: &str) -> DbResult<User> {
        use schema::users::dsl::*;

        validate_username(new_username)?;

        let update = UpdateUser {
            username: Some(new_username),
            oneshot: None,
        };

        let mut conn = self.conn().await?;
        diesel::update(users.find(user_id))
            .set(&update)
            .returning(User::as_returning())
            .get_result(&mut conn)
            .await
            .optional()?
            .ok_or(DbError::UserNotFound(user_id))
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
