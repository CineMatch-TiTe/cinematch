use cinematch_common::models::VectorType;
use cinematch_db::domain::Movie;
use cinematch_db::{AppContext, DbError, DbResult};
use log::warn;
use std::collections::HashMap;
use uuid::Uuid;

/// Min number of ratings to use sparse vector reviews
/// Ideally this should be atleast around 15, to get good results
const MIN_RATINGS: usize = 15;

/// Recommend movies from the "ratings" collection (sparse user–movie vectors).
pub async fn recommend_from_reviews(
    ctx: &impl AppContext,
    user_id: Uuid,
    party_id: Option<Uuid>,
    vector_type: VectorType,
    limit: usize,
) -> DbResult<Vec<i64>> {
    use qdrant_client::qdrant::{Condition, Filter, QueryPointsBuilder};
    let user = cinematch_db::domain::user::User::new(user_id);
    let (positive, negative, skipped) = user.get_ratings(ctx).await?;

    // if not enough ratings, fallback to recommend_movies
    let enough_ratings = positive.len() + negative.len() >= MIN_RATINGS;
    if !enough_ratings {
        warn!("Not enough ratings, fallback to recommend_movies");
        return crate::engine::standard::recommend_movies(
            ctx,
            user_id,
            party_id,
            vector_type,
            limit,
        )
        .await;
    }

    let prefs = user.preferences(ctx).await?;
    let prefs_record = prefs.record(ctx).await?;
    let mut sparse: Vec<(u32, f32)> = Vec::new();
    for &mid in &positive {
        sparse.push((mid as u32, 1.0));
    }
    for &mid in &negative {
        sparse.push((mid as u32, -1.0));
    }

    // add all eg positive negative and skipped too into the excluded list
    let excluded: Vec<i64> = positive
        .into_iter()
        .chain(negative.into_iter())
        .chain(skipped.into_iter())
        .collect();
    let excluded_slice = (!excluded.is_empty()).then_some(excluded.as_slice());

    const MAX_MATCH_ANY: usize = 2000;
    let filter_ids: Vec<i64> = excluded.iter().copied().take(MAX_MATCH_ANY).collect();

    let builder = if filter_ids.is_empty() {
        QueryPointsBuilder::new("ratings")
            .query(sparse)
            .using("ratings")
            .limit(200)
            .with_payload(true)
    } else {
        let filter = Filter::must_not([Condition::matches("movie_id", filter_ids)]);
        QueryPointsBuilder::new("ratings")
            .query(sparse)
            .using("ratings")
            .limit(200)
            .with_payload(true)
            .filter(filter)
    };

    let client = ctx.db().vector.client.clone();
    let response = client
        .query(builder)
        .await
        .map_err(|e| DbError::Other(format!("Qdrant query (ratings) error: {}", e)))?;

    // Aggregate: each result = similar user. Use payload["movie_id"] only.
    let mut movie_scores: HashMap<i64, f64> = HashMap::new();
    for point in &response.result {
        let score = point.score as f64;
        if let Some(v) = point.payload.get("movie_id")
            && let Some(qdrant_client::qdrant::value::Kind::ListValue(list)) = &v.kind
        {
            for val in &list.values {
                if let Some(qdrant_client::qdrant::value::Kind::IntegerValue(mid)) = val.kind {
                    *movie_scores.entry(mid).or_default() += score;
                }
            }
        }
    }

    let mut sorted: Vec<(i64, f64)> = movie_scores.into_iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut recommended_ids: Vec<i64> = Vec::with_capacity(limit);
    for (id, _) in &sorted {
        if Movie::new(*id)
            .matches_prefs(ctx, &prefs_record, excluded_slice)
            .await?
        {
            recommended_ids.push(*id);
            if recommended_ids.len() == limit {
                break;
            }
        }
    }

    if recommended_ids.is_empty() {
        warn!("No recommendations from reviews, using recommend_movies fallback");
        return crate::engine::standard::recommend_movies(
            ctx,
            user_id,
            party_id,
            vector_type,
            limit,
        )
        .await;
    }
    Ok(recommended_ids)
}
