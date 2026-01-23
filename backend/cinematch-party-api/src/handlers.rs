//! Party API route handlers for Actix-web
//!
//! These handlers implement the party management endpoints.
//! 
//! ## Authentication
//! 
//! All endpoints requiring authentication use JWT bearer tokens extracted from cookies.
//! The following endpoints have authentication requirements:
//! - `create_party`: Requires authenticated user
//! - `join_party`: Requires authenticated user
//! - `leave_party`: Requires authenticated user (can only leave for themselves)
//! - `kick_member`: Requires party leader
//! - `transfer_leadership`: Requires party leader
//! - `advance_phase`: Requires party leader
//! - `start_new_round`: Requires party leader (party must be in Review state)
//! - `disband_party`: Requires party leader
//! - `set_ready`: Requires authenticated party member (can only toggle own state)
//! - `get_party`: Requires authenticated party member
//! - `get_party_members`: Requires authenticated party member

use actix_web::{HttpResponse, web};
use actix_identity::Identity;
use log::{debug, trace, error};
use uuid::Uuid;

use crate::models::*;
use cinematch_common::{extract_user_id, ErrorResponse};
use cinematch_db::{Database, DbError, PartyState};

/// Application state wrapper providing database access
pub type AppState = web::Data<Database>;

// ============================================================================
// Party CRUD Operations
// ============================================================================

/// Create a new party
///
/// Creates a new party with the requesting user as the leader.
/// Returns a 4-character join code that other users can use to join.
///
/// **Auth**: Requires authenticated user.
#[utoipa::path(
    post,
    path = "/api/party",
    responses(
        (status = 201, description = "Party created successfully", body = CreatePartyResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tags = ["party"],
    security(("bearer_auth" = [])),
    operation_id = "create_party"
)]
pub async fn create_party(
    db: AppState,
    user: Option<Identity>,
) -> HttpResponse {
    let user_id = extract_user_id!(user);

    trace!("POST /api/party - user_id={}", user_id);

    // Verify the user exists
    match db.get_user(user_id).await {
        Ok(_) => {}
        Err(DbError::UserNotFound(_)) => {
            trace!("User not found: {}", user_id);
            return HttpResponse::NotFound().json(ErrorResponse::new("User not found"));
        }
        Err(e) => {
            error!("Failed to verify user: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to verify user"));
        }
    }

    debug!("Auth passed - creating party for user {}", user_id);
    match db.create_party(user_id).await {
        Ok((party, code)) => {
            debug!("Party created successfully: id={}, code={}", party.id, code.code);
            // Also add the creator as a party member
            if let Err(e) = db.add_party_member(party.id, user_id).await {
                error!("Failed to add creator as party member: {}", e);
                return HttpResponse::InternalServerError().json(ErrorResponse::new(
                    "Failed to initialize party membership",
                ));
            }

            let response = CreatePartyResponse {
                party_id: party.id,
                code: code.code,
                created_at: party.created_at,
            };
            HttpResponse::Created().json(response)
        }
        Err(e) => {
            error!("Failed to create party: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse::new(format!(
                "Failed to create party: {}",
                e
            )))
        }
    }
}

