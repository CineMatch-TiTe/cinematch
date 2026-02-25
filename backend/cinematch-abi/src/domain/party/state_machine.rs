//! Party state machine and phase transitions.

use log::{debug, error};
use std::collections::HashMap;
use uuid::Uuid;

use cinematch_db::AppContext;
use cinematch_db::PartyState;
use cinematch_db::domain::{Party, User};

use super::super::DomainError;
use super::{EndVotingTransition, PartyAdvanceOutcome, PartyValidation};
use async_trait::async_trait;
use cinematch_common::models::websocket::{PartyStateChanged, ServerMessage, VotingRoundStarted};

/// State machine operations for Party.
/// State machine operations for Party.
#[async_trait]
pub trait PartyStateMachine: PartyValidation {
    /// Advance phase (leader-only force skip).
    async fn advance_phase(
        &self,
        ctx: &impl AppContext,
        leader_id: Uuid,
    ) -> Result<PartyAdvanceOutcome, DomainError>;

    /// If all members are ready, auto-advance from Created/Picking/Review.
    async fn try_auto_advance_on_ready(
        &self,
        ctx: &impl AppContext,
    ) -> Result<Option<PartyState>, DomainError>;

    /// If all members have voted, auto-end voting.
    async fn try_auto_end_voting(
        &self,
        ctx: &impl AppContext,
    ) -> Result<Option<EndVotingTransition>, DomainError>;

    /// Force end voting due to timeout.
    async fn force_end_voting_timeout(
        &self,
        ctx: &impl AppContext,
    ) -> Result<EndVotingTransition, DomainError>;

    /// Watching → Review transition (public for timeout handler).
    async fn do_watching_to_review(&self, ctx: &impl AppContext) -> Result<(), DomainError>;
}

#[async_trait]
impl PartyStateMachine for Party {
    async fn advance_phase(
        &self,
        ctx: &impl AppContext,
        leader_id: Uuid,
    ) -> Result<PartyAdvanceOutcome, DomainError> {
        self.require_leader(ctx, leader_id).await?;
        self.require_member(ctx, leader_id).await?;

        let state = self.state(ctx).await.map_err(DomainError::from)?;

        let outcome = match state {
            PartyState::Created => {
                do_created_to_picking(ctx, self).await?;
                PartyAdvanceOutcome::PhaseChanged(PartyState::Picking)
            }
            PartyState::Picking => {
                do_picking_to_voting(ctx, self).await?;
                PartyAdvanceOutcome::PhaseChanged(PartyState::Voting)
            }
            PartyState::Voting => {
                let t = run_end_voting_internal(ctx, self, false).await?;
                PartyAdvanceOutcome::VotingEnded(t)
            }
            PartyState::Watching => {
                do_watching_to_review_internal(ctx, self).await?;
                PartyAdvanceOutcome::PhaseChanged(PartyState::Review)
            }
            PartyState::Review => {
                do_review_to_created(ctx, self).await?;
                PartyAdvanceOutcome::PhaseChanged(PartyState::Created)
            }
            PartyState::Disbanded => {
                return Err(DomainError::BadRequest(
                    "Cannot advance phase of a disbanded party".into(),
                ));
            }
        };

        match &outcome {
            PartyAdvanceOutcome::PhaseChanged(s) => {
                let msg = PartyStateChanged {
                    state: (*s).into(),
                    deadline_at: None,
                    timeout_reason: None,
                };
                ctx.broadcast_party(self.id, &ServerMessage::PartyStateChanged(msg), None);
            }
            PartyAdvanceOutcome::VotingEnded(EndVotingTransition::Round1Started) => {
                ctx.broadcast_party(
                    self.id,
                    &ServerMessage::VotingRoundStarted(VotingRoundStarted { round: 1 }),
                    None,
                );
            }
            PartyAdvanceOutcome::VotingEnded(EndVotingTransition::Round2Started) => {
                ctx.broadcast_party(
                    self.id,
                    &ServerMessage::VotingRoundStarted(VotingRoundStarted { round: 2 }),
                    None,
                );
            }
            PartyAdvanceOutcome::VotingEnded(EndVotingTransition::PhaseChanged(s)) => {
                let msg = PartyStateChanged {
                    state: (*s).into(),
                    deadline_at: None,
                    timeout_reason: None,
                };
                ctx.broadcast_party(self.id, &ServerMessage::PartyStateChanged(msg), None);
            }
        }
        debug!("Party {} advanced -> {:?}", self.id, outcome);
        Ok(outcome)
    }

