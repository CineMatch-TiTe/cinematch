use uuid::Uuid;
use chrono::Utc;
use crate::{Database, DbError, DbResult};

use crate::TasteProfile;

impl Database {
    /// Add or update a global taste (user, movie, liked)
    pub async fn add_taste(&self, user_id: Uuid, movie_id: i64, liked: bool) -> DbResult<()> {
        use crate::schema::user_tastes::dsl as ut;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        let mut conn = self.conn().await?;
        let now = Utc::now();
        diesel::insert_into(ut::user_tastes)
            .values((ut::user_id.eq(user_id), ut::movie_id.eq(movie_id), ut::liked.eq(liked), ut::updated_at.eq(now), ut::party_id.eq::<Option<Uuid>>(None)))
            .on_conflict((ut::user_id, ut::movie_id, ut::party_id))
            .do_update()
            .set((ut::liked.eq(liked), ut::updated_at.eq(now)))
            .execute(&mut conn)
            .await
            .map_err(DbError::from)?;
        Ok(())
    }

    pub async fn get_taste(&self, user_id: Uuid) -> DbResult<Vec<TasteProfile>> {
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
        let taste_profiles = results
            .into_iter()
            .map(|(point_id, liked)| TasteProfile { point_id, liked })
            .collect();
        Ok(taste_profiles)
    }

    /// Add or update a party-specific taste (user, party, movie, liked)
    pub async fn add_party_taste(&self, user_id: Uuid, party_id: Uuid, movie_id: i64, liked: bool) -> DbResult<()> {
        use crate::schema::user_tastes::dsl as ut;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        let mut conn = self.conn().await?;
        let now = Utc::now();
        diesel::insert_into(ut::user_tastes)
            .values((ut::user_id.eq(user_id), ut::party_id.eq(Some(party_id)), ut::movie_id.eq(movie_id), ut::liked.eq(liked), ut::updated_at.eq(now)))
            .on_conflict((ut::user_id, ut::movie_id, ut::party_id))
            .do_update()
            .set((ut::liked.eq(liked), ut::updated_at.eq(now)))
            .execute(&mut conn)
            .await
            .map_err(DbError::from)?;
        Ok(())
    }
}