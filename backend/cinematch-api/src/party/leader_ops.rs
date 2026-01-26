//! Thin HTTP handlers for party leader actions. Logic in `crate::handler::party`.
//! Advance is leader-only force-skip; transitions also happen automatically (all ready, all voted).

use super::{
    AppState, DbError, ErrorResponse, KickMemberRequest, TransferLeadershipRequest, extract_user_id,
};
use crate::WsStoreData;
use crate::handler::party::{EndVotingTransition, Party, PartyAdvanceOutcome, PartyError};
use crate::websocket::broadcast_party_timeout;
use crate::websocket::models::{ServerMessage, VotingRoundStarted};
use actix_identity::Identity;
use actix_web::{HttpResponse, post, web};
use log::{debug, error, trace};
use uuid::Uuid;

fn map_party_error(e: PartyError) -> HttpResponse {
    match e {
        PartyError::NotFound => {
            HttpResponse::NotFound().json(ErrorResponse::new("Party not found"))
        }
        PartyError::Forbidden(msg) => HttpResponse::Forbidden().json(ErrorResponse::new(msg)),
        PartyError::BadRequest(msg) => HttpResponse::BadRequest().json(ErrorResponse::new(msg)),
        PartyError::Db(msg) => {
            error!("Party handler DB error: {}", msg);
            HttpResponse::InternalServerError().json(ErrorResponse::new(msg))
        }
    }
}

#[utoipa::path(
    responses(
        (status = 200, description = "Phase advanced"),
        (status = 400, description = "Invalid state transition", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Leader only", body = ErrorResponse),
        (status = 404, description = "Party not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(("party_id" = Uuid, Path, description = "Party ID")),
    tags = ["party"],
    security(("cookie_auth" = [])),
    operation_id = "advance_phase"
)]
#[post("/{party_id}/advance")]
pub async fn advance_phase(
    db: AppState,
    store: WsStoreData,
    user: Identity,
    party_id: web::Path<Uuid>,
) -> HttpResponse {
    let party_id = party_id.into_inner();
    let leader_id = extract_user_id!(user);

    trace!("POST /{}/advance - leader_id={}", party_id, leader_id);

    let mut party = match Party::load(db.as_ref(), party_id).await {
        Ok(p) => p,
        Err(e) => return map_party_error(e),
    };

    match party.advance_phase(db.as_ref(), leader_id).await {
        Ok(outcome) => {
            match &outcome {
                PartyAdvanceOutcome::PhaseChanged(s) => {
                    debug!("Party {} phase advanced to {:?}", party_id, s);
                    let _ = store
                        .send_message_to_party(
                            party_id.to_string(),
                            &ServerMessage::PartyStateChanged(*s),
                            None,
                        )
                        .await;
                    let _ = broadcast_party_timeout(db.as_ref(), store.as_ref(), party_id).await;
                }
                PartyAdvanceOutcome::VotingEnded(EndVotingTransition::Round2Started) => {
                    debug!("Party {} voting round 2 started", party_id);
                    let _ = store
                        .send_message_to_party(
                            party_id.to_string(),
                            &ServerMessage::VotingRoundStarted(VotingRoundStarted { round: 2 }),
                            None,
                        )
                        .await;
                    let _ = broadcast_party_timeout(db.as_ref(), store.as_ref(), party_id).await;
                }
                PartyAdvanceOutcome::VotingEnded(EndVotingTransition::PhaseChanged(s)) => {
                    debug!("Party {} voting ended -> {:?}", party_id, s);
                    let _ = store
                        .send_message_to_party(
                            party_id.to_string(),
                            &ServerMessage::PartyStateChanged(*s),
                            None,
                        )
                        .await;
                    let _ = broadcast_party_timeout(db.as_ref(), store.as_ref(), party_id).await;
                }
            }
            HttpResponse::Ok().finish()
        }
        Err(e) => map_party_error(e),
    }
}

/// Disband party (leader only).
#[utoipa::path(
    responses(
        (status = 200, description = "Party disbanded"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Leader only", body = ErrorResponse),
        (status = 404, description = "Party not found"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(("party_id" = Uuid, Path, description = "Party ID")),
    tags = ["party"],
    security(("cookie_auth" = [])),
    operation_id = "disband_party"
)]
#[post("/{party_id}/disband")]
pub async fn disband_party(
    db: AppState,
    store: WsStoreData,
    user: Identity,
    party_id: web::Path<Uuid>,
) -> HttpResponse {
    let party_id = party_id.into_inner();
    let user_id = extract_user_id!(user);

    trace!("POST /{}/disband - leader_id={}", party_id, user_id);

    match db.get_party(party_id).await {
        Ok(party) if party.party_leader_id != user_id => {
            trace!(
                "Authorization failed: {} is not leader {}",
                user_id, party.party_leader_id
            );
            return HttpResponse::Forbidden()
                .json(ErrorResponse::new("Only party leader can disband party"));
        }
        Err(e) => {
            error!("Failed to get party: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to get party"));
        }
        _ => {}
    }

    debug!("Auth passed - disbanding party {}", party_id);
    match db.disband_party(party_id).await {
        Ok(_) => {
            debug!("Party {} disbanded successfully", party_id);
            // Notify all party members that the party was disbanded
            let msg = ServerMessage::PartyDisbanded;
            let _ = store
                .send_message_to_party(party_id.to_string(), &msg, None)
                .await;
            HttpResponse::Ok().finish()
        }
        Err(DbError::PartyNotFound(_)) => HttpResponse::NotFound().finish(),
        Err(e) => {
            error!("Failed to disband party: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse::new(format!(
                "Failed to disband party: {}",
                e
            )))
        }
    }
}

