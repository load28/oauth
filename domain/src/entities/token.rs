use crate::errors::TokenError;
use crate::value_objects::{ClientId, Scopes, UserId};
use base64::Engine;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Access Token entity
///
/// # Invariants
/// - Token is unique
/// - Has expiration time (typically 1 hour)
/// - Bound to specific client, user, and scopes
///
/// # Standards
/// - RFC 6749 Section 1.4 (Access Tokens)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessToken {
    token: String,
    client_id: ClientId,
    user_id: UserId,
    scope: Scopes,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

/// Refresh Token entity
///
/// # Invariants
/// - Token is unique and opaque
/// - Has longer expiration time (typically 30 days)
/// - Bound to specific access token, client, and user
/// - Can be rotated for security
///
/// # Standards
/// - RFC 6749 Section 1.5 (Refresh Tokens)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshToken {
    token: String,
    access_token_id: String,
    client_id: ClientId,
    user_id: UserId,
    scope: Scopes,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    revoked: bool,
}

impl AccessToken {
    /// Default expiration time for access tokens (1 hour).
    pub const DEFAULT_EXPIRATION_HOURS: i64 = 1;

    /// Creates a new access token with default expiration.
    ///
    /// # Arguments
    /// * `token` - The token string (typically JWT)
    /// * `client_id` - The client this token was issued to
    /// * `user_id` - The user this token represents
    /// * `scope` - The granted scopes
    ///
    /// # Examples
    /// ```
    /// use domain::entities::AccessToken;
    /// use domain::value_objects::{ClientId, UserId, Scopes};
    ///
    /// let token = AccessToken::new(
    ///     "eyJhbGc...".to_string(),
    ///     ClientId::new(),
    ///     UserId::new(),
    ///     Scopes::parse("openid profile email").unwrap(),
    /// );
    /// ```
    pub fn new(token: String, client_id: ClientId, user_id: UserId, scope: Scopes) -> Self {
        let now = Utc::now();
        let expires_at = now + Duration::hours(Self::DEFAULT_EXPIRATION_HOURS);

        Self {
            token,
            client_id,
            user_id,
            scope,
            expires_at,
            created_at: now,
        }
    }

    /// Creates an access token with custom expiration.
    pub fn with_expiration(
        token: String,
        client_id: ClientId,
        user_id: UserId,
        scope: Scopes,
        expires_in_seconds: i64,
    ) -> Self {
        let now = Utc::now();
        let expires_at = now + Duration::seconds(expires_in_seconds);

        Self {
            token,
            client_id,
            user_id,
            scope,
            expires_at,
            created_at: now,
        }
    }

    /// Creates an AccessToken from existing data (e.g., from database).
    pub fn from_existing(
        token: String,
        client_id: ClientId,
        user_id: UserId,
        scope: Scopes,
        expires_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            token,
            client_id,
            user_id,
            scope,
            expires_at,
            created_at,
        }
    }

    /// Returns the token string.
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Returns the client ID.
    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    /// Returns the user ID.
    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    /// Returns the granted scopes.
    pub fn scope(&self) -> &Scopes {
        &self.scope
    }

    /// Returns the expiration timestamp.
    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }

    /// Returns the creation timestamp.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Checks if the token has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Returns the remaining time in seconds until expiration.
    pub fn expires_in(&self) -> i64 {
        let now = Utc::now();
        if self.expires_at > now {
            (self.expires_at - now).num_seconds()
        } else {
            0
        }
    }

    /// Validates the access token.
    ///
    /// # Returns
    /// Ok(()) if valid
    ///
    /// # Errors
    /// - `TokenError::Expired` if the token has expired
    pub fn validate(&self) -> Result<(), TokenError> {
        if self.is_expired() {
            return Err(TokenError::Expired);
        }
        Ok(())
    }

    /// Validates if the token has sufficient scope.
    ///
    /// # Arguments
    /// * `required_scope` - The scope required for the operation
    ///
    /// # Returns
    /// Ok(()) if token has the required scope
    ///
    /// # Errors
    /// Returns TokenError::InsufficientScope if scope is missing
    pub fn validate_scope(&self, required_scope: &Scopes) -> Result<(), TokenError> {
        if self.scope.contains_all(required_scope) {
            Ok(())
        } else {
            Err(TokenError::InsufficientScope {
                required: required_scope.to_string(),
                actual: self.scope.to_string(),
            })
        }
    }
}

impl RefreshToken {
    /// Default expiration time for refresh tokens (30 days).
    pub const DEFAULT_EXPIRATION_DAYS: i64 = 30;

    /// Generates a new refresh token.
    ///
    /// # Arguments
    /// * `access_token_id` - ID of the associated access token
    /// * `client_id` - The client this token was issued to
    /// * `user_id` - The user this token represents
    /// * `scope` - The granted scopes
    ///
    /// # Examples
    /// ```
    /// use domain::entities::RefreshToken;
    /// use domain::value_objects::{ClientId, UserId, Scopes};
    ///
    /// let token = RefreshToken::generate(
    ///     "access-token-id".to_string(),
    ///     ClientId::new(),
    ///     UserId::new(),
    ///     Scopes::parse("openid profile").unwrap(),
    /// );
    /// ```
    pub fn generate(
        access_token_id: String,
        client_id: ClientId,
        user_id: UserId,
        scope: Scopes,
    ) -> Self {
        use rand::Rng;

        // Generate a cryptographically secure random token
        let random_bytes: Vec<u8> = (0..32).map(|_| rand::rng().random()).collect();
        let token = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&random_bytes);

