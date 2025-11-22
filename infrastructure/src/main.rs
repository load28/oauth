//! OAuth 2.0 Server Entry Point
//!
//! Actix-web HTTP server with full OAuth 2.0 implementation.

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use infrastructure::{
    AppState, Config, Database, SqliteAccessTokenRepository, SqliteAuthCodeRepository,
    SqliteClientRepository, SqliteRefreshTokenRepository, SqliteUserRepository,
};
use std::io;

// Type alias for concrete AppState configuration
type AppStateImpl = AppState<
    SqliteUserRepository,
    SqliteClientRepository,
    SqliteAuthCodeRepository,
    SqliteAccessTokenRepository,
    SqliteRefreshTokenRepository,
>;

// Import and re-export handlers with concrete types
mod handlers {
    use super::*;
    use actix_web::{web, HttpResponse};
    use infrastructure::*;

    pub async fn register_user(
        state: web::Data<AppStateImpl>,
        req: web::Json<RegisterUserRequest>,
    ) -> HttpResponse {
        infrastructure::register_user(state, req).await
    }

    pub async fn authenticate_user(
        state: web::Data<AppStateImpl>,
        req: web::Json<AuthenticateUserRequest>,
    ) -> HttpResponse {
        infrastructure::authenticate_user(state, req).await
    }

    pub async fn get_user(
        state: web::Data<AppStateImpl>,
        user_id: web::Path<String>,
    ) -> HttpResponse {
        infrastructure::get_user(state, user_id).await
    }

    pub async fn change_password(
        state: web::Data<AppStateImpl>,
        user_id: web::Path<String>,
        req: web::Json<ChangePasswordRequest>,
    ) -> HttpResponse {
        infrastructure::change_password(state, user_id, req).await
    }

    pub async fn register_client(
        state: web::Data<AppStateImpl>,
        req: web::Json<RegisterClientRequest>,
    ) -> HttpResponse {
        infrastructure::register_client(state, req).await
    }

    pub async fn get_client(
        state: web::Data<AppStateImpl>,
        client_id: web::Path<String>,
    ) -> HttpResponse {
        infrastructure::get_client(state, client_id).await
    }

    pub async fn delete_client(
        state: web::Data<AppStateImpl>,
        client_id: web::Path<String>,
    ) -> HttpResponse {
        infrastructure::delete_client(state, client_id).await
    }

    pub async fn authorize(
        state: web::Data<AppStateImpl>,
        query: web::Query<AuthorizeRequest>,
    ) -> HttpResponse {
        infrastructure::authorize(state, query).await
    }

    pub async fn consent(
        state: web::Data<AppStateImpl>,
        req: web::Json<ConsentRequest>,
    ) -> HttpResponse {
        infrastructure::consent(state, req).await
    }

    pub async fn token(
        state: web::Data<AppStateImpl>,
        req: web::Json<TokenRequest>,
    ) -> HttpResponse {
        infrastructure::token(state, req).await
    }

    pub async fn revoke_token(
        state: web::Data<AppStateImpl>,
        req: web::Json<RevokeTokenRequest>,
    ) -> HttpResponse {
        infrastructure::revoke_token(state, req).await
    }

    pub async fn introspect_token(
        state: web::Data<AppStateImpl>,
        req: web::Json<IntrospectTokenRequest>,
    ) -> HttpResponse {
        infrastructure::introspect_token(state, req).await
    }
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    // Initialize logging with tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Load configuration from environment
    let config = Config::from_env().expect("Failed to load configuration");

    tracing::info!("Starting OAuth 2.0 Server...");
    tracing::info!("Database: {}", config.database.url);
    tracing::info!("Server: {}:{}", config.server.host, config.server.port);

    // Initialize database connection
    let database = Database::new(&config.database.url)
        .await
        .expect("Failed to connect to database");

    tracing::info!("Database connection established");

    // Run migrations
    database
        .migrate()
        .await
        .expect("Failed to run migrations");

    tracing::info!("Database migrations completed");

    // Initialize repositories
    let user_repository = SqliteUserRepository::new(database.pool().clone());
    let client_repository = SqliteClientRepository::new(database.pool().clone());
    let auth_code_repository = SqliteAuthCodeRepository::new(database.pool().clone());
    let access_token_repository = SqliteAccessTokenRepository::new(database.pool().clone());
    let refresh_token_repository = SqliteRefreshTokenRepository::new(database.pool().clone());

    tracing::info!("Repositories initialized");

    // Create application state with dependency injection
    let app_state = AppState::new(
        config.clone(),
        user_repository,
        client_repository,
        auth_code_repository,
        access_token_repository,
        refresh_token_repository,
    );

    tracing::info!("Application state initialized");

    let server_host = config.server.host.clone();
    let server_port = config.server.port;
    let cors_origins = config.cors.allowed_origins.clone();

    // Start HTTP server
    tracing::info!("Starting HTTP server on {}:{}", server_host, server_port);

    HttpServer::new(move || {
        // Configure CORS
        let mut cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "DELETE"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ACCEPT,
                actix_web::http::header::CONTENT_TYPE,
            ])
            .max_age(3600);

        // Add allowed origins from config
        for origin in &cors_origins {
            cors = cors.allowed_origin(origin);
        }

        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(Logger::default())
            .wrap(cors)
            // User endpoints
            .route("/users/register", web::post().to(handlers::register_user))
            .route("/auth/login", web::post().to(handlers::authenticate_user))
            .route("/users/{id}", web::get().to(handlers::get_user))
            .route("/users/{id}/password", web::post().to(handlers::change_password))
            // OAuth Client endpoints
            .route("/clients/register", web::post().to(handlers::register_client))
            .route("/clients/{id}", web::get().to(handlers::get_client))
            .route("/clients/{id}", web::delete().to(handlers::delete_client))
            // OAuth Authorization endpoints
            .route("/authorize", web::get().to(handlers::authorize))
            .route("/authorize/consent", web::post().to(handlers::consent))
            // OAuth Token endpoints
            .route("/token", web::post().to(handlers::token))
            .route("/revoke", web::post().to(handlers::revoke_token))
            .route("/introspect", web::post().to(handlers::introspect_token))
    })
    .bind((server_host.as_str(), server_port))?
    .run()
    .await
}
