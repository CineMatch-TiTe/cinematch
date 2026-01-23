use super::{
    AppState, DbError, ErrorResponse, KickMemberRequest, NewRoundResponse, PartyState,
    PhaseAdvanceResponse, TransferLeadershipRequest, extract_user_id,
};
use actix_identity::Identity;
use actix_web::{HttpResponse, post, web};
use log::{debug, error, trace};
use uuid::Uuid;

#[utoipa::path(
    responses(
        (status = 200, description = "Phase advanced successfully", body = PhaseAdvanceResponse),
        (status = 400, description = "Invalid state transition", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Only party leader can advance phase", body = ErrorResponse),
        (status = 404, description = "Party not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("party_id" = Uuid, Path, description = "The party ID")
    ),
    tags = ["party"],
    security(("bearer_auth" = [])),
    operation_id = "advance_phase"
)]
#[post("/{party_id}/advance")]
pub async fn advance_phase(
    db: AppState,
    user: Identity,
    party_id: web::Path<Uuid>,
) -> HttpResponse {
    let party_id = party_id.into_inner();
    let leader_id = extract_user_id!(user);

    trace!("POST /{}/advance - leader_id={}", party_id, leader_id);

    // Get current state first
    let current_party = match db.get_party(party_id).await {
        Ok(party) => party,
        Err(DbError::PartyNotFound(_)) => {
            trace!("Party {} not found", party_id);
            return HttpResponse::NotFound().json(ErrorResponse::new("Party not found"));
        }
        Err(e) => {
            error!("Failed to get party: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new(format!("Failed to get party: {}", e)));
        }
    };

    // Verify requester is a member
    match db.is_party_member(party_id, leader_id).await {
        Ok(false) => {
            trace!("Leader {} not member of party {}", leader_id, party_id);
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

    // Verify requester is the leader
    if current_party.party_leader_id != leader_id {
        trace!(
            "Authorization failed: {} is not leader {}",
            leader_id, current_party.party_leader_id
        );
        return HttpResponse::Forbidden()
            .json(ErrorResponse::new("Only party leader can advance phase"));
    }

    debug!(
        "Auth passed - advancing party {} phase from {:?}",
        party_id, current_party.state
    );

    let next = match current_party.state {
        PartyState::Created => PartyState::Picking,
        PartyState::Picking => PartyState::Voting,
        PartyState::Voting => PartyState::Watching,
        PartyState::Watching => PartyState::Review,
        PartyState::Review => {
            return HttpResponse::BadRequest()
                .json(ErrorResponse::new("Cannot advance phase from Review state"));
        }
        PartyState::Disbanded => {
            return HttpResponse::BadRequest().json(ErrorResponse::new(
                "Cannot advance phase of a disbanded party",
            ));
        }
    };

    match db.set_phase(party_id, next).await {
        Ok(updated_party) => {
            debug!(
                "Party {} phase advanced to {:?}",
                party_id, updated_party.state
            );
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            error!("Failed to advance phase: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse::new(format!(
                "Failed to advance phase: {}",
                e
            )))
        }
    }
}

/// Start a new movie round
///
/// Starts a new movie round from the Review state.
/// Only the party leader can start a new round.
/// This resets the party to Created state and generates a new join code.
/// All members' ready states are reset to false.
///
/// **Auth**: Only the party leader can start a new round.
#[utoipa::path(
    responses(
        (status = 200, description = "New round started", body = NewRoundResponse),
        (status = 400, description = "Party is not in Review state", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Only party leader can start new round", body = ErrorResponse),
        (status = 404, description = "Party not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("party_id" = Uuid, Path, description = "The party ID")
    ),
    tags = ["party"],
    security(("bearer_auth" = [])),
    operation_id = "start_new_round"
)]
#[post("/{party_id}/new-round")]
pub async fn start_new_round(
    db: AppState,
    user: Identity,
    party_id: web::Path<Uuid>,
) -> HttpResponse {
    let party_id = party_id.into_inner();
    let user_id = extract_user_id!(user);

    // Verify requester is the leader
    match db.get_party(party_id).await {
        Ok(party) if party.party_leader_id != user_id => {
            return HttpResponse::Forbidden()
                .json(ErrorResponse::new("Only party leader can start new round"));
        }
        Err(e) => {
            error!("Failed to get party: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to get party"));
        }
        _ => {}
    }

    match db.start_new_round(party_id).await {
        Ok(code) => {
            let response = NewRoundResponse { code: code.code };
            HttpResponse::Ok().json(response)
        }
        Err(DbError::PartyNotFound(_)) => {
            HttpResponse::NotFound().json(ErrorResponse::new("Party not found"))
        }
        Err(e) => {
            error!("Failed to start new round: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse::new(format!(
                "Failed to start new round: {}",
                e
            )))
        }
    }
}

