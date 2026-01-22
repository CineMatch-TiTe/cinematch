//! Party API routing configuration
use actix_web::web;

use crate::handlers;

/// Configure all party API routes under /api/party
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/user")
            // User authentication
            .route("/login/guest", web::post().to(handlers::login_guest))
            // User management
            .route("/rename", web::patch().to(handlers::rename_user)),
    );
}
