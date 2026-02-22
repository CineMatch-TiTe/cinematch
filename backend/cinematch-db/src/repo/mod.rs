//! Repository layer - domain-oriented database operations.
//!
//! Each module provides models and CRUD operations for its domain:
//! - `user` - Users, external accounts, preferences
//! - `party` - Parties, members, codes, state machine
//! - `movie` - Movies, genres, directors, cast
//! - `vote` - Votes and shown movies
//! - `taste` - User/party taste preferences (shared across user and party contexts)

pub mod movie;
pub mod party;
pub mod schedules;
pub mod taste;
pub mod user;
pub mod vote;

// Modules are accessible via `repo::user`, `repo::party`, etc.
// Use `cinematch_db::models` for unified model re-exports.
