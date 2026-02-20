//! Timeout execution logic for phase timeouts and ready countdowns.

use chrono::Utc;
use log::{debug, error, info, warn};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::{EndVotingTransition, PartyStateMachine, get_timeout_secs};
use cinematch_common::models::websocket::{PartyStateChanged, ServerMessage, TimeoutReason};
use cinematch_db::AppContext;
use cinematch_db::domain::Party;
use cinematch_db::repo::party::models::PartyState;

use super::Scheduler;

/// Execute phase timeout (Voting/Watching).
pub async fn execute_phase_timeout<C: AppContext + Clone + 'static>(
    registry: &Arc<Scheduler>,
    party_id: Uuid,
    expected_phase: PartyState,
    ctx: C,
) {
    // Remove from tasks first (we're executing)
    registry.remove_task(party_id).await;

    // Verify party still in expected phase
    let Ok(party) = Party::from_id(&ctx, party_id).await else {
        warn!("Phase timeout: party {} not found", party_id);
        return;
    };

    let Ok(state) = party.state(&ctx).await else {
        warn!("Phase timeout: failed to get party {} state", party_id);
        return;
    };

    if state != expected_phase {
        debug!(
            "Phase timeout: party {} no longer in {:?} (now {:?}), skipping",
            party_id, expected_phase, state
        );
        return;
    }

    // Check if party is empty → disband
    let Ok(members) = party.members(&ctx).await else {
        warn!("Phase timeout: failed to get party {} members", party_id);
        return;
    };

    if members.is_empty() {
        info!("Phase timeout: party {} is empty, disbanding", party_id);
        if let Err(e) = party.disband(&ctx).await {
            error!("Failed to disband empty party {}: {:?}", party_id, e);
        }
        ctx.send_users(&[], &ServerMessage::PartyDisbanded);
        return;
    }

    // Execute phase-specific timeout
    match expected_phase {
        PartyState::Voting => {
            execute_voting_timeout(&party, party_id, ctx.clone()).await;
        }
        PartyState::Watching => {
            execute_watching_timeout(&party, party_id, ctx.clone()).await;
        }
        _ => {
            warn!(
                "Phase timeout for unexpected phase {:?} in party {}",
                expected_phase, party_id
            );
        }
    }
}

async fn execute_voting_timeout<C: AppContext>(party: &Party, party_id: Uuid, ctx: C) {
    info!(
        "[Scheduler] Executing voting timeout for party {}",
        party_id
    );
    match party.force_end_voting_timeout(&ctx).await {
        Ok(transition) => {
            // Broadcast state change with new timeout info
            let (new_state, deadline_at, reason) = match &transition {
                EndVotingTransition::Round2Started => {
                    // Round 2: same phase, new deadline
                    let (voting_secs, _) = get_timeout_secs();
                    let deadline = Utc::now() + chrono::Duration::seconds(voting_secs as i64);
                    (
                        PartyState::Voting,
                        Some(deadline),
                        Some(TimeoutReason::PhaseTimeout),
                    )
                }
                EndVotingTransition::PhaseChanged(new_phase) => match new_phase {
                    PartyState::Watching => {
                        let (_, watching_secs) = get_timeout_secs();
                        let deadline = Utc::now() + chrono::Duration::seconds(watching_secs as i64);
                        (
                            *new_phase,
                            Some(deadline),
                            Some(TimeoutReason::PhaseTimeout),
                        )
                    }
                    _ => (*new_phase, None, None),
                },
            };

            let _member_ids = match party.member_ids(&ctx).await {
                Ok(ids) => ids,
                Err(e) => {
                    error!("Failed to get member IDs for party {}: {:?}", party_id, e);
                    return;
                }
            };

            ctx.broadcast_party(
                party_id,
                &ServerMessage::PartyStateChanged(PartyStateChanged {
                    state: new_state.into(),
                    deadline_at,
                    timeout_reason: reason,
                }),
                None,
            );

            // Note: Rescheduling next timeout for Round2 or Watching is handled
            // by the domain layer via broadcast_party_timeout after transition.
        }
        Err(e) => {
            error!("Voting timeout failed for party {}: {:?}", party_id, e);
        }
    }
}

