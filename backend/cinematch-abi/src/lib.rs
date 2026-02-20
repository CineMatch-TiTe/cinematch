// Domain module with lazy-loading types and extension traits
pub mod domain;

// Prelude for convenient imports
pub mod prelude;

pub mod scheduler;
pub mod websocket;

use actix_web::{FromRequest, HttpRequest, dev::Payload, web};
use cinematch_common::models::websocket::ServerMessage;
use cinematch_db::{AppContext, Database};
use std::sync::Arc;
use uuid::Uuid;

use crate::scheduler::Scheduler;
use crate::websocket::WsRegistry;

#[derive(Clone)]
/// Application state shared across handlers and modules. Internally uses arc so cloning is cheap.
pub struct AppState {
    pub db: Arc<Database>,
    pub ws_registry: Arc<WsRegistry>,
    pub scheduler: Arc<Scheduler>,
}

// AppState no longer derefs to Database to enforce domain logic usage

// Allow using db.as_ref() to get &Database
impl AsRef<Database> for AppState {
    fn as_ref(&self) -> &Database {
        &self.db
    }
}

// Implement FromRequest to allow using AppState directly as a handler parameter
// This extracts from web::Data<AppState> and clones (cheap due to Arc internals)
impl FromRequest for AppState {
    type Error = actix_web::Error;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        match req.app_data::<web::Data<AppState>>() {
            Some(data) => std::future::ready(Ok(data.get_ref().clone())),
            None => std::future::ready(Err(actix_web::error::ErrorInternalServerError(
                "AppState not configured in app data",
            ))),
        }
    }
}

impl AppContext for AppState {
    fn db(&self) -> &Arc<Database> {
        &self.db
    }

    fn broadcast_party(&self, party_id: Uuid, msg: &ServerMessage, exclude: Option<Uuid>) {
        let db = self.db.clone();
        let ws = self.ws_registry.clone();
        let msg = msg.clone();

        tokio::spawn(async move {
            match db.get_party_members(party_id).await {
                Ok(members) => {
                    let user_ids: Vec<Uuid> = members
                        .into_iter()
                        .map(|m| m.user_id)
                        .filter(|uid| exclude != Some(*uid))
                        .collect();
                    ws.send_to_users(&user_ids, &msg);
                }
                Err(e) => {
                    log::error!("Failed to broadcast to party {}: {}", party_id, e);
                }
            }
        });
    }

    fn send_users(&self, user_ids: &[Uuid], msg: &ServerMessage) {
        self.ws_registry.send_to_users(user_ids, msg);
    }
}
