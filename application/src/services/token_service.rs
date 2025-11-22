//! Token Service Implementation
//!
//! Implements TokenUseCase for OAuth 2.0 token management.

use crate::errors::UseCaseError;
use crate::ports::{
    AccessTokenRepository, AuthCodeRepository, ClientData, ClientRepository,
    ExchangeCodeInput, IntrospectTokenInput, IntrospectTokenOutput, RefreshTokenInput,
    RefreshTokenRepository, RevokeTokenInput, TokenOutput, TokenUseCase,
};
use async_trait::async_trait;
use domain::entities::{AccessToken, AuthorizationCode, RefreshToken};
use domain::value_objects::{CodeVerifier, Scopes};
use std::sync::Arc;

/// Service implementation for OAuth token management
///
/// # Type Parameters
/// * `CR` - The client repository implementation
/// * `AR` - The auth code repository implementation
/// * `ATR` - The access token repository implementation
/// * `RTR` - The refresh token repository implementation
pub struct TokenService<
    CR: ClientRepository,
    AR: AuthCodeRepository,
    ATR: AccessTokenRepository,
    RTR: RefreshTokenRepository,
> {
    client_repository: Arc<CR>,
    auth_code_repository: Arc<AR>,
    access_token_repository: Arc<ATR>,
    refresh_token_repository: Arc<RTR>,
}

impl<
        CR: ClientRepository,
        AR: AuthCodeRepository,
        ATR: AccessTokenRepository,
        RTR: RefreshTokenRepository,
    > TokenService<CR, AR, ATR, RTR>
{
    /// Creates a new TokenService
    pub fn new(
        client_repository: Arc<CR>,
        auth_code_repository: Arc<AR>,
        access_token_repository: Arc<ATR>,
        refresh_token_repository: Arc<RTR>,
    ) -> Self {
        Self {
            client_repository,
            auth_code_repository,
            access_token_repository,
            refresh_token_repository,
        }
    }

    /// Authenticates a confidential client
    fn authenticate_confidential_client(
        client_data: &ClientData,
        provided_secret: Option<&String>,
    ) -> Result<(), UseCaseError> {
        if client_data.is_confidential() {
            let provided = provided_secret.ok_or(UseCaseError::Unauthorized)?;
            let stored = client_data
                .secret
                .as_ref()
                .ok_or(UseCaseError::Internal("Missing client secret".to_string()))?;

            if !stored.verify(provided) {
                return Err(UseCaseError::Unauthorized);
            }
        }
        Ok(())
    }
}

