//! OAuth Client Use Case Port
//!
//! Defines operations for OAuth client registration and management.

use crate::errors::UseCaseError;
use async_trait::async_trait;
use domain::entities::{Confidential, OAuthClient, Public};
use domain::value_objects::ClientId;

/// Input DTO for registering a public OAuth client
#[derive(Debug, Clone)]
pub struct RegisterPublicClientInput {
    pub name: String,
    pub redirect_uris: Vec<String>,
    pub allowed_scopes: String, // Space-separated scopes
}

/// Input DTO for registering a confidential OAuth client
#[derive(Debug, Clone)]
pub struct RegisterConfidentialClientInput {
    pub name: String,
    pub redirect_uris: Vec<String>,
    pub allowed_scopes: String, // Space-separated scopes
}

/// Output DTO for public client registration
#[derive(Debug, Clone)]
pub struct RegisterPublicClientOutput {
    pub client_id: ClientId,
    pub name: String,
    pub redirect_uris: Vec<String>,
}

/// Output DTO for confidential client registration
#[derive(Debug, Clone)]
pub struct RegisterConfidentialClientOutput {
    pub client_id: ClientId,
    pub name: String,
    pub redirect_uris: Vec<String>,
    pub client_secret: String, // Return plaintext secret only once
}

/// Use case interface for OAuth client management
///
/// # Business Rules
/// - Public clients (SPAs, mobile apps) require PKCE
/// - Confidential clients (server-side) use client secret
/// - Redirect URIs must be valid and pre-registered
#[async_trait]
pub trait ClientUseCase: Send + Sync {
    /// Registers a new public OAuth client.
    ///
    /// # Arguments
    /// * `input` - Public client registration data
    ///
    /// # Returns
    /// RegisterPublicClientOutput containing client details
    ///
    /// # Errors
    /// - `UseCaseError::Validation` if input is invalid
    /// - `UseCaseError::Repository` for database errors
    ///
    /// # Business Rules
    /// - Public clients do not receive a client secret
    /// - They must use PKCE for authorization
    async fn register_public_client(
        &self,
        input: RegisterPublicClientInput,
    ) -> Result<RegisterPublicClientOutput, UseCaseError>;

    /// Registers a new confidential OAuth client.
    ///
    /// # Arguments
    /// * `input` - Confidential client registration data
    ///
    /// # Returns
    /// RegisterConfidentialClientOutput containing client details and secret
    ///
    /// # Errors
    /// - `UseCaseError::Validation` if input is invalid
    /// - `UseCaseError::Repository` for database errors
    ///
    /// # Security Notes
    /// - The client secret is returned only once during registration
    /// - Store it securely; it cannot be retrieved later
    async fn register_confidential_client(
        &self,
        input: RegisterConfidentialClientInput,
    ) -> Result<RegisterConfidentialClientOutput, UseCaseError>;

    /// Gets a public client by ID.
    ///
    /// # Arguments
    /// * `client_id` - The client ID to retrieve
    ///
    /// # Returns
    /// The public OAuthClient
    ///
    /// # Errors
    /// - `UseCaseError::NotFound` if client doesn't exist or is not public
    /// - `UseCaseError::Repository` for database errors
    async fn get_public_client(
        &self,
        client_id: ClientId,
    ) -> Result<OAuthClient<Public>, UseCaseError>;

    /// Gets a confidential client by ID.
    ///
    /// # Arguments
    /// * `client_id` - The client ID to retrieve
    ///
    /// # Returns
    /// The confidential OAuthClient
    ///
    /// # Errors
    /// - `UseCaseError::NotFound` if client doesn't exist or is not confidential
    /// - `UseCaseError::Repository` for database errors
    async fn get_confidential_client(
        &self,
        client_id: ClientId,
    ) -> Result<OAuthClient<Confidential>, UseCaseError>;

    /// Deletes an OAuth client.
    ///
    /// # Arguments
    /// * `client_id` - The client ID to delete
    ///
    /// # Returns
    /// Ok(()) on success
    ///
    /// # Errors
    /// - `UseCaseError::NotFound` if client doesn't exist
    /// - `UseCaseError::Repository` for database errors
    ///
    /// # Business Rules
    /// - This should also revoke all tokens issued to this client
    async fn delete_client(&self, client_id: ClientId) -> Result<(), UseCaseError>;
}
