//! Output Ports (Repository Interfaces)
//!
//! These traits define how the application layer persists and retrieves data.
//! Infrastructure layer provides concrete implementations.

mod auth_code_repository;
mod client_repository;
mod token_repository;
mod user_repository;

pub use auth_code_repository::AuthCodeRepository;
pub use client_repository::{ClientData, ClientRepository};
pub use token_repository::{AccessTokenRepository, RefreshTokenRepository};
pub use user_repository::UserRepository;
