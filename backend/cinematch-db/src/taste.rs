use crate::{Database, DbError, DbResult};
use chrono::Utc;
use uuid::Uuid;

impl Database {
    /// Add or update a global taste (user, movie, liked)
    pub async fn add_taste(&self, user_id: Uuid, movie_id: i64, liked: bool) -> DbResult<()> {
        use crate::schema::user_tastes::dsl as ut;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        let mut conn = self.conn().await?;
        let now = Utc::now();
        diesel::insert_into(ut::user_tastes)
            .values((
                ut::user_id.eq(user_id),
                ut::movie_id.eq(movie_id),
                ut::liked.eq(liked),
                ut::updated_at.eq(now),
                ut::party_id.eq::<Option<Uuid>>(None),
            ))
            .on_conflict((ut::user_id, ut::movie_id, ut::party_id))
            .do_update()
            .set((ut::liked.eq(liked), ut::updated_at.eq(now)))
            .execute(&mut conn)
            .await
            .map_err(DbError::from)?;
        Ok(())
    }

    pub async fn get_taste(&self, user_id: Uuid) -> DbResult<(Vec<i64>, Vec<i64>)> {
        use crate::schema::user_tastes::dsl as ut;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        let mut conn = self.conn().await?;
        // Select all global tastes for the user (party_id is null)
        let results = ut::user_tastes
            .filter(ut::user_id.eq(user_id))
            .filter(ut::party_id.is_null())
            .select((ut::movie_id, ut::liked))
            .load::<(i64, bool)>(&mut conn)
            .await
            .map_err(DbError::from)?;
        let positive: Vec<i64> = results
            .iter()
            .filter(|(_, liked)| *liked)
            .map(|(id, _)| *id)
            .collect();
        let negative: Vec<i64> = results
            .iter()
            .filter(|(_, liked)| !*liked)
            .map(|(id, _)| *id)
            .collect();
        Ok((positive, negative))
    }

    /// List party picks for ballot building: (user_id, movie_id, liked) for all members.
    /// Use positive picks (liked = true) for "own" and "others" pools.
    pub async fn get_party_taste(&self, party_id: Uuid) -> DbResult<Vec<(Uuid, i64, bool)>> {
        use crate::schema::user_tastes::dsl as ut;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        let mut conn = self.conn().await?;
        let results = ut::user_tastes
            .filter(ut::party_id.eq(Some(party_id)))
            .select((ut::user_id, ut::movie_id, ut::liked))
            .load::<(Uuid, i64, bool)>(&mut conn)
            .await
            .map_err(DbError::from)?;
        Ok(results)
    }

    /// Add or update a party-specific taste (user, party, movie, liked)
    pub async fn add_party_taste(
        &self,
        user_id: Uuid,
        party_id: Uuid,
        movie_id: i64,
        liked: bool,
    ) -> DbResult<()> {
        use crate::schema::user_tastes::dsl as ut;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        let mut conn = self.conn().await?;
        let now = Utc::now();
        diesel::insert_into(ut::user_tastes)
            .values((
                ut::user_id.eq(user_id),
                ut::party_id.eq(Some(party_id)),
                ut::movie_id.eq(movie_id),
                ut::liked.eq(liked),
                ut::updated_at.eq(now),
            ))
            .on_conflict((ut::user_id, ut::movie_id, ut::party_id))
            .do_update()
            .set((ut::liked.eq(liked), ut::updated_at.eq(now)))
            .execute(&mut conn)
            .await
            .map_err(DbError::from)?;
        Ok(())
    }

    /// Picks (liked movie IDs) for a user in a party. Empty when not picking or no picks.
    pub async fn get_user_party_picks(&self, party_id: Uuid, user_id: Uuid) -> DbResult<Vec<i64>> {
        use crate::schema::user_tastes::dsl as ut;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        let mut conn = self.conn().await?;
        let ids = ut::user_tastes
            .filter(ut::party_id.eq(Some(party_id)))
            .filter(ut::user_id.eq(user_id))
            .filter(ut::liked.eq(true))
            .select(ut::movie_id)
            .load::<i64>(&mut conn)
            .await
            .map_err(DbError::from)?;
        Ok(ids)
    }

    /// Remove a pick (party taste) for user/party/movie. Idempotent if already absent.
    pub async fn remove_party_taste(
        &self,
        user_id: Uuid,
        party_id: Uuid,
        movie_id: i64,
    ) -> DbResult<()> {
        use crate::schema::user_tastes::dsl as ut;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        let mut conn = self.conn().await?;
        diesel::delete(
            ut::user_tastes
                .filter(ut::user_id.eq(user_id))
                .filter(ut::party_id.eq(Some(party_id)))
                .filter(ut::movie_id.eq(movie_id)),
        )
        .execute(&mut conn)
        .await
        .map_err(DbError::from)?;
        Ok(())
    }
}
