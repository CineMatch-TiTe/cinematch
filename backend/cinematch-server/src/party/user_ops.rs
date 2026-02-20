use super::{
    CreatePartyResponse, JoinQuery, MemberInfo, OptionalIdParam, PartyMembersResponse, ReadyQuery,
    ReadyStateResponse,
};

use crate::AppState;
use actix_identity::Identity;
use actix_web::{HttpResponse, get, patch, post, web};
use cinematch_common::models::ErrorResponse;
use log::debug;

use crate::api_error::ApiError;
use crate::extract_user_id;

use cinematch_abi::domain::{PartyCrud, PartyJoin, PartyValidation};
use cinematch_db::domain::{Party, User};

#[utoipa::path(
    responses(
        (status = 200, description = "Joined party", body = CreatePartyResponse),
        (status = 400, description = "Party not joinable or already in party", body = ErrorResponse),
        (status = 404, description = "Party not found", body = ErrorResponse)
    ),
    params(super::JoinQuery),
    tags = ["Member Ops"],
    security(("cookie_auth" = [])),
    operation_id = "join_party"
)]
#[post("/join")]
pub async fn join_party(
    db: AppState,
    user: Identity,
    query: web::Query<JoinQuery>,
) -> Result<web::Json<CreatePartyResponse>, ApiError> {
    let code = query.into_inner().code;
    let user_id = extract_user_id(user)?;

    // Use ABI Join by code
    let party_obj = Party::join_by_code(&db, user_id, &code).await?;

    debug!(
        "User {} successfully joined party {}",
        user_id, party_obj.id
    );

    let response = CreatePartyResponse {
        party_id: party_obj.id,
        code: code.clone(),
        created_at: party_obj.phase_entered_at(&db).await?, // Approximation
    };
    Ok(web::Json(response))
}

#[utoipa::path(
    responses(
        (status = 200, description = "Left"),
        (status = 400, description = "Not a member", body = ErrorResponse),
        (status = 404, description = "Party not found")
    ),
    params(super::OptionalIdParam),
    tags = ["Member Ops"],
    security(("cookie_auth" = [])),
    operation_id = "leave_party"
)]
#[post("/leave")]
pub async fn leave_party(
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
    party_obj.remove_member_checked(&db, user_id).await?;

    debug!("User {} left party {}", user_id, party_id);

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    responses(
        (status = 200, description = "Members", body = PartyMembersResponse),
        (status = 403, description = "Not a party member", body = ErrorResponse),
        (status = 404, description = "Party not found", body = ErrorResponse)
    ),
    params(super::OptionalIdParam),
    tags = ["Member Ops"],
    security(("cookie_auth" = [])),
    operation_id = "get_party_members"
)]
#[get("/members")]
pub async fn get_party_members(
    db: AppState,
    user: Identity,
    party_query: web::Query<OptionalIdParam>,
) -> Result<web::Json<PartyMembersResponse>, ApiError> {
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
    party_obj.require_member(&db, user_id).await?;

    let members = party_obj.member_records(&db).await?;
    let leader_id = party_obj.leader_id(&db).await?;

    let mut member_infos: Vec<MemberInfo> = Vec::with_capacity(members.len());
    for member in members {
        let user_obj = User::from_id(&db, member.user_id).await?;
        let username = user_obj
            .username(&db)
            .await
            .unwrap_or_else(|_| "Unknown".to_string());

        member_infos.push(MemberInfo {
            user_id: member.user_id,
            username,
            is_leader: member.user_id == leader_id,
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
    Ok(web::Json(response))
}

#[utoipa::path(
    responses(
        (status = 200, description = "Ready state", body = ReadyStateResponse),
        (status = 400, description = "Not a party member", body = ErrorResponse),
        (status = 404, description = "Party not found", body = ErrorResponse)
    ),
    params(
        super::ReadyQuery,
        super::OptionalIdParam
    ),
    tags = ["Member Ops"],
    security(("cookie_auth" = [])),
    operation_id = "set_ready"
)]
#[patch("/ready")]
pub async fn set_ready(
    db: AppState,
    user: Identity,
    ready_query: web::Query<ReadyQuery>,
    party_query: web::Query<OptionalIdParam>,
) -> Result<web::Json<ReadyStateResponse>, ApiError> {
    let is_ready = ready_query.is_ready;
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
    party_obj.set_member_ready(&db, user_id, is_ready).await?;

    debug!("Ready state toggled for user {}", user_id);

    if !is_ready {
        let ctx: std::sync::Arc<dyn cinematch_db::AppContext> = std::sync::Arc::new(db.clone());
        db.scheduler.cancel_and_broadcast(party_id, &ctx).await;
    }

    let (ready_count, total) = party_obj.ready_status(&db).await?;
    let all_ready = total > 0 && ready_count == total;

    if all_ready {
        let ctx: std::sync::Arc<dyn cinematch_db::AppContext> = std::sync::Arc::new(db.clone());
        db.scheduler.schedule_ready_countdown(party_id, ctx).await;
    }

    Ok(web::Json(ReadyStateResponse { all_ready }))
}
