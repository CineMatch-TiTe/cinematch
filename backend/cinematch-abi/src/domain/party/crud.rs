use cinematch_db::AppContext;
use cinematch_db::domain::{Member, Party};
use log::error;
use uuid::Uuid;

use super::super::DomainError;
use super::PartyValidation;

use async_trait::async_trait;
use cinematch_common::models::websocket::{MovieVotes, ReadyStateUpdate, ServerMessage};

/// CRUD operations for party members.
/// CRUD operations for party members.
#[async_trait]
pub trait PartyCrud: PartyValidation {
    /// Add a user to the party (checks if already member).
    async fn add_member_checked(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
    ) -> Result<Member, DomainError>;

    /// Remove a user from the party (validates membership first).
    async fn remove_member_checked(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
    ) -> Result<(), DomainError>;

    /// Kick a member (leader only).
    async fn kick(
        &self,
        ctx: &impl AppContext,
        leader_id: Uuid,
        target_id: Uuid,
    ) -> Result<(), DomainError>;

    /// Set member ready status.
    async fn set_member_ready(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
        ready: bool,
    ) -> Result<(), DomainError>;

    /// Transfer leadership to another member.
    async fn transfer_leadership_checked(
        &self,
        ctx: &impl AppContext,
        current_leader_id: Uuid,
        new_leader_id: Uuid,
    ) -> Result<(), DomainError>;

    /// Disband the party (leader only).
    async fn disband_checked(
        &self,
        ctx: &impl AppContext,
        leader_id: Uuid,
    ) -> Result<(), DomainError>;

    /// Get user's picks in this party.
    async fn get_user_picks(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
    ) -> Result<Vec<i64>, DomainError>;

    /// Add or update a pick.
    async fn set_user_pick(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
        movie_id: i64,
        liked: Option<bool>,
    ) -> Result<(), DomainError>;

    /// Remove a pick.
    async fn remove_user_pick(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
        movie_id: i64,
    ) -> Result<(), DomainError>;

    /// Cast a vote for a movie and broadcast update.
    async fn cast_vote_with_broadcast(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
        movie_id: i64,
        like: bool,
    ) -> Result<(u32, u32), DomainError>;

    /// Get what user voted for.
    async fn get_user_votes(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
    ) -> Result<Vec<i64>, DomainError>;

    /// Get current voting ballot for a user.
    async fn get_ballot(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
    ) -> Result<Vec<i64>, DomainError>;

    /// Check if user can vote.
    async fn can_vote(&self, ctx: &impl AppContext) -> Result<bool, DomainError>;

    /// Get voting round number.
    async fn voting_round(&self, ctx: &impl AppContext) -> Result<i32, DomainError>;

    /// Get number of unique members who have voted in this round.
    async fn voting_participation_count(&self, ctx: &impl AppContext)
    -> Result<usize, DomainError>;
}

#[async_trait]
impl PartyCrud for Party {
    async fn add_member_checked(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
    ) -> Result<Member, DomainError> {
        // Check if already a member
        if self.is_member_of(ctx, user_id).await? {
            return Err(DomainError::Conflict("Already in party".into()));
        }

        self.add_member(ctx, user_id)
            .await
            .map_err(DomainError::from)
    }

    async fn remove_member_checked(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
    ) -> Result<(), DomainError> {
        self.require_member(ctx, user_id).await?;

        // Check if the user leaving is the current leader
        let is_leader = self.leader_id(ctx).await? == user_id;

        self.remove_member(ctx, user_id)
            .await
            .map_err(DomainError::from)?;

        ctx.broadcast_party(self.id, &ServerMessage::PartyMemberLeft(user_id), None);

        // Auto-promote the oldest remaining member if the leader left
        if is_leader {
            let mut members = self.member_records(ctx).await.unwrap_or_default();
            if !members.is_empty() {
                // Sort by joined_at ascending to find the oldest member
                members.sort_by_key(|m| m.joined_at);
                let new_leader_id = members[0].user_id;

                if let Err(e) = self.transfer_leadership(ctx, new_leader_id).await {
                    error!("Failed to auto-promote new leader {}: {}", new_leader_id, e);
                } else {
                    ctx.broadcast_party(
                        self.id,
                        &ServerMessage::PartyLeaderChanged(new_leader_id),
                        None,
                    );
                }
            } else {
                // No members left, disband the party
                if let Err(e) = self.disband(ctx).await {
                    error!("Failed to auto-disband empty party {}: {}", self.id, e);
                }
            }
        }

        Ok(())
    }

    async fn kick(
        &self,
        ctx: &impl AppContext,
        leader_id: Uuid,
        target_id: Uuid,
    ) -> Result<(), DomainError> {
        self.require_leader(ctx, leader_id).await?;

        if leader_id == target_id {
            return Err(DomainError::BadRequest("Cannot kick yourself".into()));
        }

        self.remove_member_checked(ctx, target_id).await
    }

