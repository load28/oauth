//! Authorization Service Implementation
//!
//! Implements AuthUseCase for OAuth 2.0 authorization flow.

use crate::errors::UseCaseError;
use crate::ports::{
    AuthCodeRepository, AuthUseCase, AuthorizeInput, AuthorizeOutput, ClientData,
    ClientRepository,
};
use async_trait::async_trait;
use domain::entities::AuthorizationCode;
use domain::value_objects::{CodeChallenge, Scopes, S256};
use std::sync::Arc;

/// Service implementation for OAuth authorization flow
///
/// # Type Parameters
/// * `CR` - The client repository implementation
/// * `AR` - The auth code repository implementation
///
/// # Business Logic
/// - Validates client exists and is authorized
/// - Enforces PKCE for public clients
/// - Validates redirect URIs and scopes
/// - Generates single-use authorization codes
pub struct AuthService<CR: ClientRepository, AR: AuthCodeRepository> {
    client_repository: Arc<CR>,
    auth_code_repository: Arc<AR>,
}

impl<CR: ClientRepository, AR: AuthCodeRepository> AuthService<CR, AR> {
    /// Creates a new AuthService
    ///
    /// # Arguments
    /// * `client_repository` - Repository for retrieving clients
    /// * `auth_code_repository` - Repository for storing authorization codes
    pub fn new(client_repository: Arc<CR>, auth_code_repository: Arc<AR>) -> Self {
        Self {
            client_repository,
            auth_code_repository,
        }
    }
}

