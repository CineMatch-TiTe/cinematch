use actix_web::{
    App, HttpServer,
    middleware::{Compress, Logger},
    web,
};

use actix_cors::Cors;
use actix_identity::IdentityMiddleware;
use actix_session::{SessionMiddleware, storage::RedisSessionStore};
use actix_web::cookie::Key;
use actix_web::http;

use utoipa::OpenApi;

use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::Duration;

use log::error;

use utoipa_actix_web::AppExt;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use utoipa_swagger_ui::SwaggerUi;

// Database and application state
use cinematch_abi::AppState;
use cinematch_abi::scheduler::{Scheduler, reschedule_timeouts_on_startup};
use cinematch_abi::websocket::WsRegistry;
use cinematch_db::Database;
use cinematch_server::ApiDoc;

// use cinematch_recommendation_engine::configure_recommendation_routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let db_loc = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://user:password@localhost/cinematch".to_string());
    log::info!("Connecting to database at {}", db_loc);

    let redis_loc =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    log::info!("Connecting to Redis at {}", redis_loc);

    let vector_loc =
        std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6333".to_string());
    log::info!("Connecting to Qdrant at {}", vector_loc);

    let db_pool =
        Database::new(&db_loc, &redis_loc, &vector_loc).expect("Failed to initialize databases");

    let cfg = deadpool_redis::Config::from_url(redis_loc.clone());
    let pool = cfg
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .unwrap_or_else(|_| panic!("Failed to create Redis pool using {}", redis_loc));
    let redis_store = RedisSessionStore::new_pooled(pool).await.unwrap();

    // Run database migrations
    if let Err(e) = db_pool.run_migrations(&db_loc).await {
        error!("Failed to run database migrations: {}", e);
        return Err(std::io::Error::other("migration error"));
    }

    // Create scheduler and websocket registry
    let db = Arc::new(db_pool);
    let scheduler = Arc::new(Scheduler::new());
    let ws_registry = Arc::new(WsRegistry::new());

    // Create the unified application state
    let app_state = AppState {
        db: Arc::clone(&db),
        ws_registry: Arc::clone(&ws_registry),
        scheduler: Arc::clone(&scheduler),
    };

    // Reschedule any active timeouts from before restart
    reschedule_timeouts_on_startup(&scheduler, app_state.clone()).await;
    let data = web::Data::new(app_state);

    let server_host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let server_port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());

    // ...

    let host: Ipv4Addr = server_host.parse().map_err(|err| {
        error!("Invalid SERVER_HOST '{}': {}", server_host, err);
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid SERVER_HOST")
    })?;

    let port: u16 = server_port.parse().map_err(|err| {
        error!("Invalid SERVER_PORT '{}': {}", server_port, err);
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid SERVER_PORT")
    })?;

    log::info!("Starting server bind on [{}]:{}", host, port);

    // try read from env, else random
    // TODO! we shouldnt use as bytes since this can crash if entropy (byte count is too low)
    let secret_key = match std::env::var("SECRET_TOKEN") {
        Ok(key_str) => {
            log::info!("Using JWT secret key from environment variable");
            Key::from(key_str.as_bytes())
        }
        Err(_) => {
            log::warn!("SECRET_TOKEN not set, generating random key for debug build");
            Key::generate()
        }
    };

    // Identity session limits (10x previous: 10 days visit deadline, 300 days login deadline)
    let deadline_expiration = Duration::from_secs(10 * 24 * 60 * 60);
    let last_login_duration = Duration::from_secs(300 * 24 * 60 * 60);

    let server = HttpServer::new(move || {
        let identity_mw = IdentityMiddleware::builder()
            .visit_deadline(Some(deadline_expiration))
            .login_deadline(Some(last_login_duration))
            .build();

        App::new()
            .into_utoipa_app()
            .openapi(ApiDoc::openapi())
            .map(|app| app.wrap(Compress::default()))
            .map(|app| app.wrap(identity_mw))
            .map(|app| {
                app.wrap(SessionMiddleware::new(
                    redis_store.clone(),
                    secret_key.clone(),
                ))
            })
            .map(|app| {
                app.wrap(
                    Cors::default()
                        .allowed_origin("http://localhost:3000") // Your frontend URL (e.g., React/Vite dev server)
                        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH"])
                        .allowed_headers(vec![
                            http::header::AUTHORIZATION,
                            http::header::ACCEPT,
                            http::header::CONTENT_TYPE,
                            http::header::COOKIE,
                        ])
                        .max_age(3600),
                )
            })
            .map(|app| app.wrap(Logger::default()))
            .app_data(data.clone()) // unified application state
            // /api/auth, /api/user, /api/party, /api/ws, etc
            .service(
                utoipa_actix_web::scope("/api/auth")
                    .configure(cinematch_server::routes::configure_auth()),
            )
            .service(
                utoipa_actix_web::scope("/api/user")
                    .configure(cinematch_server::routes::configure_user()),
            )
            .service(
                utoipa_actix_web::scope("/api/party")
                    .configure(cinematch_server::routes::configure_party()),
            )
            .service(
                utoipa_actix_web::scope("/api/movie")
                    .configure(cinematch_server::routes::configure_movies()),
            )
            .service(
                utoipa_actix_web::scope("/api/recommend")
                    .configure(cinematch_server::routes::configure_recommendation()),
            )
            .service(
                utoipa_actix_web::scope("/api/ws")
                    .configure(cinematch_server::routes::configure_websocket()),
            )
            .openapi_service(|api| Redoc::with_url("/redoc", api))
            .openapi_service(|api| {
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", api)
            })
            // There is no need to create RapiDoc::with_openapi because the OpenApi is served
            // via SwaggerUi. Instead we only make rapidoc to point to the existing doc.
            //
            // If we wanted to serve the schema, the following would work:
            // .openapi_service(|api| RapiDoc::with_openapi("/api-docs/openapi2.json", api).path("/rapidoc"))
            .map(|app| app.service(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc")))
            .openapi_service(|api| Scalar::with_url("/scalar", api))
            .into_app()
    });

    server.bind((host, port))?.run().await
}
