use utoipa::ToSchema;
use uuid::Uuid;

use cinematch_db::PartyState;

/// Message types sent from server to clients
#[derive(Debug, serde::Serialize, serde::Deserialize, ToSchema)]
pub enum ServerMessage {
    NameChanged(NameChanged),
    PartyLeaderChanged(Uuid),
    PartyMemberJoined(MemberJoined),
    PartyMemberLeft(Uuid),
    PartyStateChanged(PartyState),
    UpdateReadyState(ReadyStateUpdate),
    PartyDisbanded,

    // Voting phase
    MovieVoteUpdate(MovieVotes),
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct MovieVotes {
    pub movie_id: i64,
    pub likes: u32,
    pub dislikes: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct NameChanged {
    pub user_id: Uuid,
    pub new_name: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct MemberJoined {
    pub user_id: Uuid,
    pub username: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct ReadyStateUpdate {
    pub user_id: Uuid,
    pub ready: bool,
}

/// Message types sent from clients to server
#[derive(Debug, serde::Serialize, serde::Deserialize, ToSchema)]
pub enum ClientMessage {
    VoteMovie(VoteMovie),
    ChangeName(String),
    SetReadyState(bool),
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct VoteMovie {
    pub movie_id: i64, //we're using tmdb ids
    pub vote: bool,    // true = like, false = dislike
}
