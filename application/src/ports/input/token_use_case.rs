//! Token Use Case Port
//!
//! Defines operations for OAuth 2.0 token management.

use crate::errors::UseCaseError;
use async_trait::async_trait;
use domain::value_objects::{ClientId, UserId};

/// Input DTO for token exchange (authorization code grant)
#[derive(Debug, Clone)]
pub struct ExchangeCodeInput {
    pub code: String,
    pub client_id: ClientId,
    pub client_secret: Option<String>, // Required for confidential clients
    pub redirect_uri: String,
    pub code_verifier: Option<String>, // Required if PKCE was used
}

/// Input DTO for token refresh
#[derive(Debug, Clone)]
pub struct RefreshTokenInput {
    pub refresh_token: String,
    pub client_id: ClientId,
    pub client_secret: Option<String>, // Required for confidential clients
    pub scope: Option<String>,         // Optional: request narrower scope
}

/// Input DTO for token revocation
#[derive(Debug, Clone)]
pub struct RevokeTokenInput {
    pub token: String,
    pub client_id: ClientId,
    pub client_secret: Option<String>, // Required for confidential clients
    pub token_type_hint: Option<String>, // "access_token" or "refresh_token"
}

/// Output DTO for token operations
#[derive(Debug, Clone)]
pub struct TokenOutput {
    pub access_token: String,
    pub token_type: String, // "Bearer"
    pub expires_in: i64,    // Seconds until expiration
    pub refresh_token: Option<String>,
    pub scope: String, // Space-separated scopes
}

/// Input DTO for token introspection
#[derive(Debug, Clone)]
pub struct IntrospectTokenInput {
    pub token: String,
    pub client_id: ClientId,
    pub client_secret: Option<String>,
}

/// Output DTO for token introspection
#[derive(Debug, Clone)]
pub struct IntrospectTokenOutput {
    pub active: bool,
    pub scope: Option<String>,
    pub client_id: Option<ClientId>,
    pub user_id: Option<UserId>,
    pub exp: Option<i64>, // Expiration timestamp
}

/// Use case interface for OAuth token management
///
/// # Standards
/// - RFC 6749 Section 4.1.3 (Token Endpoint)
/// - RFC 6749 Section 6 (Refreshing Tokens)
/// - RFC 7009 (Token Revocation)
/// - RFC 7662 (Token Introspection)
///
/// # Business Rules
/// - Access tokens are short-lived (1 hour)
/// - Refresh tokens are long-lived (30 days)
/// - Confidential clients must authenticate with client secret
/// - Public clients use PKCE instead of client secret
#[async_trait]
pub trait TokenUseCase: Send + Sync {
    /// Exchanges an authorization code for access and refresh tokens.
    ///
    /// # Arguments
    /// * `input` - Token exchange request data
    ///
    /// # Returns
    /// TokenOutput containing access token, refresh token, and metadata
    ///
    /// # Errors
    /// - `UseCaseError::Unauthorized` if client authentication fails
    /// - `UseCaseError::Validation` if code is invalid, expired, or already used
    /// - `UseCaseError::NotFound` if code doesn't exist
    /// - `UseCaseError::Repository` for database errors
    ///
    /// # Business Rules
    /// - Authorization code must be valid and not expired
    /// - Authorization code can only be used once
    /// - Redirect URI must match the one used in authorization
    /// - PKCE verification if challenge was provided during authorization
    /// - Confidential clients must provide valid client_secret
    async fn exchange_code(&self, input: ExchangeCodeInput)
        -> Result<TokenOutput, UseCaseError>;

    /// Refreshes an access token using a refresh token.
    ///
    /// # Arguments
    /// * `input` - Token refresh request data
    ///
    /// # Returns
    /// TokenOutput containing new access token and possibly new refresh token
    ///
    /// # Errors
    /// - `UseCaseError::Unauthorized` if client authentication fails
    /// - `UseCaseError::Validation` if refresh token is invalid or revoked
    /// - `UseCaseError::NotFound` if refresh token doesn't exist
    /// - `UseCaseError::Repository` for database errors
    ///
    /// # Business Rules
    /// - Refresh token must be valid and not revoked
    /// - Refresh token must not be expired
    /// - Client must be the same as the one that received the original token
    /// - Optionally issue a new refresh token (rotation)
    async fn refresh_token(
        &self,
        input: RefreshTokenInput,
    ) -> Result<TokenOutput, UseCaseError>;

    /// Revokes an access or refresh token.
    ///
    /// # Arguments
    /// * `input` - Token revocation request data
    ///
    /// # Returns
    /// Ok(()) on success (even if token doesn't exist)
    ///
    /// # Errors
    /// - `UseCaseError::Unauthorized` if client authentication fails
    /// - `UseCaseError::Repository` for database errors
    ///
    /// # Business Rules
    /// - Revoking a refresh token should also revoke associated access tokens
    /// - Client must be the same as the one that received the token
    /// - Revocation is idempotent (succeeds even if already revoked)
    async fn revoke_token(&self, input: RevokeTokenInput) -> Result<(), UseCaseError>;

    /// Introspects a token to check its validity and metadata.
    ///
    /// # Arguments
    /// * `input` - Token introspection request data
    ///
    /// # Returns
    /// IntrospectTokenOutput containing token status and metadata
    ///
    /// # Errors
    /// - `UseCaseError::Unauthorized` if client authentication fails
    /// - `UseCaseError::Repository` for database errors
    ///
    /// # Business Rules
    /// - Returns active=false for expired, revoked, or non-existent tokens
    /// - Client must be authorized to introspect the token
    async fn introspect_token(
        &self,
        input: IntrospectTokenInput,
    ) -> Result<IntrospectTokenOutput, UseCaseError>;
}
