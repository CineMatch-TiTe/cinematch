//! Timeout registry for party phase timeouts and ready countdowns.
//!
//! Spawns cancellable async tasks per-party. Each task sleeps until deadline
//! then fires transition logic. Cancelled automatically when party advances manually.

mod executor;
mod startup;

use std::collections::HashMap;
use std::sync::Arc;

use actix_web::rt;
use chrono::{DateTime, Utc};
use log::{debug, info};
use tokio::sync::RwLock;
use tokio::task::AbortHandle;
use uuid::Uuid;

use cinematch_common::models::websocket::{PartyTimeoutUpdate, ServerMessage, TimeoutReason};
use cinematch_db::AppContext;
use cinematch_db::domain::Party;
use cinematch_db::repo::party::models::PartyState;

pub use startup::reschedule_timeouts_on_startup;

/// Registry managing timeout tasks per party.
/// Each party has at most one active timeout (phase timeout or ready countdown).
pub struct Scheduler {
    tasks: RwLock<HashMap<Uuid, (AbortHandle, DateTime<Utc>)>>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            tasks: RwLock::new(HashMap::new()),
        }
    }

    /// Cancel any pending timeout for a party.
    pub async fn cancel(&self, party_id: Uuid) {
        if let Some((handle, _)) = self.tasks.write().await.remove(&party_id) {
            handle.abort();
            info!("[Scheduler] ❌ Cancelled timeout for party {}", party_id);
        } else {
            debug!("[Scheduler] No timeout to cancel for party {}", party_id);
        }
    }

    /// Check if a timeout is currently scheduled for a party.
    pub async fn is_scheduled(&self, party_id: Uuid) -> bool {
        self.tasks.read().await.contains_key(&party_id)
    }

    /// Get the deadline for a party's pending timeout.
    pub async fn get_deadline(&self, party_id: Uuid) -> Option<DateTime<Utc>> {
        self.tasks.read().await.get(&party_id).map(|(_, d)| *d)
    }

    /// Schedule a phase timeout (Voting/Watching).
    /// Cancels any existing timeout for this party first.
    pub async fn schedule_phase_timeout<C: AppContext + Clone + 'static>(
        self: &Arc<Self>,
        party_id: Uuid,
        phase: PartyState,
        deadline_at: DateTime<Utc>,
        ctx: C,
    ) {
        self.cancel(party_id).await;

        let duration = (deadline_at - Utc::now())
            .to_std()
            .unwrap_or(std::time::Duration::ZERO);

        debug!(
            "[Scheduler] Request to schedule {:?} phase timeout for party {}",
            phase, party_id
        );

        info!(
            "[Scheduler] Scheduling {:?} phase timeout for party {} | fires at: {} | in {:.1}s",
            phase,
            party_id,
            deadline_at.format("%H:%M:%S UTC"),
            duration.as_secs_f64()
        );

        let timeout_registry = Arc::clone(self);
        let ctx_clone = ctx.clone();
        let handle = rt::spawn(async move {
            debug!(
                "[Scheduler] Sleeping {:.1}s for {:?} timeout (party {})",
                duration.as_secs_f64(),
                phase,
                party_id
            );
            tokio::time::sleep(duration).await;
            info!(
                "[Scheduler] ⏰ {:?} timeout FIRED for party {}",
                phase, party_id
            );
            executor::execute_phase_timeout(&timeout_registry, party_id, phase, ctx_clone).await;
        });

        self.tasks
            .write()
            .await
            .insert(party_id, (handle.abort_handle(), deadline_at));
    }

    /// Schedule custom countdown.
    /// Cancels any existing timeout for this party first.
    pub async fn schedule_custom_countdown<C: AppContext + Clone + 'static>(
        self: &Arc<Self>,
        party_id: Uuid,
        delay: chrono::Duration,
        ctx: C,
    ) {
        self.cancel(party_id).await;

        let deadline_at = Utc::now() + delay;

        {
            let tasks = self.tasks.read().await;
            if tasks.contains_key(&party_id) {
                debug!(
                    "[Scheduler] Custom countdown already scheduled for party {}, skipping",
                    party_id
                );
                return;
            }
        }

        debug!(
            "[Scheduler] Request to schedule custom countdown for party {}",
            party_id
        );

        info!(
            "[Scheduler] Scheduling custom countdown for party {} | fires at: {} | in {:.1}s",
            party_id,
            deadline_at.format("%H:%M:%S UTC"),
            delay.num_seconds()
        );

        let timeout_registry = Arc::clone(self);
        let ctx_clone = ctx.clone();

        let handle = rt::spawn(async move {
            debug!(
                "[Scheduler] Sleeping {:.1}s for custom countdown (party {})",
                delay.num_seconds(),
                party_id
            );
            tokio::time::sleep(delay.to_std().unwrap_or(std::time::Duration::from_secs(1))).await;
            info!(
                "[Scheduler] ⏰ Custom countdown FIRED for party {}",
                party_id
            );
            executor::execute_custom_countdown(&timeout_registry, party_id, ctx_clone).await;
        });

        // Broadcast countdown start
        let msg = ServerMessage::PartyTimeoutUpdate(PartyTimeoutUpdate {
            phase_entered_at: None,
            timeout_secs: None,
            deadline_at: Some(deadline_at),
            reason: Some(TimeoutReason::AllReady),
        });

        ctx.broadcast_party(party_id, &msg, None);

        self.tasks
            .write()
            .await
            .insert(party_id, (handle.abort_handle(), deadline_at));
    }

    /// Schedule ready countdown.
    /// Cancels any existing timeout for this party first.
    pub async fn schedule_ready_countdown<C: AppContext + Clone + 'static>(
        self: &Arc<Self>,
        party_id: Uuid,
        ctx: C,
    ) {
        self.cancel(party_id).await;

        let countdown_secs = cinematch_common::Config::get()
            .timeouts
            .ready_countdown_secs;
        let deadline_at =
            Utc::now() + chrono::Duration::milliseconds((countdown_secs * 1000.0) as i64);

        debug!(
            "[Scheduler] Request to schedule ready countdown for party {}",
            party_id
        );

        info!(
            "[Scheduler] Scheduling ready countdown for party {} | fires at: {} | in {:.1}s",
            party_id,
            deadline_at.format("%H:%M:%S UTC"),
            countdown_secs
        );

        let timeout_registry = Arc::clone(self);
        let ctx_clone = ctx.clone();

        let handle = rt::spawn(async move {
            debug!(
                "[Scheduler] Sleeping {:.1}s for ready countdown (party {})",
                countdown_secs, party_id
            );
            tokio::time::sleep(std::time::Duration::from_secs_f32(countdown_secs)).await;
            info!(
                "[Scheduler] ⏰ Ready countdown FIRED for party {}",
                party_id
            );
            executor::execute_ready_countdown(&timeout_registry, party_id, ctx_clone).await;
        });

        // Broadcast countdown start
        let msg = ServerMessage::PartyTimeoutUpdate(PartyTimeoutUpdate {
            phase_entered_at: None,
            timeout_secs: None,
            deadline_at: Some(deadline_at),
            reason: Some(TimeoutReason::AllReady),
        });

        ctx.broadcast_party(party_id, &msg, None);

        self.tasks
            .write()
            .await
            .insert(party_id, (handle.abort_handle(), deadline_at));
    }

    /// Advance phase instantly (skipping ready countdown).
    pub async fn trigger_ready_advance_instantly<C: AppContext + Clone + 'static>(
        self: &Arc<Self>,
        party_id: Uuid,
        ctx: C,
    ) {
        self.cancel(party_id).await;
        executor::execute_ready_countdown(self, party_id, ctx).await;
    }

    /// Cancel timeout and broadcast that deadline is cleared.
    pub async fn cancel_and_broadcast<C: AppContext>(&self, party_id: Uuid, ctx: &C) {
        self.cancel(party_id).await;
        let msg = ServerMessage::PartyTimeoutUpdate(PartyTimeoutUpdate {
            phase_entered_at: None,
            timeout_secs: None,
            deadline_at: None,
            reason: None,
        });

        ctx.broadcast_party(party_id, &msg, None);
    }

    /// Enforce phase-specific timeout and broadcast to clients.
    /// Used when entering Voting or Watching phase to ensure backend schedule matches client deadline.
    pub async fn enforce_phase_timeout_and_broadcast<C: AppContext + Clone + 'static>(
        self: &Arc<Self>,
        party_id: Uuid,
        ctx: C,
    ) {
        self.cancel(party_id).await;

        let party = match Party::from_id(&ctx, party_id).await {
            Ok(p) => p,
            Err(e) => {
                log::error!(
                    "[Scheduler] enforce_phase_timeout: party {} not found: {:?}",
                    party_id,
                    e
                );
                return;
            }
        };

        let timeouts = &cinematch_common::Config::get().timeouts;
        let state = party.state(&ctx).await.unwrap_or(PartyState::Disbanded);
        let phase_entered_at = party.phase_entered_at(&ctx).await.unwrap_or(Utc::now());

        let timeout_secs = match state {
            PartyState::Voting => {
                // By default, we don't schedule Voting timeout automatically on entry.
                // However, we still broadcast the phase config snapshot so clients know timeouts are POSSIBLE.
                ctx.broadcast_party(
                    party_id,
                    &ServerMessage::PartyTimeoutUpdate(PartyTimeoutUpdate {
                        phase_entered_at: Some(phase_entered_at),
                        timeout_secs: None,
                        deadline_at: None,
                        reason: None,
                    }),
                    None,
                );
                return;
            }
            PartyState::Watching => timeouts.watching_timeout_secs,
            _ => {
                // Non-timed phase — broadcast config snapshot only, no deadline
                ctx.broadcast_party(
                    party_id,
                    &ServerMessage::PartyTimeoutUpdate(PartyTimeoutUpdate {
                        phase_entered_at: None,
                        timeout_secs: None,
                        deadline_at: None,
                        reason: None,
                    }),
                    None,
                );
                return;
            }
        };

        let deadline_at = phase_entered_at + chrono::Duration::seconds(timeout_secs as i64);

        // Schedule backend timeout
        self.schedule_phase_timeout(party.id, state, deadline_at, ctx.clone())
            .await;

        // Broadcast to party
        ctx.broadcast_party(
            party_id,
            &ServerMessage::PartyTimeoutUpdate(PartyTimeoutUpdate {
                phase_entered_at: Some(phase_entered_at),
                timeout_secs: Some(timeout_secs),
                deadline_at: Some(deadline_at),
                reason: Some(TimeoutReason::PhaseTimeout),
            }),
            None,
        );
    }

    /// Explicitly trigger and schedule the voting timeout (participation threshold met).
    pub async fn trigger_voting_timeout<C: AppContext + Clone + 'static>(
        self: &Arc<Self>,
        party_id: Uuid,
        ctx: C,
    ) {
        if self.is_scheduled(party_id).await {
            return;
        }

        let party = match Party::from_id(&ctx, party_id).await {
            Ok(p) => p,
            Err(_) => return,
        };

        let round = party.voting_round(&ctx).await.unwrap_or(None).unwrap_or(1);
        let timeout_secs = if round == 2 {
            cinematch_common::Config::get()
                .timeouts
                .voting_r2_timeout_secs
        } else {
            cinematch_common::Config::get()
                .timeouts
                .voting_r1_timeout_secs
        };

        let deadline_at = Utc::now() + chrono::Duration::seconds(timeout_secs as i64);

        self.schedule_phase_timeout(party_id, PartyState::Voting, deadline_at, ctx.clone())
            .await;

        ctx.broadcast_party(
            party_id,
            &ServerMessage::PartyTimeoutUpdate(PartyTimeoutUpdate {
                phase_entered_at: None,
                timeout_secs: Some(timeout_secs),
                deadline_at: Some(deadline_at),
                reason: Some(TimeoutReason::PhaseTimeout),
            }),
            None,
        );

        info!(
            "[Scheduler] Voting timeout triggered for party {} (Round {}) | deadline: {}",
            party_id,
            round,
            deadline_at.format("%H:%M:%S UTC")
        );
    }

    /// Remove task from internal map (called when timeout executes).
    pub(crate) async fn remove_task(&self, party_id: Uuid) {
        self.tasks.write().await.remove(&party_id);
        debug!("[Scheduler] Removed completed task for party {}", party_id);
    }

    /// Re-evaluate if all members are ready and trigger advance/countdown if so.
    /// Used when members leave, are kicked, or toggle ready status.
    pub async fn reevaluate_ready_status<C: AppContext + Clone + 'static>(
        self: &Arc<Self>,
        party_id: Uuid,
        ctx: C,
    ) {
        let party = match Party::from_id(&ctx, party_id).await {
            Ok(p) => p,
            Err(_) => return,
        };

        let (ready_count, total) = match party.ready_status(&ctx).await {
            Ok(res) => res,
            Err(_) => return,
        };

        let all_ready = total > 0 && ready_count == total;

        if all_ready {
            let state = party
                .state(&ctx)
                .await
                .unwrap_or(cinematch_db::repo::party::models::PartyState::Disbanded);

            if state == cinematch_db::repo::party::models::PartyState::Voting {
                debug!(
                    "[Scheduler] All members ready in Voting phase for party {}, instant advance!",
                    party_id
                );
                self.trigger_ready_advance_instantly(party_id, ctx.clone())
                    .await;
            } else {
                debug!(
                    "[Scheduler] All members ready in {:?} phase for party {}, scheduling countdown",
                    state, party_id
                );
                self.schedule_ready_countdown(party_id, ctx.clone()).await;
            }
        } else {
            // If NOT all ready, ensure any pending ready-countdown is cancelled.
            // Note: This only cancels if it's a ready countdown (via reason check if we added it,
            // but for now, simple cancel is fine if we're in a phase that supports ready counts).
            // We should only cancel if it's a "Wait for all ready" countdown.
            let state = party
                .state(&ctx)
                .await
                .unwrap_or(cinematch_db::repo::party::models::PartyState::Disbanded);

            if self.is_scheduled(party_id).await {
                // Do NOT cancel the timeout if the party is in a timed phase (Voting/Watching).
                // Unreadying should not cancel the voting countdown or watching timer.
                if state != cinematch_db::repo::party::models::PartyState::Voting
                    && state != cinematch_db::repo::party::models::PartyState::Watching
                {
                    self.cancel_and_broadcast(party_id, &ctx).await;
                }
            }
        }
    }
}
