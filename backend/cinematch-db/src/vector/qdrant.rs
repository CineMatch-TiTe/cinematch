use super::models::MovieData;
use anyhow::{Context, Result};
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{Distance, PointStruct};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Movie search result from vector similarity search
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MovieSearchResult {
    pub movie_id: i64,
    pub title: String,
    pub director: Option<String>,
    pub genres: Vec<String>,
    pub overview: Option<String>,
    pub cast: Vec<String>, // Just names for display
    pub score: f32,        // Similarity score
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
    pub(crate) client: Arc<Qdrant>,
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
    pub async fn create_collection_with_named_vectors(&self, collection_name: &str) -> Result<()> {
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

    /// Create both embedding collections with multiple named vectors
    pub async fn create_collections(&self) -> Result<()> {
        self.create_collection_with_named_vectors("movies").await?;
        Ok(())
    }

    /// Upload points in batches (20 per batch) for performance
    pub async fn upload_batch(&self, collection: &str, points: Vec<PointStruct>) -> Result<()> {
        if points.is_empty() {
            return Ok(());
        }

        const BATCH_SIZE: usize = 20;

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

        // Insert in alphabetical order for consistency
        payload.insert(
            "cast".to_string(),
            qdrant_client::qdrant::Value {
                kind: Some(qdrant_client::qdrant::value::Kind::StringValue(
                    serde_json::to_string(&movie.cast).unwrap_or_default(),
                )),
            },
        );

        payload.insert(
            "director".to_string(),
            qdrant_client::qdrant::Value {
                kind: Some(qdrant_client::qdrant::value::Kind::StringValue(
                    movie
                        .director
                        .as_ref()
                        .map(|d| d.clone())
                        .unwrap_or_default(),
                )),
            },
        );

        payload.insert(
            "genres".to_string(),
            qdrant_client::qdrant::Value {
                kind: Some(qdrant_client::qdrant::value::Kind::StringValue(
                    serde_json::to_string(&movie.genres).unwrap_or_default(),
                )),
            },
        );

        payload.insert(
            "imdb_id".to_string(),
            qdrant_client::qdrant::Value {
                kind: Some(qdrant_client::qdrant::value::Kind::StringValue(
                    movie
                        .imdb_id
                        .as_ref()
                        .map(|id| id.clone())
                        .unwrap_or_default(),
                )),
            },
        );

        payload.insert(
            "keywords".to_string(),
            qdrant_client::qdrant::Value {
                kind: Some(qdrant_client::qdrant::value::Kind::StringValue(
                    serde_json::to_string(&movie.keywords).unwrap_or_default(),
                )),
            },
        );

        payload.insert(
            "mediawiki_id".to_string(),
            qdrant_client::qdrant::Value {
                kind: Some(qdrant_client::qdrant::value::Kind::StringValue(
                    movie
                        .mediawiki_id
                        .as_ref()
                        .map(|id| id.clone())
                        .unwrap_or_default(),
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
            "tagline".to_string(),
            qdrant_client::qdrant::Value {
                kind: Some(qdrant_client::qdrant::value::Kind::StringValue(
                    movie
                        .tagline
                        .as_ref()
                        .map(|t| t.clone())
                        .unwrap_or_default(),
                )),
            },
        );

        payload.insert(
            "poster_url".to_string(),
            qdrant_client::qdrant::Value {
                kind: Some(qdrant_client::qdrant::value::Kind::StringValue(
                    serde_json::to_string(&movie.poster_url).unwrap_or_default(),
                )),
            },
        );

        payload.insert(
            "production_countries".to_string(),
            qdrant_client::qdrant::Value {
                kind: Some(qdrant_client::qdrant::value::Kind::StringValue(
                    serde_json::to_string(&movie.production_countries).unwrap_or_default(),
                )),
            },
        );

        payload.insert(
            "reviews".to_string(),
            qdrant_client::qdrant::Value {
                kind: Some(qdrant_client::qdrant::value::Kind::StringValue(
                    serde_json::to_string(&movie.reviews).unwrap_or_default(),
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

    /// Search for similar movies using a vector embedding
    /// vector_name: which semantic vector to search ("plot_vector", "cast_crew_vector", etc.)
    /// embedding: the query vector embedding
    /// limit: max number of results to return
    pub async fn search_similar(
        &self,
        collection: &str,
        vector_name: &str,
        embedding: Vec<f32>,
        limit: u64,
    ) -> Result<Vec<MovieSearchResult>> {
        let results = self
            .client
            .search_points(qdrant_client::qdrant::SearchPoints {
                collection_name: collection.to_string(),
                vector: embedding,
                limit,
                with_payload: Some(true.into()),
                vector_name: Some(vector_name.to_string()),
                ..Default::default()
            })
            .await
            .context("Failed to search points")?;

        // Extract search results and reconstruct movie info from payload
        let mut movies = Vec::new();
        for scored_point in results.result {
            let movie = parse_payload_to_movie(&scored_point.payload)?;
            movies.push(MovieSearchResult {
                movie_id: movie.movie_id,
                title: movie.title,
                director: movie.director,
                genres: movie.genres,
                overview: movie.overview,
                cast: movie.cast.iter().map(|m| m.name.clone()).collect(),
                score: scored_point.score,
            });
        }

        Ok(movies)
    }
}

/// Helper function to reconstruct movie data from Qdrant payload
pub fn parse_payload_to_movie(
    payload: &HashMap<String, qdrant_client::qdrant::Value>,
) -> Result<MovieData> {
    use qdrant_client::qdrant::value::Kind;

    let get_string = |key: &str| -> Option<String> {
        payload.get(key).and_then(|v| {
            if let Some(Kind::StringValue(s)) = &v.kind {
                Some(s.clone())
            } else {
                None
            }
        })
    };

    let get_int = |key: &str| -> Option<i64> {
        payload.get(key).and_then(|v| {
            if let Some(Kind::IntegerValue(i)) = &v.kind {
                Some(*i)
            } else {
                None
            }
        })
    };

    let movie_id = get_int("movie_id").unwrap_or(0);
    let title = get_string("title").unwrap_or_default();

    Ok(MovieData {
        movie_id,
        title,
        runtime: get_int("runtime").unwrap_or(0),
        average_rating: 0.0,
        popularity: 0.0,
        imdb_id: get_string("imdb_id"),
        mediawiki_id: get_string("mediawiki_id"),
        rating: get_string("rating"),
        release_date: get_int("release_date").unwrap_or(0),
        original_language: None,
        poster_url: get_string("poster_url"),
        overview: get_string("overview"),
        tagline: get_string("tagline"),
        director: get_string("director"),
        genres: serde_json::from_str(&get_string("genres").unwrap_or_default()).unwrap_or_default(),
        keywords: serde_json::from_str(&get_string("keywords").unwrap_or_default())
            .unwrap_or_default(),
        cast: serde_json::from_str(&get_string("cast").unwrap_or_default()).unwrap_or_default(),
        production_countries: serde_json::from_str(
            &get_string("production_countries").unwrap_or_default(),
        )
        .unwrap_or_default(),
        reviews: serde_json::from_str(&get_string("reviews").unwrap_or_default())
            .unwrap_or_default(),
    })
}
