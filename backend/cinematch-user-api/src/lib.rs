//! CineMatch User API
//!
//! This crate provides the HTTP API endpoints for user management.
//! It handles user authentication, guest user creation, and profile management.
//!
//! ## Authentication Flow
//!
//! ### For New Users (No Account)
//! 1. Call `POST /api/user/login/guest` to create a temporary guest user
//! 2. Use the returned `user_id` for subsequent API calls
//! 3. Guest users are ephemeral - they exist only for the duration of a party
//!
//! ### For Persistent Users (With External Account)
//! 1. Link an OAuth provider (Google, GitHub, Discord) - to be implemented
//! 2. Receive a JWT token containing the `user_id`
//! 3. Include the JWT token in the `Authorization` header for authenticated endpoints
//!
//! ### Protected Endpoints
//! - `PATCH /api/user/rename` - requires JWT authentication, user can only rename their own account

pub mod handlers;
pub mod models;
pub mod routes;

pub use handlers::AppState;
pub use routes::configure;

// Re-export models for convenience
pub use models::*;

use utoipa::OpenApi;

/// OpenAPI documentation for the User API
#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::login_guest,
        handlers::rename_user,
    ),
    components(
        schemas(
            GuestLoginResponse,
            UserResponse,
            RenameUserRequest,
            ErrorResponse,
        )
    ),
    tags(
        (name = "user", description = "User management and authentication endpoints")
    )
)]
pub struct UserApiDoc;