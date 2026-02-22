use uuid::Uuid;

pub const NAME_MIN_LENGTH: usize = 3;
pub const NAME_MAX_LENGTH: usize = 32;

/// Trait for types that have a unique identifier.
pub trait HasId {
    fn id(&self) -> Uuid;
}

// Implement HasId for Uuid itself (identity)
impl HasId for Uuid {
    fn id(&self) -> Uuid {
        *self
    }
}

// Blanket impl for references to types that implement HasId
impl<T: HasId> HasId for &T {
    fn id(&self) -> Uuid {
        (*self).id()
    }
}

pub mod config;
pub mod models;

// Re-export models for easier access
pub use config::Config;
pub use models::*;