/// Get party details
///
/// Retrieves the details of a party including its current state.
/// The join code is only returned if the party is still in Created state.
///
/// **Auth**: Requires user to be a party member.
#[utoipa::path(
    get,
    path = "/api/party/{party_id}",
    responses(
        (status = 200, description = "Party details retrieved", body = PartyResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Not a party member", body = ErrorResponse),
        (status = 404, description = "Party not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("party_id" = Uuid, Path, description = "The party's unique ID")
    ),
    tags = ["party"],
    security(("bearer_auth" = []))
)]
pub async fn get_party(
    db: AppState,
    user: Identity,
    party_id: web::Path<Uuid>,
) -> HttpResponse {
    let party_id = party_id.into_inner();
    let user_id = extract_user_id!(user);

    trace!("GET /api/party/{} - user_id={}", party_id, user_id);

    // Verify user is a member
    match db.is_party_member(party_id, user_id).await {
        Ok(false) => {
            trace!("User {} not member of party {}", user_id, party_id);
            return HttpResponse::Forbidden()
                .json(ErrorResponse::new("Not a member of this party"));
        }
        Err(_) => {
            // If error checking membership, assume not a member
            return HttpResponse::Forbidden()
                .json(ErrorResponse::new("Not a member of this party"));
        }
        Ok(true) => {}
    }

    match db.get_party(party_id).await {
        Ok(party) => {
            // Only fetch code if party is in Created state
            let code = if party.state == PartyState::Created {
                db.get_party_code(party_id)
                    .await
                    .ok()
                    .flatten()
                    .map(|c| c.code)
            } else {
                None
            };

            let response = PartyResponse {
                id: party.id,
                leader_id: party.party_leader_id,
                state: party.state.into(),
                created_at: party.created_at,
                code,
            };
            HttpResponse::Ok().json(response)
        }
        Err(DbError::PartyNotFound(_)) => {
            HttpResponse::NotFound().json(ErrorResponse::new("Party not found"))
        }
        Err(e) => {
            error!("Failed to get party: {}", e);
            HttpResponse::InternalServerError()
                .json(ErrorResponse::new(format!("Failed to get party: {}", e)))
        }
    }
}

// ============================================================================
// Join/Leave Operations
// ============================================================================

/// Join an existing party by code
///
/// Adds the requesting user to a party using the 4-character join code.
/// Only works when party is in Created state.
///
/// **Auth**: Requires authenticated user.
#[utoipa::path(
    post,
    path = "/api/party/join/{code}",
    responses(
        (status = 200, description = "Successfully joined party", body = CreatePartyResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 400, description = "Invalid code or party not joinable", body = ErrorResponse),
        (status = 404, description = "Party not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("code" = String, Path, description = "The 4-character party join code")
    ),
    tags = ["party"],
    security(("bearer_auth" = [])),
    operation_id = "join_party"
)]
pub async fn join_party(
    db: AppState,
    user: Identity,
    code: web::Path<String>,
) -> HttpResponse {
    let code = code.into_inner();
    let user_id = extract_user_id!(user);

    trace!("POST /api/party/join/{} - user_id={}", code, user_id);

    // Find party by code
    let party = match db.get_party_by_code(&code).await {
        Ok(Some(party)) => party,
        Ok(None) => {
            trace!("Party not found for code: {}", code);
            return HttpResponse::NotFound().json(ErrorResponse::new("Party not found"));
        }
        Err(e) => {
            error!("Failed to find party by code: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to find party"));
        }
    };

    // Check party is still joinable
    if party.state != PartyState::Created {
        trace!("Party {} not joinable, state: {:?}", party.id, party.state);
        return HttpResponse::BadRequest()
            .json(ErrorResponse::new("Party is no longer accepting new members"));
    }

    // Check if user is already a member
    match db.is_party_member(party.id, user_id).await {
        Ok(true) => {
            trace!("User {} already member of party {}", user_id, party.id);
            return HttpResponse::BadRequest()
                .json(ErrorResponse::new("Already a member of this party"));
        }
        Err(e) => {
            error!("Failed to check membership: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to check membership"));
        }
        Ok(false) => {}
    }

    debug!("Auth passed - adding user {} to party {}", user_id, party.id);
    // Add as member
    match db.add_party_member(party.id, user_id).await {
        Ok(_) => {
            debug!("User {} successfully joined party {}", user_id, party.id);
            let response = CreatePartyResponse {
                party_id: party.id,
                code: code.clone(),
                created_at: party.created_at,
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            error!("Failed to join party: {}", e);
            HttpResponse::InternalServerError()
                .json(ErrorResponse::new(format!("Failed to join party: {}", e)))
        }
    }
}

