//! JWT (JSON Web Token) Generation and Verification
//!
//! Implements RFC 7519 (JSON Web Token) for OAuth 2.0 access tokens.
//! Follows security best practices from RFC 8725 (JWT Best Current Practices).

use chrono::{Duration, Utc};
use domain::value_objects::{ClientId, Scopes, UserId};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// JWT-related errors
#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("Failed to encode JWT: {0}")]
    EncodingError(#[from] jsonwebtoken::errors::Error),

    #[error("Failed to decode JWT: {0}")]
    DecodingError(String),

    #[error("Invalid JWT claims: {0}")]
    InvalidClaims(String),

    #[error("JWT has expired")]
    Expired,

    #[error("Invalid audience")]
    InvalidAudience,
}

/// JWT Claims following RFC 7519
///
/// This structure contains both registered claims (defined by RFC 7519)
/// and custom claims specific to OAuth 2.0.
///
/// # RFC 7519 Registered Claims
///
/// - `sub` (Subject): The user ID for whom the token is issued
/// - `aud` (Audience): The client ID that should accept this token
/// - `exp` (Expiration Time): Unix timestamp when the token expires
/// - `iat` (Issued At): Unix timestamp when the token was issued
/// - `iss` (Issuer): Optional identifier of the authorization server
///
/// # Custom OAuth 2.0 Claims
///
/// - `scope`: Space-separated list of OAuth 2.0 scopes
///
/// # Security Notes
///
/// - Token lifetime should be short (RFC 8725 recommendation)
/// - No sensitive data is included (PII is kept minimal)
/// - Tokens cannot be revoked (use short expiration instead)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Claims {
    /// Subject: User ID
    pub sub: String,

    /// Audience: Client ID that should accept this token
    pub aud: String,

    /// Scope: OAuth 2.0 scopes (space-separated)
    pub scope: String,

    /// Expiration time: Unix timestamp
    pub exp: i64,

    /// Issued at: Unix timestamp
    pub iat: i64,

    /// Issuer: Authorization server identifier (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
}

impl Claims {
    /// Create new JWT claims
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user for whom the token is issued
    /// * `client_id` - The client that will use this token
    /// * `scopes` - OAuth 2.0 scopes granted to the token
    /// * `lifetime_seconds` - Token lifetime in seconds (from config)
    ///
    /// # Security
    ///
    /// - Follows RFC 8725: Use short-lived tokens (default: 3600 seconds / 1 hour)
    /// - Expiration is automatically calculated from current time + lifetime
    /// - Issued-at time is set to current Unix timestamp
    ///
    /// # Example
    ///
    /// ```no_run
    /// use domain::value_objects::{UserId, ClientId, Scopes};
    /// use infrastructure::jwt::Claims;
    ///
    /// let user_id = UserId::new();
    /// let client_id = ClientId::new();
    /// let scopes = Scopes::parse("openid profile email").unwrap();
    ///
    /// let claims = Claims::new(user_id, client_id, scopes, 3600);
    /// ```
    pub fn new(user_id: UserId, client_id: ClientId, scopes: Scopes, lifetime_seconds: u64) -> Self {
        let now = Utc::now();
        let exp = now + Duration::seconds(lifetime_seconds as i64);

        Self {
            sub: user_id.to_string(),
            aud: client_id.to_string(),
            scope: scopes.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            iss: Some("oauth-server".to_string()),
        }
    }

    /// Get user ID from claims
    pub fn user_id(&self) -> Result<UserId, JwtError> {
        UserId::parse(&self.sub).map_err(|_| JwtError::InvalidClaims("Invalid user ID".to_string()))
    }

    /// Get client ID from claims
    pub fn client_id(&self) -> Result<ClientId, JwtError> {
        ClientId::parse(&self.aud)
            .map_err(|_| JwtError::InvalidClaims("Invalid client ID".to_string()))
    }

    /// Get scopes from claims
    pub fn scopes(&self) -> Result<Scopes, JwtError> {
        Scopes::parse(&self.scope)
            .map_err(|_| JwtError::InvalidClaims("Invalid scopes".to_string()))
    }
}

/// Encode claims into a JWT token
///
/// Uses HS256 (HMAC with SHA-256) algorithm with a shared secret key.
///
/// # Security Notes (RFC 8725)
///
/// - For production, consider using RS256 (RSA with SHA-256) with public/private keys
/// - HS256 is acceptable for symmetric key scenarios
/// - Secret key must be at least 256 bits (32 bytes)
/// - Tokens are signed but NOT encrypted (don't include sensitive data)
///
/// # Arguments
///
/// * `claims` - The claims to encode
/// * `secret` - Secret key for signing (minimum 32 bytes)
///
/// # Returns
///
/// JWT token as a string
///
/// # Errors
///
/// Returns `JwtError::EncodingError` if encoding fails
///
/// # Example
///
/// ```ignore
/// use infrastructure::jwt::{Claims, encode_jwt};
///
/// let claims = Claims::new(user_id, client_id, scopes, 3600);
/// let token = encode_jwt(&claims, b"your-secret-key-min-32-chars")?;
/// ```
pub fn encode_jwt(claims: &Claims, secret: &[u8]) -> Result<String, JwtError> {
    let header = Header::new(Algorithm::HS256);
    let encoding_key = EncodingKey::from_secret(secret);

    encode(&header, claims, &encoding_key).map_err(JwtError::from)
}

