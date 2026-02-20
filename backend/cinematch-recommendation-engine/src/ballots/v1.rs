use crate::engine::pool::recommend_from_pool;
use cinematch_common::models::VectorType;
use cinematch_db::domain::Movie;
use cinematch_db::{AppContext, DbResult};
use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

const BALLOT_SIZE: usize = 5;
const PARTY_POOL_LIMIT: usize = 3;
const OWN_POOL_LIMIT: usize = 2;
const POPULAR_FALLBACK_LIMIT: i64 = 30;

/// Build voting ballots using Qdrant-backed recommendations.
///
/// This function constructs per-user ballots for the voting phase by mixing:
/// - Movies liked by the user (personal favorites).
/// - Movies liked by other party members (group favorites).
/// - Popular movies (fallback if the pools are too small).
pub async fn build_voting_ballots_for_party(
    ctx: &impl AppContext,
    party: &cinematch_db::domain::Party,
) -> DbResult<()> {
    party.clear_ballots(ctx).await?;

    let members = party.member_records(ctx).await?;
    if members.is_empty() {
        return Ok(());
    }

    let (picks_by_user, party_pool) = collect_party_taste(ctx, party).await?;
    let popular_ids = Movie::popular_ids(ctx, POPULAR_FALLBACK_LIMIT)
        .await
        .unwrap_or_default();

    let mut rng = create_seeded_rng();

    for member in members {
        let user_id = member.user_id;
        let own_pool = picks_by_user.get(&user_id).cloned().unwrap_or_default();

        let ballot =
            construct_user_ballot(ctx, user_id, &party_pool, &own_pool, &popular_ids, &mut rng)
                .await?;

        if !ballot.is_empty() {
            party.add_shown_movies(ctx, user_id, &ballot).await?;
        }
    }

    Ok(())
}

/// Collects likes for each user and builds the shared party pool.
async fn collect_party_taste(
    ctx: &impl AppContext,
    party: &cinematch_db::domain::Party,
) -> DbResult<(HashMap<Uuid, Vec<i64>>, Vec<i64>)> {
    let party_taste = party.get_picks(ctx).await?;
    let mut picks_by_user: HashMap<Uuid, Vec<i64>> = HashMap::new();
    let mut party_pool_set: HashSet<i64> = HashSet::new();

    for (uid, mid, liked) in party_taste {
        if liked == Some(true) {
            picks_by_user.entry(uid).or_default().push(mid);
            party_pool_set.insert(mid);
        }
    }

    let party_pool = party_pool_set.into_iter().collect();
    Ok((picks_by_user, party_pool))
}

/// Constructs a single user's ballot by recommending from party and personal pools.
async fn construct_user_ballot(
    ctx: &impl AppContext,
    user_id: Uuid,
    party_pool: &[i64],
    own_pool: &[i64],
    popular_ids: &[i64],
    rng: &mut rand::rngs::StdRng,
) -> DbResult<Vec<i64>> {
    let recs_party = recommend_from_pool(
        ctx,
        user_id,
        party_pool,
        VectorType::Combined,
        PARTY_POOL_LIMIT,
    )
    .await?;
    let recs_own =
        recommend_from_pool(ctx, user_id, own_pool, VectorType::Combined, OWN_POOL_LIMIT).await?;

    let mut ballot = recs_party;
    for mid in recs_own {
        if !ballot.contains(&mid) {
            ballot.push(mid);
        }
    }

    ballot.shuffle(rng);

    // Pad with popular movies if the pools were insufficient
    let mut used: HashSet<i64> = ballot.iter().copied().collect();
    while ballot.len() < BALLOT_SIZE {
        let mut added = false;
        for &mid in popular_ids {
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

    let mut final_ballot: Vec<i64> = ballot.into_iter().take(BALLOT_SIZE).collect();
    final_ballot.shuffle(rng);
    Ok(final_ballot)
}

fn create_seeded_rng() -> rand::rngs::StdRng {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    rand::SeedableRng::seed_from_u64(seed)
}
