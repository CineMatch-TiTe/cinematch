//! Party domain handler.
//!
//! Transitions can be automatic (all ready → advance, all voted → end voting) or
//! leader force-skip via advance. Leader-only advance is just "force skip to next phase".

use cinematch_db::{Database, DbError, PartyState};
use log::{debug, error};
use uuid::Uuid;

/// In-memory party model. Load from DB when handling a request; mutations persist immediately.
#[derive(Debug, Clone)]
pub struct Party {
    pub id: Uuid,
    pub party_leader_id: Uuid,
    pub state: PartyState,
    pub voting_round: Option<i16>,
    #[allow(dead_code)]
    pub can_vote: bool,
    pub selected_movie_id: Option<i64>,
}

/// Domain errors for party operations.
#[derive(Debug)]
pub enum PartyError {
    NotFound,
    Forbidden(String),
    BadRequest(String),
    Db(String),
}

impl From<DbError> for PartyError {
    fn from(e: DbError) -> Self {
        match &e {
            DbError::PartyNotFound(_) => PartyError::NotFound,
            DbError::UserNotInParty(_) => PartyError::Forbidden("User not in party".into()),
            _ => PartyError::Db(e.to_string()),
        }
    }
}

/// Result of end-voting: either round 2 started (stay in Voting) or phase changed.
#[derive(Debug, Clone)]
pub enum EndVotingTransition {
    Round2Started,
    PhaseChanged(PartyState),
}

/// Result of advance_phase: phase change or voting-ended outcome (round 2 / phase change).
#[derive(Debug, Clone)]
pub enum PartyAdvanceOutcome {
    PhaseChanged(PartyState),
    VotingEnded(EndVotingTransition),
}

impl Party {
    pub async fn load(db: &Database, party_id: Uuid) -> Result<Self, PartyError> {
        let row = db.get_party(party_id).await.map_err(PartyError::from)?;
        Ok(Party {
            id: row.id,
            party_leader_id: row.party_leader_id,
            state: row.state,
            voting_round: row.voting_round,
            can_vote: row.can_vote,
            selected_movie_id: row.selected_movie_id,
        })
    }

    pub fn ensure_leader(&self, user_id: Uuid) -> Result<(), PartyError> {
        if self.party_leader_id != user_id {
            return Err(PartyError::Forbidden("Not the party leader".into()));
        }
        Ok(())
    }

    pub async fn ensure_member(&self, db: &Database, user_id: Uuid) -> Result<(), PartyError> {
        let ok = db
            .is_party_member(self.id, user_id)
            .await
            .map_err(PartyError::from)?;
        if !ok {
            return Err(PartyError::Forbidden("Not a member of this party".into()));
        }
        Ok(())
    }

    /// Advance phase (leader-only force skip). Handles all transitions including
    /// Voting → end-voting logic and Review → start new round.
    pub async fn advance_phase(
        &mut self,
        db: &Database,
        leader_id: Uuid,
    ) -> Result<PartyAdvanceOutcome, PartyError> {
        self.ensure_leader(leader_id)?;
        self.ensure_member(db, leader_id).await?;

        let outcome = match self.state {
            PartyState::Created => {
                self.do_created_to_picking(db).await?;
                PartyAdvanceOutcome::PhaseChanged(PartyState::Picking)
            }
            PartyState::Picking => {
                self.do_picking_to_voting(db).await?;
                PartyAdvanceOutcome::PhaseChanged(PartyState::Voting)
            }
            PartyState::Voting => {
                let t = run_end_voting_internal(db, self, false).await?;
                PartyAdvanceOutcome::VotingEnded(t)
            }
            PartyState::Watching => {
                self.do_watching_to_review(db).await?;
                PartyAdvanceOutcome::PhaseChanged(PartyState::Review)
            }
            PartyState::Review => {
                self.do_review_to_created(db).await?;
                PartyAdvanceOutcome::PhaseChanged(PartyState::Created)
            }
            PartyState::Disbanded => {
                return Err(PartyError::BadRequest(
                    "Cannot advance phase of a disbanded party".into(),
                ));
            }
        };
        debug!("Party {} advanced -> {:?}", self.id, outcome);
        Ok(outcome)
    }

