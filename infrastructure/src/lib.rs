//! Infrastructure Layer
//!
//! Implements adapters for external systems (database, HTTP).

pub mod database;
pub mod repositories;

pub use database::Database;
pub use repositories::*;