    async fn try_auto_advance_on_ready(
        &self,
        ctx: &impl AppContext,
    ) -> Result<Option<PartyState>, DomainError> {
        let all_ready = self.are_all_ready(ctx).await.map_err(DomainError::from)?;
        if !all_ready {
            return Ok(None);
        }

        let state = self.state(ctx).await.map_err(DomainError::from)?;

        let new_state = match state {
            PartyState::Created => {
                do_created_to_voting(ctx, self).await?;
                Some(PartyState::Voting)
            }
            PartyState::Picking => {
                do_picking_to_voting(ctx, self).await?;
                Some(PartyState::Voting)
            }
            PartyState::Review => {
                do_review_to_created(ctx, self).await?;
                Some(PartyState::Created)
            }
            PartyState::Voting => {
                let t = run_end_voting_internal(ctx, self, false).await?;
                match &t {
                    EndVotingTransition::Round1Started => {
                        ctx.broadcast_party(
                            self.id,
                            &ServerMessage::VotingRoundStarted(VotingRoundStarted { round: 1 }),
                            None,
                        );
                        Some(PartyState::Voting)
                    }
                    EndVotingTransition::Round2Started => {
                        ctx.broadcast_party(
                            self.id,
                            &ServerMessage::VotingRoundStarted(VotingRoundStarted { round: 2 }),
                            None,
                        );
                        Some(PartyState::Voting) // Still in Voting state, just round 2
                    }
                    EndVotingTransition::PhaseChanged(s) => {
                        // Let the catch-all below broadcast the PhaseChanged event
                        Some(*s)
                    }
                }
            }
            _ => None,
        };

        if let Some(s) = new_state {
            if state != PartyState::Voting || s != PartyState::Voting {
                let msg = PartyStateChanged {
                    state: s.into(),
                    deadline_at: None,
                    timeout_reason: None,
                };
                ctx.broadcast_party(self.id, &ServerMessage::PartyStateChanged(msg), None);
                debug!("Party {} auto-advanced (all ready) -> {:?}", self.id, s);
            } else {
                debug!("Party {} auto-advanced voting round (all ready)", self.id);
            }
        }
        Ok(new_state)
    }

    async fn try_auto_end_voting(
        &self,
        ctx: &impl AppContext,
    ) -> Result<Option<EndVotingTransition>, DomainError> {
        let state = self.state(ctx).await.map_err(DomainError::from)?;
        if state != PartyState::Voting {
            return Ok(None);
        }

        let all_voted = self
            .have_all_members_voted(ctx)
            .await
            .map_err(DomainError::from)?;

        if !all_voted {
            return Ok(None);
        }

        let t = run_end_voting_internal(ctx, self, false).await?;

        match &t {
            EndVotingTransition::Round1Started => {
                ctx.broadcast_party(
                    self.id,
                    &ServerMessage::VotingRoundStarted(VotingRoundStarted { round: 1 }),
                    None,
                );
            }
            EndVotingTransition::Round2Started => {
                ctx.broadcast_party(
                    self.id,
                    &ServerMessage::VotingRoundStarted(VotingRoundStarted { round: 2 }),
                    None,
                );
            }
            EndVotingTransition::PhaseChanged(s) => {
                let msg = PartyStateChanged {
                    state: (*s).into(),
                    deadline_at: None,
                    timeout_reason: None,
                };
                ctx.broadcast_party(self.id, &ServerMessage::PartyStateChanged(msg), None);
            }
        }

        debug!("Party {} auto-ended voting (all voted) -> {:?}", self.id, t);
        Ok(Some(t))
    }

