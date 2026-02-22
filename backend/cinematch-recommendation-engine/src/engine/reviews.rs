use cinematch_common::models::VectorType;
use cinematch_db::domain::Movie;
use cinematch_db::{AppContext, DbError, DbResult};
use log::warn;
use qdrant_client::qdrant::{Condition, Filter, QueryPointsBuilder, QueryResponse};
use std::collections::HashMap;
use uuid::Uuid;

/// Min number of ratings to use sparse vector reviews
/// Ideally this should be atleast around 15, to get good results
const MIN_RATINGS: usize = 15;

/// Recommend movies from the "ratings" collection (sparse user–movie vectors).
///
/// This strategy uses collaborative filtering by finding users with similar rating
/// patterns in a sparse vector space.
pub async fn recommend_from_reviews(
    ctx: &impl AppContext,
    user_id: Uuid,
    party_id: Option<Uuid>,
    vector_type: VectorType,
    limit: usize,
) -> DbResult<Vec<i64>> {
    let user = cinematch_db::domain::user::User::new(user_id);
    let (positive, negative, skipped) = user.get_ratings(ctx).await?;

    if positive.len() + negative.len() < MIN_RATINGS {
        warn!("Not enough ratings, fallback to standard recommendation");
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

    let sparse_vector = build_sparse_vector(&positive, &negative);
    let excluded = collect_exclusions(&positive, &negative, &skipped);

    let similar_users = fetch_similar_users(ctx, sparse_vector, &excluded).await?;
    let movie_scores = aggregate_movie_scores(similar_users);

    let recommended_ids =
        rank_and_filter_movies(ctx, movie_scores, &prefs_record, &excluded, limit).await?;

    if recommended_ids.is_empty() {
        warn!("No recommendations from reviews, using standard fallback");
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

/// Builds a sparse vector representation of user ratings.
fn build_sparse_vector(positive: &[i64], negative: &[i64]) -> Vec<(u32, f32)> {
    let mut sparse = Vec::with_capacity(positive.len() + negative.len());
    for &mid in positive {
        sparse.push((mid as u32, 1.0));
    }
    for &mid in negative {
        sparse.push((mid as u32, -1.0));
    }
    sparse
}

/// Collects all movie IDs that should be excluded from recommendations.
fn collect_exclusions(positive: &[i64], negative: &[i64], skipped: &[i64]) -> Vec<i64> {
    positive
        .iter()
        .chain(negative.iter())
        .chain(skipped.iter())
        .copied()
        .collect()
}

/// Queries Qdrant for users with similar rating patterns.
async fn fetch_similar_users(
    ctx: &impl AppContext,
    sparse_vector: Vec<(u32, f32)>,
    excluded: &[i64],
) -> DbResult<QueryResponse> {
    const MAX_MATCH_ANY: usize = 2000;
    let filter_ids: Vec<i64> = excluded.iter().copied().take(MAX_MATCH_ANY).collect();

    let mut builder = QueryPointsBuilder::new("ratings")
        .query(sparse_vector)
        .using("ratings")
        .limit(200)
        .with_payload(true);

    if !filter_ids.is_empty() {
        let filter = Filter::must_not([Condition::matches("movie_id", filter_ids)]);
        builder = builder.filter(filter);
    }

    let client = ctx.db().vector.client.clone();
    client
        .query(builder)
        .await
        .map_err(|e| DbError::Other(format!("Qdrant query (ratings) error: {}", e)))
}

/// Aggregates movie scores from similar users' ratings.
fn aggregate_movie_scores(response: QueryResponse) -> HashMap<i64, f64> {
    let mut movie_scores = HashMap::new();
    for point in response.result {
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
    movie_scores
}

/// Ranks movies by aggregated score and filters them based on user preferences.
async fn rank_and_filter_movies(
    ctx: &impl AppContext,
    movie_scores: HashMap<i64, f64>,
    prefs: &cinematch_common::models::FullUserPreferences,
    excluded: &[i64],
    limit: usize,
) -> DbResult<Vec<i64>> {
    let mut sorted: Vec<(i64, f64)> = movie_scores.into_iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut recommended_ids = Vec::with_capacity(limit);
    let excluded_slice = if excluded.is_empty() {
        None
    } else {
        Some(excluded)
    };

    for (id, _) in sorted {
        let movie = Movie::new(id);
        if movie.matches_prefs(ctx, prefs, excluded_slice).await? {
            recommended_ids.push(id);
            if recommended_ids.len() == limit {
                break;
            }
        }
    }

    Ok(recommended_ids)
}
