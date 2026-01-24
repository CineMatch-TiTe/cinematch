use actix_web::web;
use utoipa::OpenApi;

mod party;
mod user;
mod websocket;

pub mod routes;

use cinematch_common::vote_store;

// Re-export the database type for convenience
pub use cinematch_db::Database;
pub type AppState = web::Data<Database>;
pub type VoteStore = web::Data<vote_store::VoteStore>;

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "user", description = "User management endpoints."),
        (name = "party", description = "Party management endpoints."),
        (name = "websocket", description = "WebSocket endpoints.")
    )
)]
pub struct ApiDoc;

// Optionally, add a custom modifier for security, etc.
// struct SecurityAddon;
// impl Modify for SecurityAddon { ... }

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
