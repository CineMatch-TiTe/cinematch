//! CineMatch Party API
//!
//! This crate provides the HTTP API endpoints for party management.
//! It handles party creation, joining, member management, and state transitions.

pub mod handlers;
pub mod models;
pub mod routes;

pub use handlers::AppState;
pub use routes::configure;

// Re-export models for convenience
pub use models::*;

use cinematch_common::ErrorResponse;

use utoipa::OpenApi;

/// OpenAPI documentation for the Party API
#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::create_party,
        handlers::get_party,
        handlers::join_party,
        handlers::leave_party,
        handlers::get_party_members,
        handlers::kick_member,
        handlers::transfer_leadership,
        handlers::set_ready,
        handlers::advance_phase,
        handlers::start_new_round,
        handlers::disband_party,
    ),
    components(
        schemas(
            CreatePartyResponse,
            PartyResponse,
            PartyStateDto,
            MemberInfo,
            PartyMembersResponse,
            KickMemberRequest,
            TransferLeadershipRequest,
            ReadyStateResponse,
            PhaseAdvanceResponse,
            NewRoundResponse,
            ErrorResponse,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "party", description = "Party management endpoints")
    )
)]
pub struct PartyApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );

        openapi.security = Some(vec![utoipa::openapi::security::SecurityRequirement::new(
            "bearer_auth",
            Vec::<String>::new(),
        )]);
    }
}