/// Kick a member (leader only).
#[utoipa::path(
    request_body = KickMemberRequest,
    responses(
        (status = 200, description = "Member kicked"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Leader only or not a member", body = ErrorResponse),
        (status = 404, description = "Party or member not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(("party_id" = Uuid, Path, description = "Party ID")),
    tags = ["party"],
    security(("cookie_auth" = [])),
    operation_id = "kick_member"
)]
#[post("/{party_id}/kick")]
pub async fn kick_member(
    db: AppState,
    store: WsStoreData,
    user: Identity,
    party_id: web::Path<Uuid>,
    body: web::Json<KickMemberRequest>,
) -> HttpResponse {
    let party_id = party_id.into_inner();
    let target_user_id = body.target_user_id;
    let requester_id = extract_user_id!(user);

    trace!(
        "POST /{}/kick - requester_id={}, target_user_id={}",
        party_id, requester_id, target_user_id
    );

    match db.is_party_member(party_id, requester_id).await {
        Ok(false) => {
            return HttpResponse::Forbidden()
                .json(ErrorResponse::new("Not a member of this party"));
        }
        Err(e) => {
            error!("Failed to check membership: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to check membership"));
        }
        Ok(true) => {}
    }

    let party = match db.get_party(party_id).await {
        Ok(p) => p,
        Err(DbError::PartyNotFound(_)) => {
            return HttpResponse::NotFound().json(ErrorResponse::new("Party not found"));
        }
        Err(e) => {
            error!("Failed to get party: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to get party"));
        }
    };

    if party.party_leader_id != requester_id {
        return HttpResponse::Forbidden()
            .json(ErrorResponse::new("Only party leader can kick members"));
    }

    debug!("Kicking user {} from party {}", target_user_id, party_id);
    match db.remove_party_member(party_id, target_user_id).await {
        Ok(()) => {
            debug!("User {} kicked from party {}", target_user_id, party_id);
            // Notify remaining members that the user was kicked
            let msg = ServerMessage::PartyMemberLeft(target_user_id);
            let _ = store
                .send_message_to_party(party_id.to_string(), &msg, None)
                .await;
            HttpResponse::Ok().finish()
        }
        Err(DbError::NotPartyMember) => {
            HttpResponse::NotFound().json(ErrorResponse::new("User is not a party member"))
        }
        Err(e) => {
            error!("Failed to kick member: {}", e);
            HttpResponse::InternalServerError()
                .json(ErrorResponse::new(format!("Failed to kick member: {}", e)))
        }
    }
}

/// Transfer leadership (leader only). New leader must be a party member.
#[utoipa::path(
    request_body = TransferLeadershipRequest,
    responses(
        (status = 200, description = "Leadership transferred"),
        (status = 400, description = "New leader not a member", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Leader only", body = ErrorResponse),
        (status = 404, description = "Party not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(("party_id" = Uuid, Path, description = "Party ID")),
    tags = ["party"],
    security(("cookie_auth" = [])),
    operation_id = "transfer_leadership"
)]
#[post("/{party_id}/transfer-leadership")]
pub async fn transfer_leadership(
    db: AppState,
    store: WsStoreData,
    user: Identity,
    party_id: web::Path<Uuid>,
    body: web::Json<TransferLeadershipRequest>,
) -> HttpResponse {
    let party_id = party_id.into_inner();
    let new_leader_id = body.new_leader_id;
    let requester_id = extract_user_id!(user);

    match db.get_party(party_id).await {
        Ok(party) if party.party_leader_id != requester_id => {
            return HttpResponse::Forbidden().json(ErrorResponse::new(
                "Only party leader can transfer leadership",
            ));
        }
        Err(DbError::PartyNotFound(_)) => {
            return HttpResponse::NotFound().json(ErrorResponse::new("Party not found"));
        }
        Err(e) => {
            error!("Failed to verify party leader: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to verify leadership"));
        }
        _ => {}
    }

    match db.is_party_member(party_id, new_leader_id).await {
        Ok(true) => {}
        Ok(false) => {
            return HttpResponse::BadRequest()
                .json(ErrorResponse::new("New leader must be a party member"));
        }
        Err(e) => {
            error!("Failed to check membership: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse::new(
                "Failed to verify membership for new leader",
            ));
        }
    }

    match db.transfer_party_leadership(party_id, new_leader_id).await {
        Ok(_) => {
            // Notify all party members of the leadership change
            let msg = ServerMessage::PartyLeaderChanged(new_leader_id);
            let _ = store
                .send_message_to_party(party_id.to_string(), &msg, None)
                .await;
            HttpResponse::Ok().finish()
        }
        Err(DbError::PartyNotFound(_)) => {
            HttpResponse::NotFound().json(ErrorResponse::new("Party not found"))
        }
        Err(e) => {
            error!("Failed to transfer leadership: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse::new(format!(
                "Failed to transfer leadership: {}",
                e
            )))
        }
    }
}
