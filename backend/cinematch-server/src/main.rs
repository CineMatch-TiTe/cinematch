use actix_web::{middleware::Logger, web, App, HttpServer};

use actix_cors::Cors;
use actix_web::http;
use actix_web::{cookie::Key};
use actix_identity::IdentityMiddleware;
use actix_session::{storage::RedisSessionStore, SessionMiddleware};

use utoipa::{OpenApi};
use utoipa_swagger_ui::{{SwaggerUi, Url}};
use std::net::Ipv4Addr;
use std::time::Duration;

use log::error;


use actix_wsb::Broadcaster;

mod websocket;

// Database
use cinematch_db::Database;
use cinematch_party_api::{configure as configure_party_routes, PartyApiDoc};
use cinematch_user_api::{configure as configure_user_routes, UserApiDoc};
use crate::websocket::{WebsocketApiDoc, websocket_controller};

// use cinematch_recommendation_engine::configure_recommendation_routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let db_loc = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://user:password@localhost/cinematch".to_string());
    log::info!("Connecting to database at {}", db_loc);

    let db_pool = Database::new(&db_loc).expect("Failed to initialize database");

    let redis_loc = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    log::info!("Connecting to redis at {}", redis_loc); 
    let cfg = deadpool_redis::Config::from_url(redis_loc.clone());
    let pool = cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1)).unwrap_or_else(|_| panic!("Failed to create Redis pool using {}", redis_loc));
    let redis_store = RedisSessionStore::new_pooled(pool)
        .await
        .unwrap();

    // Run database migrations
    if let Err(e) = db_pool.run_migrations(&db_loc).await {
        error!("Failed to run database migrations: {}", e);
        return Err(std::io::Error::other("migration error"));
    }

    let data = web::Data::new(db_pool);

    let server_host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let server_port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());

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
    let secret_key = match std::env::var("JWT_SECRET_KEY") {
        Ok(key_str) => {
            log::info!("Using JWT secret key from environment variable");
            Key::from(key_str.as_bytes())
        },
        Err(_) => { 
            log::warn!("JWT_SECRET_KEY not set, generating random key for debug build");
            Key::generate()
        }
    };


    let deadline_expiration = Duration::from_secs(24 * 60 * 60); // last visit this long ago will be logged out
    let last_login_duration = Duration::from_secs(30 * 24 * 60 * 60); // last login this long ago will be logged out

    let rooms = Broadcaster::new();
    let rooms_data = web::Data::new(rooms);

    // Start the server with single IPv4 binding
    let server = HttpServer::new(move || {
        let identity_mw = IdentityMiddleware::builder()
            .visit_deadline(Some(deadline_expiration))
            .login_deadline(Some(last_login_duration))
            .build();

        App::new()
            .wrap(identity_mw)
            .wrap(SessionMiddleware::new(
                 redis_store.clone(),
                 secret_key.clone(),
            ))
            .wrap(Logger::default())
            .wrap(
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

            .app_data(data.clone())
            .app_data(rooms_data.clone())
            .configure(configure_party_routes).configure(configure_user_routes)
            .route("/api/ws", web::get().to(websocket_controller))
            // .configure(configure_recommendation_engine)
            .service(SwaggerUi::new("/swagger-ui/{_:.*}").urls(vec![
                (
                    Url::new("api/party", "/api-docs/party.json"),
                    PartyApiDoc::openapi(),
                ),
                (
                    Url::new("api/users", "/api-docs/users.json"),
                    UserApiDoc::openapi(),
                ),
                (
                    Url::new("api/ws", "/api-docs/ws.json"),
                    WebsocketApiDoc::openapi(),
                ),
            ]))
    });

    server.bind((host, port))?.run().await
}