#[async_trait]
impl<CR: ClientRepository, AR: AuthCodeRepository> AuthUseCase for AuthService<CR, AR> {
    async fn authorize(&self, input: AuthorizeInput) -> Result<AuthorizeOutput, UseCaseError> {
        // 1. Find client
        let client_data: ClientData = self
            .client_repository
            .find_by_id(input.client_id)
            .await?
            .ok_or(UseCaseError::NotFound)?;

        // 2. Validate redirect URI
        if !client_data
            .redirect_uris
            .contains(&input.redirect_uri)
        {
            return Err(UseCaseError::Validation(format!(
                "Invalid redirect URI: {}",
                input.redirect_uri
            )));
        }

        // 3. Parse and validate scopes
        let requested_scopes: Scopes =
            Scopes::parse(&input.scope).map_err(|e| UseCaseError::Validation(e.to_string()))?;

        if !client_data
            .allowed_scopes
            .contains_all(&requested_scopes)
        {
            return Err(UseCaseError::Validation(format!(
                "Requested scopes not allowed: {}",
                input.scope
            )));
        }

        // 4. Handle PKCE based on client type
        let code_challenge: Option<CodeChallenge<S256>> = if client_data.is_public() {
            // Public clients MUST provide PKCE
            let challenge_string = input.code_challenge.ok_or_else(|| {
                UseCaseError::Validation("PKCE is required for public clients".to_string())
            })?;

            let method = input.code_challenge_method.ok_or_else(|| {
                UseCaseError::Validation("PKCE method is required".to_string())
            })?;

            // We only support S256 for public clients (more secure)
            if method.to_lowercase() != "s256" {
                return Err(UseCaseError::Validation(
                    "Only S256 PKCE method is supported for public clients".to_string(),
                ));
            }

            Some(CodeChallenge::new_s256(challenge_string))
        } else {
            // Confidential clients MAY provide PKCE (optional but recommended)
            if let (Some(challenge_string), Some(method)) =
                (input.code_challenge, input.code_challenge_method)
            {
                if method.to_lowercase() == "s256" {
                    Some(CodeChallenge::new_s256(challenge_string))
                } else {
                    return Err(UseCaseError::Validation(
                        "Only S256 PKCE method is supported".to_string(),
                    ));
                }
            } else {
                None
            }
        };

        // 5. Generate authorization code
        let auth_code: AuthorizationCode = AuthorizationCode::generate(
            input.client_id,
            input.user_id,
            input.redirect_uri.clone(),
            requested_scopes,
            code_challenge,
        );

        let code_string: String = auth_code.code().to_string();

        // 6. Persist authorization code
        self.auth_code_repository.save(&auth_code).await?;

        // 7. Return output
        Ok(AuthorizeOutput {
            authorization_code: code_string,
            redirect_uri: input.redirect_uri,
            state: input.state,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::RepositoryError;
    use async_trait::async_trait;
    use domain::entities::{ClientSecret, Confidential, OAuthClient, Public};
    use domain::value_objects::{ClientId, UserId};
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// Mock client repository
    struct MockClientRepository {
        clients: Mutex<HashMap<ClientId, ClientData>>,
    }

    impl MockClientRepository {
        fn new() -> Self {
            Self {
                clients: Mutex::new(HashMap::new()),
            }
        }

        fn add_public_client(&self, client: &OAuthClient<Public>) {
            let mut clients = self.clients.lock().unwrap();
            let client_data = ClientData {
                id: client.id(),
                name: client.name().to_string(),
                redirect_uris: client.redirect_uris().to_vec(),
                allowed_scopes: client.allowed_scopes().clone(),
                secret: None,
                created_at: client.created_at(),
            };
            clients.insert(client.id(), client_data);
        }

        fn add_confidential_client(&self, client: &OAuthClient<Confidential>) {
            let mut clients = self.clients.lock().unwrap();
            let client_data = ClientData {
                id: client.id(),
                name: client.name().to_string(),
                redirect_uris: client.redirect_uris().to_vec(),
                allowed_scopes: client.allowed_scopes().clone(),
                secret: client.secret().cloned(),
                created_at: client.created_at(),
            };
            clients.insert(client.id(), client_data);
        }
    }

    #[async_trait]
    impl ClientRepository for MockClientRepository {
        async fn save_public(&self, _client: &OAuthClient<Public>) -> Result<(), RepositoryError> {
            Ok(())
        }

        async fn save_confidential(
            &self,
            _client: &OAuthClient<Confidential>,
        ) -> Result<(), RepositoryError> {
            Ok(())
        }

        async fn find_by_id(&self, id: ClientId) -> Result<Option<ClientData>, RepositoryError> {
            let clients = self.clients.lock().unwrap();
            Ok(clients.get(&id).cloned())
        }

        async fn delete(&self, _id: ClientId) -> Result<(), RepositoryError> {
            Ok(())
        }
    }

    /// Mock auth code repository
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
        async fn save(&self, auth_code: &AuthorizationCode) -> Result<(), RepositoryError> {
            let mut codes = self.codes.lock().unwrap();
            codes.insert(auth_code.code().to_string(), auth_code.clone());
            Ok(())
        }

        async fn find_by_code(
            &self,
            code: &str,
        ) -> Result<Option<AuthorizationCode>, RepositoryError> {
            let codes = self.codes.lock().unwrap();
            Ok(codes.get(code).cloned())
        }

        async fn mark_as_used(&self, _code: &str) -> Result<(), RepositoryError> {
            Ok(())
        }

        async fn delete(&self, code: &str) -> Result<(), RepositoryError> {
            let mut codes = self.codes.lock().unwrap();
            codes.remove(code);
            Ok(())
        }

        async fn delete_expired(&self) -> Result<usize, RepositoryError> {
            Ok(0)
        }
    }

    #[tokio::test]
    async fn should_authorize_public_client_with_pkce() {
        let client_repo = Arc::new(MockClientRepository::new());
        let auth_code_repo = Arc::new(MockAuthCodeRepository::new());
        let service = AuthService::new(client_repo.clone(), auth_code_repo);

        // Setup public client
        let client_id = ClientId::new();
        let client = OAuthClient::<Public>::new(
            client_id,
            "Test SPA",
            vec!["http://localhost:3000/callback".to_string()],
            Scopes::parse("openid profile").unwrap(),
        );
        client_repo.add_public_client(&client);

        let input = AuthorizeInput {
            client_id,
            user_id: UserId::new(),
            redirect_uri: "http://localhost:3000/callback".to_string(),
            scope: "openid profile".to_string(),
            code_challenge: Some("test_challenge".to_string()),
            code_challenge_method: Some("S256".to_string()),
            state: Some("random_state".to_string()),
        };

        let result = service.authorize(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(!output.authorization_code.is_empty());
        assert_eq!(output.state, Some("random_state".to_string()));
    }

    #[tokio::test]
    async fn should_reject_public_client_without_pkce() {
        let client_repo = Arc::new(MockClientRepository::new());
        let auth_code_repo = Arc::new(MockAuthCodeRepository::new());
        let service = AuthService::new(client_repo.clone(), auth_code_repo);

        let client_id = ClientId::new();
        let client = OAuthClient::<Public>::new(
            client_id,
            "Test SPA",
            vec!["http://localhost:3000/callback".to_string()],
            Scopes::parse("openid").unwrap(),
        );
        client_repo.add_public_client(&client);

        let input = AuthorizeInput {
            client_id,
            user_id: UserId::new(),
            redirect_uri: "http://localhost:3000/callback".to_string(),
            scope: "openid".to_string(),
            code_challenge: None,
            code_challenge_method: None,
            state: None,
        };

        let result = service.authorize(input).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UseCaseError::Validation(_)));
    }

    #[tokio::test]
    async fn should_authorize_confidential_client_without_pkce() {
        let client_repo = Arc::new(MockClientRepository::new());
        let auth_code_repo = Arc::new(MockAuthCodeRepository::new());
        let service = AuthService::new(client_repo.clone(), auth_code_repo);

        let client_id = ClientId::new();
        let client = OAuthClient::<Confidential>::new(
            client_id,
            "Test Server App",
            ClientSecret::generate(),
            vec!["https://example.com/callback".to_string()],
            Scopes::parse("openid profile").unwrap(),
        );
        client_repo.add_confidential_client(&client);

        let input = AuthorizeInput {
            client_id,
            user_id: UserId::new(),
            redirect_uri: "https://example.com/callback".to_string(),
            scope: "openid profile".to_string(),
            code_challenge: None,
            code_challenge_method: None,
            state: None,
        };

        let result = service.authorize(input).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_reject_invalid_redirect_uri() {
        let client_repo = Arc::new(MockClientRepository::new());
        let auth_code_repo = Arc::new(MockAuthCodeRepository::new());
        let service = AuthService::new(client_repo.clone(), auth_code_repo);

        let client_id = ClientId::new();
        let client = OAuthClient::<Public>::new(
            client_id,
            "Test SPA",
            vec!["http://localhost:3000/callback".to_string()],
            Scopes::parse("openid").unwrap(),
        );
        client_repo.add_public_client(&client);

        let input = AuthorizeInput {
            client_id,
            user_id: UserId::new(),
            redirect_uri: "http://evil.com/callback".to_string(),
            scope: "openid".to_string(),
            code_challenge: Some("test_challenge".to_string()),
            code_challenge_method: Some("S256".to_string()),
            state: None,
        };

        let result = service.authorize(input).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UseCaseError::Validation(_)));
    }

