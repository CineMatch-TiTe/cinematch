use super::{
    AppState, CreatePartyResponse, DbError, ErrorResponse, PartyCode, PartyResponse, PartyState,
    extract_user_id,
};
use crate::handler::party::get_timeout_secs;
use actix_identity::Identity;
use actix_web::{HttpResponse, get, post, web};
use log::{debug, error, trace};
use uuid::Uuid;

#[utoipa::path(
    responses(
        (status = 201, description = "Party created successfully", body = CreatePartyResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tags = ["party"],
    security(("cookie_auth" = [])),
    operation_id = "create_party"
)]
#[post("")]
pub async fn create_party(db: AppState, user: Option<Identity>) -> HttpResponse {
    let user_id = extract_user_id!(user);

    trace!("POST  - user_id={}", user_id);

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

    // Check if user is already in a party
    match db.get_user_party(user_id).await {
        Ok(Some(_)) => {
            return HttpResponse::Forbidden()
                .json(ErrorResponse::new("User is already in a party"));
        }
        Ok(None) => {}
        Err(e) => {
            error!("Failed to check user party: {}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to check user party"));
        }
    }

    debug!("Auth passed - creating party for user {}", user_id);
    match db.create_party(user_id).await {
        Ok((party, code)) => {
            debug!(
                "Party created successfully: id={}, code={}",
                party.id, code.code
            );
            // Also add the creator as a party member
            if let Err(e) = db.add_party_member(party.id, user_id).await {
                error!("Failed to add creator as party member: {}", e);
                return HttpResponse::InternalServerError()
                    .json(ErrorResponse::new("Failed to initialize party membership"));
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
            HttpResponse::InternalServerError()
                .json(ErrorResponse::new(format!("Failed to create party: {}", e)))
        }
    }
}

#[utoipa::path(
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
    security(("cookie_auth" = []))
)]
#[get("/{party_id}")]
pub async fn get_party(db: AppState, user: Identity, party_id: web::Path<Uuid>) -> HttpResponse {
    let party_id = party_id.into_inner();
    let user_id = extract_user_id!(user);

    trace!("GET /{} - user_id={}", party_id, user_id);

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
            let code: Option<String> = if party.state == PartyState::Created {
                let code_result: Result<Option<PartyCode>, DbError> =
                    db.get_party_code(party_id).await;
                code_result.ok().flatten().map(|c| c.code)
            } else {
                None
            };

            let vote_status = match db.get_party_votes(party_id, Some(user_id)).await {
                Ok(votes) if party.state == PartyState::Voting => Some(votes),
                _ => None,
            };
            let (voting_timeout_secs, watching_timeout_secs) = get_timeout_secs();

            let response = PartyResponse {
                id: party.id,
                leader_id: party.party_leader_id,
                state: party.state.into(),
                created_at: party.created_at,
                code,
                vote_status,
                selected_movie_id: party.selected_movie_id,
                phase_entered_at: party.phase_entered_at,
                voting_timeout_secs,
                watching_timeout_secs,
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

#[utoipa::path(
    responses(
        (status = 200, description = "Party details retrieved", body = PartyResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "No active party found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tags = ["party"],
    security(("cookie_auth" = [])),
    operation_id = "get_my_party"
)]
#[get("")]
pub async fn get_my_party(db: AppState, user: Identity) -> HttpResponse {
    let user_id = extract_user_id!(user);
    match db.get_user_active_party(user_id).await {
        Ok(party_id) => {
            // Reuse the logic from get_party
            match db.is_party_member(party_id, user_id).await {
                Ok(false) | Err(_) => {
                    return HttpResponse::Forbidden()
                        .json(ErrorResponse::new("Not a member of this party"));
                }
                Ok(true) => {}
            }
            match db.get_party(party_id).await {
                Ok(party) => {
                    let code: Option<String> = if party.state == PartyState::Created {
                        let code_result: Result<Option<PartyCode>, DbError> =
                            db.get_party_code(party_id).await;
                        code_result.ok().flatten().map(|c| c.code)
                    } else {
                        None
                    };
                    let vote_status = match db.get_party_votes(party_id, Some(user_id)).await {
                        Ok(votes) if party.state == PartyState::Voting => Some(votes),
                        _ => None,
                    };
                    let (voting_timeout_secs, watching_timeout_secs) = get_timeout_secs();
                    let response = PartyResponse {
                        id: party.id,
                        leader_id: party.party_leader_id,
                        state: party.state.into(),
                        created_at: party.created_at,
                        code,
                        vote_status,
                        selected_movie_id: party.selected_movie_id,
                        phase_entered_at: party.phase_entered_at,
                        voting_timeout_secs,
                        watching_timeout_secs,
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
        Err(DbError::UserNotInParty(_)) => {
            HttpResponse::NotFound().json(ErrorResponse::new("No active party found"))
        }
        Err(e) => {
            error!("Failed to get user's active party: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse::new(format!(
                "Failed to get user's active party: {}",
                e
            )))
        }
    }
}