#[async_trait]
impl<
        CR: ClientRepository,
        AR: AuthCodeRepository,
        ATR: AccessTokenRepository,
        RTR: RefreshTokenRepository,
    > TokenUseCase for TokenService<CR, AR, ATR, RTR>
{
    async fn exchange_code(&self, input: ExchangeCodeInput) -> Result<TokenOutput, UseCaseError> {
        // 1. Find and validate authorization code
        let mut auth_code: AuthorizationCode = self
            .auth_code_repository
            .find_by_code(&input.code)
            .await?
            .ok_or(UseCaseError::NotFound)?;

        // Check if already used
        if auth_code.is_used() {
            return Err(UseCaseError::Validation(
                "Authorization code already used".to_string(),
            ));
        }

        // Check if expired
        if auth_code.is_expired() {
            return Err(UseCaseError::Validation(
                "Authorization code expired".to_string(),
            ));
        }

        // Validate client_id
        if auth_code.client_id() != input.client_id {
            return Err(UseCaseError::Unauthorized);
        }

        // Validate redirect_uri
        if auth_code.redirect_uri() != input.redirect_uri.as_str() {
            return Err(UseCaseError::Validation(
                "Redirect URI mismatch".to_string(),
            ));
        }

        // 2. Find and authenticate client
        let client_data: ClientData = self
            .client_repository
            .find_by_id(input.client_id)
            .await?
            .ok_or(UseCaseError::NotFound)?;

        Self::authenticate_confidential_client(&client_data, input.client_secret.as_ref())?;

        // 3. Verify PKCE if present
        if let Some(challenge) = auth_code.code_challenge() {
            let verifier_string = input.code_verifier.ok_or_else(|| {
                UseCaseError::Validation("PKCE verifier required".to_string())
            })?;

            let verifier: CodeVerifier = CodeVerifier::new(verifier_string)
                .map_err(|e| UseCaseError::Validation(e.to_string()))?;

            if !challenge
                .verify(&verifier)
                .map_err(|e| UseCaseError::Validation(e.to_string()))?
            {
                return Err(UseCaseError::Unauthorized);
            }
        }

        // 4. Generate tokens
        let access_token_string: String = uuid::Uuid::new_v4().to_string();
        let access_token: AccessToken = AccessToken::new(
            access_token_string.clone(),
            auth_code.client_id(),
            auth_code.user_id(),
            auth_code.scope().clone(),
        );

        let refresh_token: RefreshToken = RefreshToken::generate(
            access_token_string.clone(),
            auth_code.client_id(),
            auth_code.user_id(),
            auth_code.scope().clone(),
        );

        let refresh_token_string: String = refresh_token.token().to_string();

        // 5. Save tokens
        self.access_token_repository.save(&access_token).await?;
        self.refresh_token_repository.save(&refresh_token).await?;

        // 6. Mark authorization code as used
        auth_code.mark_as_used();
        self.auth_code_repository
            .mark_as_used(&input.code)
            .await?;

        // 7. Return token response
        Ok(TokenOutput {
            access_token: access_token_string,
            token_type: "Bearer".to_string(),
            expires_in: access_token.expires_in(),
            refresh_token: Some(refresh_token_string),
            scope: auth_code.scope().to_string(),
        })
    }

    async fn refresh_token(
        &self,
        input: RefreshTokenInput,
    ) -> Result<TokenOutput, UseCaseError> {
        // 1. Find refresh token
        let refresh_token: RefreshToken = self
            .refresh_token_repository
            .find_by_token(&input.refresh_token)
            .await?
            .ok_or(UseCaseError::NotFound)?;

        // 2. Validate refresh token
        if refresh_token.is_revoked() {
            return Err(UseCaseError::Validation("Token revoked".to_string()));
        }

        if refresh_token.is_expired() {
            return Err(UseCaseError::Validation("Token expired".to_string()));
        }

        if refresh_token.client_id() != input.client_id {
            return Err(UseCaseError::Unauthorized);
        }

        // 3. Authenticate client
        let client_data: ClientData = self
            .client_repository
            .find_by_id(input.client_id)
            .await?
            .ok_or(UseCaseError::NotFound)?;

        Self::authenticate_confidential_client(&client_data, input.client_secret.as_ref())?;

        // 4. Handle scope narrowing (if requested)
        let scope: Scopes = if let Some(requested_scope) = input.scope {
            let requested: Scopes =
                Scopes::parse(&requested_scope).map_err(|e| UseCaseError::Validation(e.to_string()))?;
            if !refresh_token.scope().contains_all(&requested) {
                return Err(UseCaseError::Validation(
                    "Requested scope exceeds granted scope".to_string(),
                ));
            }
            requested
        } else {
            refresh_token.scope().clone()
        };

        // 5. Generate new access token
        let access_token_string: String = uuid::Uuid::new_v4().to_string();
        let access_token: AccessToken = AccessToken::new(
            access_token_string.clone(),
            refresh_token.client_id(),
            refresh_token.user_id(),
            scope.clone(),
        );

        self.access_token_repository.save(&access_token).await?;

        // 6. Return token response (refresh token rotation not implemented)
        Ok(TokenOutput {
            access_token: access_token_string,
            token_type: "Bearer".to_string(),
            expires_in: access_token.expires_in(),
            refresh_token: Some(refresh_token.token().to_string()),
            scope: scope.to_string(),
        })
    }

    async fn revoke_token(&self, input: RevokeTokenInput) -> Result<(), UseCaseError> {
        // 1. Authenticate client
        let client_data: ClientData = self
            .client_repository
            .find_by_id(input.client_id)
            .await?
            .ok_or(UseCaseError::NotFound)?;

        Self::authenticate_confidential_client(&client_data, input.client_secret.as_ref())?;

        // 2. Try to revoke as refresh token first (most common)
        if let Ok(Some(refresh_token)) = self
            .refresh_token_repository
            .find_by_token(&input.token)
            .await
        {
            if refresh_token.client_id() == input.client_id {
                let _ = self.refresh_token_repository.revoke(&input.token).await;
                return Ok(());
            }
        }

        // 3. Try to revoke as access token
        if let Ok(Some(access_token)) = self
            .access_token_repository
            .find_by_token(&input.token)
            .await
        {
            if access_token.client_id() == input.client_id {
                let _ = self.access_token_repository.revoke(&input.token).await;
                return Ok(());
            }
        }

        // Revocation is idempotent - succeed even if token not found
        Ok(())
    }

    async fn introspect_token(
        &self,
        input: IntrospectTokenInput,
    ) -> Result<IntrospectTokenOutput, UseCaseError> {
        // 1. Authenticate client
        let client_data: ClientData = self
            .client_repository
            .find_by_id(input.client_id)
            .await?
            .ok_or(UseCaseError::NotFound)?;

        Self::authenticate_confidential_client(&client_data, input.client_secret.as_ref())?;

        // 2. Try to find as access token
        if let Ok(Some(access_token)) = self
            .access_token_repository
            .find_by_token(&input.token)
            .await
        {
            let active = !access_token.is_expired();
            return Ok(IntrospectTokenOutput {
                active,
                scope: Some(access_token.scope().to_string()),
                client_id: Some(access_token.client_id()),
                user_id: Some(access_token.user_id()),
                exp: Some(access_token.expires_at().timestamp()),
            });
        }

        // 3. Try to find as refresh token
        if let Ok(Some(refresh_token)) = self
            .refresh_token_repository
            .find_by_token(&input.token)
            .await
        {
            let active = !refresh_token.is_expired() && !refresh_token.is_revoked();
            return Ok(IntrospectTokenOutput {
                active,
                scope: Some(refresh_token.scope().to_string()),
                client_id: Some(refresh_token.client_id()),
                user_id: Some(refresh_token.user_id()),
                exp: Some(refresh_token.expires_at().timestamp()),
            });
        }

        // Token not found - return inactive
        Ok(IntrospectTokenOutput {
            active: false,
            scope: None,
            client_id: None,
            user_id: None,
            exp: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::RepositoryError;
    use crate::ports::AuthUseCase;
    use crate::services::AuthService;
    use async_trait::async_trait;
    use domain::entities::{ClientSecret, Confidential, OAuthClient, Public};
    use domain::value_objects::{ClientId, UserId};
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Mock repositories (simplified for testing)
    struct MockClientRepository {
        clients: Mutex<HashMap<ClientId, ClientData>>,
    }

    impl MockClientRepository {
        fn new() -> Self {
            Self {
                clients: Mutex::new(HashMap::new()),
            }
        }

        fn add_confidential(&self, client: &OAuthClient<Confidential>) {
            let mut clients = self.clients.lock().unwrap();
            clients.insert(
                client.id(),
                ClientData {
                    id: client.id(),
                    name: client.name().to_string(),
                    redirect_uris: client.redirect_uris().to_vec(),
                    allowed_scopes: client.allowed_scopes().clone(),
                    secret: client.secret().cloned(),
                    created_at: client.created_at(),
                },
            );
        }
    }

    #[async_trait]
    impl ClientRepository for MockClientRepository {
        async fn save_public(&self, _: &OAuthClient<Public>) -> Result<(), RepositoryError> {
            Ok(())
        }
        async fn save_confidential(
            &self,
            _: &OAuthClient<Confidential>,
        ) -> Result<(), RepositoryError> {
            Ok(())
        }
        async fn find_by_id(&self, id: ClientId) -> Result<Option<ClientData>, RepositoryError> {
            Ok(self.clients.lock().unwrap().get(&id).cloned())
        }
        async fn delete(&self, _: ClientId) -> Result<(), RepositoryError> {
            Ok(())
        }
    }

    struct MockAuthCodeRepository {
        codes: Mutex<HashMap<String, AuthorizationCode>>,
    }

    impl MockAuthCodeRepository {
        fn new() -> Self {
            Self {
                codes: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl AuthCodeRepository for MockAuthCodeRepository {
        async fn save(&self, code: &AuthorizationCode) -> Result<(), RepositoryError> {
            self.codes
                .lock()
                .unwrap()
                .insert(code.code().to_string(), code.clone());
            Ok(())
        }
        async fn find_by_code(&self, code: &str) -> Result<Option<AuthorizationCode>, RepositoryError> {
            Ok(self.codes.lock().unwrap().get(code).cloned())
        }
        async fn mark_as_used(&self, code: &str) -> Result<(), RepositoryError> {
            if let Some(auth_code) = self.codes.lock().unwrap().get_mut(code) {
                auth_code.mark_as_used();
            }
            Ok(())
        }
        async fn delete(&self, _: &str) -> Result<(), RepositoryError> {
            Ok(())
        }
        async fn delete_expired(&self) -> Result<usize, RepositoryError> {
            Ok(0)
        }
    }

    struct MockAccessTokenRepository {
        tokens: Mutex<HashMap<String, AccessToken>>,
    }

    impl MockAccessTokenRepository {
        fn new() -> Self {
            Self {
                tokens: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl AccessTokenRepository for MockAccessTokenRepository {
        async fn save(&self, token: &AccessToken) -> Result<(), RepositoryError> {
            self.tokens
                .lock()
                .unwrap()
                .insert(token.token().to_string(), token.clone());
            Ok(())
        }
        async fn find_by_token(&self, token: &str) -> Result<Option<AccessToken>, RepositoryError> {
            Ok(self.tokens.lock().unwrap().get(token).cloned())
        }
        async fn find_by_user_id(&self, _: UserId) -> Result<Vec<AccessToken>, RepositoryError> {
            Ok(vec![])
        }
        async fn revoke(&self, _: &str) -> Result<(), RepositoryError> {
            Ok(())
        }
        async fn revoke_all_for_user(&self, _: UserId) -> Result<usize, RepositoryError> {
            Ok(0)
        }
        async fn delete_expired(&self) -> Result<usize, RepositoryError> {
            Ok(0)
        }
    }

    struct MockRefreshTokenRepository {
        tokens: Mutex<HashMap<String, RefreshToken>>,
    }

    impl MockRefreshTokenRepository {
        fn new() -> Self {
            Self {
                tokens: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl RefreshTokenRepository for MockRefreshTokenRepository {
        async fn save(&self, token: &RefreshToken) -> Result<(), RepositoryError> {
            self.tokens
                .lock()
                .unwrap()
                .insert(token.token().to_string(), token.clone());
            Ok(())
        }
        async fn find_by_token(&self, token: &str) -> Result<Option<RefreshToken>, RepositoryError> {
            Ok(self.tokens.lock().unwrap().get(token).cloned())
        }
        async fn find_by_user_id(&self, _: UserId) -> Result<Vec<RefreshToken>, RepositoryError> {
            Ok(vec![])
        }
        async fn find_by_access_token_id(
            &self,
            _: &str,
        ) -> Result<Option<RefreshToken>, RepositoryError> {
            Ok(None)
        }
        async fn revoke(&self, _: &str) -> Result<(), RepositoryError> {
            Ok(())
        }
        async fn revoke_all_for_user(&self, _: UserId) -> Result<usize, RepositoryError> {
            Ok(0)
        }
        async fn revoke_all_for_client(&self, _: ClientId) -> Result<usize, RepositoryError> {
            Ok(0)
        }
        async fn delete_expired_and_revoked(&self) -> Result<usize, RepositoryError> {
            Ok(0)
        }
    }

    #[tokio::test]
    async fn should_exchange_code_for_tokens() {
        let client_repo = Arc::new(MockClientRepository::new());
        let auth_code_repo = Arc::new(MockAuthCodeRepository::new());
        let access_token_repo = Arc::new(MockAccessTokenRepository::new());
        let refresh_token_repo = Arc::new(MockRefreshTokenRepository::new());

        let auth_service = AuthService::new(client_repo.clone(), auth_code_repo.clone());
        let token_service = TokenService::new(
            client_repo.clone(),
            auth_code_repo,
            access_token_repo,
            refresh_token_repo,
        );

        // Setup client
        let client_id = ClientId::new();
        let client_secret = ClientSecret::generate();
        let secret_string = client_secret.as_str().to_string();

        let client = OAuthClient::<Confidential>::new(
            client_id,
            "Test Server",
            client_secret,
            vec!["https://example.com/callback".to_string()],
            Scopes::parse("openid profile").unwrap(),
        );
        client_repo.add_confidential(&client);

        // Create authorization code
        let auth_result = auth_service
            .authorize(crate::ports::AuthorizeInput {
                client_id,
                user_id: UserId::new(),
                redirect_uri: "https://example.com/callback".to_string(),
                scope: "openid profile".to_string(),
                code_challenge: None,
                code_challenge_method: None,
                state: None,
            })
            .await
            .unwrap();

        // Exchange code for tokens
        let token_result = token_service
            .exchange_code(ExchangeCodeInput {
                code: auth_result.authorization_code,
                client_id,
                client_secret: Some(secret_string),
                redirect_uri: "https://example.com/callback".to_string(),
                code_verifier: None,
            })
            .await;

        assert!(token_result.is_ok());
        let tokens = token_result.unwrap();
        assert!(!tokens.access_token.is_empty());
        assert!(tokens.refresh_token.is_some());
        assert_eq!(tokens.token_type, "Bearer");
    }
}
