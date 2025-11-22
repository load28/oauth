//! User Repository Port
//!
//! Defines operations for persisting and retrieving User entities.

use crate::errors::RepositoryError;
use async_trait::async_trait;
use domain::entities::User;
use domain::value_objects::{Email, UserId};

/// Repository interface for User entities
///
/// # Business Rules
/// - Users are uniquely identified by UserId
/// - Email must be unique across all users
/// - All operations are asynchronous
///
/// # Implementation Notes
/// Implementations must ensure:
/// - Thread safety (Send + Sync)
/// - Transactional consistency
/// - Email uniqueness constraint
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Saves a new user or updates an existing one.
    ///
    /// # Arguments
    /// * `user` - The user entity to save
    ///
    /// # Returns
    /// Ok(()) on success
    ///
    /// # Errors
    /// - `RepositoryError::AlreadyExists` if email already exists (for new users)
    /// - `RepositoryError::ConstraintViolation` if constraints are violated
    /// - `RepositoryError::Database` for database errors
    async fn save(&self, user: &User) -> Result<(), RepositoryError>;

    /// Finds a user by their unique ID.
    ///
    /// # Arguments
    /// * `id` - The user ID to search for
    ///
    /// # Returns
    /// - `Ok(Some(User))` if found
    /// - `Ok(None)` if not found
    ///
    /// # Errors
    /// - `RepositoryError::Database` for database errors
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepositoryError>;

    /// Finds a user by their email address.
    ///
    /// # Arguments
    /// * `email` - The email to search for
    ///
    /// # Returns
    /// - `Ok(Some(User))` if found
    /// - `Ok(None)` if not found
    ///
    /// # Errors
    /// - `RepositoryError::Database` for database errors
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepositoryError>;

    /// Checks if a user with the given email exists.
    ///
    /// # Arguments
    /// * `email` - The email to check
    ///
    /// # Returns
    /// true if exists, false otherwise
    ///
    /// # Errors
    /// - `RepositoryError::Database` for database errors
    async fn exists_by_email(&self, email: &Email) -> Result<bool, RepositoryError>;

    /// Deletes a user by their ID.
    ///
    /// # Arguments
    /// * `id` - The user ID to delete
    ///
    /// # Returns
    /// Ok(()) on success
    ///
    /// # Errors
    /// - `RepositoryError::NotFound` if user doesn't exist
    /// - `RepositoryError::Database` for database errors
    async fn delete(&self, id: UserId) -> Result<(), RepositoryError>;
}
