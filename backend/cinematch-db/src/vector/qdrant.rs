use super::models::MovieData;
use anyhow::{Context, Result};
use chrono::Datelike;
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{Distance, PointStruct};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::BATCH_SIZE;

/// Movie search result from vector similarity search
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MovieSearchResult {
    pub movie_id: i64,
    pub title: String,
    pub director: Option<String>,
    pub genres: Vec<String>,
    pub overview: Option<String>,
    pub score: f32, // Similarity score
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PointPayload {
    pub movie_id: u32,
    pub title: String,
    pub overview: String,
    pub director: String,
    pub genres: String,
}

pub struct QdrantService {
    pub client: Arc<Qdrant>,
}

impl Clone for QdrantService {
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
        }
    }
}

impl QdrantService {
    pub fn new(url: &str) -> Result<Self> {
        // Don't fail on initial connection - just build the client
        // Connection will be tested with check_service()
        let client = Qdrant::from_url(url)
            .timeout(Duration::from_secs(10))
            .build()
            .context("Failed to build Qdrant client")?;
        Ok(Self {
            client: Arc::new(client),
        })
    }

    /// Check if Qdrant service is available
    pub async fn check_service(&self) -> Result<()> {
        self.client
            .health_check()
            .await
            .context("Failed to reach Qdrant service")?;
        Ok(())
    }

    /// Create embedding collection with multiple named vectors
    /// Supports: plot_vector, cast_crew_vector, reviews_vector, combined_vector
    /// DANGER: deletes existing collection with the same name, should only be used within the importer
    pub async fn setup(&self, collection_name: &str) -> Result<()> {
        // Delete existing collection
        let _ = self.client.delete_collection(collection_name).await;

        // Create vectors config with named vectors using ParamsMap
        let mut vectors_map = std::collections::HashMap::new();

        for vector_name in &[
            "plot_vector",
            "cast_crew_vector",
            "reviews_vector",
            "combined_vector",
        ] {
            vectors_map.insert(
                vector_name.to_string(),
                qdrant_client::qdrant::VectorParams {
                    size: 1024,
                    distance: Distance::Cosine.into(),
                    ..Default::default()
                },
            );
        }

        // Create new collection with named vectors config
        self.client
            .create_collection(qdrant_client::qdrant::CreateCollection {
                collection_name: collection_name.to_string(),
                vectors_config: Some(qdrant_client::qdrant::VectorsConfig {
                    config: Some(qdrant_client::qdrant::vectors_config::Config::ParamsMap(
                        qdrant_client::qdrant::VectorParamsMap { map: vectors_map },
                    )),
                }),
                ..Default::default()
            })
            .await
            .context(format!("Failed to create collection {}", collection_name))?;

        Ok(())
    }

    /// Upload points in batches (20 per batch) for performance
    pub async fn upload_batch(&self, collection: &str, points: &[PointStruct]) -> Result<()> {
        if points.is_empty() {
            return Ok(());
        }

        for batch in points.chunks(BATCH_SIZE) {
            self.client
                .upsert_points(qdrant_client::qdrant::UpsertPoints {
                    collection_name: collection.to_string(),
                    wait: Some(true),
                    points: batch.to_vec(),
                    ..Default::default()
                })
                .await
                .context(format!(
                    "Failed to upload batch of {} points to {}",
                    batch.len(),
                    collection
                ))?;
        }

        Ok(())
    }

