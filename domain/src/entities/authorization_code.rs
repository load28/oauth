use crate::errors::AuthCodeError;
use crate::value_objects::{ClientId, CodeChallenge, CodeVerifier, S256, Scopes, UserId};
use base64::Engine;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Authorization Code entity
///
/// # Invariants
/// - Code is unique and single-use
/// - Has expiration time (typically 10 minutes)
/// - Contains PKCE challenge for public clients
/// - Bound to specific client, user, and redirect URI
///
/// # Standards
/// - RFC 6749 Section 4.1.2 (Authorization Code)
/// - RFC 7636 (PKCE)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationCode {
    code: String,
    client_id: ClientId,
    user_id: UserId,
    redirect_uri: String,
    scope: Scopes,
    code_challenge: Option<CodeChallenge<S256>>,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    used: bool,
}

impl AuthorizationCode {
    /// Default expiration time for authorization codes (10 minutes).
    const DEFAULT_EXPIRATION_MINUTES: i64 = 10;

    /// Generates a new authorization code.
    ///
    /// # Arguments
    /// * `client_id` - The client requesting authorization
    /// * `user_id` - The user granting authorization
    /// * `redirect_uri` - The redirect URI to use
    /// * `scope` - The granted scopes
    /// * `code_challenge` - Optional PKCE code challenge
    ///
    /// # Examples
    /// ```
    /// use domain::entities::AuthorizationCode;
    /// use domain::value_objects::{ClientId, UserId, Scopes, CodeVerifier};
    ///
    /// let verifier = CodeVerifier::generate();
    /// let challenge = verifier.create_s256_challenge();
    ///
    /// let code = AuthorizationCode::generate(
    ///     ClientId::new(),
    ///     UserId::new(),
    ///     "http://localhost:3000/callback".to_string(),
    ///     Scopes::parse("openid profile").unwrap(),
    ///     Some(challenge),
    /// );
    /// ```
    pub fn generate(
        client_id: ClientId,
        user_id: UserId,
        redirect_uri: String,
        scope: Scopes,
        code_challenge: Option<CodeChallenge<S256>>,
    ) -> Self {
        use rand::Rng;

        // Generate a cryptographically secure random code
        let random_bytes: Vec<u8> = (0..32).map(|_| rand::rng().random()).collect();
        let code = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&random_bytes);

        let now = Utc::now();
        let expires_at = now + Duration::minutes(Self::DEFAULT_EXPIRATION_MINUTES);

