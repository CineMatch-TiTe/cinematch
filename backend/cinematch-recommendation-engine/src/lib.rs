use cinematch_common::models::VectorType;
use cinematch_db::domain::Movie;
use cinematch_db::{AppContext, DbResult};
use qdrant_client::qdrant::{
    PointId, RecommendPoints, RecommendStrategy, WithPayloadSelector, WithVectorsSelector,
};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub mod onboarding;
mod utils;
use log::warn;

use cinematch_db::DbError;

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

const BALLOT_SIZE: usize = 5;
const PARTY_POOL_LIMIT: usize = 3;
const OWN_POOL_LIMIT: usize = 2;
const POPULAR_FALLBACK_LIMIT: i64 = 30;

/// Build voting ballots using Qdrant-backed recommendations. Party pool = all picks (shared);
/// per user: 3 from party pool + 2 from own pool, shuffle, take 5. Pad from popular if needed.
pub async fn build_voting_ballots_for_party(
    ctx: &impl AppContext,
    party: &cinematch_db::domain::Party,
) -> DbResult<()> {
    use rand::seq::SliceRandom;
    // party is passed in, no need to create it

    party.clear_ballots(ctx).await?;

    let members = party.member_records(ctx).await?;
    if members.is_empty() {
        return Ok(());
    }

    let party_taste = party.get_picks(ctx).await?;
    let mut picks_by_user: HashMap<Uuid, Vec<i64>> = HashMap::new();
    let mut party_pool_set: HashSet<i64> = HashSet::new();
    for (uid, mid, liked) in party_taste {
        // Only include positive picks (liked = Some(true))
        if liked == Some(true) {
            picks_by_user.entry(uid).or_default().push(mid);
            party_pool_set.insert(mid);
        }
    }
    let party_pool: Vec<i64> = party_pool_set.into_iter().collect();

    let popular = Movie::popular_ids(ctx, POPULAR_FALLBACK_LIMIT)
        .await
        .unwrap_or_default();
    use rand::SeedableRng;
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

    for member in &members {
        let user_id = member.user_id;
        let own_pool: Vec<i64> = picks_by_user
            .get(&user_id)
            .map(|v| v.to_vec())
            .unwrap_or_default();

        let recs_party = recommend_from_pool(
            ctx,
            user_id,
            &party_pool,
            VectorType::Combined,
            PARTY_POOL_LIMIT,
        )
        .await?;
        let recs_own = recommend_from_pool(
            ctx,
            user_id,
            &own_pool,
            VectorType::Combined,
            OWN_POOL_LIMIT,
        )
        .await?;

        let mut ballot: Vec<i64> = recs_party;
        for mid in recs_own {
            if !ballot.contains(&mid) {
                ballot.push(mid);
            }
        }
        ballot.shuffle(&mut rng);
        let mut used: HashSet<i64> = ballot.iter().copied().collect();
        while ballot.len() < BALLOT_SIZE {
            let mut added = false;
            for &mid in &popular {
                if !used.contains(&mid) {
                    ballot.push(mid);
                    used.insert(mid);
                    added = true;
                    break;
                }
            }
            if !added {
                break;
            }
        }
        let mut ballot: Vec<i64> = ballot.into_iter().take(BALLOT_SIZE).collect();
        ballot.shuffle(&mut rng);
        if !ballot.is_empty() {
            party.add_shown_movies(ctx, user_id, &ballot).await?;
        }
    }

    // Side effects (enable voting, set round) removed. Caller must handle them.
    Ok(())
}

/// Build round-2 ballots using Qdrant: same top-3 pool for everyone, recommend_from_pool per user.
/// Clears shown_movies, adds per-user ballots.
pub async fn build_round2_ballots_for_party(
    ctx: &impl AppContext,
    party: &cinematch_db::domain::Party,
    top3: &[i64],
) -> DbResult<()> {
    if top3.is_empty() {
        return Ok(());
    }
    // party passed in
    party.clear_ballots(ctx).await?;
    let members = party.member_records(ctx).await?;

    for member in &members {
        let recs = recommend_from_pool(ctx, member.user_id, top3, VectorType::Combined, 3).await?;
        let ballot: Vec<i64> = if recs.len() >= 3 {
            recs
        } else {
            let mut b = recs;
            for &mid in top3 {
                if b.len() >= 3 {
                    break;
                }
                if !b.contains(&mid) {
                    b.push(mid);
                }
            }
            b
        };
        if !ballot.is_empty() {
            party.add_shown_movies(ctx, member.user_id, &ballot).await?;
        }
    }

    // Side effects removed. Caller handles state transition.
    Ok(())
}

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
        // For now, let's use global ratings + party picks combined or just ratings.
        // Usually, we want to exclude anything they've already seen/acted upon.
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

    // get users prefs

    let genre_map = Movie::all_genres(ctx).await?;
    let prefs = user.preferences(ctx).await?;
    let prefs_record = prefs.record(ctx).await?;

    let filter = crate::utils::filter_from_prefs(&prefs_record, &genre_map);

    let mut positive = positive;
    if positive.is_empty() {
        let popular_movies = Movie::popular(ctx, 5).await?;
        positive = popular_movies.into_iter().map(|m| m.movie_id).collect();
    }

    // Get QdrantService from db.vector (assume db.vector is QdrantService)
    let client = ctx.db().vector.client.clone();

    // Convert i64 IDs to Qdrant PointId
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

    // Build the recommend request
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

    // Call Qdrant
    let response = client
        .recommend(request)
        .await
        .map_err(|e| DbError::Other(format!("Qdrant recommend error: {}", e)))?;

    // Extract movie IDs from response
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

/// Recommend movies from the "ratings" collection (sparse user–movie vectors).
pub async fn recommed_movies_from_reviews(
    ctx: &impl AppContext,
    user_id: Uuid,
    party_id: Option<Uuid>,
    vector_type: VectorType,
    limit: usize,
) -> DbResult<Vec<i64>> {
    use qdrant_client::qdrant::{Condition, Filter, QueryPointsBuilder};
    let user = cinematch_db::domain::user::User::new(user_id);
    let (positive, negative, skipped) = user.get_ratings(ctx).await?;
    let prefs = user.preferences(ctx).await?;
    let prefs_record = prefs.record(ctx).await?;
    let mut sparse: Vec<(u32, f32)> = Vec::new();
    for &mid in &positive {
        sparse.push((mid as u32, 1.0));
    }
    for &mid in &negative {
        sparse.push((mid as u32, -1.0));
    }

    if sparse.is_empty() {
        warn!("No sparse vector, fallback to recommend_movies");
        return self::recommend_movies(ctx, user_id, party_id, vector_type, limit).await;
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
        return recommend_movies(ctx, user_id, party_id, vector_type, limit).await;
    }
    Ok(recommended_ids)
}
