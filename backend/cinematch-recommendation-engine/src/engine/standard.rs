use cinematch_common::models::VectorType;
use cinematch_db::domain::Movie;
use cinematch_db::{AppContext, DbError, DbResult};
use qdrant_client::qdrant::{
    PointId, RecommendPoints, RecommendStrategy, WithPayloadSelector, WithVectorsSelector,
};
use uuid::Uuid;

/// Recommend movies for a user using Qdrant vector search, based on their taste profile.
/// If party_id is provided, excludes movies already picked (liked, disliked, or skipped) in that party.
pub async fn recommend_movies(
    ctx: &impl AppContext,
    user_id: Uuid,
    party_id: Option<Uuid>,
    vector_type: VectorType,
    limit: usize,
) -> DbResult<Vec<i64>> {
    let user = cinematch_db::domain::user::User::new(user_id);
    let (positive, negative, skipped) = if let Some(pid) = party_id {
        // If in party context, we might want to exclude party-specific picks too?
        let (pos, neg, skp) = user.get_ratings(ctx).await?;
        let picks = user.get_party_picks(ctx, pid).await?;
        let mut all_pos = pos;
        for p in picks {
            if !all_pos.contains(&p) {
                all_pos.push(p);
            }
        }
        (all_pos, neg, skp)
    } else {
        user.get_ratings(ctx).await?
    };

    let genre_map = Movie::all_genres(ctx).await?;
    let prefs = user.preferences(ctx).await?;
    let prefs_record = prefs.record(ctx).await?;

    let filter = crate::utils::filter_from_prefs(&prefs_record, &genre_map);

    let mut positive = positive;
    if positive.is_empty() {
        let popular_movies = Movie::popular(ctx, 5).await?;
        positive = popular_movies.into_iter().map(|m| m.movie_id).collect();
    }

    let client = ctx.db().vector.client.clone();

    let positive_ids: Vec<PointId> = positive
        .iter()
        .map(|&id| PointId {
            point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(
                id as u64,
            )),
        })
        .collect();
    let negative_ids: Vec<PointId> = negative
        .iter()
        .map(|&id| PointId {
            point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(
                id as u64,
            )),
        })
        .collect();

    // Exclude skipped movies and party picks from filter
    let excluded: Vec<i64> = positive
        .into_iter()
        .chain(negative.into_iter())
        .chain(skipped.into_iter())
        .collect();
    let mut final_filter = filter;
    if !excluded.is_empty() {
        let excluded_ids: Vec<PointId> = excluded
            .iter()
            .map(|&id| PointId {
                point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(
                    id as u64,
                )),
            })
            .collect();
        let mut must_not = final_filter
            .as_ref()
            .map(|f| f.must_not.clone())
            .unwrap_or_default();
        must_not.push(qdrant_client::qdrant::Condition::has_id(excluded_ids));
        final_filter = Some(qdrant_client::qdrant::Filter {
            must: final_filter
                .as_ref()
                .map(|f| f.must.clone())
                .unwrap_or_default(),
            must_not,
            ..Default::default()
        });
    }

    let request = RecommendPoints {
        collection_name: "movies".to_string(),
        positive: positive_ids,
        negative: negative_ids,
        filter: final_filter,
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
                Some(*id as i64)
            } else {
                None
            }
        })
        .collect();

    Ok(recommended_ids)
}