    /// Create a point from movie with multiple named embeddings
    /// vectors: map of vector_name -> embedding array
    /// vector names: "plot_vector", "cast_crew_vector", "reviews_vector", "combined_vector"
    pub fn create_point_with_vectors(
        movie: &MovieData,
        vectors: std::collections::HashMap<String, Vec<f32>>,
    ) -> PointStruct {
        let mut payload = HashMap::new();

        // Add release_year as an integer for filtering
        let release_year = if movie.release_date > 0 {
            let naive = chrono::DateTime::from_timestamp(movie.release_date, 0);
            naive.map(|dt| dt.year()).unwrap_or(0)
        } else {
            0
        };
        payload.insert(
            "release_year".to_string(),
            qdrant_client::qdrant::Value {
                kind: Some(qdrant_client::qdrant::value::Kind::IntegerValue(
                    release_year as i64,
                )),
            },
        );

        payload.insert(
            "directors".to_string(),
            qdrant_client::qdrant::Value {
                kind: Some(qdrant_client::qdrant::value::Kind::ListValue(
                    qdrant_client::qdrant::ListValue {
                        values: movie
                            .director
                            .iter()
                            .map(|d| qdrant_client::qdrant::Value {
                                kind: Some(qdrant_client::qdrant::value::Kind::StringValue(
                                    d.clone(),
                                )),
                            })
                            .collect(),
                    },
                )),
            },
        );

        payload.insert(
            "genres".to_string(),
            qdrant_client::qdrant::Value {
                kind: Some(qdrant_client::qdrant::value::Kind::ListValue(
                    qdrant_client::qdrant::ListValue {
                        values: movie
                            .genres
                            .iter()
                            .map(|g| qdrant_client::qdrant::Value {
                                kind: Some(qdrant_client::qdrant::value::Kind::StringValue(
                                    g.clone(),
                                )),
                            })
                            .collect(),
                    },
                )),
            },
        );

        payload.insert(
            "tags".to_string(),
            qdrant_client::qdrant::Value {
                kind: Some(qdrant_client::qdrant::value::Kind::ListValue(
                    qdrant_client::qdrant::ListValue {
                        values: movie
                            .keywords
                            .iter()
                            .map(|g| qdrant_client::qdrant::Value {
                                kind: Some(qdrant_client::qdrant::value::Kind::StringValue(
                                    g.clone(),
                                )),
                            })
                            .collect(),
                    },
                )),
            },
        );

        payload.insert(
            "movie_id".to_string(),
            qdrant_client::qdrant::Value {
                kind: Some(qdrant_client::qdrant::value::Kind::IntegerValue(
                    movie.movie_id as i64,
                )),
            },
        );

        let overview_text = movie
            .overview
            .as_ref()
            .map(|o| o.chars().take(2048).collect::<String>())
            .unwrap_or_else(|| "No overview available".to_string());

        payload.insert(
            "overview".to_string(),
            qdrant_client::qdrant::Value {
                kind: Some(qdrant_client::qdrant::value::Kind::StringValue(
                    overview_text,
                )),
            },
        );

        payload.insert(
            "runtime".to_string(),
            qdrant_client::qdrant::Value {
                kind: Some(qdrant_client::qdrant::value::Kind::IntegerValue(
                    movie.runtime,
                )),
            },
        );

        payload.insert(
            "title".to_string(),
            qdrant_client::qdrant::Value {
                kind: Some(qdrant_client::qdrant::value::Kind::StringValue(
                    movie.title.clone(),
                )),
            },
        );

        // Build named vectors from the provided embeddings
        let mut named_vectors = std::collections::HashMap::new();
        for (vector_name, embedding) in vectors {
            named_vectors.insert(
                vector_name,
                qdrant_client::qdrant::Vector {
                    vector: Some(qdrant_client::qdrant::vector::Vector::Dense(
                        qdrant_client::qdrant::DenseVector { data: embedding },
                    )),
                    ..Default::default()
                },
            );
        }

        PointStruct {
            id: Some(qdrant_client::qdrant::PointId {
                point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(
                    movie.movie_id as u64,
                )),
            }),
            vectors: Some(qdrant_client::qdrant::Vectors {
                vectors_options: Some(qdrant_client::qdrant::vectors::VectorsOptions::Vectors(
                    qdrant_client::qdrant::NamedVectors {
                        vectors: named_vectors,
                    },
                )),
            }),
            payload,
        }
    }
}
