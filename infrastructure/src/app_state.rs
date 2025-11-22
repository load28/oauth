//! Application State
//!
//! Shared state containing services and configuration for HTTP handlers.

use crate::config::Config;
use application::services::{AuthService, ClientService, TokenService, UserService};
use application::ports::{
    AuthCodeRepository, ClientRepository, AccessTokenRepository,
    RefreshTokenRepository, UserRepository,
};
use std::sync::Arc;

/// Application state shared across all HTTP handlers
///
/// Contains all services and configuration needed by the API endpoints.
/// Uses Arc for thread-safe sharing across actix-web workers.
pub struct AppState<UR, CR, AR, ATR, RTR>
where
    UR: UserRepository,
    CR: ClientRepository,
    AR: AuthCodeRepository,
    ATR: AccessTokenRepository,
    RTR: RefreshTokenRepository,
{
    pub config: Arc<Config>,
    pub user_service: Arc<UserService<UR>>,
    pub client_service: Arc<ClientService<CR>>,
    pub auth_service: Arc<AuthService<CR, AR>>,
    pub token_service: Arc<TokenService<CR, AR, ATR, RTR>>,
}

impl<UR, CR, AR, ATR, RTR> AppState<UR, CR, AR, ATR, RTR>
where
    UR: UserRepository,
    CR: ClientRepository,
    AR: AuthCodeRepository,
    ATR: AccessTokenRepository,
    RTR: RefreshTokenRepository,
{
    /// Create new application state
    pub fn new(
        config: Config,
        user_repository: UR,
        client_repository: CR,
        auth_code_repository: AR,
        access_token_repository: ATR,
        refresh_token_repository: RTR,
    ) -> Self {
        // Wrap repositories in Arc for sharing
        let user_repo = Arc::new(user_repository);
        let client_repo = Arc::new(client_repository);
        let auth_code_repo = Arc::new(auth_code_repository);
        let access_token_repo = Arc::new(access_token_repository);
        let refresh_token_repo = Arc::new(refresh_token_repository);

        // Create services with repositories
        let user_service = Arc::new(UserService::new(user_repo));
        let client_service = Arc::new(ClientService::new(client_repo.clone()));
        let auth_service = Arc::new(AuthService::new(client_repo.clone(), auth_code_repo.clone()));
        let token_service = Arc::new(TokenService::new(
            client_repo,
            auth_code_repo,
            access_token_repo,
            refresh_token_repo,
        ));

        Self {
            config: Arc::new(config),
            user_service,
            client_service,
            auth_service,
            token_service,
        }
    }
}

// Implement Clone for AppState to allow sharing across actix-web workers
impl<UR, CR, AR, ATR, RTR> Clone for AppState<UR, CR, AR, ATR, RTR>
where
    UR: UserRepository,
    CR: ClientRepository,
    AR: AuthCodeRepository,
    ATR: AccessTokenRepository,
    RTR: RefreshTokenRepository,
{
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            user_service: Arc::clone(&self.user_service),
            client_service: Arc::clone(&self.client_service),
            auth_service: Arc::clone(&self.auth_service),
            token_service: Arc::clone(&self.token_service),
        }
    }
}
