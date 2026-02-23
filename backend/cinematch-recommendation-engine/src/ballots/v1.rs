use crate::engine::pool::recommend_from_pool;
use cinematch_common::models::VectorType;
use cinematch_db::domain::Movie;
use cinematch_db::{AppContext, DbResult};
use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

const BALLOT_SIZE: usize = 5;
const POPULAR_FALLBACK_LIMIT: i64 = 30;

/// Build voting ballots using Qdrant-backed recommendations.
///
/// This function constructs a Shared Party Ballot for the voting phase:
/// - Combines all members' picks into a single party pool.
/// - Uses the recommendation engine (via the leader's profile) to select the best 5 movies from the pool.
/// - Pads with popular movies if the party pool is too small.
/// - Assigns the exact same ballot to everyone in the party.
pub async fn build_voting_ballots_for_party(
    ctx: &impl AppContext,
    party: &cinematch_db::domain::Party,
) -> DbResult<()> {
    party.clear_ballots(ctx).await?;

    let members = party.member_records(ctx).await?;
    if members.is_empty() {
        return Ok(());
    }

    // Determine the user to query Qdrant with (e.g., the leader)
    let leader_id = party.leader_id(ctx).await?;

    let (_, party_pool) = collect_party_taste(ctx, party).await?;
    let popular_ids = Movie::popular_ids(ctx, POPULAR_FALLBACK_LIMIT, None).await?;

    let mut rng = create_seeded_rng();

    let shared_pool_size =
        std::cmp::max(1, (members.len() * 3) / std::cmp::max(1, members.len() / 2));

    // Get shared movies that best represent the whole party
    let shared_movies = recommend_from_pool(
        ctx,
        leader_id,
        &party_pool,
        VectorType::Combined,
        shared_pool_size,
    )
    .await?;

    // Now build individual hybrid ballots
    for member in members {
        let own_pool = collect_user_taste(ctx, party, member.user_id).await?;

        let user_ballot = construct_hybrid_ballot(
            ctx,
            member.user_id,
            &shared_movies,
            &own_pool,
            &party_pool,
            &popular_ids,
            &mut rng,
        )
        .await?;

        if !user_ballot.is_empty() {
            party
                .add_shown_movies(ctx, member.user_id, &user_ballot)
                .await?;
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

/// Collects likes for a single user.
async fn collect_user_taste(
    ctx: &impl AppContext,
    party: &cinematch_db::domain::Party,
    user_id: Uuid,
) -> DbResult<Vec<i64>> {
    let user_picks = party.get_user_picks(ctx, user_id).await?;
    let mut own_pool_set: HashSet<i64> = HashSet::new();

    for mid in user_picks {
        own_pool_set.insert(mid);
    }
    Ok(own_pool_set.into_iter().collect())
}

/// Constructs a hybrid round-1 ballot: 3 Shared Core + 2 Personal Pad.
async fn construct_hybrid_ballot(
    ctx: &impl AppContext,
    user_id: Uuid,
    shared_movies: &[i64],
    own_pool: &[i64],
    party_pool: &[i64],
    popular_ids: &[i64],
    rng: &mut rand::rngs::StdRng,
) -> DbResult<Vec<i64>> {
    let mut ballot = Vec::new();
    let mut used = HashSet::new();

    // 1. Pick exactly 3 from the Shared Movies pool (or whatever is available)
    let mut shared_candidates = shared_movies.to_vec();
    shared_candidates.shuffle(rng);
    for &mid in &shared_candidates {
        if ballot.len() >= 3 {
            break;
        }
        if !used.contains(&mid) {
            ballot.push(mid);
            used.insert(mid);
        }
    }

    // 2. Pad to 3 with remaining party picks if the shared pool was short
    if ballot.len() < 3 {
        let mut fallback_party = party_pool.to_vec();
        fallback_party.shuffle(rng);
        for &mid in fallback_party.iter() {
            if ballot.len() >= 3 {
                break;
            }
            if !used.contains(&mid) {
                ballot.push(mid);
                used.insert(mid);
            }
        }
    }

    // 3. Pick 2 "Personal Pad" movies from the user's explicit likes
    let personal_recs =
        recommend_from_pool(ctx, user_id, own_pool, VectorType::Combined, 2).await?;

    for &mid in &personal_recs {
        if ballot.len() >= BALLOT_SIZE {
            break;
        }
        if !used.contains(&mid) {
            ballot.push(mid);
            used.insert(mid);
        }
    }

    // 4. Pad to 5 using the remainder of the party pool
    if ballot.len() < BALLOT_SIZE {
        let mut fallback_party = party_pool.to_vec();
        fallback_party.shuffle(rng);
        for &mid in fallback_party.iter() {
            if ballot.len() >= BALLOT_SIZE {
                break;
            }
            if !used.contains(&mid) {
                ballot.push(mid);
                used.insert(mid);
            }
        }
    }

    // 5. Final safety pad with popular movies
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

    // Ensure strict BALLOT_SIZE bounds and shuffle before sending down the pipe
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
