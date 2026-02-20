use utoipa::ToSchema;

#[derive(Debug, serde::Serialize, serde::Deserialize, ToSchema)]
pub enum ClientMessage {
    VoteMovie(VoteMovie),
    ChangeName(String),
    SetReadyState(bool),
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct VoteMovie {
    pub movie_id: i64, // we're using tmdb ids
    pub vote: bool,    // true = like, false = dislike
}
