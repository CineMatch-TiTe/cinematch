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

/// Get ready countdown duration from config (LazyLock, read once from env).
pub fn get_ready_countdown_secs() -> f32 {
    cinematch_common::Config::get().ready_countdown_secs
}

/// Registry managing timeout tasks per party.
/// Each party has at most one active timeout (phase timeout or ready countdown).
pub struct Scheduler {
    tasks: RwLock<HashMap<Uuid, AbortHandle>>,
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
        if let Some(handle) = self.tasks.write().await.remove(&party_id) {
            handle.abort();
            info!("[Scheduler] ❌ Cancelled timeout for party {}", party_id);
        } else {
            debug!("[Scheduler] No timeout to cancel for party {}", party_id);
        }
    }

    /// Schedule a phase timeout (Voting/Watching).
    /// Cancels any existing timeout for this party first.
    pub async fn schedule_phase_timeout(
        self: &Arc<Self>,
        party_id: Uuid,
        phase: PartyState,
        deadline_at: DateTime<Utc>,
        ctx: Arc<dyn AppContext>,
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
        let ctx_clone = Arc::clone(&ctx);
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
            .insert(party_id, handle.abort_handle());
    }

    /// Schedule ready countdown.
    /// Cancels any existing timeout for this party first.
    pub async fn schedule_ready_countdown(
        self: &Arc<Self>,
        party_id: Uuid,
        ctx: Arc<dyn AppContext>,
    ) {
        self.cancel(party_id).await;

        let countdown_secs = get_ready_countdown_secs();
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
        let ctx_clone = Arc::clone(&ctx);
        let ctx_for_broadcast = Arc::clone(&ctx);

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
            voting_timeout_secs: None,
            watching_timeout_secs: None,
            deadline_at: Some(deadline_at),
            reason: Some(TimeoutReason::AllReady),
        });

        let party = Party::new(party_id);
        let member_ids = match party.member_ids(&ctx_for_broadcast).await {
            Ok(ids) => ids,
            Err(e) => {
                debug!("Failed to get party members for timeout broadcast: {}", e);
                vec![]
            }
        };

        ctx.send_users(&member_ids, &msg);

        self.tasks
            .write()
            .await
            .insert(party_id, handle.abort_handle());
    }

    /// Cancel timeout and broadcast that deadline is cleared.
    /// Cancel timeout and broadcast that deadline is cleared.
    pub async fn cancel_and_broadcast(&self, party_id: Uuid, ctx: &Arc<dyn AppContext>) {
        self.cancel(party_id).await;
        let msg = ServerMessage::PartyTimeoutUpdate(PartyTimeoutUpdate {
            phase_entered_at: None,
            voting_timeout_secs: None,
            watching_timeout_secs: None,
            deadline_at: None,
            reason: None,
        });

        let party = Party::new(party_id);
        let member_ids = match party.member_ids(ctx).await {
            Ok(ids) => ids,
            Err(e) => {
                debug!("Failed to get party members for timeout broadcast: {}", e);
                vec![]
            }
        };
        ctx.send_users(&member_ids, &msg);
    }

    /// Enforce phase-specific timeout and broadcast to clients.
    /// Used when entering Voting or Watching phase to ensure backend schedule matches client deadline.
    pub async fn enforce_phase_timeout_and_broadcast(
        self: &Arc<Self>,
        party_id: Uuid,
        ctx: Arc<dyn AppContext>,
    ) {
        use crate::domain::get_timeout_secs;

        let party = match Party::from_id(&ctx, party_id).await {
            Ok(p) => p,
            Err(e) => {
                log::error!(
                    "[Scheduler] verify_phase_timeout: failed to get party {}: {:?}",
                    party_id,
                    e
                );
                return;
            }
        };

        let (voting_secs, watching_secs) = get_timeout_secs();
        let state = party.state(&ctx).await.unwrap_or(PartyState::Disbanded);
        let phase_entered_at = party.phase_entered_at(&ctx).await.unwrap_or(Utc::now());

        let (timeout_update, deadline_opt) = match state {
            PartyState::Voting | PartyState::Watching => {
                let timeout_secs = match state {
                    PartyState::Voting => voting_secs,
                    PartyState::Watching => watching_secs,
                    _ => 0,
                };
                let deadline_at = phase_entered_at + chrono::Duration::seconds(timeout_secs as i64);

                (
                    PartyTimeoutUpdate {
                        phase_entered_at: Some(phase_entered_at),
                        voting_timeout_secs: Some(voting_secs),
                        watching_timeout_secs: Some(watching_secs),
                        deadline_at: Some(deadline_at),
                        reason: Some(TimeoutReason::PhaseTimeout),
                    },
                    Some(deadline_at),
                )
            }
            _ => {
                // No active timeout for other phases
                (
                    PartyTimeoutUpdate {
                        phase_entered_at: None,
                        voting_timeout_secs: Some(voting_secs),
                        watching_timeout_secs: Some(watching_secs),
                        deadline_at: None,
                        reason: None,
                    },
                    None,
                )
            }
        };

        // Schedule the timeout in the scheduler
        if let Some(deadline) = deadline_opt {
            self.schedule_phase_timeout(party.id, state, deadline, ctx.clone())
                .await;
        }

        // Broadcast to party members
        let msg = ServerMessage::PartyTimeoutUpdate(timeout_update);
        let member_ids = match party.member_ids(&ctx).await {
            Ok(ids) => ids,
            Err(e) => {
                log::error!(
                    "[Scheduler] verify_phase_timeout: failed to get members for party {}: {:?}",
                    party_id,
                    e
                );
                return;
            }
        };
        ctx.send_users(&member_ids, &msg);
    }

    /// Remove task from internal map (called when timeout executes).
    pub(crate) async fn remove_task(&self, party_id: Uuid) {
        self.tasks.write().await.remove(&party_id);
        debug!("[Scheduler] Removed completed task for party {}", party_id);
    }
}
