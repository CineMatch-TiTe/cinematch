//! Recommendation engine for Cinematch.
//!
//! This crate provides the core algorithms for movie recommendations using Qdrant vector search.
//! It is organized into several modules:
//! - `engine`: Core recommendation logic (standard, reviews-based, and pool-based).
//! - `ballots`: Logic for building voting ballots for parties.
//! - `utils`: Qdrant filter builders and other utilities.

pub mod ballots;
pub mod engine;
pub mod utils;

// Re-export core functions for backward compatibility and convenience
pub use engine::pool::recommend_from_pool;
pub use engine::reviews::recommend_from_reviews;
pub use engine::standard::recommend_movies;

// Re-export for ABI compatibility (maintaining the typo for now if needed, but better to fix it)
/// Typo-fix alias for `recommend_from_reviews`
pub use engine::reviews::recommend_from_reviews as recommed_movies_from_reviews;

pub use ballots::v1::build_voting_ballots_for_party;
pub use ballots::v2::build_round2_ballots_for_party;

pub use cinematch_db::DbError;
