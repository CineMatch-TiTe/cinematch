//! Taste preferences - shared across user and party contexts.
//!
//! Handles both global user ratings (liked/disliked/skipped movies) and
//! party-specific picks during the picking phase.

use crate::{Database, DbError, DbResult};
use chrono::Utc;
use uuid::Uuid;

impl Database {
    /// Add or update a global movie rating (user, movie, liked, rating)
    /// liked: Some(true) = like, Some(false) = dislike, None = skip/none
    pub(crate) async fn add_rating(
        &self,
        user_id: Uuid,
        movie_id: i64,
        liked: Option<bool>,
        rating: Option<i32>,
    ) -> DbResult<()> {
        use crate::schema::user_ratings::dsl as ur;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        let mut conn = self.conn().await?;
        let now = Utc::now();
        diesel::insert_into(ur::user_ratings)
            .values((
                ur::user_id.eq(user_id),
                ur::movie_id.eq(movie_id),
                ur::liked.eq(liked),
                ur::rating.eq(rating),
                ur::updated_at.eq(now),
            ))
            .on_conflict((ur::user_id, ur::movie_id))
            .do_update()
            .set((
                ur::liked.eq(liked),
                ur::rating.eq(rating),
                ur::updated_at.eq(now),
            ))
            .execute(&mut conn)
            .await
            .map_err(DbError::from)?;
        Ok(())
    }

    /// Retrieve global rating and liked status for a specific movie.
    pub(crate) async fn get_movie_rating(
        &self,
        user_id: Uuid,
        movie_id: i64,
    ) -> DbResult<Option<(Option<bool>, Option<i32>, chrono::DateTime<Utc>)>> {
        use crate::schema::user_ratings::dsl as ur;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        let mut conn = self.conn().await?;
        ur::user_ratings
            .filter(ur::user_id.eq(user_id))
            .filter(ur::movie_id.eq(movie_id))
            .select((ur::liked, ur::rating, ur::updated_at))
            .first::<(Option<bool>, Option<i32>, chrono::DateTime<Utc>)>(&mut conn)
            .await
            .optional()
            .map_err(DbError::from)
    }

    /// Get user ratings: (positive, negative, skipped)
    pub(crate) async fn get_user_ratings(
        &self,
        user_id: Uuid,
    ) -> DbResult<(Vec<i64>, Vec<i64>, Vec<i64>)> {
        use crate::schema::user_ratings::dsl as ur;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        let mut conn = self.conn().await?;
        let results = ur::user_ratings
            .filter(ur::user_id.eq(user_id))
            .select((ur::movie_id, ur::liked))
            .load::<(i64, Option<bool>)>(&mut conn)
            .await
            .map_err(DbError::from)?;
        let mut positive: Vec<i64> = Vec::new();
        let mut negative: Vec<i64> = Vec::new();
        let mut skipped: Vec<i64> = Vec::new();
        for (id, liked_opt) in results {
            match liked_opt {
                Some(true) => positive.push(id),
                Some(false) => negative.push(id),
                None => skipped.push(id),
            }
        }
        Ok((positive, negative, skipped))
    }

    /// Add or update a party-specific pick (session-based)
    pub(crate) async fn add_party_pick(
        &self,
        user_id: Uuid,
        party_id: Uuid,
        movie_id: i64,
        liked: Option<bool>,
    ) -> DbResult<()> {
        use crate::schema::party_picks::dsl as pp;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        let mut conn = self.conn().await?;
        let now = Utc::now();
        diesel::insert_into(pp::party_picks)
            .values((
                pp::user_id.eq(user_id),
                pp::party_id.eq(party_id),
                pp::movie_id.eq(movie_id),
                pp::liked.eq(liked),
                pp::updated_at.eq(now),
            ))
            .on_conflict((pp::user_id, pp::movie_id, pp::party_id))
            .do_update()
            .set((pp::liked.eq(liked), pp::updated_at.eq(now)))
            .execute(&mut conn)
            .await
            .map_err(DbError::from)?;
        Ok(())
    }

    /// List party picks for ballot building: (user_id, movie_id, liked) for all members.
    pub(crate) async fn get_party_picks(
        &self,
        party_id: Uuid,
    ) -> DbResult<Vec<(Uuid, i64, Option<bool>)>> {
        use crate::schema::party_picks::dsl as pp;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        let mut conn = self.conn().await?;
        let results = pp::party_picks
            .filter(pp::party_id.eq(party_id))
            .select((pp::user_id, pp::movie_id, pp::liked))
            .load::<(Uuid, i64, Option<bool>)>(&mut conn)
            .await
            .map_err(DbError::from)?;
        Ok(results)
    }

    /// Picks (liked movie IDs) for a user in a party.
    pub(crate) async fn get_user_party_picks(
        &self,
        party_id: Uuid,
        user_id: Uuid,
    ) -> DbResult<Vec<i64>> {
        use crate::schema::party_picks::dsl as pp;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        let mut conn = self.conn().await?;
        let ids = pp::party_picks
            .filter(pp::party_id.eq(party_id))
            .filter(pp::user_id.eq(user_id))
            .filter(pp::liked.eq(true))
            .select(pp::movie_id)
            .load::<i64>(&mut conn)
            .await
            .map_err(DbError::from)?;
        Ok(ids)
    }

    /// Remove a pick from a party session.
    pub(crate) async fn remove_party_pick(
        &self,
        user_id: Uuid,
        party_id: Uuid,
        movie_id: i64,
    ) -> DbResult<()> {
        use crate::schema::party_picks::dsl as pp;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        let mut conn = self.conn().await?;
        diesel::delete(
            pp::party_picks
                .filter(pp::user_id.eq(user_id))
                .filter(pp::party_id.eq(party_id))
                .filter(pp::movie_id.eq(movie_id)),
        )
        .execute(&mut conn)
        .await
        .map_err(DbError::from)?;
        Ok(())
    }
}
