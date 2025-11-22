//! OAuth Client Repository Port
//!
//! Defines operations for persisting and retrieving OAuthClient entities.

use crate::errors::RepositoryError;
use async_trait::async_trait;
use domain::entities::{Confidential, OAuthClient, Public};
use domain::value_objects::ClientId;

/// Repository interface for OAuth Client entities
///
/// # Business Rules
/// - Clients are uniquely identified by ClientId
/// - Supports both Public and Confidential client types
/// - Client secrets are stored securely (hashed in production)
///
/// # Implementation Notes
/// Since OAuthClient uses phantom types for Public/Confidential,
/// we store and retrieve them as generic OAuthClient and the caller
/// determines the type based on whether a secret exists.
#[async_trait]
pub trait ClientRepository: Send + Sync {
    /// Saves a Public client.
    ///
    /// # Arguments
    /// * `client` - The public client to save
    ///
    /// # Returns
    /// Ok(()) on success
    ///
    /// # Errors
    /// - `RepositoryError::AlreadyExists` if client ID already exists
    /// - `RepositoryError::Database` for database errors
    async fn save_public(&self, client: &OAuthClient<Public>) -> Result<(), RepositoryError>;

    /// Saves a Confidential client.
    ///
    /// # Arguments
    /// * `client` - The confidential client to save
    ///
    /// # Returns
    /// Ok(()) on success
    ///
    /// # Errors
    /// - `RepositoryError::AlreadyExists` if client ID already exists
    /// - `RepositoryError::Database` for database errors
    async fn save_confidential(
        &self,
        client: &OAuthClient<Confidential>,
    ) -> Result<(), RepositoryError>;

    /// Finds a client by ID.
    ///
    /// # Arguments
    /// * `id` - The client ID to search for
    ///
    /// # Returns
    /// - `Ok(Some(client))` if found (as Public or Confidential based on secret presence)
    /// - `Ok(None)` if not found
    ///
    /// # Errors
    /// - `RepositoryError::Database` for database errors
    ///
    /// # Implementation Notes
    /// The caller should check `is_confidential()` or `is_public()` to determine
    /// the client type, then use appropriate type conversion.
    async fn find_by_id(&self, id: ClientId) -> Result<Option<ClientData>, RepositoryError>;

    /// Deletes a client by ID.
    ///
    /// # Arguments
    /// * `id` - The client ID to delete
    ///
    /// # Returns
    /// Ok(()) on success
    ///
    /// # Errors
    /// - `RepositoryError::NotFound` if client doesn't exist
    /// - `RepositoryError::Database` for database errors
    async fn delete(&self, id: ClientId) -> Result<(), RepositoryError>;
}

/// Client data returned from repository
///
/// This type-erased representation allows the repository to return
/// client data without committing to a specific phantom type.
/// The caller can reconstruct the appropriate typed client.
#[derive(Debug, Clone)]
pub struct ClientData {
    pub id: ClientId,
    pub name: String,
    pub redirect_uris: Vec<String>,
    pub allowed_scopes: domain::value_objects::Scopes,
    pub secret: Option<domain::entities::ClientSecret>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl ClientData {
    /// Reconstructs a Public client from this data.
    ///
    /// # Panics
    /// Panics if this data contains a secret (use `to_confidential` instead)
    pub fn to_public(self) -> OAuthClient<Public> {
        assert!(
            self.secret.is_none(),
            "Cannot convert client with secret to Public"
        );
        OAuthClient::from_existing(
            self.id,
            self.name,
            self.redirect_uris,
            self.allowed_scopes,
            None,
            self.created_at,
        )
    }

    /// Reconstructs a Confidential client from this data.
    ///
    /// # Panics
    /// Panics if this data doesn't contain a secret (use `to_public` instead)
    pub fn to_confidential(self) -> OAuthClient<Confidential> {
        assert!(
            self.secret.is_some(),
            "Cannot convert client without secret to Confidential"
        );
        OAuthClient::from_existing(
            self.id,
            self.name,
            self.redirect_uris,
            self.allowed_scopes,
            self.secret,
            self.created_at,
        )
    }

    /// Checks if this represents a confidential client.
    pub fn is_confidential(&self) -> bool {
        self.secret.is_some()
    }

    /// Checks if this represents a public client.
    pub fn is_public(&self) -> bool {
        self.secret.is_none()
    }
}
