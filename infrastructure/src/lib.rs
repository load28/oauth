//! Infrastructure Layer
//!
//! Implements adapters for external systems (database, HTTP).

pub mod app_state;
pub mod config;
pub mod database;
pub mod dtos;
pub mod handlers;
pub mod jwt;
pub mod repositories;

pub use app_state::AppState;
pub use config::Config;
pub use database::Database;
pub use dtos::*;
pub use handlers::*;
pub use jwt::{decode_jwt, encode_jwt, Claims, JwtError};
pub use repositories::*;
