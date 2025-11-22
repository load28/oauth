//! Application-level errors
//!
//! These errors represent failures in use case execution.

use thiserror::Error;

/// Repository operation errors
#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Entity not found")]
    NotFound,

    #[error("Entity already exists")]
    AlreadyExists,

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Use case execution errors
#[derive(Debug, Error)]
pub enum UseCaseError {
    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("Domain error: {0}")]
    Domain(String),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Not found")]
    NotFound,

    #[error("Internal error: {0}")]
    Internal(String),
}
