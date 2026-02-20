//! User API routing configuration
use utoipa_actix_web::service_config::ServiceConfig;

use crate::{auth, movie, party, recommendation, user, websocket};

/// Configure auth API routes under /auth
pub fn configure_auth() -> impl FnOnce(&mut ServiceConfig) {
    |cfg: &mut ServiceConfig| {
        cfg.service(auth::handlers::login_guest)
            .service(auth::handlers::logout_user);
    }
}

/// Configure all user API routes under /user
pub fn configure_user() -> impl FnOnce(&mut ServiceConfig) {
    |cfg: &mut ServiceConfig| {
        cfg.service(user::handlers::get_current_user)
            .service(user::handlers::rename_user)
            .service(user::handlers::update_taste)
            .service(user::pref::get_user_pref)
            .service(user::pref::edit_user_pref);
    }
}

pub fn configure_party() -> impl FnOnce(&mut ServiceConfig) {
    |cfg: &mut ServiceConfig| {
        cfg.service(party::crud::create_party)
            .service(party::crud::get_party)
            .service(party::picks::get_picks)
            .service(party::picks::pick_movie)
            .service(party::picks::delete_pick)
            .service(party::user_ops::join_party)
            .service(party::user_ops::leave_party)
            .service(party::user_ops::get_party_members)
            .service(party::leader_ops::kick_member)
            .service(party::leader_ops::transfer_leadership)
            .service(party::user_ops::set_ready)
            .service(party::leader_ops::advance_phase)
            .service(party::leader_ops::disband_party)
            .service(party::votes::get_vote)
            .service(party::votes::vote_movie);
    }
}

pub fn configure_movies() -> impl FnOnce(&mut ServiceConfig) {
    |cfg: &mut ServiceConfig| {
        cfg.service(movie::handlers::get_movie)
            .service(movie::handlers::get_genres)
            .service(movie::handlers::search);
    }
}

pub fn configure_recommendation() -> impl FnOnce(&mut ServiceConfig) {
    |cfg: &mut ServiceConfig| {
        cfg.service(recommendation::handlers::get_recommendations);
    }
}

pub fn configure_websocket() -> impl FnOnce(&mut ServiceConfig) {
    |cfg: &mut ServiceConfig| {
        cfg.service(websocket::websocket_controller);
    }
}
