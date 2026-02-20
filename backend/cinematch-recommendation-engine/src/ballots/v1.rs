use crate::engine::pool::recommend_from_pool;
use cinematch_common::models::VectorType;
use cinematch_db::domain::Movie;
use cinematch_db::{AppContext, DbResult};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

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
    use rand::SeedableRng;
    use rand::seq::SliceRandom;

    party.clear_ballots(ctx).await?;

    let members = party.member_records(ctx).await?;
    if members.is_empty() {
        return Ok(());
    }

    let party_taste = party.get_picks(ctx).await?;
    let mut picks_by_user: HashMap<Uuid, Vec<i64>> = HashMap::new();
    let mut party_pool_set: HashSet<i64> = HashSet::new();
    for (uid, mid, liked) in party_taste {
        if liked == Some(true) {
            picks_by_user.entry(uid).or_default().push(mid);
            party_pool_set.insert(mid);
        }
    }
    let party_pool: Vec<i64> = party_pool_set.into_iter().collect();

    let popular = Movie::popular_ids(ctx, POPULAR_FALLBACK_LIMIT)
        .await
        .unwrap_or_default();

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

    Ok(())
}