    async fn do_created_to_picking(&mut self, db: &Database) -> Result<(), PartyError> {
        db.release_party_code(self.id).await.map_err(|e| {
            error!("Failed to release party code: {}", e);
            PartyError::Db("Failed to release party code".into())
        })?;
        db.set_phase(self.id, PartyState::Picking)
            .await
            .map_err(|e| {
                error!("Failed to advance phase: {}", e);
                PartyError::Db("Failed to advance phase".into())
            })?;
        self.state = PartyState::Picking;
        Ok(())
    }

    async fn do_picking_to_voting(&mut self, db: &Database) -> Result<(), PartyError> {
        db.set_phase(self.id, PartyState::Voting)
            .await
            .map_err(|e| {
                error!("Failed to advance phase: {}", e);
                PartyError::Db("Failed to advance phase".into())
            })?;
        cinematch_recommendation_engine::build_voting_ballots_for_party(db, self.id)
            .await
            .map_err(|e| {
                error!("Failed to build voting ballots (Qdrant): {}", e);
                PartyError::Db("Failed to build voting ballots".into())
            })?;
        self.state = PartyState::Voting;
        Ok(())
    }

    async fn do_watching_to_review(&mut self, db: &Database) -> Result<(), PartyError> {
        db.set_phase(self.id, PartyState::Review)
            .await
            .map_err(|e| {
                error!("Failed to advance phase: {}", e);
                PartyError::Db("Failed to advance phase".into())
            })?;
        self.state = PartyState::Review;
        Ok(())
    }

    async fn do_review_to_created(&mut self, db: &Database) -> Result<(), PartyError> {
        let code = db.start_new_round(self.id).await.map_err(|e| {
            error!("Failed to start new round: {}", e);
            PartyError::Db(format!("Failed to start new round: {}", e))
        })?;
        self.state = PartyState::Created;
        self.voting_round = None;
        self.selected_movie_id = None;
        debug!("Party {} started new round, code {}", self.id, code.code);
        Ok(())
    }
}

