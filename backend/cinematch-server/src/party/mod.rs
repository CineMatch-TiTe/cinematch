pub mod crud;
pub mod leader_ops;
pub mod picks;
pub mod user_ops;
pub mod votes;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use std::collections::HashMap;

// Re-export types that are used in responses
pub use crate::AppState;
pub use cinematch_db::PartyState;
// ============================================================================
// Party Responses
// ============================================================================

/// Response when creating a new party
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreatePartyResponse {
    /// The unique party ID
    pub party_id: Uuid,
    /// The 4-character join code (e.g., "A1B2")
    pub code: String,
    /// When the party was created
    pub created_at: DateTime<Utc>,
}

/// Response with party details
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PartyResponse {
    /// The unique party ID
    pub id: Uuid,
    /// The party leader's user ID
    pub leader_id: Uuid,
    /// Current state of the party
    pub state: PartyStateDto,
    /// When the party was created
    pub created_at: DateTime<Utc>,
    /// The join code (only present in Created state)
    pub code: Option<String>,
    /// Vote totals per movie (only in Voting): movie_id → (likes, dislikes)
    pub vote_status: Option<HashMap<i64, (u32, u32)>>,
    /// Selected winner movie ID (set when voting ends with a majority; in Watching/Review).
    pub selected_movie_id: Option<i64>,
    /// When the party entered the current phase (Voting, Watching, etc.). Used with timeout secs for client countdown.
    pub phase_entered_at: DateTime<Utc>,
    /// Voting phase auto-end timeout (seconds). From VOTING_TIMEOUT_SECS.
    pub voting_timeout_secs: u32,
    /// Watching phase auto-advance timeout (seconds). From WATCHING_TIMEOUT_SECS.
    pub watching_timeout_secs: u32,
}

/// Party state for API responses
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PartyStateDto {
    /// Initial state, people can join
    Created,
    /// People are picking movies (taste), no new joins
    Picking,
    /// Voting on picked movies
    Voting,
    /// Movie is being watched
    Watching,
    /// Review phase after watching
    Review,
    /// Party has ended
    Disbanded,
}

impl From<PartyState> for PartyStateDto {
    fn from(state: PartyState) -> Self {
        match state {
            PartyState::Created => PartyStateDto::Created,
            PartyState::Picking => PartyStateDto::Picking,
            PartyState::Voting => PartyStateDto::Voting,
            PartyState::Watching => PartyStateDto::Watching,
            PartyState::Review => PartyStateDto::Review,
            PartyState::Disbanded => PartyStateDto::Disbanded,
        }
    }
}

// ============================================================================
// Member Models
// ============================================================================

/// Information about a party member
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MemberInfo {
    /// The user's unique ID
    pub user_id: Uuid,
    /// The user's display name
    pub username: String,
    /// Whether this member is the party leader
    pub is_leader: bool,
    /// Whether this member has marked themselves as ready
    pub is_ready: bool,
    /// When the member joined the party
    pub joined_at: DateTime<Utc>,
}

/// Response with list of party members
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PartyMembersResponse {
    /// List of all members in the party
    pub members: Vec<MemberInfo>,
    /// Total count of members
    pub count: usize,
    /// Number of members who are ready
    pub ready_count: usize,
    /// Whether all members are ready
    pub all_ready: bool,
}

// ============================================================================
// Ready State Models
// ============================================================================

/// Response after toggling ready state
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ReadyStateResponse {
    /// Whether all members are now ready
    pub all_ready: bool,
}

// ============================================================================
// Vote Models
// ============================================================================

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VoteMovieResponse {
    /// Current vote totals for the movie
    pub likes: u32,
    pub dislikes: u32,
}

/// Per-movie vote totals (for GET /vote)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VoteTotals {
    pub likes: u32,
    pub dislikes: u32,
}

/// Response for GET /api/party/{party_id}/picks: your picks (movie IDs) for this party.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({"movie_ids": [12345, 67890]}))]
pub struct GetPicksResponse {
    /// TMDB movie IDs you have picked. Empty when not in Created/Picking.
    pub movie_ids: Vec<i64>,
}

/// Response for GET /api/party/{party_id}/vote: ballot and vote info for the current user.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GetVoteResponse {
    /// Movie IDs the user can vote on (their ballot). Use on refresh / poll.
    pub movie_ids: Vec<i64>,
    /// 1 = first round, 2 = top-3 round. None when not in Voting.
    pub voting_round: Option<i16>,
    pub can_vote: bool,
    /// Totals per movie (keys = movie_ids). Omitted when empty.
    pub vote_totals: std::collections::HashMap<i64, VoteTotals>,
}

// ============================================================================
// Phase Control Models (Leader Only)
// ============================================================================

/// Response after advancing to next phase
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PhaseAdvanceResponse {
    /// New party state after advancing
    pub new_state: PartyStateDto,
}

/// Response after starting a new round
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NewRoundResponse {
    /// New join code for the round
    pub code: String,
}

/// Response after ending voting (round 2 started, winner selected, or no winner -> Picking)
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EndVotingResponse {
    /// True when round 2 (top 3) has started
    pub round_2_started: bool,
    /// Set when a winner was selected and party moved to Watching
    pub winner_movie_id: Option<i64>,
    /// True when no 50%+ winner; party moved back to Picking, tastes updated from round-2 votes
    pub next_round_picking: bool,
}

// ============================================================================
// Status Response
// ============================================================================

/// Generic status response
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StatusResponse {
    /// Status message
    pub status: String,
}

#[allow(dead_code)]
impl StatusResponse {
    pub fn ok() -> Self {
        Self {
            status: "ok".to_string(),
        }
    }

    pub fn new(status: impl Into<String>) -> Self {
        Self {
            status: status.into(),
        }
    }
}
#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct OptionalIdParam {
    /// Optional resource ID (UUID). If omitted, the system auto-selects from session.
    pub id: Option<Uuid>,
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct MovieIdQuery {
    pub movie_id: i64,
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct JoinQuery {
    pub code: String,
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct ReadyQuery {
    pub is_ready: bool,
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct KickQuery {
    pub target_user_id: Uuid,
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct TransferQuery {
    pub new_leader_id: Uuid,
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct VoteQuery {
    pub movie_id: i64,
    pub like: bool,
}