    async fn set_member_ready(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
        ready: bool,
    ) -> Result<(), DomainError> {
        self.require_member(ctx, user_id).await?;

        // Get the member and set ready
        let member = Member::from_ids(ctx, user_id, self.id)
            .await
            .map_err(DomainError::from)?;

        member
            .set_ready(ctx, ready)
            .await
            .map_err(DomainError::from)?;

        let msg = ServerMessage::UpdateReadyState(ReadyStateUpdate { user_id, ready });
        ctx.broadcast_party(self.id, &msg, None);

        Ok(())
    }

    async fn transfer_leadership_checked(
        &self,
        ctx: &impl AppContext,
        current_leader_id: Uuid,
        new_leader_id: Uuid,
    ) -> Result<(), DomainError> {
        self.require_leader(ctx, current_leader_id).await?;
        self.require_member(ctx, new_leader_id).await?;

        self.transfer_leadership(ctx, new_leader_id)
            .await
            .map_err(|e| {
                error!("Failed to transfer leadership: {}", e);
                DomainError::from(e)
            })?;

        ctx.broadcast_party(
            self.id,
            &ServerMessage::PartyLeaderChanged(new_leader_id),
            None,
        );

        Ok(())
    }

    async fn disband_checked(
        &self,
        ctx: &impl AppContext,
        leader_id: Uuid,
    ) -> Result<(), DomainError> {
        self.require_leader(ctx, leader_id).await?;

        let members = self.member_ids(ctx).await.unwrap_or_default();
        self.disband(ctx).await.map_err(DomainError::from)?;

        if !members.is_empty() {
            ctx.send_users(&members, &ServerMessage::PartyDisbanded);
        }

        Ok(())
    }

    async fn get_user_picks(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
    ) -> Result<Vec<i64>, DomainError> {
        self.require_member(ctx, user_id).await?;
        self.get_user_picks(ctx, user_id)
            .await
            .map_err(DomainError::from)
    }

    async fn set_user_pick(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
        movie_id: i64,
        liked: Option<bool>,
    ) -> Result<(), DomainError> {
        self.require_member(ctx, user_id).await?;
        self.add_pick(ctx, user_id, movie_id, liked)
            .await
            .map_err(DomainError::from)
    }

    async fn remove_user_pick(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
        movie_id: i64,
    ) -> Result<(), DomainError> {
        self.require_member(ctx, user_id).await?;
        self.remove_pick(ctx, user_id, movie_id)
            .await
            .map_err(DomainError::from)
    }

    async fn cast_vote_with_broadcast(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
        movie_id: i64,
        like: bool,
    ) -> Result<(u32, u32), DomainError> {
        // Party::can_user_vote is usually called before. But we can call it here.
        if !self
            .can_user_vote(ctx, user_id, movie_id)
            .await
            .map_err(DomainError::from)?
        {
            return Err(DomainError::Forbidden("Cannot vote for this movie".into()));
        }

        self.cast_vote(ctx, user_id, movie_id, like)
            .await
            .map_err(DomainError::from)?;

        let (likes, dislikes) = self
            .get_movie_vote_totals(ctx, movie_id)
            .await
            .map_err(DomainError::from)?;
        let (likes, dislikes) = (likes as u32, dislikes as u32);

        let ws_message = ServerMessage::MovieVoteUpdate(MovieVotes {
            movie_id,
            likes,
            dislikes,
        });

        ctx.broadcast_party(self.id, &ws_message, None);

        Ok((likes, dislikes))
    }

    async fn get_user_votes(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
    ) -> Result<Vec<i64>, DomainError> {
        let votes = self
            .get_user_votes(ctx, user_id)
            .await
            .map_err(DomainError::from)?
            .into_iter()
            .map(|v| v.movie_id)
            .collect();
        Ok(votes)
    }

    async fn get_ballot(
        &self,
        ctx: &impl AppContext,
        user_id: Uuid,
    ) -> Result<Vec<i64>, DomainError> {
        let ballot = self
            .get_ballot(ctx, user_id)
            .await
            .map_err(DomainError::from)?;
        Ok(ballot)
    }

    async fn can_vote(&self, ctx: &impl AppContext) -> Result<bool, DomainError> {
        match self.state(ctx).await.map_err(DomainError::from)? {
            cinematch_db::PartyState::Voting => Ok(true),
            _ => Ok(false),
        }
    }

    async fn voting_round(&self, _ctx: &impl AppContext) -> Result<i32, DomainError> {
        Ok(1)
    }

    async fn voting_participation_count(
        &self,
        ctx: &impl AppContext,
    ) -> Result<usize, DomainError> {
        self.get_voting_participation_count(ctx)
            .await
            .map_err(DomainError::from)
    }
}
