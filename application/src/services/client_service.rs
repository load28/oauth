//! OAuth Client Service Implementation
//!
//! Implements ClientUseCase for OAuth client registration and management.

use crate::errors::UseCaseError;
use crate::ports::{
    ClientData, ClientRepository, ClientUseCase, RegisterConfidentialClientInput,
    RegisterConfidentialClientOutput, RegisterPublicClientInput, RegisterPublicClientOutput,
};
use async_trait::async_trait;
use domain::entities::{ClientSecret, Confidential, OAuthClient, Public};
use domain::value_objects::{ClientId, Scopes};
use std::sync::Arc;

/// Service implementation for OAuth client management
///
/// # Type Parameters
/// * `R` - The repository implementation (must implement ClientRepository)
///
/// # Business Logic
/// - Public clients are for SPAs and mobile apps (require PKCE)
/// - Confidential clients are for server-side apps (use client secret)
/// - Client secrets are returned only once during registration
pub struct ClientService<R: ClientRepository> {
    client_repository: Arc<R>,
}

impl<R: ClientRepository> ClientService<R> {
    /// Creates a new ClientService
    ///
    /// # Arguments
    /// * `client_repository` - Repository for persisting clients
    pub fn new(client_repository: Arc<R>) -> Self {
        Self { client_repository }
    }
}

#[async_trait]
impl<R: ClientRepository> ClientUseCase for ClientService<R> {
    async fn register_public_client(
        &self,
        input: RegisterPublicClientInput,
    ) -> Result<RegisterPublicClientOutput, UseCaseError> {
        // 1. Validate redirect URIs
        if input.redirect_uris.is_empty() {
            return Err(UseCaseError::Validation(
                "At least one redirect URI is required".to_string(),
            ));
        }

        // 2. Parse and validate scopes
        let allowed_scopes: Scopes = Scopes::parse(&input.allowed_scopes)
            .map_err(|e| UseCaseError::Validation(e.to_string()))?;

        // 3. Create public client entity
        let client_id: ClientId = ClientId::new();
        let client: OAuthClient<Public> = OAuthClient::<Public>::new(
            client_id,
            input.name.clone(),
            input.redirect_uris.clone(),
            allowed_scopes,
        );

        // 4. Persist client
        self.client_repository.save_public(&client).await?;

        // 5. Return output
        Ok(RegisterPublicClientOutput {
            client_id: client.id(),
            name: input.name,
            redirect_uris: input.redirect_uris,
        })
    }

    async fn register_confidential_client(
        &self,
        input: RegisterConfidentialClientInput,
    ) -> Result<RegisterConfidentialClientOutput, UseCaseError> {
        // 1. Validate redirect URIs
        if input.redirect_uris.is_empty() {
            return Err(UseCaseError::Validation(
                "At least one redirect URI is required".to_string(),
            ));
        }

        // 2. Parse and validate scopes
        let allowed_scopes: Scopes = Scopes::parse(&input.allowed_scopes)
            .map_err(|e| UseCaseError::Validation(e.to_string()))?;

        // 3. Generate client secret
        let client_secret: ClientSecret = ClientSecret::generate();
        let client_secret_string: String = client_secret.as_str().to_string();

        // 4. Create confidential client entity
        let client_id: ClientId = ClientId::new();
        let client: OAuthClient<Confidential> = OAuthClient::<Confidential>::new(
            client_id,
            input.name.clone(),
            client_secret,
            input.redirect_uris.clone(),
            allowed_scopes,
        );

        // 5. Persist client
        self.client_repository.save_confidential(&client).await?;

        // 6. Return output (including plaintext secret, only shown once)
        Ok(RegisterConfidentialClientOutput {
            client_id: client.id(),
            name: input.name,
            redirect_uris: input.redirect_uris,
            client_secret: client_secret_string,
        })
    }

    async fn get_public_client(
        &self,
        client_id: ClientId,
    ) -> Result<OAuthClient<Public>, UseCaseError> {
        let client_data: ClientData = self
            .client_repository
            .find_by_id(client_id)
            .await?
            .ok_or(UseCaseError::NotFound)?;

        if !client_data.is_public() {
            return Err(UseCaseError::NotFound);
        }

        Ok(client_data.to_public())
    }

    async fn get_confidential_client(
        &self,
        client_id: ClientId,
    ) -> Result<OAuthClient<Confidential>, UseCaseError> {
        let client_data: ClientData = self
            .client_repository
            .find_by_id(client_id)
            .await?
            .ok_or(UseCaseError::NotFound)?;

        if !client_data.is_confidential() {
            return Err(UseCaseError::NotFound);
        }

        Ok(client_data.to_confidential())
    }

