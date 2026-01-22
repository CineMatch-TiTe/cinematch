use actix_web::{middleware::Logger, web, App, HttpServer};
use utoipa::{OpenApi};
use utoipa_swagger_ui::{{SwaggerUi, Url}};
use std::net::Ipv4Addr;

use log::error;
use ed25519_compact::KeyPair;

use actix_jwt_auth_middleware::use_jwt::UseJWTOnApp;
use actix_jwt_auth_middleware::{Authority, TokenSigner};
use jwt_compact::alg::Ed25519;

// Database
use cinematch_db::Database;
use cinematch_party_api::{configure as configure_party_routes, PartyApiDoc};
use cinematch_user_api::{configure as configure_user_routes, UserApiDoc};
// use cinematch_recommendation_engine::configure_recommendation_routes;

use cinematch_common::UserClaims;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let db_loc = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://user:password@localhost/cinematch".to_string());
    log::info!("Connecting to database at {}", db_loc); 

    let db_pool = Database::new(&db_loc).expect("Failed to initialize database");

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


    let KeyPair {
        pk: public_key,
        sk: secret_key,
    } = KeyPair::generate();

    // Start the server with single IPv4 binding
    let server = HttpServer::new(move || {

        let authority = Authority::<UserClaims, Ed25519, _, _>::new()
            .refresh_authorizer(|| async move { Ok(()) })
            .token_signer(Some(
                TokenSigner::new()
                    .signing_key(secret_key.clone())
                    .algorithm(Ed25519)
                    .build()
                    .expect("Failed to build token signer"),
            ))
            .verifying_key(public_key)
            .build()
            .expect("Failed to build authority");

        App::new()
            .app_data(data.clone())
            .wrap(Logger::default())
            .use_jwt(authority.clone(), web::scope("/api/party")) // claims available at all routes
            .use_jwt(authority, web::scope("/api/user/rename"))
            .configure(configure_party_routes)
            .configure(configure_user_routes)
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
            ]))
    });

    server.bind((host, port))?.run().await
}