    #[tokio::test]
    async fn should_reject_invalid_scopes() {
        let client_repo = Arc::new(MockClientRepository::new());
        let auth_code_repo = Arc::new(MockAuthCodeRepository::new());
        let service = AuthService::new(client_repo.clone(), auth_code_repo);

        let client_id = ClientId::new();
        let client = OAuthClient::<Public>::new(
            client_id,
            "Test SPA",
            vec!["http://localhost:3000/callback".to_string()],
            Scopes::parse("openid profile").unwrap(),
        );
        client_repo.add_public_client(&client);

        let input = AuthorizeInput {
            client_id,
            user_id: UserId::new(),
            redirect_uri: "http://localhost:3000/callback".to_string(),
            scope: "openid profile email admin".to_string(), // 'admin' not allowed
            code_challenge: Some("test_challenge".to_string()),
            code_challenge_method: Some("S256".to_string()),
            state: None,
        };

        let result = service.authorize(input).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UseCaseError::Validation(_)));
    }

    #[tokio::test]
    async fn should_reject_nonexistent_client() {
        let client_repo = Arc::new(MockClientRepository::new());
        let auth_code_repo = Arc::new(MockAuthCodeRepository::new());
        let service = AuthService::new(client_repo, auth_code_repo);

        let input = AuthorizeInput {
            client_id: ClientId::new(), // Non-existent client
            user_id: UserId::new(),
            redirect_uri: "http://localhost:3000/callback".to_string(),
            scope: "openid".to_string(),
            code_challenge: Some("test_challenge".to_string()),
            code_challenge_method: Some("S256".to_string()),
            state: None,
        };

        let result = service.authorize(input).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UseCaseError::NotFound));
    }
}
