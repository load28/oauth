use crate::errors::ClientError;
use crate::value_objects::{ClientId, Scopes};
use base64::Engine;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// Marker type for Public OAuth clients (e.g., SPAs, mobile apps)
#[derive(Debug, Clone, Copy)]
pub struct Public;

/// Marker type for Confidential OAuth clients (e.g., server-side apps)
#[derive(Debug, Clone, Copy)]
pub struct Confidential;

/// OAuth Client Secret (only for Confidential clients)
///
/// # Security
/// - Should be stored hashed in production
/// - For this implementation, we store it as-is (can be enhanced)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientSecret(String);

impl ClientSecret {
    /// Creates a new client secret.
    pub fn new(secret: impl Into<String>) -> Self {
        Self(secret.into())
    }

    /// Generates a new random client secret.
    pub fn generate() -> Self {
        use rand::Rng;
        let random_bytes: Vec<u8> = (0..32).map(|_| rand::rng().random()).collect();
        let secret = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&random_bytes);
        Self(secret)
    }

    /// Returns the secret as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Verifies if the provided secret matches.
    pub fn verify(&self, secret: &str) -> bool {
        // In production, use constant-time comparison
        self.0 == secret
    }
}

/// OAuth 2.0 Client entity
///
/// Uses Typestate pattern to enforce client type at compile time.
///
/// # Type Parameters
/// * `T` - Client type marker (Public or Confidential)
///
/// # Invariants
/// - ClientId is unique and immutable
/// - Public clients have no secret
/// - Confidential clients must have a secret
/// - Redirect URIs are validated
/// - Scopes are validated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthClient<T> {
    id: ClientId,
    name: String,
    redirect_uris: Vec<String>,
    allowed_scopes: Scopes,
    #[serde(skip)]
    secret: Option<ClientSecret>,
    created_at: DateTime<Utc>,
    #[serde(skip)]
    _client_type: PhantomData<T>,
}

impl OAuthClient<Public> {
    /// Creates a new Public OAuth client.
    ///
    /// # Arguments
    /// * `id` - Unique client identifier
    /// * `name` - Human-readable client name
    /// * `redirect_uris` - List of allowed redirect URIs
    /// * `allowed_scopes` - Scopes this client can request
    ///
    /// # Examples
    /// ```
    /// use domain::entities::{OAuthClient, Public};
    /// use domain::value_objects::{ClientId, Scopes};
    ///
    /// let client = OAuthClient::<Public>::new(
    ///     ClientId::new(),
    ///     "My SPA App",
    ///     vec!["http://localhost:3000/callback".to_string()],
    ///     Scopes::parse("openid profile email").unwrap(),
    /// );
    /// ```
    pub fn new(
        id: ClientId,
        name: impl Into<String>,
        redirect_uris: Vec<String>,
        allowed_scopes: Scopes,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            redirect_uris,
            allowed_scopes,
            secret: None,
            created_at: Utc::now(),
            _client_type: PhantomData,
        }
    }

    /// Returns true (Public clients don't require PKCE by RFC, but we enforce it).
    pub fn requires_pkce(&self) -> bool {
        true
    }
}

impl OAuthClient<Confidential> {
    /// Creates a new Confidential OAuth client with a secret.
    ///
    /// # Arguments
    /// * `id` - Unique client identifier
    /// * `name` - Human-readable client name
    /// * `secret` - Client secret for authentication
    /// * `redirect_uris` - List of allowed redirect URIs
    /// * `allowed_scopes` - Scopes this client can request
    pub fn new(
        id: ClientId,
        name: impl Into<String>,
        secret: ClientSecret,
        redirect_uris: Vec<String>,
        allowed_scopes: Scopes,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            redirect_uris,
            allowed_scopes,
            secret: Some(secret),
            created_at: Utc::now(),
            _client_type: PhantomData,
        }
    }

    /// Verifies the client secret.
    ///
    /// # Arguments
    /// * `secret` - The secret to verify
    ///
    /// # Returns
    /// Ok(()) if secret matches
    ///
    /// # Errors
    /// Returns ClientError::InvalidClientSecret if verification fails
    pub fn verify_secret(&self, secret: &str) -> Result<(), ClientError> {
        match &self.secret {
            Some(s) if s.verify(secret) => Ok(()),
            _ => Err(ClientError::InvalidClientSecret),
        }
    }

    /// Returns false (Confidential clients can optionally use PKCE but not required).
    pub fn requires_pkce(&self) -> bool {
        false
    }

    /// Returns the client secret (for testing or admin purposes).
    pub fn secret(&self) -> Option<&ClientSecret> {
        self.secret.as_ref()
    }
}

