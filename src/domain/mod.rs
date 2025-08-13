//! Domain layer for network infrastructure

pub mod aggregates;
pub mod commands;
pub mod configuration;
pub mod errors;
pub mod events;
pub mod state_machines;
pub mod value_objects;
#[cfg(feature = "workflows")]
pub mod workflows;

// Re-export commonly used types
pub use commands::*;
pub use errors::*;
pub use events::*;
pub use value_objects::*;