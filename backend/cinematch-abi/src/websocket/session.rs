//! WebSocket session actor with heartbeat handling.

use actix::{Actor, ActorContext, AsyncContext, Handler, StreamHandler};
use actix_web_actors::ws;
use log::{debug, trace};
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

use super::{WsMessage, WsRegistry};
use cinematch_common::models::websocket::ClientMessage;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// WebSocket session actor - one per connection
pub struct WsSession {
    pub user_id: Uuid,
    pub registry: Arc<WsRegistry>,
    pub last_heartbeat: Instant,
}

impl WsSession {
    pub fn new(user_id: Uuid, registry: Arc<WsRegistry>) -> Self {
        Self {
            user_id,
            registry,
            last_heartbeat: Instant::now(),
        }
    }

    /// Start heartbeat process
    fn start_heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.last_heartbeat) > CLIENT_TIMEOUT {
                debug!(
                    "WsSession: heartbeat failed for user {}, disconnecting",
                    act.user_id
                );
                ctx.stop();
                return;
            }

            // Send timestamp as ping payload for RTT measurement
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            ctx.ping(&now.to_be_bytes());
        });
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_heartbeat(ctx);
        self.registry
            .add_session(self.user_id, ctx.address().recipient());
        debug!("WsSession: started for user {}", self.user_id);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> actix::Running {
        self.registry
            .remove_session(&self.user_id, &ctx.address().recipient());
        debug!("WsSession: stopping for user {}", self.user_id);
        actix::Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                log::error!("WsSession: protocol error for user {}: {}", self.user_id, e);
                ctx.stop();
                return;
            }
        };

        match msg {
            ws::Message::Ping(bytes) => {
                self.last_heartbeat = Instant::now();
                ctx.pong(&bytes);
            }
            ws::Message::Pong(_) => {
                self.last_heartbeat = Instant::now();
            }
            ws::Message::Text(text) => {
                // Parse and handle client messages
                if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                    trace!(
                        "WsSession: received message from user {}: {:?}",
                        self.user_id, client_msg
                    );
                    // Client messages are handled by HTTP endpoints, not WebSocket
                    // WebSocket is primarily for server→client push notifications
                }
            }
            ws::Message::Binary(_) => {
                // Binary messages not supported
            }
            ws::Message::Close(reason) => {
                debug!(
                    "WsSession: close requested for user {}: {:?}",
                    self.user_id, reason
                );
                ctx.close(reason);
                ctx.stop();
            }
            _ => (),
        }
    }
}

/// Handle messages sent from server to client
impl Handler<WsMessage> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}
