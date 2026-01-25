use super::{
    AppState, CreatePartyResponse, DbError, ErrorResponse, PartyCode, PartyResponse, PartyState,
    extract_user_id,
};
use actix_identity::Identity;
use actix_web::{HttpResponse, get, post, web};
use log::{debug, error, trace};
use uuid::Uuid;


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
    security(("bearer_auth" = []))
)]
#[get("/{party_id}/vote/{movie_id}")]
pub async fn vote_movie(db: AppState, user: Identity, party_id: web::Path<Uuid>, movie_id: web::Path<i64>) -> HttpResponse {
    let party_id = party_id.into_inner();
    let movie_id = movie_id.into_inner();
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
