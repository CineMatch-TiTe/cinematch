use std::collections::HashMap;

use anyhow::Result;
use log::error;

use crate::Database;
use crate::conn::qdrant::QdrantService;
use crate::conn::qdrant::models::MovieData;

/// Batch size for Postgres inserts during import
const PG_BATCH_SIZE: usize = 100;

/// User data for rating import: (user_id, indices, values, movie_ids)
pub type RatingImportData = (i64, Vec<u32>, Vec<f32>, Vec<i64>);

impl Database {
    /// Upload movies to both Qdrant (vector store) and Postgres (relational store).
    ///
    /// This is the **public entry point** for bulk movie import.
    /// The importer should call this after generating embeddings.
    ///
    /// # Arguments
    /// * `movies` - Slice of movie data to insert
    /// * `embeddings` - Map of movie_id → (vector_name → embedding). Movies without
    ///   embeddings will still be inserted into Postgres but skipped for Qdrant.
    /// * `collection` - Qdrant collection name (e.g. "movies")
    ///
    /// # Returns
    /// A tuple of `(qdrant_points_uploaded, pg_rows_inserted)`.
    pub async fn upload_movies(
        &self,
        movies: &[MovieData],
        embeddings: &HashMap<i64, HashMap<String, Vec<f32>>>,
        collection: &str,
    ) -> Result<(u64, u64)> {
        let mut qdrant_uploaded = 0u64;
        let mut pg_inserted = 0u64;

        // --- Step 1: Build Qdrant points from movies + embeddings ---
        let mut points = Vec::new();
        for movie in movies {
            if let Some(vectors) = embeddings.get(&movie.movie_id).filter(|v| !v.is_empty()) {
                let point = QdrantService::create_point_with_vectors(movie, vectors.clone());
                points.push(point);
            }
        }

        // --- Step 2: Upload points to Qdrant ---
        if !points.is_empty() {
            match self.vector.upload_batch(collection, &points).await {
                Ok(_) => {
                    qdrant_uploaded = points.len() as u64;
                }
                Err(e) => {
                    error!("Failed to upload batch to Qdrant: {}", e);
                    // Continue to postgres — don't fail the whole batch
                }
            }
        }

        // --- Step 3: Insert movie data into Postgres in batches ---
        for chunk in movies.chunks(PG_BATCH_SIZE) {
            match self.insert_movie_data_batch(chunk).await {
                Ok(_) => {
                    pg_inserted += chunk.len() as u64;
                }
                Err(e) => {
                    error!(
                        "Failed to insert batch of {} movies to Postgres: {}",
                        chunk.len(),
                        e
                    );
                    // Continue with next batch
                }
            }
        }

        Ok((qdrant_uploaded, pg_inserted))
    }

    /// Insert movies into Postgres only (no Qdrant).
    ///
    /// Useful when vectors are handled separately or not needed.
    pub async fn insert_movies_pg(&self, movies: &[MovieData]) -> Result<u64> {
        let mut inserted = 0u64;
        for chunk in movies.chunks(PG_BATCH_SIZE) {
            match self.insert_movie_data_batch(chunk).await {
                Ok(_) => {
                    inserted += chunk.len() as u64;
                }
                Err(e) => {
                    error!(
                        "Failed to insert batch of {} movies to Postgres: {}",
                        chunk.len(),
                        e
                    );
                }
            }
        }
        Ok(inserted)
    }

    /// Create or recreate the ratings Qdrant collection with sparse vector config.
    pub async fn upload_ratings_setup(&self, collection_name: &str) -> Result<()> {
        use qdrant_client::qdrant::{CreateCollection, SparseVectorConfig, SparseVectorParams};

        // Delete existing
        let _ = self.vector.client.delete_collection(collection_name).await;

        // Create with sparse vector config
        let mut sparse_map = HashMap::new();
        sparse_map.insert("ratings".to_string(), SparseVectorParams::default());

        self.vector
            .client
            .create_collection(CreateCollection {
                collection_name: collection_name.to_string(),
                vectors_config: None, // no dense vectors
                sparse_vectors_config: Some(SparseVectorConfig { map: sparse_map }),
                ..Default::default()
            })
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create ratings collection: {}", e))?;

        Ok(())
    }

