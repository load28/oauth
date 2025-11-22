//! OAuth Client-related Data Transfer Objects

use serde::{Deserialize, Serialize};

/// Request to register a new OAuth client
///
/// # Client Types
///
/// - Public client: No client_secret provided
/// - Confidential client: client_secret provided
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterClientRequest {
    pub name: String,
    pub redirect_uris: Vec<String>,
    pub scopes: String,

    /// Optional secret for confidential clients
    /// If provided, client is confidential; otherwise public
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
}

/// Response after successful client registration
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterClientResponse {
    pub client_id: String,
    pub client_type: String, // "public" or "confidential"
    pub name: String,
    pub redirect_uris: Vec<String>,
    pub allowed_scopes: String,

    /// Only returned for confidential clients
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,

    pub created_at: String,
}

/// Response for GET /clients/{id}
#[derive(Debug, Serialize, Deserialize)]
pub struct GetClientResponse {
    pub client_id: String,
    pub client_type: String,
    pub name: String,
    pub redirect_uris: Vec<String>,
    pub allowed_scopes: String,
    pub created_at: String,
}
