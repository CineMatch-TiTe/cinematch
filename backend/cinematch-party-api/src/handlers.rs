//! Party API route handlers for Actix-web
//!
//! These handlers implement the party management endpoints.
//! 
//! ## Authentication Notes (TODO)
//! 
//! When auth is implemented, the following endpoints should require authentication:
//! - `create_party`: Requires authenticated user (user_id from JWT)
//! - `join_party`: Requires authenticated user (user_id from JWT)
//! - `leave_party`: User can only leave for themselves (user_id from JWT must match)
//! - `kick_member`: Only party leader can kick members
//! - `transfer_leadership`: Only party leader can transfer
//! - `advance_phase`: Only party leader can advance phase
//! - `start_new_round`: Only party leader can start new round (Review state only)
//! - `disband_party`: Only party leader can disband
//! - `toggle_ready`: User can only toggle their own ready state
//! - `get_party`: Any authenticated party member can view
//! - `get_party_members`: Any authenticated party member can view

use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::models::*;
use cinematch_common::UserClaims;
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
/// **Auth**: Requires authenticated user (JWT bearer).
#[utoipa::path(
    post,
    path = "/api/party",
    responses(
        (status = 201, description = "Party created successfully", body = CreatePartyResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tags = ["party"],
    security(("bearer_auth" = [])),
    operation_id = "create_party"
)]
pub async fn create_party(
    db: AppState,
    claims: UserClaims,
) -> HttpResponse {
    let user_id = claims.user_id;

    // Verify the user exists
    match db.get_user(user_id).await {
        Ok(_) => {}
        Err(DbError::UserNotFound(_)) => {
            return HttpResponse::NotFound().json(ErrorResponse::new("User not found"));
        }
        Err(e) => {
            log::error!("Failed to verify user: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to verify user"));
        }
    }

    match db.create_party(user_id).await {
        Ok((party, code)) => {
            // Also add the creator as a party member
            if let Err(e) = db.add_party_member(party.id, user_id).await {
                log::error!("Failed to add creator as party member: {}", e);
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
            log::error!("Failed to create party: {}", e);
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
/// **Auth**: Should require user to be a party member.
#[utoipa::path(
    get,
    path = "/api/party/{party_id}",
    responses(
        (status = 200, description = "Party details retrieved", body = PartyResponse),
        (status = 404, description = "Party not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("party_id" = Uuid, Path, description = "The party's unique ID")
    ),
    tags = ["party"]
)]
pub async fn get_party(db: AppState, party_id: web::Path<Uuid>) -> HttpResponse {
    let party_id = party_id.into_inner();

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
            log::error!("Failed to get party: {}", e);
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
/// **Auth**: Requires authenticated user (JWT bearer).
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
    claims: UserClaims,
    code: web::Path<String>,
) -> HttpResponse {
    let code = code.into_inner();
    let user_id = claims.user_id;

    // Find party by code
    let party = match db.get_party_by_code(&code).await {
        Ok(Some(party)) => party,
        Ok(None) => {
            return HttpResponse::NotFound().json(ErrorResponse::new("Party not found"));
        }
        Err(e) => {
            log::error!("Failed to find party by code: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to find party"));
        }
    };

    // Check party is still joinable
    if party.state != PartyState::Created {
        return HttpResponse::BadRequest()
            .json(ErrorResponse::new("Party is no longer accepting new members"));
    }

    // Check if user is already a member
    match db.is_party_member(party.id, user_id).await {
        Ok(true) => {
            return HttpResponse::BadRequest()
                .json(ErrorResponse::new("Already a member of this party"));
        }
        Err(e) => {
            log::error!("Failed to check membership: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to check membership"));
        }
        Ok(false) => {}
    }

    // Add as member
    match db.add_party_member(party.id, user_id).await {
        Ok(_) => {
            let response = CreatePartyResponse {
                party_id: party.id,
                code: code.clone(),
                created_at: party.created_at,
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            log::error!("Failed to join party: {}", e);
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
/// **Auth**: User can only leave for themselves (JWT bearer).
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
    claims: UserClaims,
    party_id: web::Path<Uuid>,
) -> HttpResponse {
    let party_id = party_id.into_inner();
    let user_id = claims.user_id;

    // Verify user is a member
    match db.is_party_member(party_id, user_id).await {
        Ok(false) => {
            return HttpResponse::BadRequest()
                .json(ErrorResponse::new("Not a member of this party"));
        }
        Err(e) => {
            log::error!("Failed to check membership: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to check membership"));
        }
        Ok(true) => {}
    }

    match db.leave_party(party_id, user_id).await {
        Ok(Some(_party)) => HttpResponse::Ok().finish(),
        Ok(None) => {
            // Party was disbanded (user was last member)
            HttpResponse::Ok().finish()
        }
        Err(DbError::NotPartyMember) => {
            HttpResponse::BadRequest().json(ErrorResponse::new("Not a member of this party"))
        }
        Err(DbError::PartyNotFound(_)) => {
            HttpResponse::NotFound().json(ErrorResponse::new("Party not found"))
        }
        Err(e) => {
            log::error!("Failed to leave party: {}", e);
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
/// **Auth**: Should require user to be a party member.
#[utoipa::path(
    get,
    path = "/api/party/{party_id}/members",
    responses(
        (status = 200, description = "Party members retrieved", body = PartyMembersResponse),
        (status = 404, description = "Party not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("party_id" = Uuid, Path, description = "The party ID")
    ),
    tags = ["party"]
)]
pub async fn get_party_members(db: AppState, party_id: web::Path<Uuid>) -> HttpResponse {
    let party_id = party_id.into_inner();

    // Get party first to get leader ID
    let party = match db.get_party(party_id).await {
        Ok(party) => party,
        Err(DbError::PartyNotFound(_)) => {
            return HttpResponse::NotFound().json(ErrorResponse::new("Party not found"));
        }
        Err(e) => {
            log::error!("Failed to get party: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new(format!("Failed to get party: {}", e)));
        }
    };

    // Get member records with ready state
    let member_records = match db.get_party_member_records(party_id).await {
        Ok(records) => records,
        Err(e) => {
            log::error!("Failed to get member records: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new(format!("Failed to get members: {}", e)));
        }
    };

    // Get user details for each member
    match db.get_party_members(party_id).await {
        Ok(users) => {
            let mut member_infos: Vec<MemberInfo> = Vec::with_capacity(users.len());
            
            for user in users {
                // Find the corresponding member record to get is_ready and joined_at
                let member_record = member_records
                    .iter()
                    .find(|m| m.user_id == user.id);
                
                let (is_ready, joined_at) = match member_record {
                    Some(record) => (record.is_ready, record.joined_at),
                    None => (false, user.created_at), // Fallback
                };

                member_infos.push(MemberInfo {
                    user_id: user.id,
                    username: user.username,
                    is_leader: user.id == party.party_leader_id,
                    is_ready,
                    joined_at,
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
            log::error!("Failed to get party members: {}", e);
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
/// **Auth**: Only party leader can kick members (JWT bearer).
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
    claims: UserClaims,
    party_id: web::Path<Uuid>,
    body: web::Json<KickMemberRequest>,
) -> HttpResponse {
    let party_id = party_id.into_inner();
    let target_user_id = body.target_user_id;
    let requester_id = claims.user_id;

    // Verify requester is a member
    match db.is_party_member(party_id, requester_id).await {
        Ok(false) => {
            return HttpResponse::Forbidden()
                .json(ErrorResponse::new("Not a member of this party"));
        }
        Err(e) => {
            log::error!("Failed to check membership: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to check membership"));
        }
        Ok(true) => {}
    }

    // Get party to find leader
    let party = match db.get_party(party_id).await {
        Ok(party) => party,
        Err(DbError::PartyNotFound(_)) => {
            return HttpResponse::NotFound().json(ErrorResponse::new("Party not found"));
        }
        Err(e) => {
            log::error!("Failed to get party: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new(format!("Failed to get party: {}", e)));
        }
    };

    if party.party_leader_id != requester_id {
        return HttpResponse::Forbidden().json(ErrorResponse::new(
            "Only party leader can kick members",
        ));
    }

    match db.kick_party_member(party_id, requester_id, target_user_id).await {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(DbError::CannotKickSelf) => {
            HttpResponse::BadRequest().json(ErrorResponse::new("Cannot kick yourself, use leave instead"))
        }
        Err(DbError::NotPartyLeader) => {
            HttpResponse::Forbidden().json(ErrorResponse::new("Only party leader can kick members"))
        }
        Err(DbError::NotPartyMember) => {
            HttpResponse::NotFound().json(ErrorResponse::new("User is not a party member"))
        }
        Err(e) => {
            log::error!("Failed to kick member: {}", e);
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
/// **Auth**: Only party leader can transfer leadership (JWT bearer).
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
    claims: UserClaims,
    party_id: web::Path<Uuid>,
    body: web::Json<TransferLeadershipRequest>,
) -> HttpResponse {
    let party_id = party_id.into_inner();
    let new_leader_id = body.new_leader_id;
    let requester_id = claims.user_id;

    // Verify requester is a member
    match db.is_party_member(party_id, requester_id).await {
        Ok(false) => {
            return HttpResponse::Forbidden()
                .json(ErrorResponse::new("Not a member of this party"));
        }
        Err(e) => {
            log::error!("Failed to check membership: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to check membership"));
        }
        Ok(true) => {}
    }

    // Verify new leader is a party member
    match db.is_party_member(party_id, new_leader_id).await {
        Ok(true) => {}
        Ok(false) => {
            return HttpResponse::BadRequest()
                .json(ErrorResponse::new("New leader must be a party member"));
        }
        Err(e) => {
            log::error!("Failed to check membership: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to verify membership"));
        }
    }

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
            log::error!("Failed to verify party leader: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to verify leadership"));
        }
        _ => {}
    }

    match db.transfer_party_leadership(party_id, new_leader_id).await {
        Ok(_party) => HttpResponse::Ok().finish(),
        Err(DbError::PartyNotFound(_)) => {
            HttpResponse::NotFound().json(ErrorResponse::new("Party not found"))
        }
        Err(e) => {
            log::error!("Failed to transfer leadership: {}", e);
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
/// **Auth**: User can only toggle their own ready state (JWT bearer).
#[utoipa::path(
    post,
    path = "/api/party/{party_id}/ready",
    responses(
        (status = 200, description = "Ready state toggled", body = ReadyStateResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 400, description = "Not a party member", body = ErrorResponse),
        (status = 404, description = "Party not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("party_id" = Uuid, Path, description = "The party ID")
    ),
    tags = ["party"],
    security(("bearer_auth" = [])),
    operation_id = "toggle_ready"
)]
pub async fn toggle_ready(
    db: AppState,
    claims: UserClaims,
    party_id: web::Path<Uuid>,
) -> HttpResponse {
    let party_id = party_id.into_inner();
    let user_id = claims.user_id;

    // Verify user is a member
    match db.is_party_member(party_id, user_id).await {
        Ok(false) => {
            return HttpResponse::BadRequest()
                .json(ErrorResponse::new("Not a member of this party"));
        }
        Err(e) => {
            log::error!("Failed to check membership: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to check membership"));
        }
        Ok(true) => {}
    }

    match db.toggle_member_ready(party_id, user_id).await {
        Ok(_new_ready_state) => {
            // Get the ready status for the party
            match db.get_ready_status(party_id).await {
                Ok((ready_count, total)) => {
                    let response = ReadyStateResponse {
                        all_ready: total > 0 && ready_count == total,
                    };
                    HttpResponse::Ok().json(response)
                }
                Err(e) => {
                    log::error!("Failed to get ready status: {}", e);
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
            log::error!("Failed to toggle ready state: {}", e);
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
/// **Auth**: Only party leader can advance phase (JWT bearer).
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
    claims: UserClaims,
    party_id: web::Path<Uuid>,
) -> HttpResponse {
    let party_id = party_id.into_inner();
    let leader_id = claims.user_id;

    // Get current state first
    let current_party = match db.get_party(party_id).await {
        Ok(party) => party,
        Err(DbError::PartyNotFound(_)) => {
            return HttpResponse::NotFound().json(ErrorResponse::new("Party not found"));
        }
        Err(e) => {
            log::error!("Failed to get party: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new(format!("Failed to get party: {}", e)));
        }
    };

    // Verify requester is a member
    match db.is_party_member(party_id, leader_id).await {
        Ok(false) => {
            return HttpResponse::Forbidden()
                .json(ErrorResponse::new("Not a member of this party"));
        }
        Err(e) => {
            log::error!("Failed to check membership: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to check membership"));
        }
        Ok(true) => {}
    }

    // Verify requester is the leader
    if current_party.party_leader_id != leader_id {
        return HttpResponse::Forbidden()
            .json(ErrorResponse::new("Only party leader can advance phase"));
    }

    match db.advance_party_phase(party_id, leader_id).await {
        Ok(updated_party) => {
            let response = PhaseAdvanceResponse {
                new_state: updated_party.state.into(),
            };
            HttpResponse::Ok().json(response)
        }
        Err(DbError::NotPartyLeader) => {
            HttpResponse::Forbidden().json(ErrorResponse::new("Only party leader can advance phase"))
        }
        Err(DbError::InvalidStateTransition(msg)) => {
            HttpResponse::BadRequest().json(ErrorResponse::new(format!("Invalid transition: {}", msg)))
        }
        Err(DbError::PartyNotFound(_)) => {
            HttpResponse::NotFound().json(ErrorResponse::new("Party not found"))
        }
        Err(e) => {
            log::error!("Failed to advance phase: {}", e);
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
/// **Auth**: Only party leader can start new round (JWT bearer).
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
    claims: UserClaims,
    party_id: web::Path<Uuid>,
) -> HttpResponse {
    let party_id = party_id.into_inner();
    let leader_id = claims.user_id;

    // Verify requester is a member
    match db.is_party_member(party_id, leader_id).await {
        Ok(false) => {
            return HttpResponse::Forbidden()
                .json(ErrorResponse::new("Not a member of this party"));
        }
        Err(e) => {
            log::error!("Failed to check membership: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to check membership"));
        }
        Ok(true) => {}
    }

    // Verify requester is the leader
    match db.get_party(party_id).await {
        Ok(party) if party.party_leader_id != leader_id => {
            return HttpResponse::Forbidden()
                .json(ErrorResponse::new("Only party leader can start new round"));
        }
        Err(e) => {
            log::error!("Failed to get party: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to get party"));
        }
        _ => {}
    }

    match db.start_new_round(party_id, leader_id).await {
        Ok((_party, code)) => {
            let response = NewRoundResponse {
                code: code.code,
            };
            HttpResponse::Ok().json(response)
        }
        Err(DbError::NotPartyLeader) => {
            HttpResponse::Forbidden().json(ErrorResponse::new("Only party leader can start new round"))
        }
        Err(DbError::InvalidStateTransition(msg)) => {
            HttpResponse::BadRequest().json(ErrorResponse::new(format!("Cannot start new round: {}", msg)))
        }
        Err(DbError::PartyNotFound(_)) => {
            HttpResponse::NotFound().json(ErrorResponse::new("Party not found"))
        }
        Err(e) => {
            log::error!("Failed to start new round: {}", e);
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
/// **Auth**: Only party leader can disband (JWT bearer).
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
    claims: UserClaims,
    party_id: web::Path<Uuid>,
) -> HttpResponse {
    let party_id = party_id.into_inner();
    let leader_id = claims.user_id;

    // Verify requester is a member
    match db.is_party_member(party_id, leader_id).await {
        Ok(false) => {
            return HttpResponse::Forbidden()
                .json(ErrorResponse::new("Not a member of this party"));
        }
        Err(e) => {
            log::error!("Failed to check membership: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to check membership"));
        }
        Ok(true) => {}
    }

    // Verify requester is the leader
    match db.get_party(party_id).await {
        Ok(party) if party.party_leader_id != leader_id => {
            return HttpResponse::Forbidden()
                .json(ErrorResponse::new("Only party leader can disband party"));
        }
        Err(e) => {
            log::error!("Failed to get party: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to get party"));
        }
        _ => {}
    }

    match db.disband_party(party_id, leader_id).await {
        Ok(_party) => HttpResponse::Ok().finish(),
        Err(DbError::NotPartyLeader) => {
            HttpResponse::Forbidden().json(ErrorResponse::new("Only party leader can disband party"))
        }
        Err(DbError::PartyNotFound(_)) => {
            HttpResponse::NotFound().json(ErrorResponse::new("Party not found"))
        }
        Err(e) => {
            log::error!("Failed to disband party: {}", e);
            HttpResponse::InternalServerError()
                .json(ErrorResponse::new(format!("Failed to disband party: {}", e)))
        }
    }
}
