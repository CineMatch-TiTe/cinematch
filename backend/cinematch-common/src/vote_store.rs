use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Default)]
pub struct VoteStore {
    // key is party_id
    votes: Mutex<HashMap<Uuid, HashMap<i64, MovieVote>>>,
}

impl VoteStore {
    pub fn new() -> Self {
        Self {
            votes: Mutex::new(HashMap::new()),
        }
    }

    /// Add a movie to the party's vote store
    pub fn add_movie_to_party(&self, party_id: Uuid, movie_id: i64) -> Result<(), ()> {
        let mut votes_lock = self.votes.lock().map_err(|_| ())?;
        let party_votes = votes_lock.entry(party_id).or_insert_with(HashMap::new);
        party_votes.entry(movie_id).or_insert_with(|| MovieVote::new());
        Ok(())
    }

    /// Cast a vote for a movie in a party by a user
    pub fn cast_vote(
        &self,
        party_id: Uuid,
        user_id: Uuid,
        movie_id: i64,
        vote: bool,
    ) -> Result<(), ()> {
        let mut votes_lock = self.votes.lock().map_err(|_| ())?;
        let party_votes = votes_lock.entry(party_id).or_insert_with(HashMap::new);
        if let Some(movie_vote) = party_votes.get_mut(&movie_id) {
            movie_vote.cast_vote(user_id, vote);
            Ok(())
        } else {
            // Movie not present in party
            Err(())
        }
    }

    /// Get a user's vote for a movie in a party
    pub fn get_user_vote(
        &self,
        party_id: Uuid,
        user_id: Uuid,
        movie_id: i64,
    ) -> Result<Option<bool>, ()> {
        let votes_lock = self.votes.lock().map_err(|_| ())?;
        Ok(votes_lock
            .get(&party_id)
            .and_then(|movie_votes| {
                movie_votes
                    .get(&movie_id)
                    .map(|mv| mv.get_user_vote(&user_id))
            })
            .flatten())
    }

    /// Get total likes and dislikes for a movie in a party
    pub fn get_movie_totals(
        &self,
        party_id: Uuid,
        movie_id: i64,
    ) -> Result<Option<(u32, u32)>, ()> {
        let votes_lock = self.votes.lock().map_err(|_| ())?;
        Ok(votes_lock
            .get(&party_id)
            .and_then(|movie_votes| movie_votes.get(&movie_id).map(|mv| mv.get_totals())))
    }

    pub fn get_party_votes(
        &self,
        party_id: Uuid,
    ) -> Result<Option<HashMap<i64, (u32, u32)>>, ()> {
        let votes_lock = self.votes.lock().map_err(|_| ())?;
        Ok(votes_lock.get(&party_id).map(|movie_votes| {
            movie_votes
                .iter()
                .map(|(movie_id, mv)| (*movie_id, mv.get_totals()))
                .collect()
        }))
    }

    /// Cleanup votes for a party
    pub fn cleanup_party(&self, party_id: Uuid) -> Result<(), ()> {
        let mut votes_lock = self.votes.lock().map_err(|_| ())?;
        votes_lock.remove(&party_id);
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct MovieVote {
    pub(crate) total_likes: u32,
    pub(crate) total_dislikes: u32,
    pub(crate) user_votes: HashMap<Uuid, bool>, // key is user_id, value is vote (true=like, false=dislike)
}

impl MovieVote {
    pub(crate) fn new() -> Self {
        Self {
            total_likes: 0,
            total_dislikes: 0,
            user_votes: HashMap::new(),
        }
    }

    pub(crate) fn cast_vote(&mut self, user_id: Uuid, vote: bool) {
        if let Some(previous_vote) = self.user_votes.insert(user_id, vote) {
            // User has voted before, adjust counts
            if previous_vote {
                self.total_likes -= 1;
            } else {
                self.total_dislikes -= 1;
            }
        }

        // Add new vote
        if vote {
            self.total_likes += 1;
        } else {
            self.total_dislikes += 1;
        }
    }

    pub(crate) fn get_user_vote(&self, user_id: &Uuid) -> Option<bool> {
        self.user_votes.get(user_id).cloned()
    }

    pub(crate) fn get_totals(&self) -> (u32, u32) {
        (self.total_likes, self.total_dislikes)
    }
}
