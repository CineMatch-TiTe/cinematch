use actix_web::web;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::{Modify, OpenApi};

pub mod handler;
mod movie;
mod party;
mod user;
mod websocket;

pub mod routes;

pub use cinematch_db::Database;
pub use websocket::WsStore;

pub type AppState = web::Data<Database>;

/// App data for WebSocket: single store containing broadcaster and conn_id→user_id map.
pub type WsStoreData = web::Data<std::sync::Arc<websocket::WsStore>>;

/// Adds cookie-based auth security scheme. Actix-identity uses the `id` cookie
/// (httpOnly, path=/, samesite=Lax, secure). Obtain via `POST /api/user/login/guest`.
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "cookie_auth",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("id"))),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    tags(
        (name = "user", description = "User management endpoints."),
        (name = "party", description = "Party management endpoints."),
        (name = "movie", description = "Movie retrieval and search endpoints."),
        (name = "websocket", description = "WebSocket endpoints.")
    )
)]
pub struct ApiDoc;

// pub async fn run_api_server() -> std::io::Result<()> {
//     env_logger::init();
//     HttpServer::new(|| {
//         App::new()
//             .wrap(Logger::default())
//             // Register your API routes here
//             // .configure(user::configure)
//             // .configure(party::configure)
//             .openapi_service(ApiDoc::openapi())
//             .service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", ApiDoc::openapi()))
//     })
//     .bind((Ipv4Addr::UNSPECIFIED, 8080))?
//     .run()
//     .await
// }
