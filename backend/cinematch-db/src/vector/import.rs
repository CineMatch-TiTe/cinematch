use crate::{Database, vector::models::MovieData};

impl Database {
    pub async fn upload_movies(&self, _movies: &[MovieData]) -> anyhow::Result<()> {
        // upload both to qdrant and to pg in batches, first uploading the vectors to qdrant to get the point ids then to pg

        Ok(())
    }
}
