//! Token Repository Port
//!
//! Defines operations for persisting and retrieving Access and Refresh tokens.

use crate::errors::RepositoryError;
use async_trait::async_trait;
use domain::entities::{AccessToken, RefreshToken};
use domain::value_objects::{ClientId, UserId};

/// Repository interface for Access Token entities
///
/// # Business Rules
/// - Access tokens are short-lived (1 hour)
/// - Tokens are uniquely identified by their token string
/// - Expired tokens should be cleaned up periodically
#[async_trait]
pub trait AccessTokenRepository: Send + Sync {
    /// Saves a new access token.
    ///
    /// # Arguments
    /// * `token` - The access token to save
    ///
    /// # Returns
    /// Ok(()) on success
    ///
    /// # Errors
    /// - `RepositoryError::AlreadyExists` if token already exists
    /// - `RepositoryError::Database` for database errors
    async fn save(&self, token: &AccessToken) -> Result<(), RepositoryError>;

    /// Finds an access token by its token string.
    ///
    /// # Arguments
    /// * `token` - The token string to search for
    ///
    /// # Returns
    /// - `Ok(Some(AccessToken))` if found
    /// - `Ok(None)` if not found
    ///
    /// # Errors
    /// - `RepositoryError::Database` for database errors
    async fn find_by_token(&self, token: &str) -> Result<Option<AccessToken>, RepositoryError>;

    /// Finds all access tokens for a user.
    ///
    /// # Arguments
    /// * `user_id` - The user ID to search for
    ///
    /// # Returns
    /// Vector of access tokens (may be empty)
    ///
    /// # Errors
    /// - `RepositoryError::Database` for database errors
    async fn find_by_user_id(&self, user_id: UserId)
        -> Result<Vec<AccessToken>, RepositoryError>;

    /// Revokes an access token.
    ///
    /// # Arguments
    /// * `token` - The token string to revoke
    ///
    /// # Returns
    /// Ok(()) on success
    ///
    /// # Errors
    /// - `RepositoryError::NotFound` if token doesn't exist
    /// - `RepositoryError::Database` for database errors
    async fn revoke(&self, token: &str) -> Result<(), RepositoryError>;

    /// Revokes all access tokens for a user.
    ///
    /// # Arguments
    /// * `user_id` - The user ID whose tokens should be revoked
    ///
    /// # Returns
    /// The number of tokens revoked
    ///
    /// # Errors
    /// - `RepositoryError::Database` for database errors
    async fn revoke_all_for_user(&self, user_id: UserId) -> Result<usize, RepositoryError>;

    /// Deletes all expired access tokens.
    ///
    /// # Returns
    /// The number of tokens deleted
    ///
    /// # Errors
    /// - `RepositoryError::Database` for database errors
    async fn delete_expired(&self) -> Result<usize, RepositoryError>;
}

/// Repository interface for Refresh Token entities
///
/// # Business Rules
/// - Refresh tokens are long-lived (30 days)
/// - Tokens are uniquely identified by their token string
/// - Revoked tokens cannot be reused
/// - Refresh token rotation is supported
#[async_trait]
pub trait RefreshTokenRepository: Send + Sync {
    /// Saves a new refresh token.
    ///
    /// # Arguments
    /// * `token` - The refresh token to save
    ///
    /// # Returns
    /// Ok(()) on success
    ///
    /// # Errors
    /// - `RepositoryError::AlreadyExists` if token already exists
    /// - `RepositoryError::Database` for database errors
    async fn save(&self, token: &RefreshToken) -> Result<(), RepositoryError>;

    /// Finds a refresh token by its token string.
    ///
    /// # Arguments
    /// * `token` - The token string to search for
    ///
    /// # Returns
    /// - `Ok(Some(RefreshToken))` if found
    /// - `Ok(None)` if not found
    ///
    /// # Errors
    /// - `RepositoryError::Database` for database errors
    async fn find_by_token(&self, token: &str) -> Result<Option<RefreshToken>, RepositoryError>;

    /// Finds all refresh tokens for a user.
    ///
    /// # Arguments
    /// * `user_id` - The user ID to search for
    ///
    /// # Returns
    /// Vector of refresh tokens (may be empty)
    ///
    /// # Errors
    /// - `RepositoryError::Database` for database errors
    async fn find_by_user_id(
        &self,
        user_id: UserId,
    ) -> Result<Vec<RefreshToken>, RepositoryError>;

    /// Finds refresh tokens by access token ID.
    ///
    /// # Arguments
    /// * `access_token_id` - The access token ID to search for
    ///
    /// # Returns
    /// - `Ok(Some(RefreshToken))` if found
    /// - `Ok(None)` if not found
    ///
    /// # Errors
    /// - `RepositoryError::Database` for database errors
    async fn find_by_access_token_id(
        &self,
        access_token_id: &str,
    ) -> Result<Option<RefreshToken>, RepositoryError>;

    /// Revokes a refresh token.
    ///
    /// # Arguments
    /// * `token` - The token string to revoke
    ///
    /// # Returns
    /// Ok(()) on success
    ///
    /// # Errors
    /// - `RepositoryError::NotFound` if token doesn't exist
    /// - `RepositoryError::Database` for database errors
    ///
    /// # Business Rules
    /// This should also revoke associated access tokens
    async fn revoke(&self, token: &str) -> Result<(), RepositoryError>;

    /// Revokes all refresh tokens for a user.
    ///
    /// # Arguments
    /// * `user_id` - The user ID whose tokens should be revoked
    ///
    /// # Returns
    /// The number of tokens revoked
    ///
    /// # Errors
    /// - `RepositoryError::Database` for database errors
    async fn revoke_all_for_user(&self, user_id: UserId) -> Result<usize, RepositoryError>;

    /// Revokes all refresh tokens for a client.
    ///
    /// # Arguments
    /// * `client_id` - The client ID whose tokens should be revoked
    ///
    /// # Returns
    /// The number of tokens revoked
    ///
    /// # Errors
    /// - `RepositoryError::Database` for database errors
    async fn revoke_all_for_client(&self, client_id: ClientId) -> Result<usize, RepositoryError>;

    /// Deletes all expired and revoked refresh tokens.
    ///
    /// # Returns
    /// The number of tokens deleted
    ///
    /// # Errors
    /// - `RepositoryError::Database` for database errors
    async fn delete_expired_and_revoked(&self) -> Result<usize, RepositoryError>;
}