/// Leave a party
///
/// Removes the requesting user from the party.
/// If the leader leaves, leadership is transferred to the oldest member.
/// If no members remain, the party is disbanded.
///
/// **Auth**: Requires authenticated user. Users can only leave for themselves.
#[utoipa::path(
    post,
    path = "/api/party/{party_id}/leave",
    responses(
        (status = 200, description = "Successfully left party"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 400, description = "Not a member of this party", body = ErrorResponse),
        (status = 404, description = "Party not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("party_id" = Uuid, Path, description = "The party ID to leave")
    ),
    tags = ["party"],
    security(("bearer_auth" = [])),
    operation_id = "leave_party"
)]
pub async fn leave_party(
    db: AppState,
    user: Identity,
    party_id: web::Path<Uuid>,
) -> HttpResponse {
    let party_id = party_id.into_inner();
    let user_id = extract_user_id!(user);

    match db.remove_party_member(party_id, user_id).await {
        Ok(_) => {
            debug!("User {} left party {}", user_id, party_id);
            HttpResponse::Ok().finish()
        }
        Err(DbError::NotPartyMember) => {
            HttpResponse::BadRequest().json(ErrorResponse::new("Not a member of this party"))
        }
        Err(DbError::PartyNotFound(_)) => {
            HttpResponse::NotFound().finish()
        }
        Err(e) => {
            error!("Failed to leave party: {}", e);
            HttpResponse::InternalServerError()
                .json(ErrorResponse::new(format!("Failed to leave party: {}", e)))
        }
    }
}

// ============================================================================
// Member Management (Leader Only)
// ============================================================================

