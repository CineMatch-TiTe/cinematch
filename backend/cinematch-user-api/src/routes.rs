//! User API routing configuration
use actix_web::web;

use crate::handlers;

/// Configure all user API routes under /user (protected routes, already under /api scope)
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/user")
            // Guest login
            .route("/login/guest", web::post().to(handlers::login_guest))
            // Logout
            .route("/logout", web::post().to(handlers::logout_user))
            // User info and profile
            .route("", web::get().to(handlers::get_current_user))
            // User management
            .route("/rename/{user_id}", web::patch().to(handlers::rename_user)),
    );
}