    async fn force_end_voting_timeout(
        &self,
        ctx: &impl AppContext,
    ) -> Result<EndVotingTransition, DomainError> {
        let t = run_end_voting_internal(ctx, self, true).await?;

        match &t {
            EndVotingTransition::Round1Started => {
                ctx.broadcast_party(
                    self.id,
                    &ServerMessage::VotingRoundStarted(VotingRoundStarted { round: 1 }),
                    None,
                );
            }
            EndVotingTransition::Round2Started => {
                ctx.broadcast_party(
                    self.id,
                    &ServerMessage::VotingRoundStarted(VotingRoundStarted { round: 2 }),
                    None,
                );
            }
            EndVotingTransition::PhaseChanged(s) => {
                let msg = PartyStateChanged {
                    state: (*s).into(),
                    deadline_at: None,
                    timeout_reason: None,
                };
                ctx.broadcast_party(self.id, &ServerMessage::PartyStateChanged(msg), None);
            }
        }
        Ok(t)
    }

    async fn do_watching_to_review(&self, ctx: &impl AppContext) -> Result<(), DomainError> {
        do_watching_to_review_internal(ctx, self).await?;
        let msg = PartyStateChanged {
            state: PartyState::Review.into(),
            deadline_at: None,
            timeout_reason: None,
        };
        ctx.broadcast_party(self.id, &ServerMessage::PartyStateChanged(msg), None);
        Ok(())
    }
}

// ============================================================================
// Internal helper functions
// ============================================================================

/// Created → Picking: Release join code, switch phase.
async fn do_created_to_picking(ctx: &impl AppContext, party: &Party) -> Result<(), DomainError> {
    party.release_code(ctx).await.map_err(|e| {
        error!("Failed to release party code: {}", e);
        DomainError::Internal(format!("Failed to release party code: {}", e))
    })?;

    party
        .set_phase(ctx, PartyState::Picking)
        .await
        .map_err(|e| {
            error!("Failed to advance phase: {}", e);
            DomainError::Internal(format!("Failed to advance phase: {}", e))
        })?;

    Ok(())
}

/// Created → Voting: Release join code, build ballots, switch phase.
async fn do_created_to_voting(ctx: &impl AppContext, party: &Party) -> Result<(), DomainError> {
    party.release_code(ctx).await.map_err(|e| {
        error!("Failed to release party code: {}", e);
        DomainError::Internal(format!("Failed to release party code: {}", e))
    })?;

    // Switch to Voting (resets ready states)
    party
        .set_phase(ctx, PartyState::Voting)
        .await
        .map_err(|e| {
            error!("Failed to advance phase: {}", e);
            DomainError::Internal(format!("Failed to advance phase: {}", e))
        })?;

    // Generate ballots (pure-ish)
    cinematch_recommendation_engine::build_voting_ballots_for_party(ctx, party)
        .await
        .map_err(|e| {
            error!("Failed to build voting ballots (Qdrant): {}", e);
            DomainError::Internal(format!("Failed to build voting ballots: {}", e))
        })?;

    // Enable voting state
    party.enable_voting(ctx).await.map_err(|e| {
        error!("Failed to enable voting: {}", e);
        DomainError::Internal(format!("Failed to enable voting: {}", e))
    })?;

    party.set_voting_round(ctx, Some(1)).await.map_err(|e| {
        error!("Failed to set voting round: {}", e);
        DomainError::Internal(format!("Failed to set voting round: {}", e))
    })?;

    Ok(())
}

/// Picking → Voting: Build ballots from Qdrant, switch phase.
async fn do_picking_to_voting(ctx: &impl AppContext, party: &Party) -> Result<(), DomainError> {
    party
        .set_phase(ctx, PartyState::Voting)
        .await
        .map_err(|e| {
            error!("Failed to advance phase: {}", e);
            DomainError::Internal(format!("Failed to advance phase: {}", e))
        })?;

    cinematch_recommendation_engine::build_voting_ballots_for_party(ctx, party)
        .await
        .map_err(|e| {
            error!("Failed to build voting ballots (Qdrant): {}", e);
            DomainError::Internal(format!("Failed to build voting ballots: {}", e))
        })?;

    party.enable_voting(ctx).await.map_err(|e| {
        error!("Failed to enable voting: {}", e);
        DomainError::Internal(format!("Failed to enable voting: {}", e))
    })?;

    party.set_voting_round(ctx, Some(1)).await.map_err(|e| {
        error!("Failed to set voting round: {}", e);
        DomainError::Internal(format!("Failed to set voting round: {}", e))
    })?;

    Ok(())
}

