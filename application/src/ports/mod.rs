//! Ports (Interfaces)
//!
//! Defines the contracts for the application layer:
//! - Input Ports: Use case interfaces (driven by infrastructure)
//! - Output Ports: Repository interfaces (implemented by infrastructure)

pub mod input;
pub mod output;

// Re-export for convenience
pub use input::*;
pub use output::*;
