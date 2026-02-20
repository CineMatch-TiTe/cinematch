//! Timeout rescheduling on server startup.

use std::sync::Arc;

use chrono::Utc;
use log::{debug, error, info};

use crate::domain::get_timeout_secs;
use cinematch_db::domain::Party;
use cinematch_db::repo::party::models::PartyState;

use super::Scheduler;
use super::executor;

/// Reschedule all timeouts on server startup.
/// Call after creating the registry and before starting the HTTP server.
use cinematch_db::AppContext;

/// Reschedule all timeouts on server startup.
/// Call after creating the registry and before starting the HTTP server.
pub async fn reschedule_timeouts_on_startup<C: AppContext + Clone + 'static>(
    registry: &Arc<Scheduler>,
    ctx: C,
) {
    let (voting_secs, watching_secs) = get_timeout_secs();
    let now = Utc::now();

    info!(
        "[Scheduler] Rescheduling timeouts on startup (voting={}s, watching={}s)",
        voting_secs, watching_secs
    );

    // Reschedule phase timeouts (Voting/Watching)
    match Party::get_in_timed_phases(ctx.db().clone()).await {
        Ok(parties) => {
            for (party_id, state, phase_entered_at) in parties {
                let timeout_secs = match state {
                    PartyState::Voting => voting_secs,
                    PartyState::Watching => watching_secs,
                    _ => continue,
                };

                let deadline = phase_entered_at + chrono::Duration::seconds(timeout_secs as i64);
                let remaining = (deadline - now).num_seconds();

                debug!(
                    "[Scheduler] Startup: party {} in {:?} | entered at {} | deadline {} | remaining: {}s",
                    party_id,
                    state,
                    phase_entered_at.format("%H:%M:%S"),
                    deadline.format("%H:%M:%S"),
                    remaining
                );

                if deadline <= now {
                    // Already past deadline, fire immediately
                    info!(
                        "Startup: party {} {:?} timeout already past, executing immediately",
                        party_id, state
                    );
                    executor::execute_phase_timeout(registry, party_id, state, ctx.clone()).await;
                } else {
                    registry
                        .schedule_phase_timeout(party_id, state, deadline, ctx.clone())
                        .await;
                }
            }
        }
        Err(e) => {
            error!("Failed to query parties for timeout reschedule: {:?}", e);
        }
    }

    // Reschedule ready countdowns (Created/Picking with all ready)
    match Party::get_all_ready_in_phases(
        ctx.db().clone(),
        &[PartyState::Created, PartyState::Picking],
    )
    .await
    {
        Ok(party_ids) => {
            for party_id in party_ids {
                info!(
                    "Startup: party {} all ready, scheduling countdown",
                    party_id
                );
                registry
                    .schedule_ready_countdown(party_id, ctx.clone())
                    .await;
            }
        }
        Err(e) => {
            error!(
                "Failed to query all-ready parties for timeout reschedule: {:?}",
                e
            );
        }
    }

    info!("[Scheduler] Startup rescheduling complete");
}
