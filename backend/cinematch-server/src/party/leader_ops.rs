//! Thin HTTP handlers for party leader actions.
//! Uses domain logic from `cinematch_abi::domain`.

use super::{AppState, KickQuery, OptionalIdParam, TransferQuery};
use crate::api_error::ApiError;
use crate::extract_user_id;

use actix_identity::Identity;
use actix_web::{HttpResponse, post, web};
use cinematch_common::models::ErrorResponse;
use log::debug;

// Import domain types and traits from cinematch_abi
use cinematch_abi::domain::{
    EndVotingTransition, PartyAdvanceOutcome, PartyCrud, PartyStateMachine,
};
use cinematch_db::domain::{Party, User};

#[utoipa::path(
    responses(
        (status = 200, description = "Phase advanced"),
        (status = 400, description = "Invalid state transition", body = ErrorResponse),
        (status = 403, description = "Leader only", body = ErrorResponse),
        (status = 404, description = "Party not found", body = ErrorResponse)
    ),
    params(super::OptionalIdParam),
    tags = ["Leader Tools"],
    security(("cookie_auth" = [])),
    operation_id = "advance_phase"
)]
#[post("/advance")]
pub async fn advance_phase(
    db: AppState,
    user: Identity,
    party_query: web::Query<OptionalIdParam>,
) -> Result<HttpResponse, ApiError> {
    let leader_id = extract_user_id(user)?;
    let party_id = match party_query.id {
        Some(id) => id,
        None => {
            let user_obj = User::from_id(&db, leader_id).await?;
            user_obj
                .current_party(&db)
                .await?
                .ok_or_else(|| ApiError::NotFound("No active party found".to_string()))?
                .id
        }
    };

    let party = Party::from_id(&db, party_id).await?;

    let outcome = party.advance_phase(&db, leader_id).await?;
    match &outcome {
        PartyAdvanceOutcome::PhaseChanged(s) => {
            debug!("Party {} phase advanced to {:?}", party_id, s);
            db.scheduler
                .enforce_phase_timeout_and_broadcast(party_id, std::sync::Arc::new(db.clone()))
                .await;
        }
        PartyAdvanceOutcome::VotingEnded(EndVotingTransition::Round2Started) => {
            debug!("Party {} voting round 2 started", party_id);
            db.scheduler
                .enforce_phase_timeout_and_broadcast(party_id, std::sync::Arc::new(db.clone()))
                .await;
        }
        PartyAdvanceOutcome::VotingEnded(EndVotingTransition::PhaseChanged(s)) => {
            debug!("Party {} voting ended -> {:?}", party_id, s);
            db.scheduler
                .enforce_phase_timeout_and_broadcast(party_id, std::sync::Arc::new(db.clone()))
                .await;
        }
    }
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    responses(
        (status = 200, description = "Party disbanded"),
        (status = 403, description = "Leader only", body = ErrorResponse),
        (status = 404, description = "Party not found")
    ),
    params(super::OptionalIdParam),
    tags = ["Leader Tools"],
    security(("cookie_auth" = [])),
    operation_id = "disband_party"
)]
#[post("/disband")]
pub async fn disband_party(
    db: AppState,
    user: Identity,
    party_query: web::Query<OptionalIdParam>,
) -> Result<HttpResponse, ApiError> {
    let user_id = extract_user_id(user)?;
    let party_id = match party_query.id {
        Some(id) => id,
        None => {
            let user_obj = User::from_id(&db, user_id).await?;
            user_obj
                .current_party(&db)
                .await?
                .ok_or_else(|| ApiError::NotFound("No active party found".to_string()))?
                .id
        }
    };

    let party_obj = Party::from_id(&db, party_id).await?;
    party_obj.disband_checked(&db, user_id).await?;

    debug!("Party {} disbanded successfully", party_id);
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    responses(
        (status = 200, description = "Member kicked"),
        (status = 403, description = "Leader only or not a member", body = ErrorResponse),
        (status = 404, description = "Party or member not found", body = ErrorResponse)
    ),
    params(
        super::KickQuery,
        super::OptionalIdParam
    ),
    tags = ["Leader Tools"],
    security(("cookie_auth" = [])),
    operation_id = "kick_member"
)]
#[post("/kick")]
pub async fn kick_member(
    db: AppState,
    user: Identity,
    kick_query: web::Query<KickQuery>,
    party_query: web::Query<OptionalIdParam>,
) -> Result<HttpResponse, ApiError> {
    let requester_id = extract_user_id(user)?;
    let target_user_id = kick_query.target_user_id;
    let party_id = match party_query.id {
        Some(id) => id,
        None => {
            let user_obj = User::from_id(&db, requester_id).await?;
            user_obj
                .current_party(&db)
                .await?
                .ok_or_else(|| ApiError::NotFound("No active party found".to_string()))?
                .id
        }
    };

    let party_obj = Party::from_id(&db, party_id).await?;
    party_obj.kick(&db, requester_id, target_user_id).await?;

    debug!("User {} kicked from party {}", target_user_id, party_id);
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    responses(
        (status = 200, description = "Leadership transferred"),
        (status = 400, description = "New leader not a member", body = ErrorResponse),
        (status = 403, description = "Leader only", body = ErrorResponse),
        (status = 404, description = "Party not found", body = ErrorResponse)
    ),
    params(
        super::TransferQuery,
        super::OptionalIdParam
    ),
    tags = ["Leader Tools"],
    security(("cookie_auth" = [])),
    operation_id = "transfer_leadership"
)]
#[post("/transfer-leadership")]
pub async fn transfer_leadership(
    db: AppState,
    user: Identity,
    transfer_query: web::Query<TransferQuery>,
    party_query: web::Query<OptionalIdParam>,
) -> Result<HttpResponse, ApiError> {
    let requester_id = extract_user_id(user)?;
    let new_leader_id = transfer_query.new_leader_id;
    let party_id = match party_query.id {
        Some(id) => id,
        None => {
            let user_obj = User::from_id(&db, requester_id).await?;
            user_obj
                .current_party(&db)
                .await?
                .ok_or_else(|| ApiError::NotFound("No active party found".to_string()))?
                .id
        }
    };

    let party_obj = Party::from_id(&db, party_id).await?;
    party_obj
        .transfer_leadership_checked(&db, requester_id, new_leader_id)
        .await?;
    Ok(HttpResponse::Ok().finish())
}
