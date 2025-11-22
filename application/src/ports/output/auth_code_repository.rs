//! Authorization Code Repository Port
//!
//! Defines operations for persisting and retrieving AuthorizationCode entities.

use crate::errors::RepositoryError;
use async_trait::async_trait;
use domain::entities::AuthorizationCode;

/// Repository interface for Authorization Code entities
///
/// # Business Rules
/// - Authorization codes are single-use and short-lived (10 minutes)
/// - Codes are uniquely identified by their code string
/// - Expired codes should be cleaned up periodically
///
/// # Security Notes
/// - Codes must be stored securely
/// - Used codes should be marked to prevent replay attacks
/// - Consider implementing automatic cleanup of expired codes
#[async_trait]
pub trait AuthCodeRepository: Send + Sync {
    /// Saves a new authorization code.
    ///
    /// # Arguments
    /// * `auth_code` - The authorization code to save
    ///
    /// # Returns
    /// Ok(()) on success
    ///
    /// # Errors
    /// - `RepositoryError::AlreadyExists` if code already exists
    /// - `RepositoryError::Database` for database errors
    async fn save(&self, auth_code: &AuthorizationCode) -> Result<(), RepositoryError>;

    /// Finds an authorization code by its code string.
    ///
    /// # Arguments
    /// * `code` - The authorization code string to search for
    ///
    /// # Returns
    /// - `Ok(Some(AuthorizationCode))` if found
    /// - `Ok(None)` if not found
    ///
    /// # Errors
    /// - `RepositoryError::Database` for database errors
    async fn find_by_code(&self, code: &str) -> Result<Option<AuthorizationCode>, RepositoryError>;

    /// Marks an authorization code as used.
    ///
    /// # Arguments
    /// * `code` - The authorization code string to mark as used
    ///
    /// # Returns
    /// Ok(()) on success
    ///
    /// # Errors
    /// - `RepositoryError::NotFound` if code doesn't exist
    /// - `RepositoryError::Database` for database errors
    ///
    /// # Business Rules
    /// This operation should be atomic to prevent race conditions
    async fn mark_as_used(&self, code: &str) -> Result<(), RepositoryError>;

    /// Deletes an authorization code.
    ///
    /// # Arguments
    /// * `code` - The authorization code string to delete
    ///
    /// # Returns
    /// Ok(()) on success (even if code doesn't exist)
    ///
    /// # Errors
    /// - `RepositoryError::Database` for database errors
    async fn delete(&self, code: &str) -> Result<(), RepositoryError>;

    /// Deletes all expired authorization codes.
    ///
    /// # Returns
    /// The number of codes deleted
    ///
    /// # Errors
    /// - `RepositoryError::Database` for database errors
    ///
    /// # Implementation Notes
    /// This should be called periodically to clean up expired codes
    async fn delete_expired(&self) -> Result<usize, RepositoryError>;
}
