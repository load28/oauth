//! Domain-level errors
//!
//! These errors represent business rule violations and domain logic failures.
//! They should never include infrastructure details.

use thiserror::Error;

/// User-related domain errors
#[derive(Debug, Error)]
pub enum UserError {
    #[error("Email already in use: {0}")]
    EmailAlreadyExists(String),

    #[error("User not found")]
    NotFound,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User account is locked")]
    AccountLocked,

    #[error("Cannot change to the same email")]
    SameEmail,
}

/// OAuth client-related domain errors
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Invalid client ID")]
    InvalidClientId,

    #[error("Invalid client secret")]
    InvalidClientSecret,

    #[error("Redirect URI not registered: {0}")]
    InvalidRedirectUri(String),

    #[error("Requested scope not allowed: {0}")]
    ScopeNotAllowed(String),

    #[error("Client not found")]
    NotFound,
}

/// Authorization code-related domain errors
#[derive(Debug, Error)]
pub enum AuthCodeError {
    #[error("Authorization code has expired")]
    Expired,

    #[error("Authorization code not found")]
    NotFound,

    #[error("Authorization code already used")]
    AlreadyUsed,

    #[error("Invalid PKCE code verifier")]
    InvalidCodeVerifier,

    #[error("PKCE required but not provided")]
    PkceRequired,
}

/// Token-related domain errors
#[derive(Debug, Error)]
pub enum TokenError {
    #[error("Token has expired")]
    Expired,

    #[error("Invalid token")]
    Invalid,

    #[error("Token not found")]
    NotFound,

    #[error("Token has been revoked")]
    Revoked,

    #[error("Insufficient scope: required {required}, got {actual}")]
    InsufficientScope { required: String, actual: String },
}
