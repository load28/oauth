//! OAuth Authorization-related Data Transfer Objects

use serde::{Deserialize, Serialize};

/// Request to initiate OAuth authorization (GET /authorize)
///
/// Query parameters following OAuth 2.0 RFC 6749
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorizeRequest {
    /// OAuth 2.0 response type (must be "code")
    pub response_type: String,

    /// Client ID
    pub client_id: String,

    /// Redirect URI (must match registered URI)
    pub redirect_uri: String,

    /// Requested scopes (space-separated)
    pub scope: String,

    /// Client state (opaque value for CSRF protection)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    /// PKCE code challenge (required for public clients)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_challenge: Option<String>,

    /// PKCE code challenge method ("plain" or "S256")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_challenge_method: Option<String>,
}

/// Response to authorization request
///
/// In real implementation, this would redirect to login page
/// For simplicity, we return JSON
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorizeResponse {
    pub message: String,
    pub client_id: String,
    pub requested_scopes: String,
    pub redirect_uri: String,

    /// User must be authenticated first
    pub requires_authentication: bool,
}

/// Request to grant authorization consent (POST /authorize/consent)
#[derive(Debug, Serialize, Deserialize)]
pub struct ConsentRequest {
    /// User ID who is granting consent
    pub user_id: String,

    /// Client ID requesting authorization
    pub client_id: String,

    /// Redirect URI
    pub redirect_uri: String,

    /// Approved scopes (space-separated)
    pub scope: String,

    /// Client state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    /// PKCE code challenge
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_challenge: Option<String>,
}

/// Response after granting consent
///
/// Returns authorization code that client can exchange for tokens
#[derive(Debug, Serialize, Deserialize)]
pub struct ConsentResponse {
    /// Authorization code
    pub code: String,

    /// Client state (if provided in request)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    /// Redirect URI where client should receive the code
    pub redirect_uri: String,
}
