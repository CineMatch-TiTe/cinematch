use cinematch_common::models::VectorType;
use cinematch_db::domain::Movie;
use cinematch_db::{AppContext, DbError, DbResult};
use qdrant_client::qdrant::{
    Filter, PointId, RecommendPoints, RecommendResponse, RecommendStrategy, WithPayloadSelector,
    WithVectorsSelector,
};
use std::collections::HashSet;
use uuid::Uuid;

/// Recommend up to `limit` movies from `pool` (HasId filter) for a user, using taste + prefs.
///
/// This strategy restricts recommendations to a specific set of movie IDs,
/// useful for voting rounds where candidates are already pre-selected.
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
    let (positive, _, skipped) = user.get_ratings(ctx).await?;
    let (prefs, genre_map) = fetch_prefs_and_genres(ctx, &user).await?;

    let filtered_pool = filter_skipped_from_pool(pool, &skipped);
    let filter = build_pool_filter(&filtered_pool, &prefs, &genre_map);

    let seed_ids = determine_seed_ids(ctx, &positive, &filtered_pool).await?;
    let negative_ids = fetch_negative_ids(ctx, &user).await?;

    let client = ctx.db().vector.client.clone();
    let request = build_pool_request(seed_ids, negative_ids, filter, limit, vector_type);

    let response = client
        .recommend(request)
        .await
        .map_err(|e| DbError::Other(format!("Qdrant recommend error: {}", e)))?;

    Ok(extract_and_validate_ids(response, &filtered_pool))
}

/// Fetches user preferences and the global genre map.
async fn fetch_prefs_and_genres(
    ctx: &impl AppContext,
    user: &cinematch_db::domain::user::User,
) -> DbResult<(
    cinematch_common::models::FullUserPreferences,
    std::collections::HashMap<String, Uuid>,
)> {
    let genre_map = Movie::all_genres(ctx).await?;
    let prefs = user.preferences(ctx).await?;
    let prefs_record = prefs.record(ctx).await?;
    Ok((prefs_record, genre_map))
}

/// Excludes movies that the user has already skipped from the candidate pool.
fn filter_skipped_from_pool(pool: &[i64], skipped: &[i64]) -> Vec<i64> {
    let skipped_set: HashSet<i64> = skipped.iter().copied().collect();
    pool.iter()
        .copied()
        .filter(|id| !skipped_set.contains(id))
        .collect()
}

/// Builds the Qdrant filter combining the pool restriction and user preferences.
fn build_pool_filter(
    pool: &[i64],
    prefs: &cinematch_common::models::FullUserPreferences,
    genre_map: &std::collections::HashMap<String, Uuid>,
) -> Filter {
    let prefs_filter = crate::utils::filter_from_prefs(prefs, genre_map);
    crate::utils::filter_pool_and_prefs(pool, prefs_filter.as_ref())
}

/// Determines the positive seed IDs for recommendation.
/// Falls back to popular movies or pool members if no user ratings exist.
async fn determine_seed_ids(
    ctx: &impl AppContext,
    positive: &[i64],
    filtered_pool: &[i64],
) -> DbResult<Vec<PointId>> {
    let mut seeds = positive.to_vec();
    if seeds.is_empty() {
        let popular = Movie::popular(ctx, 5, None).await?;
        seeds = popular.into_iter().map(|m| m.movie_id).collect();
    }

    let mut point_ids: Vec<PointId> = seeds
        .iter()
        .filter(|&id| filtered_pool.contains(id))
        .map(|&id| to_point_id(id))
        .collect();

    if point_ids.is_empty() && !filtered_pool.is_empty() {
        point_ids = filtered_pool
            .iter()
            .take(5)
            .map(|&id| to_point_id(id))
            .collect();
    }

    Ok(point_ids)
}

/// Fetches the negative rating IDs for the user.
async fn fetch_negative_ids(
    ctx: &impl AppContext,
    user: &cinematch_db::domain::user::User,
) -> DbResult<Vec<PointId>> {
    let (_, negative, _) = user.get_ratings(ctx).await?;
    Ok(negative.iter().map(|&id| to_point_id(id)).collect())
}

/// Constructs the Qdrant RecommendPoints request for pool-based recommendation.
fn build_pool_request(
    positive: Vec<PointId>,
    negative: Vec<PointId>,
    filter: Filter,
    limit: usize,
    vector_type: VectorType,
) -> RecommendPoints {
    RecommendPoints {
        collection_name: "movies".to_string(),
        positive,
        negative,
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
        using: Some(vector_type.as_str().to_string()),
        strategy: Some(RecommendStrategy::AverageVector.into()),
        ..Default::default()
    }
}

/// Extracts IDs from the response and ensures they belong to the filtered pool.
fn extract_and_validate_ids(response: RecommendResponse, filtered_pool: &[i64]) -> Vec<i64> {
    response
        .result
        .iter()
        .filter_map(|point| {
            point
                .id
                .as_ref()
                .and_then(|pid| pid.point_id_options.as_ref())
                .and_then(|opt| match opt {
                    qdrant_client::qdrant::point_id::PointIdOptions::Num(id) => {
                        let mid = *id as i64;
                        if filtered_pool.contains(&mid) {
                            Some(mid)
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
        })
        .collect()
}

fn to_point_id(id: i64) -> PointId {
    PointId {
        point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(
            id as u64,
        )),
    }
}
