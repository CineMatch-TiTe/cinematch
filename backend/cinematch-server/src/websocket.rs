//! WebSocket handler and broadcast helpers using actor-based WsRegistry.

use actix_identity::Identity;
use actix_web::{HttpRequest, HttpResponse, get, web::Payload};
use actix_web_actors::ws;
use log::{error, trace};
use uuid::Uuid;

use crate::AppState;
use crate::api_error::ApiError;
use crate::extract_user_id;
use cinematch_abi::websocket::session::WsSession;
use cinematch_db::domain::Party;

// Re-export models from common for handlers to use
pub use cinematch_common::models::websocket::ServerMessage;

/// WebSocket upgrade endpoint. Requires authentication via cookie.
#[utoipa::path(
    responses(
        (status = 101, description = "WebSocket upgrade"),
        (status = 401, description = "Unauthorized")
    ),
    tags = ["websocket"],
    security(("cookie_auth" = [])),
    operation_id = "websocket"
)]
#[get("")]
pub async fn websocket_controller(
    req: HttpRequest,
    stream: Payload,
    db: AppState,
    user: Identity,
) -> Result<HttpResponse, ApiError> {
    let user_id = extract_user_id(user)?;

    trace!("WebSocket upgrade for user {}", user_id);

    let session = WsSession::new(user_id, db.ws_registry.clone());
    ws::start(session, &req, stream).map_err(ApiError::from)
}

/// Broadcast a message to all members of a party.
/// Fetches member IDs from the database and sends via WsRegistry.
pub async fn broadcast_to_party<T: serde::Serialize>(
    db: &AppState,
    party_id: Uuid,
    msg: &T,
    exclude_user: Option<Uuid>,
) {
    let party = match Party::from_id(db, party_id).await {
        Ok(p) => p,
        Err(e) => {
            error!("broadcast_to_party: party {} not found: {:?}", party_id, e);
            return;
        }
    };

    let member_ids = match party.member_ids(db).await {
        Ok(ids) => ids,
        Err(e) => {
            error!(
                "broadcast_to_party: failed to get members for party {}: {:?}",
                party_id, e
            );
            return;
        }
    };

    let recipients: Vec<Uuid> = if let Some(exclude) = exclude_user {
        member_ids.into_iter().filter(|id| *id != exclude).collect()
    } else {
        member_ids
    };

    db.ws_registry.send_to_users(&recipients, msg);
}
