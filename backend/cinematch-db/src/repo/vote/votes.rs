use crate::models::{NewShownMovie, NewVote, Vote};
use crate::schema::{shown_movies, votes};
use crate::{Database, DbError, DbResult};
use diesel::AggregateExpressionMethods;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use rand::seq::SliceRandom;
use uuid::Uuid;

use std::collections::{HashMap, HashSet};

const BALLOT_SIZE: usize = 5;
const OWN_PICKS: usize = 2;
const OTHERS_PICKS: usize = 3;
const POPULAR_FALLBACK_LIMIT: i64 = 30;

impl Database {
    /// Clear shown_movies and votes for a party. Call before building new ballots.
    pub(crate) async fn clear_shown_movies_for_party(&self, party_id: Uuid) -> DbResult<()> {
        use crate::schema::shown_movies::dsl as shown_dsl;
        use crate::schema::votes::dsl as votes_dsl;
        use diesel_async::AsyncConnection;
        use diesel_async::scoped_futures::ScopedFutureExt;
        let mut conn = self.conn().await?;

        conn.transaction::<(), DbError, _>(|conn| {
            async move {
                diesel::delete(shown_dsl::shown_movies.filter(shown_dsl::party_id.eq(party_id)))
                    .execute(conn)
                    .await?;
                diesel::delete(votes_dsl::votes.filter(votes_dsl::party_id.eq(party_id)))
                    .execute(conn)
                    .await?;
                Ok(())
            }
            .scope_boxed()
        })
        .await?;

        Ok(())
    }
    /// Build per-user ballots when entering Voting: 5 movies each (2 own, 3 others when possible).
    /// Clears existing shown_movies for the party, then inserts new ballots and enables voting.
    pub(crate) async fn build_voting_ballots(&self, party_id: Uuid) -> DbResult<()> {
        self.clear_shown_movies_for_party(party_id).await?;

        let members = self.get_party_members(party_id).await?;
        if members.is_empty() {
            return Ok(());
        }

        let party_taste = self.get_party_picks(party_id).await?;
        let mut picks_by_user: HashMap<Uuid, Vec<i64>> = HashMap::new();
        for (uid, mid, liked) in party_taste {
            // Only include positive picks (liked = Some(true))
            if liked != Some(true) {
                continue;
            }
            picks_by_user.entry(uid).or_default().push(mid);
        }

        let popular = self
            .get_popular_movie_ids(POPULAR_FALLBACK_LIMIT, None)
            .await
            .unwrap_or_default();
        let mut rng = rand::rng();

        for member in &members {
            let user_id = member.user_id;
            let own: Vec<i64> = picks_by_user
                .get(&user_id)
                .map(|v| v.iter().copied().take(OWN_PICKS).collect())
                .unwrap_or_default();
            let mut need_own = OWN_PICKS.saturating_sub(own.len());
            let mut own_pool = own;
            if need_own > 0 {
                let (global_pos, _, _) = self.get_user_ratings(user_id).await.unwrap_or_default();
                for &mid in global_pos.iter().take(need_own + own_pool.len()) {
                    if !own_pool.contains(&mid) {
                        own_pool.push(mid);
                        if own_pool.len() >= OWN_PICKS {
                            break;
                        }
                    }
                }
            }
            need_own = OWN_PICKS.saturating_sub(own_pool.len());
            if need_own > 0 {
                for &mid in &popular {
                    if !own_pool.contains(&mid) {
                        own_pool.push(mid);
                        if own_pool.len() >= OWN_PICKS {
                            break;
                        }
                    }
                }
            }

            let mut others_picks: Vec<i64> = picks_by_user
                .iter()
                .filter(|(uid, _)| *uid != &user_id)
                .flat_map(|(_, ids)| ids.iter().copied())
                .collect::<HashSet<_>>()
                .into_iter()
                .filter(|mid| !own_pool.contains(mid))
                .collect();
            others_picks.shuffle(&mut rng);
            let mut others_pool: Vec<i64> = others_picks.into_iter().take(OTHERS_PICKS).collect();
            let need_others = OTHERS_PICKS.saturating_sub(others_pool.len());
            if need_others > 0 {
                let used: HashSet<i64> = own_pool
                    .iter()
                    .copied()
                    .chain(others_pool.iter().copied())
                    .collect();
                for &mid in &popular {
                    if !used.contains(&mid) {
                        others_pool.push(mid);
                        if others_pool.len() >= OTHERS_PICKS {
                            break;
                        }
                    }
                }
            }

            let mut ballot: Vec<i64> = own_pool;
            ballot.extend(others_pool);
            ballot.shuffle(&mut rng);
            let take = BALLOT_SIZE.min(ballot.len());
            let ballot = ballot.into_iter().take(take).collect::<Vec<_>>();
            if !ballot.is_empty() {
                self.add_shown_movies(party_id, user_id, &ballot).await?;
            }
        }

        self.enable_voting(party_id).await?;
        self.set_voting_round(party_id, Some(1)).await?;
        Ok(())
    }

