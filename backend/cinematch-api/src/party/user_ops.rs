use super::{
    CreatePartyResponse, DbError, ErrorResponse, MemberInfo, PartyMembersResponse, PartyState,
    ReadyStateResponse, SetReadyRequest, extract_user_id,
};

use crate::{AppState, WsStoreData};
use actix_identity::Identity;
use actix_web::{HttpResponse, get, patch, post, web};
use log::{debug, error, trace};
use uuid::Uuid;

use crate::websocket::broadcast_party_timeout;
use crate::websocket::models::{MemberJoined, ServerMessage};

#[utoipa::path(
    responses(
        (status = 200, description = "Joined party", body = CreatePartyResponse),
        (status = 400, description = "Party not joinable or already in party", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Party not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(("code" = String, Path, description = "4-character join code")),
    tags = ["party"],
    security(("cookie_auth" = [])),
    operation_id = "join_party"
)]
#[post("/join/{code}")]
pub async fn join_party(
    db: AppState,
    user: Identity,
    store: WsStoreData,
    code: web::Path<String>,
) -> HttpResponse {
    let code = code.into_inner();
    let user_id = extract_user_id!(user);

    trace!("POST /join/{} - user_id={}", code, user_id);

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
        return HttpResponse::BadRequest().json(ErrorResponse::new(
            "Party is no longer accepting new members",
        ));
    }

    // Check if user is already in a party
    match db.get_user_party(user_id).await {
        Ok(Some(_)) => {
            return HttpResponse::BadRequest()
                .json(ErrorResponse::new("User is already in a party"));
        }
        Ok(None) => {}
        Err(e) => {
            error!("Failed to check user party: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to check user party"));
        }
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

    debug!(
        "Auth passed - adding user {} to party {}",
        user_id, party.id
    );
    // Add as member
    match db.add_party_member(party.id, user_id).await {
        Ok(_) => {
            debug!("User {} successfully joined party {}", user_id, party.id);

            let username = match db.get_user(user_id).await {
                Ok(user) => user.username,
                Err(e) => {
                    error!("Failed to fetch username for user {}: {}", user_id, e);
                    "Unknown".to_string()
                }
            };

            let msg = ServerMessage::PartyMemberJoined(MemberJoined { user_id, username });
            let _ = store
                .send_message_to_party(party.id.to_string(), &msg, Some(&[user_id]))
                .await;

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

/// Leave the party. Leader leaves → transfer to oldest member; no members left → disband.
#[utoipa::path(
    responses(
        (status = 200, description = "Left"),
        (status = 400, description = "Not a member", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Party not found"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(("party_id" = Uuid, Path, description = "Party ID")),
    tags = ["party"],
    security(("cookie_auth" = [])),
    operation_id = "leave_party"
)]
#[post("/{party_id}/leave")]
pub async fn leave_party(
    db: AppState,
    user: Identity,
    store: WsStoreData,
    party_id: web::Path<Uuid>,
) -> HttpResponse {
    let party_id = party_id.into_inner();
    let user_id = extract_user_id!(user);

    match db.remove_party_member(party_id, user_id).await {
        Ok(_) => {
            debug!("User {} left party {}", user_id, party_id);
            // Notify remaining members
            let msg = ServerMessage::PartyMemberLeft(user_id);
            let _ = store
                .send_message_to_party(party_id.to_string(), &msg, None)
                .await;
            HttpResponse::Ok().finish()
        }
        Err(DbError::NotPartyMember) => {
            HttpResponse::BadRequest().json(ErrorResponse::new("Not a member of this party"))
        }
        Err(DbError::PartyNotFound(_)) => HttpResponse::NotFound().finish(),
        Err(e) => {
            error!("Failed to leave party: {}", e);
            HttpResponse::InternalServerError()
                .json(ErrorResponse::new(format!("Failed to leave party: {}", e)))
        }
    }
}

#[utoipa::path(
    responses(
        (status = 200, description = "Members", body = PartyMembersResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Not a party member", body = ErrorResponse),
        (status = 404, description = "Party not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(("party_id" = Uuid, Path, description = "Party ID")),
    tags = ["party"],
    security(("cookie_auth" = [])),
    operation_id = "get_party_members"
)]
#[get("/{party_id}/members")]
pub async fn get_party_members(
    db: AppState,
    user: Identity,
    party_id: web::Path<Uuid>,
) -> HttpResponse {
    let party_id = party_id.into_inner();
    let user_id = extract_user_id!(user);

    trace!("GET /{}/members - user_id={}", party_id, user_id);

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
                let user = user_records.iter().find(|u| u.id == member.user_id);

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

#[utoipa::path(
    request_body = SetReadyRequest,
    responses(
        (status = 200, description = "Ready state", body = ReadyStateResponse),
        (status = 400, description = "Not a party member", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Party not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(("party_id" = Uuid, Path, description = "Party ID")),
    tags = ["party"],
    security(("cookie_auth" = [])),
    operation_id = "set_ready"
)]
#[patch("/{party_id}/ready")]
pub async fn set_ready(
    db: AppState,
    store: WsStoreData,
    user: Identity,
    party_id: web::Path<Uuid>,
    body: web::Json<SetReadyRequest>,
) -> HttpResponse {
    let party_id = party_id.into_inner();
    let ready = body.into_inner();
    let user_id = extract_user_id!(user);

    trace!("POST /{}/ready - user_id={}", party_id, user_id);

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

    debug!(
        "Auth passed - toggling ready state for user {} in party {}",
        user_id, party_id
    );
    match db.set_member_ready(party_id, user_id, ready.is_ready).await {
        Ok(_) => {
            debug!("Ready state toggled for user {}", user_id);
            // Auto-advance when all ready (Created→Picking or Picking→Voting or Review→Created).
            // Advancing resets readiness, so we must return all_ready from *before* the reset.
            let advanced =
                match crate::handler::party::try_auto_advance_on_ready(db.as_ref(), party_id).await
                {
                    Ok(Some(s)) => Some(s),
                    _ => None,
                };
            if let Some(new_state) = advanced {
                let msg = ServerMessage::PartyStateChanged(new_state);
                let _ = store
                    .send_message_to_party(party_id.to_string(), &msg, None)
                    .await;
                let _ = broadcast_party_timeout(db.as_ref(), store.as_ref(), party_id).await;
                // We advanced because everyone was ready; readiness is now reset. Return all_ready: true.
                return HttpResponse::Ok().json(ReadyStateResponse { all_ready: true });
            }
            match db.get_ready_status(party_id).await {
                Ok((ready_count, total)) => {
                    let response = ReadyStateResponse {
                        all_ready: total > 0 && ready_count == total,
                    };
                    HttpResponse::Ok().json(response)
                }
                Err(e) => {
                    error!("Failed to get ready status: {}", e);
                    let response = ReadyStateResponse { all_ready: false };
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