/// Decode and validate a JWT token
///
/// Validates the token signature and registered claims:
/// - Signature verification using the secret key
/// - Expiration time (`exp` claim)
/// - Issued-at time (`iat` claim if present)
///
/// # Security Notes
///
/// - Always validates expiration (prevents token reuse)
/// - Verifies signature (prevents tampering)
/// - Optionally validates audience (aud claim) if provided
///
/// # Arguments
///
/// * `token` - JWT token string to decode
/// * `secret` - Secret key used for signing
/// * `expected_audience` - Optional client ID to validate (recommended)
///
/// # Returns
///
/// Decoded claims if validation succeeds
///
/// # Errors
///
/// - `JwtError::DecodingError`: Invalid token format or signature
/// - `JwtError::Expired`: Token has expired
/// - `JwtError::InvalidAudience`: Audience mismatch
///
/// # Example
///
/// ```ignore
/// use infrastructure::jwt::decode_jwt;
///
/// let claims = decode_jwt(&token, b"your-secret-key", Some("client-id"))?;
/// println!("Token issued for user: {}", claims.sub);
/// ```
pub fn decode_jwt(
    token: &str,
    secret: &[u8],
    expected_audience: Option<&str>,
) -> Result<Claims, JwtError> {
    let mut validation = Validation::new(Algorithm::HS256);

    // Configure audience validation if provided
    if let Some(aud) = expected_audience {
        validation.set_audience(&[aud]);
    } else {
        // Disable audience validation if not provided
        validation.validate_aud = false;
    }

    let decoding_key = DecodingKey::from_secret(secret);

    decode::<Claims>(token, &decoding_key, &validation)
        .map(|token_data| token_data.claims)
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::Expired,
            jsonwebtoken::errors::ErrorKind::InvalidAudience => JwtError::InvalidAudience,
            _ => JwtError::DecodingError(e.to_string()),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::value_objects::{ClientId, Scopes, UserId};

    const TEST_SECRET: &[u8] = b"test-secret-key-must-be-at-least-32-characters-long";

    #[test]
    fn should_encode_and_decode_jwt() {
        let user_id = UserId::new();
        let client_id = ClientId::new();
        let scopes = Scopes::parse("openid profile email").unwrap();

        let claims = Claims::new(user_id, client_id, scopes.clone(), 3600);
        let original_claims = claims.clone();

        // Encode
        let token = encode_jwt(&claims, TEST_SECRET).expect("Failed to encode JWT");
        assert!(!token.is_empty());

        // Decode
        let decoded_claims = decode_jwt(&token, TEST_SECRET, Some(&client_id.to_string()))
            .expect("Failed to decode JWT");

        assert_eq!(decoded_claims.sub, original_claims.sub);
        assert_eq!(decoded_claims.aud, original_claims.aud);
        assert_eq!(decoded_claims.scope, original_claims.scope);
        assert_eq!(decoded_claims.exp, original_claims.exp);
        assert_eq!(decoded_claims.iat, original_claims.iat);
    }

    #[test]
    fn should_reject_invalid_signature() {
        let user_id = UserId::new();
        let client_id = ClientId::new();
        let scopes = Scopes::parse("openid").unwrap();

        let claims = Claims::new(user_id, client_id, scopes, 3600);
        let token = encode_jwt(&claims, TEST_SECRET).unwrap();

        // Try to decode with wrong secret
        let wrong_secret = b"wrong-secret-key-completely-different-32-chars-minimum";
        let result = decode_jwt(&token, wrong_secret, None);

        assert!(result.is_err());
        assert!(matches!(result, Err(JwtError::DecodingError(_))));
    }

    #[test]
    fn should_reject_expired_token() {
        let user_id = UserId::new();
        let client_id = ClientId::new();
        let scopes = Scopes::parse("openid").unwrap();

        // Create token that expired 1 hour ago (well past any leeway)
        let now = Utc::now();
        let claims = Claims {
            sub: user_id.to_string(),
            aud: client_id.to_string(),
            scope: scopes.to_string(),
            exp: (now - Duration::hours(1)).timestamp(), // Expired 1 hour ago
            iat: (now - Duration::hours(2)).timestamp(), // Issued 2 hours ago
            iss: Some("oauth-server".to_string()),
        };

        let token = encode_jwt(&claims, TEST_SECRET).unwrap();
        let result = decode_jwt(&token, TEST_SECRET, None);

        assert!(result.is_err());
        assert!(matches!(result, Err(JwtError::Expired)));
    }

    #[test]
    fn should_validate_audience() {
        let user_id = UserId::new();
        let client_id = ClientId::new();
        let scopes = Scopes::parse("openid").unwrap();

        let claims = Claims::new(user_id, client_id, scopes, 3600);
        let token = encode_jwt(&claims, TEST_SECRET).unwrap();

        // Try to decode with wrong audience
        let result = decode_jwt(&token, TEST_SECRET, Some("wrong-client-id"));

        assert!(result.is_err());
        assert!(matches!(result, Err(JwtError::InvalidAudience)));
    }

    #[test]
    fn should_parse_claims_to_domain_types() {
        let user_id = UserId::new();
        let client_id = ClientId::new();
        let scopes = Scopes::parse("openid profile").unwrap();

        let claims = Claims::new(user_id, client_id, scopes.clone(), 3600);

        // Parse back to domain types
        let parsed_user_id = claims.user_id().expect("Should parse user ID");
        let parsed_client_id = claims.client_id().expect("Should parse client ID");
        let parsed_scopes = claims.scopes().expect("Should parse scopes");

        assert_eq!(parsed_user_id, user_id);
        assert_eq!(parsed_client_id, client_id);
        assert_eq!(parsed_scopes, scopes);
    }
}