/// Watching → Review: Simple phase change.
async fn do_watching_to_review_internal(
    ctx: &impl AppContext,
    party: &Party,
) -> Result<(), DomainError> {
    party
        .set_phase(ctx, PartyState::Review)
        .await
        .map_err(|e| {
            error!("Failed to advance phase: {}", e);
            DomainError::Internal(format!("Failed to advance phase: {}", e))
        })?;
    Ok(())
}

/// Review → Created: Start new round, regenerate join code.
async fn do_review_to_created(ctx: &impl AppContext, party: &Party) -> Result<(), DomainError> {
    party.start_new_round(ctx).await.map_err(|e| {
        error!("Failed to start new round: {}", e);
        DomainError::Internal(format!("Failed to start new round: {}", e))
    })?;
    debug!("Party {} started new round", party.id);
    Ok(())
}

/// Run end-voting logic (tally, round 2 / winner / back to Picking).
async fn run_end_voting_internal(
    ctx: &impl AppContext,
    party: &Party,
    force_timeout: bool,
) -> Result<EndVotingTransition, DomainError> {
    party.disable_voting(ctx).await.map_err(|e| {
        error!("Failed to disable voting: {}", e);
        DomainError::Internal("Failed to disable voting".into())
    })?;

    let vote_map = party.get_votes(ctx, None).await.map_err(|e| {
        error!("Failed to get party votes: {}", e);
        DomainError::Internal("Failed to tally votes".into())
    })?;

    let round = party.voting_round(ctx).await.unwrap_or(None);
    let is_round2 = round == Some(2);

    if !is_round2 {
        return handle_round1_end(ctx, party, &vote_map, force_timeout).await;
    }

    handle_round2_end(ctx, party, &vote_map, force_timeout).await
}

/// Handle end of round 1 voting.
async fn handle_round1_end(
    ctx: &impl AppContext,
    party: &Party,
    vote_map: &HashMap<i64, (u32, u32)>,
    force_timeout: bool,
) -> Result<EndVotingTransition, DomainError> {
    if vote_map.is_empty() {
        if force_timeout {
            // Restart Voting Phase 1 instead of rolling back to Picking
            debug!("Round 1 timeout with zero votes, restarting phase 1 ballots");
            do_picking_to_voting(ctx, party).await?;
            return Ok(EndVotingTransition::Round1Started);
        }
        let _ = party.enable_voting(ctx).await;
        return Err(DomainError::BadRequest(
            "No votes cast; cannot start round 2".into(),
        ));
    }

    // Sort by score (likes - dislikes)
    let mut by_score: Vec<(i64, u32, u32)> = vote_map
        .iter()
        .map(|(&mid, &(likes, dislikes))| (mid, likes, dislikes))
        .collect();
    by_score.sort_by(|a, b| {
        let sa = a.1 as i32 - a.2 as i32;
        let sb = b.1 as i32 - b.2 as i32;
        sb.cmp(&sa)
    });
    let top3: Vec<i64> = by_score.into_iter().take(3).map(|(m, _, _)| m).collect();

    cinematch_recommendation_engine::build_round2_ballots_for_party(ctx, party, &top3)
        .await
        .map_err(|e| {
            error!("Failed to build round 2 ballots (Qdrant): {}", e);
            DomainError::Internal("Failed to build round 2 ballots".into())
        })?;

    party.enable_voting(ctx).await.map_err(|e| {
        error!("Failed to enable voting for round 2: {}", e);
        DomainError::Internal("Failed to enable voting".into())
    })?;

    party.set_voting_round(ctx, Some(2)).await.map_err(|e| {
        error!("Failed to set voting round 2: {}", e);
        DomainError::Internal("Failed to set voting round".into())
    })?;

    party.set_phase_entered_at_now(ctx).await.map_err(|e| {
        error!("Failed to update phase time: {}", e);
        DomainError::Internal("Failed to update phase time".into())
    })?;

    Ok(EndVotingTransition::Round2Started)
}