/// Get all members in a party
///
/// Retrieves the list of all members currently in the party,
/// including their ready status.
///
/// **Auth**: Requires user to be a party member.
#[utoipa::path(
    get,
    path = "/api/party/{party_id}/members",
    responses(
        (status = 200, description = "Party members retrieved", body = PartyMembersResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Not a party member", body = ErrorResponse),
        (status = 404, description = "Party not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("party_id" = Uuid, Path, description = "The party ID")
    ),
    tags = ["party"],
    security(("bearer_auth" = []))
)]
pub async fn get_party_members(
    db: AppState,
    user: Identity,
    party_id: web::Path<Uuid>,
) -> HttpResponse {
    let party_id = party_id.into_inner();
    let user_id = extract_user_id!(user);

    trace!("GET /api/party/{}/members - user_id={}", party_id, user_id);

    // Verify user is a member
    match db.is_party_member(party_id, user_id).await {
        Ok(false) => {
            trace!("User {} not member of party {}", user_id, party_id);
            return HttpResponse::Forbidden()
                .json(ErrorResponse::new("Not a member of this party"));
        }
        Err(_) => {
            // If error checking membership, assume not a member
            return HttpResponse::Forbidden()
                .json(ErrorResponse::new("Not a member of this party"));
        }
        Ok(true) => {}
    }

    // Get party first to get leader ID
    let party = match db.get_party(party_id).await {
        Ok(party) => party,
        Err(DbError::PartyNotFound(_)) => {
            return HttpResponse::NotFound().json(ErrorResponse::new("Party not found"));
        }
        Err(e) => {
            error!("Failed to get party: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new(format!("Failed to get party: {}", e)));
        }
    };

    // Get member records with ready state
    let user_records = match db.get_party_users(party_id).await {
        Ok(records) => records,
        Err(e) => {
            error!("Failed to get member records: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new(format!("Failed to get members: {}", e)));
        }
    };

    // Get user details for each member
    match db.get_party_members(party_id).await {
        Ok(members) => {
            let mut member_infos: Vec<MemberInfo> = Vec::with_capacity(members.len());
            
            for member in members {
                // Find the corresponding member record to get is_ready and joined_at
                let user = user_records
                    .iter()
                    .find(|u| u.id == member.user_id);

                let user_name = match user {
                    Some(u) => u.username.clone(),
                    None => {
                        error!("User record not found for member: {}", member.user_id);
                        continue; // Skip this member
                    }
                };
                
                member_infos.push(MemberInfo {
                    user_id: member.user_id,
                    username: user_name,
                    is_leader: member.user_id == party.party_leader_id,
                    is_ready: member.is_ready,
                    joined_at: member.joined_at,
                });
            }

            let count = member_infos.len();
            let ready_count = member_infos.iter().filter(|m| m.is_ready).count();
            let all_ready = count > 0 && ready_count == count;
            
            let response = PartyMembersResponse {
                members: member_infos,
                count,
                ready_count,
                all_ready,
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            error!("Failed to get party members: {}", e);
            HttpResponse::InternalServerError()
                .json(ErrorResponse::new(format!("Failed to get members: {}", e)))
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
    post,
    path = "/api/party/{party_id}/kick",
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
pub async fn kick_member(
    db: AppState,
    user: Identity,
    party_id: web::Path<Uuid>,
    body: web::Json<KickMemberRequest>,
) -> HttpResponse {
    let party_id = party_id.into_inner();
    let target_user_id = body.target_user_id;
    let requester_id = extract_user_id!(user);

    trace!("POST /api/party/{}/kick - requester_id={}, target_user_id={}", party_id, requester_id, target_user_id);

    // Verify requester is a member
    match db.is_party_member(party_id, requester_id).await {
        Ok(false) => {
            trace!("Requester {} not member of party {}", requester_id, party_id);
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
        trace!("Authorization failed: requester {} is not leader {}", requester_id, party.party_leader_id);
        return HttpResponse::Forbidden().json(ErrorResponse::new(
            "Only party leader can kick members",
        ));
    }

    debug!("Auth passed - kicking user {} from party {}", target_user_id, party_id);
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
    post,
    path = "/api/party/{party_id}/transfer-leadership",
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
            return HttpResponse::Forbidden()
                .json(ErrorResponse::new("Only party leader can transfer leadership"));
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
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to verify membership for new leader"));
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

// ============================================================================
// Ready State Operations
// ============================================================================

/// Toggle member ready state
///
/// Toggles the ready state for the requesting user in this party.
/// Returns whether all members are ready after the toggle.
///
/// **Auth**: Users can only toggle their own ready state.
#[utoipa::path(
    post,
    path = "/api/party/{party_id}/ready",
    request_body = SetReadyRequest,
    responses(
        (status = 200, description = "Ready set"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 400, description = "Not a party member"),
        (status = 404, description = "Party not found"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("party_id" = Uuid, Path, description = "The party ID")
    ),
    tags = ["party"],
    security(("bearer_auth" = [])),
    operation_id = "toggle_ready"
)]
pub async fn set_ready(
    db: AppState,
    user: Identity,
    party_id: web::Path<Uuid>,
    body: web::Json<SetReadyRequest>,
) -> HttpResponse {
    let party_id = party_id.into_inner();
    let ready = body.into_inner();
    let user_id = extract_user_id!(user);

    trace!("POST /api/party/{}/ready - user_id={}", party_id, user_id);

    // Verify user is a member
    match db.is_party_member(party_id, user_id).await {
        Ok(false) => {
            trace!("User {} not member of party {}", user_id, party_id);
            return HttpResponse::BadRequest()
                .json(ErrorResponse::new("Not a member of this party"));
        }
        Err(e) => {
            error!("Failed to check membership: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to check membership"));
        }
        Ok(true) => {}
    }

    debug!("Auth passed - toggling ready state for user {} in party {}", user_id, party_id);
    match db.set_member_ready(party_id, user_id, ready.is_ready).await {
        Ok(_) => {
            debug!("Ready state toggled for user {}", user_id);
            // Get the ready status for the party
            match db.get_ready_status(party_id).await {
                Ok((ready_count, total)) => {
                    let response = ReadyStateResponse {
                        all_ready: total > 0 && ready_count == total,
                    };
                    HttpResponse::Ok().json(response)
                }
                Err(e) => {
                    error!("Failed to get ready status: {}", e);
                    // Still return success since the toggle worked
                    let response = ReadyStateResponse {
                        all_ready: false,
                    };
                    HttpResponse::Ok().json(response)
                }
            }
        }
        Err(DbError::NotPartyMember) => {
            HttpResponse::BadRequest().json(ErrorResponse::new("Not a member of this party"))
        }
        Err(DbError::PartyNotFound(_)) => {
            HttpResponse::NotFound().json(ErrorResponse::new("Party not found"))
        }
        Err(e) => {
            error!("Failed to toggle ready state: {}", e);
            HttpResponse::InternalServerError()
                .json(ErrorResponse::new(format!("Failed to toggle ready: {}", e)))
        }
    }
}

// ============================================================================
// Phase Control (Leader Only)
// ============================================================================

/// Advance to next phase
///
/// Advances the party to the next phase in the lifecycle.
/// Only the party leader can advance the phase.
/// This resets all members' ready states to false.
///
/// State transitions: Created → Picking → Voting → Watching → Review
///
/// When transitioning from Created, the join code is released.
///
/// **Auth**: Only the party leader can advance the phase.
#[utoipa::path(
    post,
    path = "/api/party/{party_id}/advance",
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
pub async fn advance_phase(
    db: AppState,
    user: Identity,
    party_id: web::Path<Uuid>,
) -> HttpResponse {
    let party_id = party_id.into_inner();
    let leader_id = extract_user_id!(user);

    trace!("POST /api/party/{}/advance - leader_id={}", party_id, leader_id);

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
        trace!("Authorization failed: {} is not leader {}", leader_id, current_party.party_leader_id);
        return HttpResponse::Forbidden()
            .json(ErrorResponse::new("Only party leader can advance phase"));
    }

    debug!("Auth passed - advancing party {} phase from {:?}", party_id, current_party.state);

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
            return HttpResponse::BadRequest()
                .json(ErrorResponse::new("Cannot advance phase of a disbanded party"));
        }
    };

    match db.set_phase(party_id, next).await {
        Ok(updated_party) => {
            debug!("Party {} phase advanced to {:?}", party_id, updated_party.state);
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            error!("Failed to advance phase: {}", e);
            HttpResponse::InternalServerError()
                .json(ErrorResponse::new(format!("Failed to advance phase: {}", e)))
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
    post,
    path = "/api/party/{party_id}/new-round",
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
            let response = NewRoundResponse {
                code: code.code,
            };
            HttpResponse::Ok().json(response)
        }
        Err(DbError::PartyNotFound(_)) => {
            HttpResponse::NotFound().json(ErrorResponse::new("Party not found"))
        }
        Err(e) => {
            error!("Failed to start new round: {}", e);
            HttpResponse::InternalServerError()
                .json(ErrorResponse::new(format!("Failed to start new round: {}", e)))
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
    post,
    path = "/api/party/{party_id}/disband",
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
pub async fn disband_party(
    db: AppState,
    user: Identity,
    party_id: web::Path<Uuid>,
) -> HttpResponse {
    let party_id = party_id.into_inner();
    let user_id = extract_user_id!(user);

    trace!("POST /api/party/{}/disband - leader_id={}", party_id, user_id);

    // Verify requester is the leader
    match db.get_party(party_id).await {
        Ok(party) if party.party_leader_id != user_id => {
            trace!("Authorization failed: {} is not leader {}", user_id, party.party_leader_id);
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
        Err(DbError::PartyNotFound(_)) => {
            HttpResponse::NotFound().finish()
        }
        Err(e) => {
            error!("Failed to disband party: {}", e);
            HttpResponse::InternalServerError()
                .json(ErrorResponse::new(format!("Failed to disband party: {}", e)))
        }
    }
}