        Self {
            code,
            client_id,
            user_id,
            redirect_uri,
            scope,
            code_challenge,
            expires_at,
            created_at: now,
            used: false,
        }
    }

    /// Creates an AuthorizationCode from existing data (e.g., from database).
    #[allow(clippy::too_many_arguments)]
    pub fn from_existing(
        code: String,
        client_id: ClientId,
        user_id: UserId,
        redirect_uri: String,
        scope: Scopes,
        code_challenge: Option<CodeChallenge<S256>>,
        expires_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
        used: bool,
    ) -> Self {
        Self {
            code,
            client_id,
            user_id,
            redirect_uri,
            scope,
            code_challenge,
            expires_at,
            created_at,
            used,
        }
    }

    /// Returns the authorization code string.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Returns the client ID.
    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    /// Returns the user ID.
    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    /// Returns the redirect URI.
    pub fn redirect_uri(&self) -> &str {
        &self.redirect_uri
    }

    /// Returns the granted scopes.
    pub fn scope(&self) -> &Scopes {
        &self.scope
    }

    /// Returns the PKCE code challenge if present.
    pub fn code_challenge(&self) -> Option<&CodeChallenge<S256>> {
        self.code_challenge.as_ref()
    }

    /// Returns the expiration timestamp.
    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }

    /// Returns the creation timestamp.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Returns whether the code has been used.
    pub fn is_used(&self) -> bool {
        self.used
    }

    /// Checks if the authorization code has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Validates the authorization code before exchange.
    ///
    /// # Arguments
    /// * `client_id` - The client attempting to exchange
    /// * `redirect_uri` - The redirect URI provided
    /// * `code_verifier` - Optional PKCE code verifier
    ///
    /// # Returns
    /// Ok(()) if validation succeeds
    ///
    /// # Errors
    /// - `AuthCodeError::Expired` if the code has expired
    /// - `AuthCodeError::AlreadyUsed` if the code was already used
    /// - `AuthCodeError::PkceRequired` if PKCE is required but not provided
    /// - `AuthCodeError::InvalidCodeVerifier` if PKCE verification fails
    ///
    /// # Business Rules
    /// - Code must not be expired
    /// - Code must not be already used
    /// - Client ID must match
    /// - Redirect URI must match exactly
    /// - PKCE verification if challenge present
    pub fn validate(
        &self,
        client_id: ClientId,
        redirect_uri: &str,
        code_verifier: Option<&CodeVerifier>,
    ) -> Result<(), AuthCodeError> {
        // Check expiration
        if self.is_expired() {
            return Err(AuthCodeError::Expired);
        }

        // Check if already used
        if self.used {
            return Err(AuthCodeError::AlreadyUsed);
        }

        // Verify client ID matches
        if self.client_id != client_id {
            return Err(AuthCodeError::NotFound);
        }

        // Verify redirect URI matches exactly
        if self.redirect_uri != redirect_uri {
            return Err(AuthCodeError::NotFound);
        }

        // PKCE verification
        match (&self.code_challenge, code_verifier) {
            (Some(challenge), Some(verifier)) => {
                // Challenge present, verifier provided - verify
                if !challenge.verify(verifier).unwrap_or(false) {
                    return Err(AuthCodeError::InvalidCodeVerifier);
                }
            }
            (Some(_), None) => {
                // Challenge present but no verifier - error
                return Err(AuthCodeError::PkceRequired);
            }
            (None, _) => {
                // No challenge - no verification needed (confidential client)
            }
        }

        Ok(())
    }

    /// Marks the authorization code as used.
    ///
    /// # Business Rules
    /// - Code can only be used once (prevents replay attacks)
    /// - This should be called after successful token exchange
    pub fn mark_as_used(&mut self) {
        self.used = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_code_with_pkce() -> (AuthorizationCode, CodeVerifier) {
        let verifier = CodeVerifier::generate();
        let challenge = verifier.create_s256_challenge();

        let code = AuthorizationCode::generate(
            ClientId::new(),
            UserId::new(),
            "http://localhost:3000/callback".to_string(),
            Scopes::parse("openid profile").unwrap(),
            Some(challenge),
        );

        (code, verifier)
    }

    #[test]
    fn should_generate_unique_codes() {
        let code1 = AuthorizationCode::generate(
            ClientId::new(),
            UserId::new(),
            "http://localhost:3000/callback".to_string(),
            Scopes::parse("openid").unwrap(),
            None,
        );

        let code2 = AuthorizationCode::generate(
            ClientId::new(),
            UserId::new(),
            "http://localhost:3000/callback".to_string(),
            Scopes::parse("openid").unwrap(),
            None,
        );

        assert_ne!(code1.code(), code2.code());
    }

    #[test]
    fn should_not_be_expired_immediately() {
        let code = AuthorizationCode::generate(
            ClientId::new(),
            UserId::new(),
            "http://localhost:3000/callback".to_string(),
            Scopes::parse("openid").unwrap(),
            None,
        );

        assert!(!code.is_expired());
    }

    #[test]
    fn should_validate_with_correct_pkce() {
        let (code, verifier) = create_test_code_with_pkce();

        let result = code.validate(
            code.client_id(),
            code.redirect_uri(),
            Some(&verifier),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn should_reject_wrong_pkce_verifier() {
        let (code, _) = create_test_code_with_pkce();
        let wrong_verifier = CodeVerifier::generate();

        let result = code.validate(
            code.client_id(),
            code.redirect_uri(),
            Some(&wrong_verifier),
        );

        assert!(matches!(result, Err(AuthCodeError::InvalidCodeVerifier)));
    }

    #[test]
    fn should_reject_missing_pkce_verifier() {
        let (code, _) = create_test_code_with_pkce();

        let result = code.validate(
            code.client_id(),
            code.redirect_uri(),
            None,
        );

        assert!(matches!(result, Err(AuthCodeError::PkceRequired)));
    }

    #[test]
    fn should_reject_wrong_client_id() {
        let (code, verifier) = create_test_code_with_pkce();
        let wrong_client_id = ClientId::new();

        let result = code.validate(
            wrong_client_id,
            code.redirect_uri(),
            Some(&verifier),
        );

        assert!(matches!(result, Err(AuthCodeError::NotFound)));
    }

    #[test]
    fn should_reject_wrong_redirect_uri() {
        let (code, verifier) = create_test_code_with_pkce();

        let result = code.validate(
            code.client_id(),
            "http://evil.com/callback",
            Some(&verifier),
        );

        assert!(matches!(result, Err(AuthCodeError::NotFound)));
    }

    #[test]
    fn should_mark_as_used() {
        let (mut code, _) = create_test_code_with_pkce();
        assert!(!code.is_used());

        code.mark_as_used();
        assert!(code.is_used());
    }

    #[test]
    fn should_reject_already_used_code() {
        let (mut code, verifier) = create_test_code_with_pkce();
        code.mark_as_used();

        let result = code.validate(
            code.client_id(),
            code.redirect_uri(),
            Some(&verifier),
        );

        assert!(matches!(result, Err(AuthCodeError::AlreadyUsed)));
    }

    #[test]
    fn should_allow_confidential_client_without_pkce() {
        let code = AuthorizationCode::generate(
            ClientId::new(),
            UserId::new(),
            "https://example.com/callback".to_string(),
            Scopes::parse("openid").unwrap(),
            None, // No PKCE challenge
        );

        let result = code.validate(
            code.client_id(),
            code.redirect_uri(),
            None, // No verifier
        );

        assert!(result.is_ok());
    }
}
