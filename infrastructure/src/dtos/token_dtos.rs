//! OAuth Token-related Data Transfer Objects

use serde::{Deserialize, Serialize};

/// Request to exchange authorization code for tokens (POST /token)
///
/// Follows OAuth 2.0 RFC 6749 Token Endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenRequest {
    /// Grant type (must be "authorization_code")
    pub grant_type: String,

    /// Authorization code from consent
    pub code: String,

    /// Redirect URI (must match original request)
    pub redirect_uri: String,

    /// Client ID
    pub client_id: String,

    /// Client secret (required for confidential clients)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,

    /// PKCE code verifier (required if code_challenge was used)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_verifier: Option<String>,
}

/// Response containing access and refresh tokens
///
/// Follows OAuth 2.0 RFC 6749 Token Response
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    /// Access token (JWT format)
    pub access_token: String,

    /// Token type (always "Bearer")
    pub token_type: String,

    /// Expires in seconds
    pub expires_in: i64,

    /// Refresh token for getting new access tokens
    pub refresh_token: String,

    /// Granted scopes (space-separated)
    pub scope: String,
}

/// Request to revoke a token (POST /revoke)
///
/// Follows OAuth 2.0 RFC 7009 Token Revocation
#[derive(Debug, Serialize, Deserialize)]
pub struct RevokeTokenRequest {
    /// Token to revoke
    pub token: String,

    /// Token type hint ("access_token" or "refresh_token")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type_hint: Option<String>,

    /// Client ID
    pub client_id: String,

    /// Client secret (for confidential clients)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
}

/// Response after token revocation
#[derive(Debug, Serialize, Deserialize)]
pub struct RevokeTokenResponse {
    pub success: bool,
    pub message: String,
}

/// Request to introspect a token (POST /introspect)
///
/// Follows OAuth 2.0 RFC 7662 Token Introspection
#[derive(Debug, Serialize, Deserialize)]
pub struct IntrospectTokenRequest {
    /// Token to introspect
    pub token: String,

    /// Token type hint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type_hint: Option<String>,
}

/// Response with token introspection details
#[derive(Debug, Serialize, Deserialize)]
pub struct IntrospectTokenResponse {
    /// Whether token is active
    pub active: bool,

    /// Scope (if active)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,

    /// Client ID (if active)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,

    /// User ID (if active)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// Token type (if active)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,

    /// Expiration timestamp (if active)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<i64>,

    /// Issued at timestamp (if active)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<i64>,
}