/// Disband party
///
/// Disbands the party completely. Only the party leader can disband.
/// This sets the party state to Disbanded and removes the join code.
///
/// **Auth**: Only the party leader can disband the party.
#[utoipa::path(
    responses(
        (status = 200, description = "Party disbanded"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Only party leader can disband", body = ErrorResponse),
        (status = 404, description = "Party not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("party_id" = Uuid, Path, description = "The party ID")
    ),
    tags = ["party"],
    security(("bearer_auth" = [])),
    operation_id = "disband_party"
)]
#[post("/{party_id}/disband")]
pub async fn disband_party(
    db: AppState,
    user: Identity,
    party_id: web::Path<Uuid>,
) -> HttpResponse {
    let party_id = party_id.into_inner();
    let user_id = extract_user_id!(user);

    trace!("POST /{}/disband - leader_id={}", party_id, user_id);

    // Verify requester is the leader
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
        Ok(_party) => {
            debug!("Party {} disbanded successfully", party_id);
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

/// Kick a member from the party
///
/// Removes a member from the party. Only the party leader can kick members.
/// The leader cannot kick themselves (use leave instead).
///
/// **Auth**: Only the party leader can kick members.
#[utoipa::path(
    request_body = KickMemberRequest,
    responses(
        (status = 200, description = "Member kicked successfully"),
        (status = 400, description = "Cannot kick self or invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Only party leader can kick members", body = ErrorResponse),
        (status = 404, description = "Party or member not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("party_id" = Uuid, Path, description = "The party ID")
    ),
    tags = ["party"],
    security(("bearer_auth" = [])),
    operation_id = "kick_member"
)]
#[post("/{party_id}/kick")]
pub async fn kick_member(
    db: AppState,
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

    // Verify requester is a member
    match db.is_party_member(party_id, requester_id).await {
        Ok(false) => {
            trace!(
                "Requester {} not member of party {}",
                requester_id, party_id
            );
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

    // Get party to find leader
    let party = match db.get_party(party_id).await {
        Ok(party) => party,
        Err(DbError::PartyNotFound(_)) => {
            trace!("Party {} not found", party_id);
            return HttpResponse::NotFound().json(ErrorResponse::new("Party not found"));
        }
        Err(e) => {
            error!("Failed to get party: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new(format!("Failed to get party: {}", e)));
        }
    };

    if party.party_leader_id != requester_id {
        trace!(
            "Authorization failed: requester {} is not leader {}",
            requester_id, party.party_leader_id
        );
        return HttpResponse::Forbidden()
            .json(ErrorResponse::new("Only party leader can kick members"));
    }

    debug!(
        "Auth passed - kicking user {} from party {}",
        target_user_id, party_id
    );
    match db.remove_party_member(party_id, target_user_id).await {
        Ok(()) => {
            debug!("User {} kicked from party {}", target_user_id, party_id);
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

/// Transfer party leadership
///
/// Transfers leadership to another party member.
/// Only the current leader can transfer leadership.
///
/// **Auth**: Only the party leader can transfer leadership.
#[utoipa::path(
    request_body = TransferLeadershipRequest,
    responses(
        (status = 200, description = "Leadership transferred"),
        (status = 400, description = "New leader is not a party member", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Only party leader can transfer leadership", body = ErrorResponse),
        (status = 404, description = "Party not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("party_id" = Uuid, Path, description = "The party ID")
    ),
    tags = ["party"],
    security(("bearer_auth" = [])),
    operation_id = "transfer_leadership"
)]
#[post("/{party_id}/transfer-leadership")]
pub async fn transfer_leadership(
    db: AppState,
    user: Identity,
    party_id: web::Path<Uuid>,
    body: web::Json<TransferLeadershipRequest>,
) -> HttpResponse {
    let party_id = party_id.into_inner();
    let new_leader_id = body.new_leader_id;
    let requester_id = extract_user_id!(user);

    // Verify requester is the current leader
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

    // Verify new leader is a party member
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
        Ok(_party) => HttpResponse::Ok().finish(),
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
