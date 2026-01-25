//! User API routing configuration
use actix_web::web;
use utoipa_actix_web::service_config::ServiceConfig;

use crate::{party, user, websocket, movie};

/// Configure all user API routes under /user (protected routes, already under /api scope)
///
///

pub fn configure_user() -> impl FnOnce(&mut ServiceConfig) {
    |cfg: &mut ServiceConfig| {
        cfg.service(user::handlers::login_guest)
            .service(user::handlers::logout_user)
            .service(user::handlers::get_current_user)
            .service(user::handlers::rename_user)
            .service(user::handlers::update_taste);
    }
}

pub fn configure_party() -> impl FnOnce(&mut ServiceConfig) {
    |cfg: &mut ServiceConfig| {
        cfg.service(party::crud::create_party)
            .service(party::crud::get_party)
            .service(party::user_ops::join_party)
            .service(party::user_ops::leave_party)
            .service(party::user_ops::get_party_members)
            .service(party::leader_ops::kick_member)
            .service(party::leader_ops::transfer_leadership)
            .service(party::user_ops::set_ready)
            .service(party::leader_ops::advance_phase)
            .service(party::leader_ops::start_new_round)
            .service(party::leader_ops::disband_party)
            .service(party::user_ops::vote_movie)
            .service(party::crud::get_my_party);
    }
}

pub fn configure_movies() -> impl FnOnce(&mut ServiceConfig) {
    |cfg: &mut ServiceConfig| {
        cfg.service(movie::handlers::get_movie)
            .service(movie::handlers::get_genres);
    }
}

pub fn configure_websocket() -> impl FnOnce(&mut ServiceConfig) {
    |cfg: &mut ServiceConfig| {
        cfg.service(websocket::websocket_controller);
    }
}
