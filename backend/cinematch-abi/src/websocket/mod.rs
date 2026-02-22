//! Actor-based WebSocket registry for real-time party communication.
//!
//! Uses Actix actors for connection management with heartbeat/timeout handling.

use actix::{Message, Recipient};
use log::trace;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

pub mod session;

/// Message sent to the WebSocket session actor
#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

pub type WebsocketRegistry = actix_web::web::Data<std::sync::Arc<WsRegistry>>;

/// Global registry of active WebSocket sessions, keyed by user_id.
/// Each user can have multiple connections (e.g., multiple browser tabs).
pub struct WsRegistry {
    /// Maps user_id to a list of active connection recipients
    sessions: RwLock<HashMap<Uuid, Vec<Recipient<WsMessage>>>>,
}

impl Default for WsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl WsRegistry {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }

    /// Add a session for a user
    pub fn add_session(&self, user_id: Uuid, recipient: Recipient<WsMessage>) {
        let mut sessions = self
            .sessions
            .write()
            .expect("WsRegistry: failed to acquire write lock");
        sessions.entry(user_id).or_default().push(recipient);
        trace!("WsRegistry: added session for user {}", user_id);
    }

    /// Remove a session for a user
    pub fn remove_session(&self, user_id: &Uuid, recipient: &Recipient<WsMessage>) {
        let mut sessions = self
            .sessions
            .write()
            .expect("WsRegistry: failed to acquire write lock");
        if let Some(user_sessions) = sessions.get_mut(user_id) {
            user_sessions.retain(|r| r != recipient);
            if user_sessions.is_empty() {
                sessions.remove(user_id);
            }
        }
        trace!("WsRegistry: removed session for user {}", user_id);
    }

    /// Send a message to specific users
    pub fn send_to_users<T: Serialize>(&self, user_ids: &[Uuid], msg: &T) {
        let msg_str = match serde_json::to_string(msg) {
            Ok(s) => s,
            Err(e) => {
                log::error!("WsRegistry: failed to serialize message: {}", e);
                return;
            }
        };
        let sessions = self
            .sessions
            .read()
            .expect("WsRegistry: failed to acquire read lock");
        for user_id in user_ids {
            if let Some(user_sessions) = sessions.get(user_id) {
                for session in user_sessions {
                    session.do_send(WsMessage(msg_str.clone()));
                }
            }
        }
    }
}
