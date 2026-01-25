

use uuid::Uuid;
use cinematch_db::{Database, DbResult};
use qdrant_client::qdrant::{RecommendPoints, WithPayloadSelector, WithVectorsSelector, PointId, RecommendStrategy};
use std::collections::HashMap;
use cinematch_db::vector::qdrant::{QdrantService, MovieSearchResult};

use cinematch_db::DbError;

/// Recommend movies for a user using Qdrant vector search, based on their taste profile.
pub async fn recommend_movies(
	db: &Database,
	user_id: Uuid,
	limit: usize,
) -> DbResult<Vec<i64>> {
	let (positive, negative) = db.get_taste(user_id).await?;

	let mut positive = positive;
	if positive.is_empty() {
		let popular_movies = db.get_popular_movies(5).await?;
		positive = popular_movies.into_iter().map(|m| m.movie_id).collect();
	}

	// Get QdrantService from db.vector (assume db.vector is QdrantService)
	let client = db.vector.client.clone();


	// Convert i64 IDs to Qdrant PointId
	let positive_ids: Vec<PointId> = positive.iter().map(|&id| PointId {
		point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(id as u64)),
	}).collect();
	let negative_ids: Vec<PointId> = negative.iter().map(|&id| PointId {
		point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(id as u64)),
	}).collect();

	// Build the recommend request
	let request = RecommendPoints {
		collection_name: "movies".to_string(),
		positive: positive_ids,
		negative: negative_ids,
		filter: None,
		limit: limit as u64,
		with_payload: Some(WithPayloadSelector {
			selector_options: Some(qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(true)),
		}),
		with_vectors: Some(WithVectorsSelector {
			selector_options: Some(qdrant_client::qdrant::with_vectors_selector::SelectorOptions::Enable(false)),
		}),
		score_threshold: None,
		offset: None,
        using: Some("combined_vector".to_string()),
		strategy: Some(RecommendStrategy::BestScore.into()),
		..Default::default()
	};

	// Call Qdrant
	let response = client.recommend(request).await.map_err(|e| {
        DbError::Other(format!("Qdrant recommend error: {}", e))
    })?;

	// Extract movie IDs from response
    let recommended_ids: Vec<i64> = response.result.iter().filter_map(|point| {
        if let Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(id)) = &point.id.as_ref().and_then(|pid| pid.point_id_options.as_ref()) {
            Some(*id as i64)
        } else {
            None
        }
    }).collect();

    Ok(recommended_ids)
}