/// Run end-voting logic (tally, round 2 / winner / back to Picking). Updates `party` in place.
/// No leader check. When `force_timeout`, no-votes rounds still transition to Picking instead of error.
async fn run_end_voting_internal(
    db: &Database,
    party: &mut Party,
    force_timeout: bool,
) -> Result<EndVotingTransition, PartyError> {
    db.disable_voting(party.id).await.map_err(|e| {
        error!("Failed to disable voting: {}", e);
        PartyError::Db("Failed to disable voting".into())
    })?;

    let vote_map = db.get_party_votes(party.id, None).await.map_err(|e| {
        error!("Failed to get party votes: {}", e);
        PartyError::Db("Failed to tally votes".into())
    })?;

    let round = db.get_voting_round(party.id).await.unwrap_or(None);
    let is_round2 = round == Some(2);

    if !is_round2 {
        if vote_map.is_empty() {
            if force_timeout {
                db.clear_shown_movies_for_party(party.id)
                    .await
                    .map_err(|e| {
                        error!("Failed to clear ballots: {}", e);
                        PartyError::Db("Failed to clear ballots".into())
                    })?;
                db.set_phase(party.id, PartyState::Picking)
                    .await
                    .map_err(|e| {
                        error!("Failed to move to Picking: {}", e);
                        PartyError::Db("Failed to move to Picking".into())
                    })?;
                let _ = db.set_voting_round(party.id, None).await;
                party.state = PartyState::Picking;
                party.voting_round = None;
                return Ok(EndVotingTransition::PhaseChanged(PartyState::Picking));
            }
            let _ = db.enable_voting(party.id).await;
            return Err(PartyError::BadRequest(
                "No votes cast; cannot start round 2".into(),
            ));
        }
        let mut by_score: Vec<(i64, u32, u32)> = vote_map
            .into_iter()
            .map(|(mid, (likes, dislikes))| (mid, likes, dislikes))
            .collect();
        by_score.sort_by(|a, b| {
            let sa = a.1 as i32 - a.2 as i32;
            let sb = b.1 as i32 - b.2 as i32;
            sb.cmp(&sa)
        });
        let top3: Vec<i64> = by_score.into_iter().take(3).map(|(m, _, _)| m).collect();
        cinematch_recommendation_engine::build_round2_ballots_for_party(db, party.id, &top3)
            .await
            .map_err(|e| {
                error!("Failed to build round 2 ballots (Qdrant): {}", e);
                PartyError::Db("Failed to build round 2 ballots".into())
            })?;
        party.voting_round = Some(2);
        return Ok(EndVotingTransition::Round2Started);
    }

    if vote_map.is_empty() {
        if force_timeout {
            db.clear_shown_movies_for_party(party.id)
                .await
                .map_err(|e| {
                    error!("Failed to clear ballots: {}", e);
                    PartyError::Db("Failed to clear ballots".into())
                })?;
            db.set_phase(party.id, PartyState::Picking)
                .await
                .map_err(|e| {
                    error!("Failed to move to Picking: {}", e);
                    PartyError::Db("Failed to move to Picking".into())
                })?;
            let _ = db.set_voting_round(party.id, None).await;
            party.state = PartyState::Picking;
            party.voting_round = None;
            return Ok(EndVotingTransition::PhaseChanged(PartyState::Picking));
        }
        let _ = db.set_voting_round(party.id, None).await;
        return Err(PartyError::BadRequest(
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
    let has_majority = total_likes > 0 && winner_likes >= fifty_pct;

    if has_majority {
        db.set_selected_movie_id(party.id, Some(winner_id))
            .await
            .map_err(|e| {
                error!("Failed to set selected movie: {}", e);
                PartyError::Db("Failed to set selected movie".into())
            })?;
        db.set_phase(party.id, PartyState::Watching)
            .await
            .map_err(|e| {
                error!("Failed to advance to Watching: {}", e);
                PartyError::Db("Failed to advance to Watching".into())
            })?;
        let _ = db.set_voting_round(party.id, None).await;
        party.state = PartyState::Watching;
        party.selected_movie_id = Some(winner_id);
        party.voting_round = None;
        return Ok(EndVotingTransition::PhaseChanged(PartyState::Watching));
    }

    let members = db.get_party_members(party.id).await.map_err(|e| {
        error!("Failed to get party members: {}", e);
        PartyError::Db("Failed to get party members".into())
    })?;
    for member in &members {
        let user_votes = db
            .get_user_votes(party.id, member.user_id)
            .await
            .unwrap_or_default();
        for v in user_votes {
            let _ = db
                .add_party_taste(member.user_id, party.id, v.movie_id, Some(v.vote_value))
                .await;
        }
    }
    db.clear_shown_movies_for_party(party.id)
        .await
        .map_err(|e| {
            error!("Failed to clear ballots: {}", e);
            PartyError::Db("Failed to clear ballots".into())
        })?;
    db.set_phase(party.id, PartyState::Picking)
        .await
        .map_err(|e| {
            error!("Failed to move to Picking: {}", e);
            PartyError::Db("Failed to move to Picking".into())
        })?;
    let _ = db.set_voting_round(party.id, None).await;
    party.state = PartyState::Picking;
    party.voting_round = None;
    Ok(EndVotingTransition::PhaseChanged(PartyState::Picking))
}

/// If all members are ready and state is Created, Picking, or Review, advance automatically.
/// Call after set_ready. No leader check.
/// Returns `Some(new_state)` when we advanced, `None` when we didn't.
pub async fn try_auto_advance_on_ready(
    db: &Database,
    party_id: Uuid,
) -> Result<Option<PartyState>, PartyError> {
    let all_ready = db
        .are_all_members_ready(party_id)
        .await
        .map_err(PartyError::from)?;
    if !all_ready {
        return Ok(None);
    }
    let mut party = Party::load(db, party_id).await?;
    let new_state = match party.state {
        PartyState::Created => {
            party.do_created_to_picking(db).await?;
            Some(PartyState::Picking)
        }
        PartyState::Picking => {
            party.do_picking_to_voting(db).await?;
            Some(PartyState::Voting)
        }
        PartyState::Review => {
            party.do_review_to_created(db).await?;
            Some(PartyState::Created)
        }
        _ => None,
    };
    if new_state.is_some() {
        debug!(
            "Party {} auto-advanced (all ready) -> {:?}",
            party_id, new_state
        );
    }
    Ok(new_state)
}

/// If all members have voted and state is Voting, run end-voting automatically.
/// Call after vote_movie. No leader check.
/// Returns `Some(transition)` when we ran end-voting, `None` otherwise.
pub async fn try_auto_end_voting(
    db: &Database,
    party_id: Uuid,
) -> Result<Option<EndVotingTransition>, PartyError> {
    let state = db.get_state(party_id).await.map_err(PartyError::from)?;
    if state != PartyState::Voting {
        return Ok(None);
    }
    let all_voted = db
        .have_all_members_voted(party_id)
        .await
        .map_err(PartyError::from)?;
    if !all_voted {
        return Ok(None);
    }
    let mut party = Party::load(db, party_id).await?;
    let t = run_end_voting_internal(db, &mut party, false).await?;
    debug!(
        "Party {} auto-ended voting (all voted) -> {:?}",
        party_id, t
    );
    Ok(Some(t))
}

/// Voting and watching timeout durations (seconds). From env or defaults.
pub fn get_timeout_secs() -> (u32, u32) {
    let voting = std::env::var("VOTING_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(180);
    let watching = std::env::var("WATCHING_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(900);
    (voting, watching)
}

/// Run the timeouts tick: auto-end overdue Voting, auto-advance overdue Watching → Review.
/// Call periodically (e.g. every 30s). Uses env VOTING_TIMEOUT_SECS and WATCHING_TIMEOUT_SECS, or defaults.
pub async fn run_timeouts_tick(db: &Database) -> Result<(), PartyError> {
    use chrono::Utc;

    let (voting_secs, watching_secs) = get_timeout_secs();

    let now = Utc::now();
    let voting_deadline = now - chrono::Duration::seconds(voting_secs as i64);
    let watching_deadline = now - chrono::Duration::seconds(watching_secs as i64);

    let voting_parties = db
        .get_parties_for_timeout(PartyState::Voting, voting_deadline)
        .await
        .map_err(PartyError::from)?;
    for party_id in voting_parties {
        if let Ok(mut party) = Party::load(db, party_id).await {
            if let Err(e) = run_end_voting_internal(db, &mut party, true).await {
                log::error!("Timeout end-voting party {}: {:?}", party_id, e);
            } else {
                debug!("Party {} voting timed out, auto-ended", party_id);
            }
        }
    }

    let watching_parties = db
        .get_parties_for_timeout(PartyState::Watching, watching_deadline)
        .await
        .map_err(PartyError::from)?;
    for party_id in watching_parties {
        if let Ok(mut party) = Party::load(db, party_id).await {
            if party.state != PartyState::Watching {
                continue;
            }
            if let Err(e) = party.do_watching_to_review(db).await {
                log::error!("Timeout Watching→Review party {}: {:?}", party_id, e);
            } else {
                debug!(
                    "Party {} watching timed out, auto-advanced to Review",
                    party_id
                );
            }
        }
    }

    Ok(())
}
