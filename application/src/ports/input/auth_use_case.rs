//! Authorization Use Case Port
//!
//! Defines operations for the OAuth 2.0 authorization flow.

use crate::errors::UseCaseError;
use async_trait::async_trait;
use domain::value_objects::{ClientId, UserId};

/// Input DTO for authorization request
#[derive(Debug, Clone)]
pub struct AuthorizeInput {
    pub client_id: ClientId,
    pub user_id: UserId,
    pub redirect_uri: String,
    pub scope: String, // Space-separated scopes
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>, // "plain" or "S256"
    pub state: Option<String>,
}

/// Output DTO for authorization
#[derive(Debug, Clone)]
pub struct AuthorizeOutput {
    pub authorization_code: String,
    pub redirect_uri: String,
    pub state: Option<String>,
}

/// Use case interface for OAuth authorization flow
///
/// # Standards
/// RFC 6749 Section 4.1 (Authorization Code Grant)
/// RFC 7636 (PKCE)
///
/// # Business Rules
/// - User must be authenticated before authorization
/// - Client must be registered and valid
/// - Redirect URI must match registered URIs
/// - Public clients must use PKCE
#[async_trait]
pub trait AuthUseCase: Send + Sync {
    /// Processes an authorization request and generates an authorization code.
    ///
    /// # Arguments
    /// * `input` - Authorization request data
    ///
    /// # Returns
    /// AuthorizeOutput containing the authorization code and redirect info
    ///
    /// # Errors
    /// - `UseCaseError::NotFound` if client doesn't exist
    /// - `UseCaseError::Validation` if redirect_uri or scopes are invalid
    /// - `UseCaseError::Forbidden` if PKCE is required but not provided
    /// - `UseCaseError::Repository` for database errors
    ///
    /// # Business Rules
    /// - Public clients (no secret) MUST provide PKCE
    /// - Confidential clients MAY provide PKCE (optional)
    /// - Authorization code expires in 10 minutes
    /// - Authorization code is single-use
    async fn authorize(&self, input: AuthorizeInput) -> Result<AuthorizeOutput, UseCaseError>;
}