async fn execute_watching_timeout<C: AppContext>(party: &Party, party_id: Uuid, ctx: C) {
    info!(
        "[Scheduler] Executing watching timeout for party {}",
        party_id
    );
    if let Err(e) = party.do_watching_to_review(&ctx).await {
        error!("Watching timeout failed for party {}: {:?}", party_id, e);
        return;
    }

    let _member_ids = match party.member_ids(&ctx).await {
        Ok(ids) => ids,
        Err(e) => {
            error!("Failed to get member IDs for party {}: {:?}", party_id, e);
            return;
        }
    };

    ctx.broadcast_party(
        party_id,
        &ServerMessage::PartyStateChanged(PartyStateChanged {
            state: PartyState::Review.into(),
            deadline_at: None,
            timeout_reason: None,
        }),
        None,
    );
}

/// Execute ready countdown (all members ready → advance phase).
pub async fn execute_ready_countdown<C: AppContext + Clone + 'static>(
    registry: &Arc<Scheduler>,
    party_id: Uuid,
    ctx: C,
) {
    // Verify all still ready
    let Ok(party) = Party::from_id(&ctx, party_id).await else {
        warn!("Ready countdown: party {} not found", party_id);
        return;
    };

    let Ok(all_ready) = party.are_all_ready(&ctx).await else {
        warn!(
            "Ready countdown: couldn't check ready status for party {}",
            party_id
        );
        return;
    };

    if !all_ready {
        debug!(
            "Ready countdown: party {} no longer all ready, skipping",
            party_id
        );
        return;
    }

    // Check if party is empty → disband
    let Ok(members) = party.members(&ctx).await else {
        warn!("Ready countdown: failed to get party {} members", party_id);
        return;
    };

    if members.is_empty() {
        info!("Ready countdown: party {} is empty, disbanding", party_id);
        if let Err(e) = party.disband(&ctx).await {
            error!("Failed to disband empty party {}: {:?}", party_id, e);
        }
        return;
    }

    let Ok(_state) = party.state(&ctx).await else {
        warn!("Ready countdown: failed to get party {} state", party_id);
        return;
    };

    // Advance phase using shared logic (handles side effects like clearing ballots, releasing codes)
    debug!(
        "[Scheduler] Ready countdown: attempting to advance party {}",
        party_id
    );

    let transition_result = party.try_auto_advance_on_ready(&ctx).await;

    match transition_result {
        Ok(Some(new_phase)) => {
            info!(
                "[Scheduler] Ready countdown: successfully advanced party {} to {:?}",
                party_id, new_phase
            );

            // Calculate timeout for the new phase if applicable
            let (deadline_at, reason) = match new_phase {
                PartyState::Voting => {
                    let (voting_secs, _) = get_timeout_secs();
                    let deadline = Utc::now() + chrono::Duration::seconds(voting_secs as i64);
                    (Some(deadline), Some(TimeoutReason::PhaseTimeout))
                }
                PartyState::Watching => {
                    let (_, watching_secs) = get_timeout_secs();
                    let deadline = Utc::now() + chrono::Duration::seconds(watching_secs as i64);
                    (Some(deadline), Some(TimeoutReason::PhaseTimeout))
                }
                _ => (None, None),
            };

            // Broadcast state change with timeout info
            let _member_ids = match party.member_ids(&ctx).await {
                Ok(ids) => ids,
                Err(e) => {
                    error!("Failed to get member IDs for party {}: {:?}", party_id, e);
                    return;
                }
            };

            ctx.broadcast_party(
                party_id,
                &ServerMessage::PartyStateChanged(PartyStateChanged {
                    state: new_phase.into(),
                    deadline_at,
                    timeout_reason: reason,
                }),
                None,
            );

            // Schedule the timeout in the scheduler
            if let Some(deadline) = deadline_at {
                registry
                    .schedule_phase_timeout(party_id, new_phase, deadline, ctx)
                    .await;
            }
        }
        Ok(None) => {
            // This can happen if readiness check inside try_auto_advance_on_ready failed
            // (e.g. someone unreadied at the last millisecond)
            warn!(
                "[Scheduler] Ready countdown: party {} not advanced (verification failed or state changed)",
                party_id
            );
        }
        Err(e) => {
            error!("Failed to advance party {} phase: {:?}", party_id, e);
        }
    }
}