    /// Upload a batch of user rating points with sparse vectors.
    ///
    /// Each point: (user_id, indices, values, movie_ids_payload)
    pub async fn upload_ratings_batch(
        &self,
        collection: &str,
        batch: &[RatingImportData],
    ) -> Result<u64> {
        use qdrant_client::qdrant::{NamedVectors, PointStruct, UpsertPoints, Value, Vector};

        if batch.is_empty() {
            return Ok(0);
        }

        let points: Vec<PointStruct> = batch
            .iter()
            .map(|(user_id, indices, values, movie_ids)| {
                // Build payload
                let mut payload_map = HashMap::new();
                payload_map.insert("user_id".to_string(), Value::from(*user_id));
                payload_map.insert(
                    "movie_id".to_string(),
                    Value::from(
                        movie_ids
                            .iter()
                            .map(|&id| Value::from(id))
                            .collect::<Vec<_>>(),
                    ),
                );

                // Build sparse vector from (index, value) pairs
                let sparse_pairs: Vec<(u32, f32)> = indices
                    .iter()
                    .zip(values.iter())
                    .map(|(&i, &v)| (i, v))
                    .collect();
                let sparse_vec = Vector::from(sparse_pairs);

                let mut named = HashMap::new();
                named.insert("ratings".to_string(), sparse_vec);

                PointStruct {
                    id: Some(qdrant_client::qdrant::PointId::from(*user_id as u64)),
                    payload: payload_map,
                    vectors: Some(NamedVectors { vectors: named }.into()),
                }
            })
            .collect();

        let count = points.len() as u64;

        self.vector
            .client
            .upsert_points(UpsertPoints {
                collection_name: collection.to_string(),
                wait: Some(true),
                points,
                ..Default::default()
            })
            .await
            .map_err(|e| anyhow::anyhow!("Failed to upload ratings batch: {}", e))?;

        Ok(count)
    }

    /// Wipe all imported data: drop Qdrant collections + truncate Postgres movie tables.
    pub async fn wipe_all(&self) -> Result<()> {
        use crate::schema::*;
        use diesel_async::RunQueryDsl;

        // Drop Qdrant collections
        let _ = self.vector.client.delete_collection("movies").await;
        let _ = self.vector.client.delete_collection("ratings").await;

        // Truncate Postgres movie-related tables
        let mut conn = self.conn().await.map_err(|e| anyhow::anyhow!("{}", e))?;

        // Delete join tables first (FK dependencies)
        diesel::delete(movie_directors::table)
            .execute(&mut conn)
            .await?;
        diesel::delete(movie_genres::table)
            .execute(&mut conn)
            .await?;
        diesel::delete(movie_keywords::table)
            .execute(&mut conn)
            .await?;
        diesel::delete(movie_cast::table).execute(&mut conn).await?;
        diesel::delete(movie_production_countries::table)
            .execute(&mut conn)
            .await?;
        diesel::delete(movie_trailers::table)
            .execute(&mut conn)
            .await?;

        // Delete lookup tables
        diesel::delete(directors::table).execute(&mut conn).await?;
        diesel::delete(genres::table).execute(&mut conn).await?;
        diesel::delete(keywords::table).execute(&mut conn).await?;
        diesel::delete(cast_members::table)
            .execute(&mut conn)
            .await?;
        diesel::delete(production_countries::table)
            .execute(&mut conn)
            .await?;
        diesel::delete(trailers::table).execute(&mut conn).await?;

        // Delete movies last
        diesel::delete(movies::table).execute(&mut conn).await?;

        Ok(())
    }
}
