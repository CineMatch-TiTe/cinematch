
use crate::Database;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::DbError;
use crate::DbResult;
use crate::schema;
impl Database {
    pub async fn get_genres(&self) -> DbResult<Vec<String>> {
        use schema::genres::dsl::*;

        let mut conn = self.conn().await?;
        let mut results: Vec<String> = match genres
            .select(name)
            .load::<String>(&mut conn)
            .await {
            Ok(res) => res,
            Err(e) => {
                return Err(DbError::Query(e));
            }
        };
        results.sort();
        results.dedup();
        Ok(results)
    }
}