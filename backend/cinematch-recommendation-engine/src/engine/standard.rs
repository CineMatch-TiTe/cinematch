use cinematch_common::models::VectorType;
use cinematch_db::domain::Movie;
use cinematch_db::{AppContext, DbError, DbResult};
use qdrant_client::qdrant::{
    Condition, Filter, PointId, RecommendPoints, RecommendResponse, RecommendStrategy,
    WithPayloadSelector, WithVectorsSelector,
};
use uuid::Uuid;

/// Recommend movies for a user using Qdrant vector search, based on their taste profile.
///
/// This is the standard recommendation strategy that uses a weighted average of positive
/// and negative movie embeddings to find similar content.
pub async fn recommend_movies(
    ctx: &impl AppContext,
    user_id: Uuid,
    party_id: Option<Uuid>,
    vector_type: VectorType,
    limit: usize,
) -> DbResult<Vec<i64>> {
    let (positive, negative, skipped) = fetch_user_taste(ctx, user_id, party_id).await?;
    let genre_map = Movie::all_genres(ctx).await?;
    let user = cinematch_db::domain::user::User::new(user_id);
    let prefs = user.preferences(ctx).await?;
    let prefs_record = prefs.record(ctx).await?;

    // Combine preference filters with exclusions (seen/skipped movies)
    let filter =
        build_recommendation_filter(&prefs_record, &genre_map, &positive, &negative, &skipped);

    // Fallback if no positive signals exist: use popular movies as seeds
    let positive_ids = if positive.is_empty() {
        let popular = Movie::popular(ctx, 5).await?;
        to_point_ids(&popular.into_iter().map(|m| m.movie_id).collect::<Vec<_>>())
    } else {
        to_point_ids(&positive)
    };

    let negative_ids = to_point_ids(&negative);

    let request = build_qdrant_request(positive_ids, negative_ids, filter, limit, vector_type);

    let client = ctx.db().vector.client.clone();
    let response = client
        .recommend(request)
        .await
        .map_err(|e| DbError::Other(format!("Qdrant recommend error: {}", e)))?;

    Ok(extract_movie_ids(response))
}

/// Fetches the user's positive, negative, and skipped ratings.
/// If in a party context, also incorporates picks from that party.
async fn fetch_user_taste(
    ctx: &impl AppContext,
    user_id: Uuid,
    party_id: Option<Uuid>,
) -> DbResult<(Vec<i64>, Vec<i64>, Vec<i64>)> {
    let user = cinematch_db::domain::user::User::new(user_id);
    let (mut positive, negative, skipped) = user.get_ratings(ctx).await?;

    if let Some(pid) = party_id {
        let picks = user.get_party_picks(ctx, pid).await?;
        for p in picks {
            if !positive.contains(&p) {
                positive.push(p);
            }
        }
    }

    Ok((positive, negative, skipped))
}

/// Builds a Qdrant filter that combines user preferences and excludes already interactive movies.
fn build_recommendation_filter(
    prefs: &cinematch_common::models::FullUserPreferences,
    genre_map: &std::collections::HashMap<String, Uuid>,
    positive: &[i64],
    negative: &[i64],
    skipped: &[i64],
) -> Option<Filter> {
    let base_filter = crate::utils::filter_from_prefs(prefs, genre_map);

    let mut excluded = Vec::new();
    excluded.extend_from_slice(positive);
    excluded.extend_from_slice(negative);
    excluded.extend_from_slice(skipped);

    if excluded.is_empty() {
        return base_filter;
    }

    let excluded_ids = to_point_ids(&excluded);
    let mut must_not = base_filter
        .as_ref()
        .map(|f| f.must_not.clone())
        .unwrap_or_default();

    must_not.push(Condition::has_id(excluded_ids));

    Some(Filter {
        must: base_filter
            .as_ref()
            .map(|f| f.must.clone())
            .unwrap_or_default(),
        must_not,
        ..Default::default()
    })
}

/// Converts a slice of movie IDs to Qdrant PointId format.
fn to_point_ids(ids: &[i64]) -> Vec<PointId> {
    ids.iter()
        .map(|&id| PointId {
            point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(
                id as u64,
            )),
        })
        .collect()
}

/// Constructs the final Qdrant RecommendPoints request.
fn build_qdrant_request(
    positive: Vec<PointId>,
    negative: Vec<PointId>,
    filter: Option<Filter>,
    limit: usize,
    vector_type: VectorType,
) -> RecommendPoints {
    RecommendPoints {
        collection_name: "movies".to_string(),
        positive,
        negative,
        filter,
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

/// Extracts movie IDs from the Qdrant recommendation response.
fn extract_movie_ids(response: RecommendResponse) -> Vec<i64> {
    response
        .result
        .iter()
        .filter_map(|point| {
            point
                .id
                .as_ref()
                .and_then(|pid| pid.point_id_options.as_ref())
                .and_then(|opt| match opt {
                    qdrant_client::qdrant::point_id::PointIdOptions::Num(id) => Some(*id as i64),
                    _ => None,
                })
        })
        .collect()
}
