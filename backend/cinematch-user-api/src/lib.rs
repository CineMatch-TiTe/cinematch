//! CineMatch User API
//!
//! This crate provides the HTTP API endpoints for user management.
//! It handles user authentication, guest user creation, and profile management.
//!
//! ## Authentication Flow
//!
//! ### For New Users (No Account)
//! 1. Call `POST /auth/login` to create a temporary guest user
//! 2. Use the returned `user_id` for subsequent API calls
//! 3. Guest users are ephemeral - they exist only for the duration of a party
//!
//! ### For Persistent Users (With External Account)
//! 1. Link an OAuth provider (Google, GitHub, Discord) - to be implemented
//! 2. Receive a JWT token containing the `user_id`
//! 3. Include the JWT token in the `Authorization` header for authenticated endpoints
//!
//! ### Authentication Management
//! - `POST /auth/login` - Create a temporary guest user (no auth required)
//! - `POST /auth/logout` - Clear authentication cookies (no auth required)
//!
//! ### Protected Endpoints
//! - `GET /api/user` - Get current user info and token expiry (requires JWT)
//! - `PATCH /api/user/rename/{user_id}` - Rename user account (requires JWT, can only rename own account)

pub mod handlers;
pub mod models;
pub mod routes;

pub use handlers::AppState;
pub use routes::configure;

// Re-export models for convenience
pub use models::*;

use cinematch_common::ErrorResponse;

use utoipa::OpenApi;

/// OpenAPI documentation for the User API
#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::login_guest,
        handlers::logout_user,
        handlers::get_current_user,
        handlers::rename_user,
    ),
    components(
        schemas(
            GuestLoginResponse,
            UserResponse,
            CurrentUserResponse,
            RenameUserRequest,
            ErrorResponse,
        )
    ),
    tags(
        (name = "user", description = "User management and authentication endpoints")
    )
)]
pub struct UserApiDoc;