    async fn delete_client(&self, client_id: ClientId) -> Result<(), UseCaseError> {
        self.client_repository.delete(client_id).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::RepositoryError;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// Mock repository for testing
    struct MockClientRepository {
        clients: Mutex<HashMap<ClientId, ClientData>>,
    }

    impl MockClientRepository {
        fn new() -> Self {
            Self {
                clients: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl ClientRepository for MockClientRepository {
        async fn save_public(&self, client: &OAuthClient<Public>) -> Result<(), RepositoryError> {
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
            Ok(())
        }

        async fn save_confidential(
            &self,
            client: &OAuthClient<Confidential>,
        ) -> Result<(), RepositoryError> {
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
            Ok(())
        }

        async fn find_by_id(&self, id: ClientId) -> Result<Option<ClientData>, RepositoryError> {
            let clients = self.clients.lock().unwrap();
            Ok(clients.get(&id).cloned())
        }

        async fn delete(&self, id: ClientId) -> Result<(), RepositoryError> {
            let mut clients = self.clients.lock().unwrap();
            clients
                .remove(&id)
                .ok_or(RepositoryError::NotFound)
                .map(|_| ())
        }
    }

    #[tokio::test]
    async fn should_register_public_client() {
        let repo = Arc::new(MockClientRepository::new());
        let service = ClientService::new(repo);

        let input = RegisterPublicClientInput {
            name: "My SPA".to_string(),
            redirect_uris: vec!["http://localhost:3000/callback".to_string()],
            allowed_scopes: "openid profile email".to_string(),
        };

        let result = service.register_public_client(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.name, "My SPA");
    }

    #[tokio::test]
    async fn should_register_confidential_client() {
        let repo = Arc::new(MockClientRepository::new());
        let service = ClientService::new(repo);

        let input = RegisterConfidentialClientInput {
            name: "My Server App".to_string(),
            redirect_uris: vec!["https://example.com/callback".to_string()],
            allowed_scopes: "openid profile email".to_string(),
        };

        let result = service.register_confidential_client(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.name, "My Server App");
        assert!(!output.client_secret.is_empty());
    }

    #[tokio::test]
    async fn should_reject_client_without_redirect_uris() {
        let repo = Arc::new(MockClientRepository::new());
        let service = ClientService::new(repo);

        let input = RegisterPublicClientInput {
            name: "Invalid Client".to_string(),
            redirect_uris: vec![],
            allowed_scopes: "openid".to_string(),
        };

        let result = service.register_public_client(input).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UseCaseError::Validation(_)));
    }

    #[tokio::test]
    async fn should_get_public_client_by_id() {
        let repo = Arc::new(MockClientRepository::new());
        let service = ClientService::new(repo);

        let input = RegisterPublicClientInput {
            name: "My SPA".to_string(),
            redirect_uris: vec!["http://localhost:3000/callback".to_string()],
            allowed_scopes: "openid".to_string(),
        };

        let output = service.register_public_client(input).await.unwrap();
        let result = service.get_public_client(output.client_id).await;

        assert!(result.is_ok());
        let client = result.unwrap();
        assert_eq!(client.name(), "My SPA");
    }

    #[tokio::test]
    async fn should_get_confidential_client_by_id() {
        let repo = Arc::new(MockClientRepository::new());
        let service = ClientService::new(repo);

        let input = RegisterConfidentialClientInput {
            name: "My Server App".to_string(),
            redirect_uris: vec!["https://example.com/callback".to_string()],
            allowed_scopes: "openid".to_string(),
        };

        let output = service.register_confidential_client(input).await.unwrap();
        let result = service.get_confidential_client(output.client_id).await;

        assert!(result.is_ok());
        let client = result.unwrap();
        assert_eq!(client.name(), "My Server App");
    }

    #[tokio::test]
    async fn should_reject_getting_public_as_confidential() {
        let repo = Arc::new(MockClientRepository::new());
        let service = ClientService::new(repo);

        let input = RegisterPublicClientInput {
            name: "My SPA".to_string(),
            redirect_uris: vec!["http://localhost:3000/callback".to_string()],
            allowed_scopes: "openid".to_string(),
        };

        let output = service.register_public_client(input).await.unwrap();
        let result = service.get_confidential_client(output.client_id).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UseCaseError::NotFound));
    }

    #[tokio::test]
    async fn should_delete_client() {
        let repo = Arc::new(MockClientRepository::new());
        let service = ClientService::new(repo);

        let input = RegisterPublicClientInput {
            name: "My SPA".to_string(),
            redirect_uris: vec!["http://localhost:3000/callback".to_string()],
            allowed_scopes: "openid".to_string(),
        };

        let output = service.register_public_client(input).await.unwrap();
        let delete_result = service.delete_client(output.client_id).await;

        assert!(delete_result.is_ok());

        let get_result = service.get_public_client(output.client_id).await;
        assert!(get_result.is_err());
    }
}
