//! Schedule-related database models.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::schedules;

// ============================================================================
// Enums
// ============================================================================

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    diesel_derive_enum::DbEnum,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
#[ExistingTypePath = "crate::schema::sql_types::TimeoutType"]
pub enum TimeoutType {
    VotingStarting,
    VotingEnding,
    WatchingEnding,
    ReadyTimeout,
}

impl From<TimeoutType> for cinematch_common::models::TimeoutType {
    fn from(t: TimeoutType) -> Self {
        match t {
            TimeoutType::VotingStarting => cinematch_common::models::TimeoutType::VotingStarting,
            TimeoutType::VotingEnding => cinematch_common::models::TimeoutType::VotingEnding,
            TimeoutType::WatchingEnding => cinematch_common::models::TimeoutType::WatchingEnding,
            TimeoutType::ReadyTimeout => cinematch_common::models::TimeoutType::ReadyTimeout,
        }
    }
}

impl From<cinematch_common::models::TimeoutType> for TimeoutType {
    fn from(t: cinematch_common::models::TimeoutType) -> Self {
        match t {
            cinematch_common::models::TimeoutType::VotingStarting => TimeoutType::VotingStarting,
            cinematch_common::models::TimeoutType::VotingEnding => TimeoutType::VotingEnding,
            cinematch_common::models::TimeoutType::WatchingEnding => TimeoutType::WatchingEnding,
            cinematch_common::models::TimeoutType::ReadyTimeout => TimeoutType::ReadyTimeout,
        }
    }
}

// ============================================================================
// Schedule Models
// ============================================================================

/// Queryable Schedule from database
#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schedules)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Schedule {
    pub id: Uuid,
    pub party_id: Option<Uuid>,
    pub timeout_type: TimeoutType,
    pub created_at: DateTime<Utc>,
    pub execute_at: DateTime<Utc>,
}

/// For inserting a new schedule
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = schedules)]
pub struct NewSchedule {
    pub party_id: Option<Uuid>,
    pub timeout_type: TimeoutType,
    pub execute_at: DateTime<Utc>,
}

/// For updating a schedule (not used - schedules are deleted when executed/cancelled)
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = schedules)]
pub struct UpdateSchedule {
    // when the timeout is scheduled to execute
    pub execute_at: DateTime<Utc>,
}