        let now = Utc::now();
        let expires_at = now + Duration::days(Self::DEFAULT_EXPIRATION_DAYS);

        Self {
            token,
            access_token_id,
            client_id,
            user_id,
            scope,
            expires_at,
            created_at: now,
            revoked: false,
        }
    }

    /// Creates a RefreshToken from existing data (e.g., from database).
    #[allow(clippy::too_many_arguments)]
    pub fn from_existing(
        token: String,
        access_token_id: String,
        client_id: ClientId,
        user_id: UserId,
        scope: Scopes,
        expires_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
        revoked: bool,
    ) -> Self {
        Self {
            token,
            access_token_id,
            client_id,
            user_id,
            scope,
            expires_at,
            created_at,
            revoked,
        }
    }

    /// Returns the token string.
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Returns the associated access token ID.
    pub fn access_token_id(&self) -> &str {
        &self.access_token_id
    }

    /// Returns the client ID.
    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    /// Returns the user ID.
    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    /// Returns the granted scopes.
    pub fn scope(&self) -> &Scopes {
        &self.scope
    }

    /// Returns the expiration timestamp.
    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }

    /// Returns the creation timestamp.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Returns whether the token has been revoked.
    pub fn is_revoked(&self) -> bool {
        self.revoked
    }

    /// Checks if the token has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Validates the refresh token.
    ///
    /// # Returns
    /// Ok(()) if valid
    ///
    /// # Errors
    /// - `TokenError::Expired` if expired
    /// - `TokenError::Revoked` if revoked
    pub fn validate(&self) -> Result<(), TokenError> {
        if self.revoked {
            return Err(TokenError::Revoked);
        }

        if self.is_expired() {
            return Err(TokenError::Expired);
        }

        Ok(())
    }

    /// Revokes the refresh token.
    ///
    /// # Business Rules
    /// - Once revoked, the token cannot be used again
    /// - This is permanent and cannot be undone
    pub fn revoke(&mut self) {
        self.revoked = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_access_token() {
        let token = AccessToken::new(
            "test-token".to_string(),
            ClientId::new(),
            UserId::new(),
            Scopes::parse("openid profile").unwrap(),
        );

        assert_eq!(token.token(), "test-token");
        assert!(!token.is_expired());
    }

    #[test]
    fn should_validate_access_token() {
        let token = AccessToken::new(
            "test-token".to_string(),
            ClientId::new(),
            UserId::new(),
            Scopes::parse("openid profile email").unwrap(),
        );

        assert!(token.validate().is_ok());
    }

    #[test]
    fn should_validate_sufficient_scope() {
        let token = AccessToken::new(
            "test-token".to_string(),
            ClientId::new(),
            UserId::new(),
            Scopes::parse("openid profile email").unwrap(),
        );

        let required = Scopes::parse("openid profile").unwrap();
        assert!(token.validate_scope(&required).is_ok());
    }

    #[test]
    fn should_reject_insufficient_scope() {
        let token = AccessToken::new(
            "test-token".to_string(),
            ClientId::new(),
            UserId::new(),
            Scopes::parse("openid profile").unwrap(),
        );

        let required = Scopes::parse("openid email").unwrap();
        assert!(matches!(
            token.validate_scope(&required),
            Err(TokenError::InsufficientScope { .. })
        ));
    }

    #[test]
    fn should_calculate_expires_in() {
        let token = AccessToken::new(
            "test-token".to_string(),
            ClientId::new(),
            UserId::new(),
            Scopes::parse("openid").unwrap(),
        );

        let expires_in = token.expires_in();
        assert!(expires_in > 3500); // Slightly less than 1 hour
        assert!(expires_in <= 3600); // Not more than 1 hour
    }

    #[test]
    fn should_generate_unique_refresh_tokens() {
        let token1 = RefreshToken::generate(
            "access1".to_string(),
            ClientId::new(),
            UserId::new(),
            Scopes::parse("openid").unwrap(),
        );

        let token2 = RefreshToken::generate(
            "access2".to_string(),
            ClientId::new(),
            UserId::new(),
            Scopes::parse("openid").unwrap(),
        );

        assert_ne!(token1.token(), token2.token());
    }

    #[test]
    fn should_validate_refresh_token() {
        let token = RefreshToken::generate(
            "access-token-id".to_string(),
            ClientId::new(),
            UserId::new(),
            Scopes::parse("openid").unwrap(),
        );

        assert!(token.validate().is_ok());
        assert!(!token.is_expired());
        assert!(!token.is_revoked());
    }

    #[test]
    fn should_revoke_refresh_token() {
        let mut token = RefreshToken::generate(
            "access-token-id".to_string(),
            ClientId::new(),
            UserId::new(),
            Scopes::parse("openid").unwrap(),
        );

        assert!(!token.is_revoked());

        token.revoke();

        assert!(token.is_revoked());
        assert!(matches!(token.validate(), Err(TokenError::Revoked)));
    }

    #[test]
    fn should_create_access_token_with_custom_expiration() {
        let token = AccessToken::with_expiration(
            "test-token".to_string(),
            ClientId::new(),
            UserId::new(),
            Scopes::parse("openid").unwrap(),
            300, // 5 minutes
        );

        let expires_in = token.expires_in();
        assert!(expires_in > 200); // More than 3 minutes
        assert!(expires_in <= 300); // Not more than 5 minutes
    }
}
