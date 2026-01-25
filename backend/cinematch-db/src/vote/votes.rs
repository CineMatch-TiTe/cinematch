use crate::models::{Vote, NewVote, NewShownMovie};
use crate::schema::{votes, shown_movies};
use crate::{Database, DbError, DbResult};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use std::collections::HashMap;

impl Database {
    /// Add movies to shown_movies for a user in a party
    /// User can only vote for movies in this list
    pub async fn add_shown_movies(&self, party_id: Uuid, user_id: Uuid, movie_ids: &[i64]) -> DbResult<()> {
        let mut conn = self.conn().await?;
        let new_shown: Vec<NewShownMovie> = movie_ids.iter().map(|&movie_id| NewShownMovie {
            party_id,
            user_id,
            movie_id,
        }).collect();
        diesel::insert_into(shown_movies::table)
            .values(&new_shown)
            .on_conflict_do_nothing()
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    /// Check if a user can vote for a movie in a party
    pub async fn can_vote(&self, party_id: Uuid, user_id: Uuid, movie_id: i64) -> DbResult<bool> {
        use crate::schema::parties::dsl as parties_dsl;
        let mut conn = self.conn().await?;
        // Check if voting is enabled for the party
        let can_vote_party = parties_dsl::parties
            .filter(parties_dsl::id.eq(party_id))
            .select(parties_dsl::can_vote)
            .first::<bool>(&mut conn)
            .await?;
        if !can_vote_party {
            return Ok(false);
        }
        let exists = shown_movies::table
            .filter(shown_movies::party_id.eq(party_id))
            .filter(shown_movies::user_id.eq(user_id))
            .filter(shown_movies::movie_id.eq(movie_id))
            .select(shown_movies::movie_id)
            .first::<i64>(&mut conn)
            .await
            .optional()?;
        Ok(exists.is_some())
    }

    /// Cast a vote (insert or update)
    pub async fn cast_vote(&self, party_id: Uuid, user_id: Uuid, movie_id: i64, vote_value: bool) -> DbResult<Vote> {
        if !self.can_vote(party_id, user_id, movie_id).await? {
            return Err(DbError::Other("User cannot vote for this movie".into()));
        }
        let mut conn = self.conn().await?;
        let new_vote = NewVote { party_id, user_id, movie_id, vote_value };
        let vote = diesel::insert_into(votes::table)
            .values(&new_vote)
            .on_conflict((votes::party_id, votes::user_id, votes::movie_id))
            .do_update()
            .set(votes::vote_value.eq(vote_value))
            .get_result::<Vote>(&mut conn)
            .await?;
        Ok(vote)
    }

    /// Get like/dislike totals for a movie in a party (or all parties if party_id is None)
    pub async fn get_vote_totals(&self, movie_id: i64, party_id: Option<Uuid>) -> DbResult<(i64, i64)> {
        use diesel::dsl::{count_star};
        let mut conn = self.conn().await?;
        let mut base = votes::table.filter(votes::movie_id.eq(movie_id)).into_boxed();
        if let Some(pid) = party_id {
            base = base.filter(votes::party_id.eq(pid));
        }
        let likes = base
            .filter(votes::vote_value.eq(true))
            .select(count_star())
            .first(&mut conn)
            .await?;
        let mut base = votes::table.filter(votes::movie_id.eq(movie_id)).into_boxed();
        if let Some(pid) = party_id {
            base = base.filter(votes::party_id.eq(pid));
        }
        let dislikes = base
            .filter(votes::vote_value.eq(false))
            .select(count_star())
            .first(&mut conn)
            .await?;
        Ok((likes, dislikes))
    }

    /// Get all votes for a user in a party
    pub async fn get_user_votes(&self, party_id: Uuid, user_id: Uuid) -> DbResult<Vec<Vote>> {
        let mut conn = self.conn().await?;
        let votes = votes::table
            .filter(votes::party_id.eq(party_id))
            .filter(votes::user_id.eq(user_id))
            .load::<Vote>(&mut conn)
            .await?;
        Ok(votes)
    }

    /// Get all votes in a party, hashmap of movie_id -> (likes, dislikes)
    /// If user_id is Some, only include movies the user can vote for
    pub async fn get_party_votes(&self, party_id: Uuid, user_id: Option<Uuid>) -> DbResult<HashMap<i64, (u32, u32)>> {
        let mut conn = self.conn().await?;
        let votes = votes::table
            .filter(votes::party_id.eq(party_id))
            .select((votes::movie_id, votes::vote_value))
            .load::<(i64, bool)>(&mut conn)
            .await?;

        let allowed_movies: Option<std::collections::HashSet<i64>> = if let Some(uid) = user_id {
            use crate::schema::shown_movies;
            let movie_ids = shown_movies::table
                .filter(shown_movies::party_id.eq(party_id))
                .filter(shown_movies::user_id.eq(uid))
                .select(shown_movies::movie_id)
                .load::<i64>(&mut conn)
                .await?;
            Some(movie_ids.into_iter().collect())
        } else {
            None
        };

        let mut vote_map: HashMap<i64, (u32, u32)> = HashMap::new();
        for (movie_id, vote_value) in votes {
            if let Some(ref allowed) = allowed_movies {
                if !allowed.contains(&movie_id) {
                    continue;
                }
            }
            let entry = vote_map.entry(movie_id).or_insert((0, 0));
            if vote_value {
                entry.0 += 1;
            } else {
                entry.1 += 1;
            }
        }
        Ok(vote_map)
    }

    pub async fn enable_voting(&self, party_id: Uuid) -> DbResult<()> {
        use crate::schema::parties::dsl::*;
        let mut conn = self.conn().await?;
        diesel::update(parties.filter(id.eq(party_id)))
            .set(can_vote.eq(true))
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn disable_voting(&self, party_id: Uuid) -> DbResult<()> {
        use crate::schema::parties::dsl::*;
        let mut conn = self.conn().await?;
        diesel::update(parties.filter(id.eq(party_id)))
            .set(can_vote.eq(false))
            .execute(&mut conn)
            .await?;
        Ok(())
    }
}
