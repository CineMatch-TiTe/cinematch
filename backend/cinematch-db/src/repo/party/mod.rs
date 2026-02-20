//! Party-related database operations.

pub mod models;
mod ops;

pub use models::*;
// ops adds impl methods to Database (no explicit re-exports needed)
