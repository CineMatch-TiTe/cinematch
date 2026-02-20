//! Vote-related database models.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::{shown_movies, votes};

// ============================================================================
// Shown Movie Models (tracking what movies were shown to users)
// ============================================================================

#[derive(Debug, Clone, Queryable, Identifiable, Associations)]
#[diesel(table_name = shown_movies)]
#[diesel(primary_key(party_id, user_id, movie_id))]
#[diesel(belongs_to(crate::repo::party::models::Party, foreign_key = party_id))]
#[diesel(belongs_to(crate::repo::user::models::User, foreign_key = user_id))]
#[diesel(belongs_to(crate::repo::movie::models::Movie, foreign_key = movie_id))]
pub struct ShownMovie {
    pub party_id: Uuid,
    pub user_id: Uuid,
    pub movie_id: i64,
    pub shown_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = shown_movies)]
pub struct NewShownMovie {
    pub party_id: Uuid,
    pub user_id: Uuid,
    pub movie_id: i64,
}

// ============================================================================
// Vote Models
// ============================================================================

#[derive(Debug, Clone, Queryable, Identifiable, Associations)]
#[diesel(table_name = votes)]
#[diesel(primary_key(party_id, user_id, movie_id))]
#[diesel(belongs_to(crate::repo::party::models::Party, foreign_key = party_id))]
#[diesel(belongs_to(crate::repo::user::models::User, foreign_key = user_id))]
#[diesel(belongs_to(crate::repo::movie::models::Movie, foreign_key = movie_id))]
pub struct Vote {
    pub party_id: Uuid,
    pub user_id: Uuid,
    pub movie_id: i64,
    pub vote_value: bool,
    pub voted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = votes)]
pub struct NewVote {
    pub party_id: Uuid,
    pub user_id: Uuid,
    pub movie_id: i64,
    pub vote_value: bool,
}
