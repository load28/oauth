//! Application Layer
//!
//! Contains use case logic (business rules) and port definitions.
//! This layer orchestrates domain entities and defines interfaces
//! for external adapters.

pub mod errors;
pub mod ports;
pub mod services;

// Re-export commonly used types
pub use errors::*;
