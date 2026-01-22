//! Party API routing configuration
use actix_web::web;

use crate::handlers;

/// Configure all party API routes under /api/party
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/party")
            // Party CRUD
            .route("", web::post().to(handlers::create_party))
            .route("/{party_id}", web::get().to(handlers::get_party))
            // Join/Leave
            .route("/join/{code}", web::post().to(handlers::join_party))
            .route("/{party_id}/leave", web::post().to(handlers::leave_party))
            // Members
            .route("/{party_id}/members", web::get().to(handlers::get_party_members))
            .route("/{party_id}/kick", web::post().to(handlers::kick_member))
            // Leadership
            .route(
                "/{party_id}/transfer-leadership",
                web::post().to(handlers::transfer_leadership),
            )
            // Ready State
            .route("/{party_id}/ready", web::post().to(handlers::toggle_ready))
            // Phase Control (Leader Only)
            .route("/{party_id}/advance", web::post().to(handlers::advance_phase))
            .route("/{party_id}/new-round", web::post().to(handlers::start_new_round))
            .route("/{party_id}/disband", web::post().to(handlers::disband_party)),
    );
}