/// Handle end of round 2 voting.
async fn handle_round2_end(
    ctx: &impl AppContext,
    party: &Party,
    vote_map: &HashMap<i64, (u32, u32)>,
    force_timeout: bool,
) -> Result<EndVotingTransition, DomainError> {
    if vote_map.is_empty() {
        if force_timeout {
            // Restart Voting Phase 1 instead of rolling back to Picking
            debug!("Round 2 timeout with zero votes, restarting phase 1 ballots");
            do_picking_to_voting(ctx, party).await?;
            return Ok(EndVotingTransition::Round1Started);
        }
        let _ = party.set_voting_round(ctx, None).await;
        return Err(DomainError::BadRequest(
            "No votes in round 2; cannot select winner".into(),
        ));
    }

    let (winner_id, winner_likes) = vote_map
        .iter()
        .max_by_key(|(_, (likes, _))| *likes)
        .map(|(&mid, &(likes, _))| (mid, likes))
        .unwrap();

    let total_likes: u32 = vote_map.values().map(|(l, _)| *l).sum();
    let fifty_pct = total_likes / 2;
    // Strict majority required to win (> 50%)
    let has_majority = total_likes > 0 && winner_likes > fifty_pct;

    if has_majority {
        return select_winner(ctx, party, winner_id).await;
    }

    // No majority (tie or everyone rejected everything)
    // Identify top movie(s) and carry over to Picking pool (as leader picks)
    // so they appear again for everyone at next phase 1
    let mut by_likes: Vec<(i64, u32)> = vote_map
        .iter()
        .map(|(&mid, &(likes, _))| (mid, likes))
        .collect();
    by_likes.sort_by(|a, b| b.1.cmp(&a.1));

    if let Ok(leader_id) = party.leader_id(ctx).await {
        // Take top 1, or top 2 if they have the same (highest) number of likes
        let top_likes = by_likes.first().map(|(_, l)| *l).unwrap_or(0);
        if top_likes > 0 {
            for (mid, likes) in by_likes.into_iter().take(2) {
                if likes == top_likes {
                    debug!(
                        "Carrying over movie {} to picks (top votes: {}) so it persists in restart",
                        mid, likes
                    );
                    let _ = party.add_pick(ctx, leader_id, mid, Some(true)).await;
                }
            }
        }
    }

    // Record votes as taste and restart Voting Phase 1 instead of rolling back to Picking
    debug!(
        "Round 2 finished without majority winner, recording taste and restarting phase 1 ballots"
    );
    record_votes_as_taste(ctx, party).await?;
    do_picking_to_voting(ctx, party).await?;
    Ok(EndVotingTransition::Round1Started)
}

/// Select a winner and move to Watching state.
async fn select_winner(
    ctx: &impl AppContext,
    party: &Party,
    movie_id: i64,
) -> Result<EndVotingTransition, DomainError> {
    party
        .set_selected_movie_id(ctx, Some(movie_id))
        .await
        .map_err(|e| {
            error!("Failed to set selected movie: {}", e);
            DomainError::Internal("Failed to set selected movie".into())
        })?;

    party
        .set_phase(ctx, PartyState::Watching)
        .await
        .map_err(|e| {
            error!("Failed to advance to Watching: {}", e);
            DomainError::Internal("Failed to advance to Watching".into())
        })?;

    let _ = party.set_voting_round(ctx, None).await;

    Ok(EndVotingTransition::PhaseChanged(PartyState::Watching))
}

/// Record user votes as taste data for future recommendations.
/// This syncs the party session results to the global user profile.
async fn record_votes_as_taste(ctx: &impl AppContext, party: &Party) -> Result<(), DomainError> {
    let member_ids = party.member_ids(ctx).await.map_err(DomainError::from)?;

    for user_id in member_ids {
        let user_votes = party.get_user_votes(ctx, user_id).await.unwrap_or_default();

        for v in user_votes {
            // Sync to party session picks
            let _ = party
                .add_pick(ctx, user_id, v.movie_id, Some(v.vote_value))
                .await;

            // Sync to global user ratings (persists sentiment across parties)
            let user = User::new(user_id);
            let _ = user
                .add_rating(ctx, v.movie_id, Some(v.vote_value), None)
                .await;
        }
    }
    Ok(())
}
