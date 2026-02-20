//! User extension trait for business logic.

use cinematch_db::domain::User;
// UUID imported via cinematch_db::prelude

use super::DomainError;
use async_trait::async_trait;
use cinematch_common::models::websocket::{NameChanged, ServerMessage};

/// Type alias for user rating data: (liked, score, timestamp)
pub type UserRating = Option<(Option<bool>, Option<i32>, chrono::DateTime<chrono::Utc>)>;

/// Extension trait for User business logic.
/// Extension trait for User business logic.
#[async_trait]
pub trait UserLogic {
    /// Rename the user (validates name length).
    async fn rename(
        &self,
        ctx: &impl cinematch_db::AppContext,
        new_name: &str,
    ) -> Result<(), DomainError>;

    /// Check if user is in any party.
    async fn is_in_party(&self, ctx: &impl cinematch_db::AppContext) -> Result<bool, DomainError>;

    /// Set user global star rating (0-10).
    async fn rate_movie(
        &self,
        ctx: &impl cinematch_db::AppContext,
        movie_id: i64,
        rating: i32,
    ) -> Result<(), DomainError>;

    /// Update user global rating (liked status and/or 0-10 score).
    async fn update_rating(
        &self,
        ctx: &impl cinematch_db::AppContext,
        movie_id: i64,
        liked: Option<bool>,
        rating: Option<i32>,
    ) -> Result<(), DomainError>;

    /// Get user global rating for a specific movie.
    async fn get_rating(
        &self,
        ctx: &impl cinematch_db::AppContext,
        movie_id: i64,
    ) -> Result<UserRating, DomainError>;
}

#[async_trait]
#[async_trait]
impl UserLogic for User {
    async fn rename(
        &self,
        ctx: &impl cinematch_db::AppContext,
        new_name: &str,
    ) -> Result<(), DomainError> {
        let name = validate_username(new_name)?;
        self.set_username(ctx, &name)
            .await
            .map_err(DomainError::from)?;

        // Broadcast if in party
        if let Ok(Some(party)) = self.current_party(ctx).await {
            // We need party ID. Party object has it.
            // We need to broadcast to party members.
            if let Ok(members) = party.member_ids(ctx).await {
                let msg = ServerMessage::NameChanged(NameChanged {
                    user_id: self.id,
                    new_name: name,
                });
                ctx.send_users(&members, &msg);
            }
        }
        Ok(())
    }

    async fn is_in_party(&self, ctx: &impl cinematch_db::AppContext) -> Result<bool, DomainError> {
        let party = self.current_party(ctx).await.map_err(DomainError::from)?;
        Ok(party.is_some())
    }

    async fn update_rating(
        &self,
        ctx: &impl cinematch_db::AppContext,
        movie_id: i64,
        liked: Option<bool>,
        rating: Option<i32>,
    ) -> Result<(), DomainError> {
        self.add_rating(ctx, movie_id, liked, rating)
            .await
            .map_err(DomainError::from)
    }

    async fn rate_movie(
        &self,
        ctx: &impl cinematch_db::AppContext,
        movie_id: i64,
        rating: i32,
    ) -> Result<(), DomainError> {
        self.add_rating(ctx, movie_id, None, Some(rating))
            .await
            .map_err(DomainError::from)
    }

    async fn get_rating(
        &self,
        ctx: &impl cinematch_db::AppContext,
        movie_id: i64,
    ) -> Result<UserRating, DomainError> {
        self.get_movie_rating(ctx, movie_id)
            .await
            .map_err(DomainError::from)
    }
}

/// Validate username length (3-32 chars, trimmed, no control chars).
fn validate_username(name: &str) -> Result<String, DomainError> {
    let name: String = name.trim().chars().filter(|c| !c.is_control()).collect();

    if name.len() < cinematch_common::NAME_MIN_LENGTH
        || name.len() > cinematch_common::NAME_MAX_LENGTH
    {
        return Err(DomainError::BadRequest(format!(
            "Username must be between {} and {} characters",
            cinematch_common::NAME_MIN_LENGTH,
            cinematch_common::NAME_MAX_LENGTH
        )));
    }

    Ok(name)
}

/// Helper extension for User creation (static-like).
/// Helper extension for User creation (static-like).
#[async_trait]
pub trait UserCreation {
    /// Create a guest user with a checked/generated name.
    async fn create_guest_checked(
        ctx: std::sync::Arc<dyn cinematch_db::AppContext>,
        username: Option<String>,
    ) -> Result<User, DomainError>;

    /// Generate a random guest name.
    fn generate_guest_name() -> String;
}

#[async_trait]
impl UserCreation for User {
    async fn create_guest_checked(
        ctx: std::sync::Arc<dyn cinematch_db::AppContext>,
        username: Option<String>,
    ) -> Result<User, DomainError> {
        let name = match username {
            Some(n) => validate_username(&n)?,
            None => Self::generate_guest_name(),
        };

        User::create_guest(&ctx, &name)
            .await
            .map_err(DomainError::from)
    }

    fn generate_guest_name() -> String {
        let random_suffix = uuid::Uuid::new_v4()
            .to_string()
            .chars()
            .take(8)
            .collect::<String>();
        format!("guest_{}", random_suffix)
    }
}
