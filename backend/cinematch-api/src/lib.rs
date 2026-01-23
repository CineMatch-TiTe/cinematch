use actix_web::web;
use utoipa::OpenApi;

// Import your API modules here
mod party;
mod user;
mod websocket;

pub mod routes;

//mod routes;

// With actix_extras enabled, utoipa can auto-derive OpenAPI from actix-web handlers and macros.
// Just annotate your handlers with #[utoipa::path(...)] and use actix-web macros like #[get], #[post], etc.
// Tags can be set in the handler attributes or in the #[openapi(...)] macro.

// Re-export the database type for convenience
pub use cinematch_db::Database;
pub type AppState = web::Data<Database>;

#[derive(OpenApi)]
#[openapi(
    // No need to list paths/components if actix_extras is enabled and handlers are annotated
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
