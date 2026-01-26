use cinematch_db::{Database, DbResult};
use qdrant_client::qdrant::{
    PointId, RecommendPoints, RecommendStrategy, WithPayloadSelector, WithVectorsSelector,
};
use uuid::Uuid;

mod utils;

use cinematch_db::DbError;

/// Recommend up to `limit` movies from `pool` (HasId filter) for a user, using taste + prefs.
/// Uses Qdrant recommend. Returns IDs from the pool only. If pool is empty, returns [].
pub async fn recommend_from_pool(
    db: &Database,
    user_id: Uuid,
    pool: &[i64],
    limit: usize,
) -> DbResult<Vec<i64>> {
    if pool.is_empty() || limit == 0 {
        return Ok(vec![]);
    }

    let (positive, negative, skipped) = db.get_taste(user_id).await?;
    let genre_map = db.get_genres().await?;
    let prefs = db.get_user_preferences(user_id).await?;
    let prefs_filter = crate::utils::filter_from_prefs(&prefs, &genre_map);

    let mut positive = positive;
    if positive.is_empty() {
        let popular_movies = db.get_popular_movies(5).await?;
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

    let client = db.vector.client.clone();
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
        using: Some("combined_vector".to_string()),
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
pub async fn build_voting_ballots_for_party(db: &Database, party_id: Uuid) -> DbResult<()> {
    use rand::seq::SliceRandom;
    use std::collections::{HashMap, HashSet};

    db.clear_shown_movies_for_party(party_id).await?;

    let members = db.get_party_members(party_id).await?;
    if members.is_empty() {
        return Ok(());
    }

    let party_taste = db.get_party_taste(party_id).await?;
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

    let popular = db
        .get_popular_movie_ids(POPULAR_FALLBACK_LIMIT)
        .await
        .unwrap_or_default();
    let mut rng = rand::rng();

    for member in &members {
        let user_id = member.user_id;
        let own_pool: Vec<i64> = picks_by_user
            .get(&user_id)
            .map(|v| v.to_vec())
            .unwrap_or_default();

        let recs_party = recommend_from_pool(db, user_id, &party_pool, PARTY_POOL_LIMIT).await?;
        let recs_own = recommend_from_pool(db, user_id, &own_pool, OWN_POOL_LIMIT).await?;

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
            db.add_shown_movies(party_id, user_id, &ballot).await?;
        }
    }

    db.enable_voting(party_id).await?;
    db.set_voting_round(party_id, Some(1)).await?;
    Ok(())
}

/// Build round-2 ballots using Qdrant: same top-3 pool for everyone, recommend_from_pool per user.
/// Clears shown_movies, adds per-user ballots, enables voting, sets round 2 and phase_entered_at.
pub async fn build_round2_ballots_for_party(
    db: &Database,
    party_id: Uuid,
    top3: &[i64],
) -> DbResult<()> {
    if top3.is_empty() {
        return Ok(());
    }
    db.clear_shown_movies_for_party(party_id).await?;
    let members = db.get_party_members(party_id).await?;
    for member in &members {
        let recs = recommend_from_pool(db, member.user_id, top3, 3).await?;
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
            db.add_shown_movies(party_id, member.user_id, &ballot)
                .await?;
        }
    }
    db.enable_voting(party_id).await?;
    db.set_voting_round(party_id, Some(2)).await?;
    db.set_phase_entered_at_now(party_id).await?;
    Ok(())
}

/// Recommend movies for a user using Qdrant vector search, based on their taste profile.
/// If party_id is provided, excludes movies already picked (liked, disliked, or skipped) in that party.
pub async fn recommend_movies(
    db: &Database,
    user_id: Uuid,
    limit: usize,
    party_id: Option<Uuid>,
) -> DbResult<Vec<i64>> {
    let (positive, negative, skipped) = db.get_taste(user_id).await?;

    // get users prefs

    let genre_map = db.get_genres().await?;
    let prefs = db.get_user_preferences(user_id).await?;

    let filter = crate::utils::filter_from_prefs(&prefs, &genre_map);

    // Collect all movies to exclude
    let mut excluded: std::collections::HashSet<i64> = skipped.into_iter().collect();

    // If party_id provided, exclude movies already picked in that party (liked, disliked, or skipped)
    if let Some(pid) = party_id {
        let party_taste = db.get_party_taste(pid).await?;
        for (_, mid, _) in party_taste {
            excluded.insert(mid);
        }
    }

    let mut positive = positive;
    if positive.is_empty() {
        let popular_movies = db.get_popular_movies(5).await?;
        positive = popular_movies.into_iter().map(|m| m.movie_id).collect();
    }

    // Get QdrantService from db.vector (assume db.vector is QdrantService)
    let client = db.vector.client.clone();

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
        using: Some("combined_vector".to_string()),
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
