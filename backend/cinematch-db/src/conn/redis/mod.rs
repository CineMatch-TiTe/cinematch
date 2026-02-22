//! Redis caching layer for cinematch-db.
//!
//! Provides generic cache helpers with automatic serialization/deserialization.

pub mod cache;

pub use cache::*;
