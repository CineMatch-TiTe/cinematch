use crate::models::PartyState;
use crate::models::movie::MovieData;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use uuid::Uuid;

/// Reason for the timeout deadline
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, ToSchema)]
pub enum TimeoutReason {
    /// Standard phase timeout (voting/watching duration)
    PhaseTimeout,
    /// All members ready → countdown started
    AllReady,
}

/// Party state change with optional timeout info
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct PartyStateChanged {
    pub state: PartyState,
    /// When the timeout deadline is (if any)
    pub deadline_at: Option<DateTime<Utc>>,
    /// Why the timeout was set
    pub timeout_reason: Option<TimeoutReason>,
    /// The selected movie ID, relevant when state transitions to Watching
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_movie_id: Option<i64>,
}

/// Message types sent from server to clients
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub enum ServerMessage {
    RecommendMovie(Box<MovieData>),
    NameChanged(NameChanged),
    PartyLeaderChanged(Uuid),
    PartyMemberJoined(MemberJoined),
    PartyMemberLeft(Uuid),
    PartyStateChanged(PartyStateChanged),
    UpdateReadyState(ReadyStateUpdate),
    ResetReadiness,
    PartyDisbanded,
    PartyMemberRated(PartyMemberRated),

    // Voting phase
    MovieVoteUpdate(MovieVotes),
    /// Emitted when voting round 2 (top 3) starts. Party stays in Voting; clients should refetch ballot.
    VotingRoundStarted(VotingRoundStarted),

    /// Timeout config for the current phase. Sent whenever phase changes (or round 2). Clients use for countdown.
    PartyTimeoutUpdate(PartyTimeoutUpdate),

    /// Optional new code for the party
    PartyCodeChanged(String),
}

/// Timeout info for the current phase.
/// For phase timeouts: includes phase_entered_at and duration (clients compute deadline).
/// For ready countdowns: includes deadline_at directly.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct PartyTimeoutUpdate {
    /// When the phase started (for phase-based countdowns)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_entered_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Active timeout duration in seconds (for the current phase/round)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u32>,
    /// Direct deadline (for ready countdowns or clearing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Why the timeout was set
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<TimeoutReason>,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct VotingRoundStarted {
    pub round: u16,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct MovieVotes {
    pub movie_id: i64,
    pub likes: u32,
    pub dislikes: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct NameChanged {
    pub user_id: Uuid,
    pub new_name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct MemberJoined {
    pub user_id: Uuid,
    pub username: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct ReadyStateUpdate {
    pub user_id: Uuid,
    pub ready: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct PartyMemberRated {
    pub user_id: Uuid,
    pub rating: i32,
    pub party_average: f32,
}