// Common methods for both client types
impl<T> OAuthClient<T> {
    /// Creates an OAuthClient from existing data (e.g., from database).
    pub fn from_existing(
        id: ClientId,
        name: String,
        redirect_uris: Vec<String>,
        allowed_scopes: Scopes,
        secret: Option<ClientSecret>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name,
            redirect_uris,
            allowed_scopes,
            secret,
            created_at,
            _client_type: PhantomData,
        }
    }

    /// Returns the client ID.
    pub fn id(&self) -> ClientId {
        self.id
    }

    /// Returns the client name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the allowed redirect URIs.
    pub fn redirect_uris(&self) -> &[String] {
        &self.redirect_uris
    }

    /// Returns the allowed scopes.
    pub fn allowed_scopes(&self) -> &Scopes {
        &self.allowed_scopes
    }

    /// Returns the creation timestamp.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Validates if a redirect URI is allowed for this client.
    ///
    /// # Arguments
    /// * `redirect_uri` - The redirect URI to validate
    ///
    /// # Returns
    /// Ok(()) if valid
    ///
    /// # Errors
    /// Returns ClientError::InvalidRedirectUri if not allowed
    ///
    /// # Business Rules
    /// - Exact match required (no wildcard support for security)
    pub fn validate_redirect_uri(&self, redirect_uri: &str) -> Result<(), ClientError> {
        if self.redirect_uris.contains(&redirect_uri.to_string()) {
            Ok(())
        } else {
            Err(ClientError::InvalidRedirectUri(redirect_uri.to_string()))
        }
    }

    /// Validates if the requested scopes are allowed for this client.
    ///
    /// # Arguments
    /// * `requested_scopes` - The scopes being requested
    ///
    /// # Returns
    /// Ok(()) if all requested scopes are allowed
    ///
    /// # Errors
    /// Returns ClientError::ScopeNotAllowed if any scope is not allowed
    pub fn validate_scopes(&self, requested_scopes: &Scopes) -> Result<(), ClientError> {
        for scope in requested_scopes.iter() {
            if !self.allowed_scopes.contains(scope) {
                return Err(ClientError::ScopeNotAllowed(scope.to_string()));
            }
        }
        Ok(())
    }

    /// Checks if the client is confidential (has a secret).
    pub fn is_confidential(&self) -> bool {
        self.secret.is_some()
    }

    /// Checks if the client is public (no secret).
    pub fn is_public(&self) -> bool {
        self.secret.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_public_client() {
        let id = ClientId::new();
        let scopes = Scopes::parse("openid profile email").unwrap();
        let client = OAuthClient::<Public>::new(
            id,
            "Test SPA",
            vec!["http://localhost:3000/callback".to_string()],
            scopes,
        );

        assert_eq!(client.name(), "Test SPA");
        assert!(client.is_public());
        assert!(!client.is_confidential());
        assert!(client.requires_pkce());
    }

    #[test]
    fn should_create_confidential_client() {
        let id = ClientId::new();
        let secret = ClientSecret::generate();
        let scopes = Scopes::parse("openid profile email").unwrap();

        let client = OAuthClient::<Confidential>::new(
            id,
            "Test Backend",
            secret,
            vec!["https://example.com/callback".to_string()],
            scopes,
        );

        assert_eq!(client.name(), "Test Backend");
        assert!(client.is_confidential());
        assert!(!client.is_public());
        assert!(!client.requires_pkce());
    }

    #[test]
    fn should_validate_redirect_uri() {
        let client = OAuthClient::<Public>::new(
            ClientId::new(),
            "Test App",
            vec!["http://localhost:3000/callback".to_string()],
            Scopes::parse("openid").unwrap(),
        );

        assert!(client
            .validate_redirect_uri("http://localhost:3000/callback")
            .is_ok());
        assert!(client
            .validate_redirect_uri("http://evil.com/callback")
            .is_err());
    }

    #[test]
    fn should_validate_scopes() {
        let allowed = Scopes::parse("openid profile email").unwrap();
        let client = OAuthClient::<Public>::new(
            ClientId::new(),
            "Test App",
            vec!["http://localhost:3000/callback".to_string()],
            allowed,
        );

        let valid_request = Scopes::parse("openid profile").unwrap();
        assert!(client.validate_scopes(&valid_request).is_ok());

        let invalid_request = Scopes::parse("openid write").unwrap();
        assert!(client.validate_scopes(&invalid_request).is_err());
    }

    #[test]
    fn should_verify_client_secret() {
        let secret = ClientSecret::new("my-secret");
        let client = OAuthClient::<Confidential>::new(
            ClientId::new(),
            "Test Backend",
            secret,
            vec!["https://example.com/callback".to_string()],
            Scopes::parse("openid").unwrap(),
        );

        assert!(client.verify_secret("my-secret").is_ok());
        assert!(client.verify_secret("wrong-secret").is_err());
    }

    #[test]
    fn should_generate_random_secret() {
        let secret1 = ClientSecret::generate();
        let secret2 = ClientSecret::generate();

        assert_ne!(secret1.as_str(), secret2.as_str());
        assert!(secret1.as_str().len() > 20);
    }
}
