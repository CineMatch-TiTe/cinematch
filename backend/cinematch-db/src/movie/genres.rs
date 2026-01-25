
use crate::Database;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::DbError;
use crate::DbResult;
use crate::schema;
impl Database {
    /// Get all genres as a map: name -> id
    pub async fn get_genres(&self) -> DbResult<std::collections::HashMap<String, Uuid>> {
        use crate::schema::genres::dsl::*;
        let mut conn = self.conn().await?;
        let rows = genres
            .select((name, genre_id))
            .load::<(String, Uuid)>(&mut conn)
            .await
            .map_err(DbError::from)?;
        Ok(rows.into_iter().collect())
    }
}