    /// Build round-2 ballots: only top 3 movies, same for all users. Clears existing, inserts, enables voting, sets round 2.
    pub(crate) async fn build_round2_ballots(&self, party_id: Uuid, top3: &[i64]) -> DbResult<()> {
        self.clear_shown_movies_for_party(party_id).await?;

        let members = self.get_party_members(party_id).await?;
        for member in &members {
            if !top3.is_empty() {
                self.add_shown_movies(party_id, member.user_id, top3)
                    .await?;
            }
        }

        self.enable_voting(party_id).await?;
        self.set_voting_round(party_id, Some(2)).await?;
        self.set_phase_entered_at_now(party_id).await?;
        Ok(())
    }

    /// Add movies to shown_movies for a user in a party
    /// User can only vote for movies in this list
    pub(crate) async fn add_shown_movies(
        &self,
        party_id: Uuid,
        user_id: Uuid,
        movie_ids: &[i64],
    ) -> DbResult<()> {
        let mut conn = self.conn().await?;
        let new_shown: Vec<NewShownMovie> = movie_ids
            .iter()
            .map(|&movie_id| NewShownMovie {
                party_id,
                user_id,
                movie_id,
            })
            .collect();
        diesel::insert_into(shown_movies::table)
            .values(&new_shown)
            .on_conflict_do_nothing()
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    /// Movie IDs on the user's ballot (what they can vote on) for this party. Empty when not in Voting or no ballot.
    pub(crate) async fn get_user_ballot(
        &self,
        party_id: Uuid,
        user_id: Uuid,
    ) -> DbResult<Vec<i64>> {
        let mut conn = self.conn().await?;
        let ids = shown_movies::table
            .filter(shown_movies::party_id.eq(party_id))
            .filter(shown_movies::user_id.eq(user_id))
            .select(shown_movies::movie_id)
            .order_by(shown_movies::shown_at)
            .load::<i64>(&mut conn)
            .await?;
        Ok(ids)
    }

    /// Check if a user can vote for a movie in a party
    pub(crate) async fn can_vote(
        &self,
        party_id: Uuid,
        user_id: Uuid,
        movie_id: i64,
    ) -> DbResult<bool> {
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
    pub(crate) async fn cast_vote(
        &self,
        party_id: Uuid,
        user_id: Uuid,
        movie_id: i64,
        vote_value: bool,
    ) -> DbResult<Vote> {
        if !self.can_vote(party_id, user_id, movie_id).await? {
            return Err(DbError::Other("User cannot vote for this movie".into()));
        }
        let mut conn = self.conn().await?;
        let new_vote = NewVote {
            party_id,
            user_id,
            movie_id,
            vote_value,
        };
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
    pub(crate) async fn get_vote_totals(
        &self,
        movie_id: i64,
        party_id: Option<Uuid>,
    ) -> DbResult<(i64, i64)> {
        use diesel::dsl::count_star;
        let mut conn = self.conn().await?;
        let mut base = votes::table
            .filter(votes::movie_id.eq(movie_id))
            .into_boxed();
        if let Some(pid) = party_id {
            base = base.filter(votes::party_id.eq(pid));
        }
        let likes = base
            .filter(votes::vote_value.eq(true))
            .select(count_star())
            .first(&mut conn)
            .await?;
        let mut base = votes::table
            .filter(votes::movie_id.eq(movie_id))
            .into_boxed();
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

    /// True if every current party member has cast at least one vote (for this round).
    pub(crate) async fn have_all_members_voted(&self, party_id: Uuid) -> DbResult<bool> {
        let members = self.get_party_members(party_id).await?;
        if members.is_empty() {
            return Ok(false);
        }
        for m in &members {
            let user_votes = self.get_user_votes(party_id, m.user_id).await?;
            if user_votes.is_empty() {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Count how many unique members have cast at least one vote for this party round.
    pub(crate) async fn get_voting_participation_count(&self, party_id: Uuid) -> DbResult<usize> {
        let mut conn = self.conn().await?;
        let count: i64 = votes::table
            .filter(votes::party_id.eq(party_id))
            .select(diesel::dsl::count(votes::user_id).aggregate_distinct())
            .get_result::<i64>(&mut conn)
            .await?;

        Ok(count as usize)
    }

    /// Get all votes for a user in a party
    pub(crate) async fn get_user_votes(
        &self,
        party_id: Uuid,
        user_id: Uuid,
    ) -> DbResult<Vec<Vote>> {
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
    pub(crate) async fn get_party_votes(
        &self,
        party_id: Uuid,
        user_id: Option<Uuid>,
    ) -> DbResult<HashMap<i64, (u32, u32)>> {
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
            if let Some(ref allowed) = allowed_movies
                && !allowed.contains(&movie_id)
            {
                continue;
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

    pub(crate) async fn enable_voting(&self, party_id: Uuid) -> DbResult<()> {
        use crate::schema::parties::dsl::*;
        let mut conn = self.conn().await?;
        diesel::update(parties.filter(id.eq(party_id)))
            .set(can_vote.eq(true))
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    pub(crate) async fn disable_voting(&self, party_id: Uuid) -> DbResult<()> {
        use crate::schema::parties::dsl::*;
        let mut conn = self.conn().await?;
        diesel::update(parties.filter(id.eq(party_id)))
            .set(can_vote.eq(false))
            .execute(&mut conn)
            .await?;
        Ok(())
    }
}
