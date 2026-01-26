#![allow(clippy::await_holding_lock)]

use actix_identity::Identity;
use actix_web::{HttpRequest, HttpResponse, get, rt::spawn, web::Payload};

use actix_ws::{Item, Message};
use actix_wsb::Broadcaster;
use log::{debug, error, info, trace};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use cinematch_common::extract_user_id;

pub mod models;

use crate::AppState;
use crate::handler::party::get_timeout_secs;
use crate::websocket::models::{ClientMessage, PartyTimeoutUpdate, ServerMessage};
use cinematch_common::models::ErrorResponse;

/// WebSocket store: broadcaster + conn_id→user_id map. Wrapped in `Arc` and `web::Data` as app state.
/// `Broadcaster::new()` returns `Arc<RwLock<Broadcaster>>`.
#[derive(Clone)]
pub struct WsStore {
    pub broadcaster: Arc<RwLock<Broadcaster>>,
    pub conn_map: Arc<RwLock<HashMap<String, Uuid>>>,
}

impl Default for WsStore {
    fn default() -> Self {
        Self::new()
    }
}

impl WsStore {
    pub fn new() -> Self {
        Self {
            broadcaster: Broadcaster::new(),
            conn_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Broadcast to all connections in the room whose user_id (from map) is NOT in ignore_users.
    /// Creates the room if it doesn't exist (no-op if no connections).
    pub async fn send_message_to_party(
        &self,
        room_id: String,
        message: &ServerMessage,
        ignore_users: Option<&[Uuid]>,
    ) {
        let msg_text = match serde_json::to_string(message) {
            Ok(text) => text,
            Err(e) => {
                error!("Failed to serialize server message: {}", e);
                return;
            }
        };

        let mut write_broadcaster = self.broadcaster.write().unwrap();

        // If room doesn't exist, it means no connections are in this party yet
        if write_broadcaster.check_room(&room_id).is_none() {
            trace!("Room {} has no connections, skipping broadcast", room_id);
            return;
        }

        if let Some(ignore_list) = ignore_users {
            let exclude_conn_ids: HashSet<String> = {
                let map = self.conn_map.read().unwrap();
                map.iter()
                    .filter(|(_, uid)| ignore_list.contains(uid))
                    .map(|(cid, _)| cid.clone())
                    .collect()
            };
            write_broadcaster
                .room(&room_id)
                .broadcast_if(msg_text, |connection| {
                    !exclude_conn_ids.contains(&connection.id)
                })
                .await;
        } else {
            write_broadcaster.room(&room_id).broadcast(msg_text).await;
        }
    }

    /// Send only to connections whose user_id (from map) equals target_user_id.
    /// Creates the room if it doesn't exist (no-op if no connections).
    pub async fn send_message_to_user(
        &self,
        room_id: String,
        target_user_id: Uuid,
        message: &ServerMessage,
    ) {
        let msg_text = match serde_json::to_string(message) {
            Ok(text) => text,
            Err(e) => {
                error!("Failed to serialize server message: {}", e);
                return;
            }
        };

        let target_conn_ids: HashSet<String> = {
            let map = self.conn_map.read().unwrap();
            map.iter()
                .filter(|(_, uid)| **uid == target_user_id)
                .map(|(cid, _)| cid.clone())
                .collect()
        };

        // If no connections for this user, nothing to send
        if target_conn_ids.is_empty() {
            trace!(
                "No connections found for user {} in room {}",
                target_user_id, room_id
            );
            return;
        }

        let mut write_broadcaster = self.broadcaster.write().unwrap();

        // If room doesn't exist, it means no connections are in this party yet
        if write_broadcaster.check_room(&room_id).is_none() {
            trace!(
                "Room {} has no connections, skipping message to user {}",
                room_id, target_user_id
            );
            return;
        }

        write_broadcaster
            .room(&room_id)
            .broadcast_if(msg_text, |connection| {
                target_conn_ids.contains(&connection.id)
            })
            .await;
    }
}

/// Broadcast `PartyTimeoutUpdate` to the party room. Call after phase changes (including round 2).
pub async fn broadcast_party_timeout(
    db: &cinematch_db::Database,
    store: &std::sync::Arc<WsStore>,
    party_id: Uuid,
) {
    let party = match db.get_party(party_id).await {
        Ok(p) => p,
        Err(_) => return,
    };
    let (voting, watching) = get_timeout_secs();
    let msg = ServerMessage::PartyTimeoutUpdate(PartyTimeoutUpdate {
        phase_entered_at: party.phase_entered_at,
        voting_timeout_secs: voting,
        watching_timeout_secs: watching,
    });
    store
        .send_message_to_party(party_id.to_string(), &msg, None)
        .await;
}

#[utoipa::path(
    responses(
        (status = 200, description = "WebSocket upgrade; real-time party updates"),
        (status = 400, description = "Handshake failed (expect WebSocket upgrade)"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 406, description = "User not in a party")
    ),
    tags = ["websocket"],
    security(("cookie_auth" = [])),
    operation_id = "websocket_controller"
)]
#[get("")]
pub async fn websocket_controller(
    req: HttpRequest,
    body: Payload,
    db: AppState,
    store: crate::WsStoreData,
    user: Option<Identity>,
) -> HttpResponse {
    let requester_id = extract_user_id!(user);

    let party_id = match db.get_user_active_party(requester_id).await {
        Ok(party_id) => party_id,
        Err(e) => {
            error!("User {} is not in a party: {}", requester_id, e);
            return HttpResponse::NotAcceptable().finish();
        }
    };

    let room_id = party_id.to_string();
    let conn_id = Uuid::new_v4().to_string();

    info!(
        "WebSocket connection established for user {} in party {}",
        requester_id, room_id
    );

    {
        let mut m = store.conn_map.write().unwrap();
        m.insert(conn_id.clone(), requester_id);
    }

    let (_response, session, mut msg_stream) = match actix_ws::handle(&req, body) {
        Ok(res) => res,
        Err(e) => {
            error!("WebSocket handshake failed: {}", e);
            let _ = store.conn_map.write().unwrap().remove(&conn_id);
            return HttpResponse::BadRequest().finish();
        }
    };

    let get_broadcaster = Broadcaster::handle(&store.broadcaster, &room_id, &conn_id, session);
    let conn_map = store.conn_map.clone();

    spawn(async move {
        while let Some(result) = msg_stream.recv().await {
            match result {
                Ok(msg) => match msg {
                    Message::Text(msg) => {
                        let _message: ClientMessage = match serde_json::from_str(&msg) {
                            Ok(m) => m,
                            Err(e) => {
                                error!("Failed to parse client message: {}", e);
                                continue;
                            }
                        };
                        let mut w = get_broadcaster.write().unwrap();
                        w.room(&room_id).broadcast(msg.to_string()).await;
                    }
                    Message::Close(reason) => {
                        debug!("WebSocket connection {} closed: {:?}", conn_id, reason);
                        let _ = conn_map.write().unwrap().remove(&conn_id);
                        let _ = get_broadcaster
                            .write()
                            .unwrap()
                            .room(&room_id)
                            .close_conn(reason, &conn_id)
                            .await;
                        break;
                    }
                    Message::Pong(bytes) => {
                        let mut w = get_broadcaster.write().unwrap();
                        w.room(&room_id).ping(bytes.to_vec()).await;
                    }
                    Message::Ping(bytes) => {
                        let mut w = get_broadcaster.write().unwrap();
                        w.room(&room_id).pong(bytes.to_vec()).await;
                    }
                    Message::Continuation(item) => {
                        let mut w = get_broadcaster.write().unwrap();
                        let room = w.room(&room_id);
                        let msg = format!(r"hello, your continuation message: {:#?}", item);
                        let start = Item::FirstBinary(msg.into());
                        let _ = room.continuation(start).await;
                        let cont_cont = Item::Continue(r"continue".into());
                        let _ = room.continuation(cont_cont).await;
                        let last = Item::Last(r"end".into());
                        let _ = room.continuation(last).await;
                    }
                    _ => {}
                },
                Err(e) => {
                    error!("WebSocket stream error for connection {}: {}", conn_id, e);
                    // Clean up connection on stream error
                    let _ = conn_map.write().unwrap().remove(&conn_id);
                    break;
                }
            }
        }
        // Ensure cleanup on stream end (normal or error)
        let _ = conn_map.write().unwrap().remove(&conn_id);
        debug!("WebSocket connection {} cleaned up", conn_id);
    });

    HttpResponse::Ok().finish()
}
