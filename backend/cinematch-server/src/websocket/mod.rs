use actix_identity::Identity;
use actix_web::{rt::spawn, web::{get, Data, Payload, Query}, App, HttpRequest, HttpResponse, HttpServer, Responder};

use actix_wsb::Broadcaster;
use uuid::Uuid;
use std::sync::{Arc, RwLock};
use actix_ws::{Item, Message};

use log::error;

use cinematch_common::extract_user_id;

pub mod models;

use crate::websocket::models::{ClientMessage, ServerMessage};
use crate::Database;

use cinematch_common::ErrorResponse;

// returns 403 if not authorized


use utoipa::OpenApi;

/// OpenAPI documentation for the WebSocket API
#[derive(OpenApi)]
#[openapi(
    paths(
        websocket_controller
    ),
    components(
        schemas(
            ClientMessage,
            ServerMessage,
            ErrorResponse,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "ws", description = "WebSocket endpoints")
    )
)]
pub struct WebsocketApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );

        openapi.security = Some(vec![utoipa::openapi::security::SecurityRequirement::new(
            "bearer_auth",
            Vec::<String>::new(),
        )]);
    }
}


pub async fn send_message_to_user(broadcaster: &Arc<RwLock<Broadcaster>>, room_id: String, user_id: &str, message: &ServerMessage) {
    let msg_text = match serde_json::to_string(message) {
        Ok(text) => text,
        Err(e) => {
            error!("Failed to serialize server message: {}", e);
            return;
        }
    };

    let mut write_broadcaster = broadcaster.write().unwrap();

    // verify room
    if write_broadcaster.check_room(&room_id).is_none() {
        error!("Room {} does not exist in broadcaster", room_id);
        return;
    }

    write_broadcaster.room(&room_id).broadcast_if(msg_text, |connection| connection.id == user_id).await;
}

pub async fn send_message_to_party(broadcaster: &Arc<RwLock<Broadcaster>>, room_id: String, message: &ServerMessage, ignore_users: Option<&Vec<Uuid>>) {
    let msg_text = match serde_json::to_string(message) {
        Ok(text) => text,
        Err(e) => {
            error!("Failed to serialize server message: {}", e);
            return;
        }
    };

    let mut write_broadcaster = broadcaster.write().unwrap();

    // verify room
    if write_broadcaster.check_room(&room_id).is_none() {
        error!("Room {} does not exist in broadcaster", room_id);
        return;
    }

    //write_broadcaster.room(&room_id).broadcast(msg_text).await;
    if let Some(ignore_list) = ignore_users {
        write_broadcaster.room(&room_id).broadcast_if(msg_text, |connection| {
            let conn_uuid = match Uuid::parse_str(&connection.id) {
                Ok(uuid) => uuid,
                Err(_) => return true, // if parsing fails, do not ignore
            };
            !ignore_list.contains(&conn_uuid)
        }).await;
    } else {
        write_broadcaster.room(&room_id).broadcast(msg_text).await;
    }
}

#[utoipa::path(
    get,
    path = "/api/ws",
    responses(
        (status = 200, description = "WebSocket connection established"),
        (status = 401, description = "Unauthorized"),
        (status = 406, description = "Not Acceptable - user is not in a party"),
        (status = 400, description = "WebSocket handshake failed: WebSocket upgrade is expected"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tags = ["ws"],
    operation_id = "websocket_controller"
)]
pub async fn websocket_controller(req: HttpRequest, body: Payload, db: Data<Database>, broadcaster: Data<Arc<RwLock<Broadcaster>>>, user: Option<Identity>) -> HttpResponse {

    // validates user is authenticated
    let requester_id = extract_user_id!(user);

    let party_id = match db.get_user_active_party(requester_id).await {
        Ok(party_id) => party_id,
        Err(e) => {
            error!("User {} is not in a party: {}", requester_id, e);
            return HttpResponse::NotAcceptable().finish();
        }
    };

    let user_id = requester_id.to_string();
    let room_id = party_id.to_string();

    let (_response, session, mut msg_stream) = match actix_ws::handle(&req, body) {
        Ok(res) => res,
        Err(e) => {
            error!("WebSocket handshake failed: {}", e);
            return HttpResponse::BadRequest().finish();
        }
    };

    let get_broadcaster = Broadcaster::handle(&broadcaster, &room_id, &user_id, session);

    // // ".each_room_immut()" example
    // get_broadcaster.read().unwrap().each_room_immut(|room| println!("Hello to room {}!", room.id));
    
    // // ".each_room()" example
    // let mut num = 0;
    
    // get_broadcaster.read().unwrap().each_room(|room| {
    //     for _ in room.connectors.iter() {
    //         num = num + 1
    //     }
    // });
    
    spawn(async move {
        while let Some(Ok(msg)) = msg_stream.recv().await {
            match msg {
                Message::Text(msg) => {

                    // handle incoming messages

                    let message: ClientMessage = match serde_json::from_str(&msg) {
                        Ok(m) => m,
                        Err(e) => {
                            error!("Failed to parse client message: {}", e);
                            continue;
                        }
                    };

                    let mut write_broadcaster = get_broadcaster.write().unwrap();

                    write_broadcaster.room(&room_id).broadcast(msg.to_string()).await;
                },
                 Message::Close(reason) => {
                    // warning, that closes and removes all the connections but not removes the room: 
                    //let _ = get_broadcaster.write().unwrap().room(room_id.clone()).close(reason).await;
                    
                    // if you want to remove a room with removing all the connections, use this instead:
                    // let _ = get_broadcaster.write().unwrap().remove_room(room_id.clone()).await;

                    // you can conditionally close connections:
                    //let _ = get_broadcaster.write().unwrap().room(room_id.clone()).close_if(reason, |conn| conn.id == query.id.clone().unwrap()).await;
                    
                    // or, use the new api:

                    let _ = get_broadcaster.write().unwrap().room(&room_id).close_conn(reason, &user_id).await;
                    break;
                 },
                 Message::Pong(bytes) => {
                    let mut write_broadcaster = get_broadcaster.write().unwrap();

                    write_broadcaster.room(&room_id).ping(bytes.to_vec()).await;
                 },
                 Message::Ping(bytes) => {
                    let mut write_broadcaster = get_broadcaster.write().unwrap();

                    write_broadcaster.room(&room_id).pong(bytes.to_vec()).await;
                 },
                 Message::Continuation(item) => {
                    let mut write_broadcaster = get_broadcaster.write().unwrap();

                    let room = write_broadcaster.room(&room_id);

                    let msg = format!(r"hello, your continuation message: {:#?}", item);
                    
                    let start = Item::FirstBinary(msg.into());
                    let _ = room.continuation(start).await;

                    let cont_cont = Item::Continue(r"continue".into());
                    let _ = room.continuation(cont_cont).await;

                    let last = Item::Last(r"end".into());
                    let _ = room.continuation(last);

                 }
                 _ => ()
            }
        }
    });
    
    HttpResponse::Ok().finish()
}
