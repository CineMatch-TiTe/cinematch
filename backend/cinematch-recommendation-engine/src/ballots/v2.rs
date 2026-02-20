use crate::engine::pool::recommend_from_pool;
use cinematch_common::models::VectorType;
use cinematch_db::{AppContext, DbResult};

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

    Ok(())
}
