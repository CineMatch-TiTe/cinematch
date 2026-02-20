use cinematch_common::models::VectorType;
use cinematch_db::domain::Movie;
use cinematch_db::{AppContext, DbError, DbResult};
use qdrant_client::qdrant::{
    PointId, RecommendPoints, RecommendStrategy, WithPayloadSelector, WithVectorsSelector,
};
use uuid::Uuid;

/// Recommend up to `limit` movies from `pool` (HasId filter) for a user, using taste + prefs.
/// Uses Qdrant recommend. Returns IDs from the pool only. If pool is empty, returns [].
pub async fn recommend_from_pool(
    ctx: &impl AppContext,
    user_id: Uuid,
    pool: &[i64],
    vector_type: VectorType,
    limit: usize,
) -> DbResult<Vec<i64>> {
    if pool.is_empty() || limit == 0 {
        return Ok(vec![]);
    }

    let user = cinematch_db::domain::user::User::new(user_id);
    let (positive, negative, skipped) = user.get_ratings(ctx).await?;
    let genre_map = Movie::all_genres(ctx).await?;
    let prefs = user.preferences(ctx).await?;
    let prefs_record = prefs.record(ctx).await?;
    let prefs_filter = crate::utils::filter_from_prefs(&prefs_record, &genre_map);

    let mut positive = positive;
    if positive.is_empty() {
        let popular_movies = Movie::popular(ctx, 5).await?;
        positive = popular_movies.into_iter().map(|m| m.movie_id).collect();
    }

    // Exclude skipped movies from pool
    let skipped_set: std::collections::HashSet<i64> = skipped.into_iter().collect();
    let filtered_pool: Vec<i64> = pool
        .iter()
        .copied()
        .filter(|id| !skipped_set.contains(id))
        .collect();

    let filter = crate::utils::filter_pool_and_prefs(&filtered_pool, prefs_filter.as_ref());

    let client = ctx.db().vector.client.clone();
    let positive_ids: Vec<PointId> = positive
        .iter()
        .filter(|&id| filtered_pool.contains(id))
        .map(|&id| PointId {
            point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(
                id as u64,
            )),
        })
        .collect();

    let positive_ids = if positive_ids.is_empty() {
        filtered_pool
            .iter()
            .take(5)
            .map(|&id| PointId {
                point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(
                    id as u64,
                )),
            })
            .collect::<Vec<_>>()
    } else {
        positive_ids
    };

    let negative_ids: Vec<PointId> = negative
        .iter()
        .map(|&id| PointId {
            point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(
                id as u64,
            )),
        })
        .collect();

    let request = RecommendPoints {
        collection_name: "movies".to_string(),
        positive: positive_ids,
        negative: negative_ids,
        filter: Some(filter),
        limit: limit as u64,
        with_payload: Some(WithPayloadSelector {
            selector_options: Some(
                qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(true),
            ),
        }),
        with_vectors: Some(WithVectorsSelector {
            selector_options: Some(
                qdrant_client::qdrant::with_vectors_selector::SelectorOptions::Enable(false),
            ),
        }),
        score_threshold: None,
        offset: None,
        using: Some(vector_type.as_str().to_string()),
        strategy: Some(RecommendStrategy::AverageVector.into()),
        ..Default::default()
    };

    let response = client
        .recommend(request)
        .await
        .map_err(|e| DbError::Other(format!("Qdrant recommend error: {}", e)))?;

    let recommended_ids: Vec<i64> = response
        .result
        .iter()
        .filter_map(|point| {
            if let Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(id)) = &point
                .id
                .as_ref()
                .and_then(|pid| pid.point_id_options.as_ref())
            {
                let mid = *id as i64;
                if filtered_pool.contains(&mid) {
                    Some(mid)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    Ok(recommended_ids)
}
