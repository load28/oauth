//! Common test utilities for integration tests

use actix_web::{web, App, HttpResponse};
use infrastructure::{
    AppState, AuthenticateUserRequest, AuthorizeRequest, ChangePasswordRequest, Config,
    ConsentRequest, Database, IntrospectTokenRequest, RegisterClientRequest,
    RegisterUserRequest, RevokeTokenRequest, SqliteAccessTokenRepository,
    SqliteAuthCodeRepository, SqliteClientRepository, SqliteRefreshTokenRepository,
    SqliteUserRepository, TokenRequest,
};

/// Type alias for the concrete AppState used in tests
pub type TestAppState = AppState<
    SqliteUserRepository,
    SqliteClientRepository,
    SqliteAuthCodeRepository,
    SqliteAccessTokenRepository,
    SqliteRefreshTokenRepository,
>;

/// Creates a test database with migrations applied
pub async fn create_test_database() -> Database {
    let db = Database::new("sqlite::memory:")
        .await
        .expect("Failed to create test database");

    db.migrate()
        .await
        .expect("Failed to run migrations");

    db
}

/// Creates a test configuration
pub fn create_test_config() -> Config {
    std::env::set_var("DATABASE_URL", "sqlite::memory:");
    std::env::set_var("JWT_SECRET", "test-secret-key-min-32-characters");
    std::env::set_var("SERVER_HOST", "127.0.0.1");
    std::env::set_var("SERVER_PORT", "8080");

    Config::from_env().expect("Failed to create test config")
}

/// Creates a test AppState with in-memory database
pub async fn create_test_app_state() -> TestAppState {
    let config = create_test_config();
    let database = create_test_database().await;

    let user_repository = SqliteUserRepository::new(database.pool().clone());
    let client_repository = SqliteClientRepository::new(database.pool().clone());
    let auth_code_repository = SqliteAuthCodeRepository::new(database.pool().clone());
    let access_token_repository = SqliteAccessTokenRepository::new(database.pool().clone());
    let refresh_token_repository = SqliteRefreshTokenRepository::new(database.pool().clone());

    AppState::new(
        config,
        user_repository,
        client_repository,
        auth_code_repository,
        access_token_repository,
        refresh_token_repository,
    )
}

// Handler wrappers with concrete types for testing
async fn register_user(
    state: web::Data<TestAppState>,
    req: web::Json<RegisterUserRequest>,
) -> HttpResponse {
    infrastructure::register_user(state, req).await
}

async fn authenticate_user(
    state: web::Data<TestAppState>,
    req: web::Json<AuthenticateUserRequest>,
) -> HttpResponse {
    infrastructure::authenticate_user(state, req).await
}

async fn get_user(state: web::Data<TestAppState>, user_id: web::Path<String>) -> HttpResponse {
    infrastructure::get_user(state, user_id).await
}

async fn change_password(
    state: web::Data<TestAppState>,
    user_id: web::Path<String>,
    req: web::Json<ChangePasswordRequest>,
) -> HttpResponse {
    infrastructure::change_password(state, user_id, req).await
}

async fn register_client(
    state: web::Data<TestAppState>,
    req: web::Json<RegisterClientRequest>,
) -> HttpResponse {
    infrastructure::register_client(state, req).await
}

async fn get_client(
    state: web::Data<TestAppState>,
    client_id: web::Path<String>,
) -> HttpResponse {
    infrastructure::get_client(state, client_id).await
}

async fn delete_client(
    state: web::Data<TestAppState>,
    client_id: web::Path<String>,
) -> HttpResponse {
    infrastructure::delete_client(state, client_id).await
}

async fn authorize(
    state: web::Data<TestAppState>,
    query: web::Query<AuthorizeRequest>,
) -> HttpResponse {
    infrastructure::authorize(state, query).await
}

async fn consent(
    state: web::Data<TestAppState>,
    req: web::Json<ConsentRequest>,
) -> HttpResponse {
    infrastructure::consent(state, req).await
}

async fn token(state: web::Data<TestAppState>, req: web::Json<TokenRequest>) -> HttpResponse {
    infrastructure::token(state, req).await
}

async fn revoke_token(
    state: web::Data<TestAppState>,
    req: web::Json<RevokeTokenRequest>,
) -> HttpResponse {
    infrastructure::revoke_token(state, req).await
}

async fn introspect_token(
    state: web::Data<TestAppState>,
    req: web::Json<IntrospectTokenRequest>,
) -> HttpResponse {
    infrastructure::introspect_token(state, req).await
}

/// Creates an Actix-web App instance with all routes configured
pub fn create_test_app(
    state: TestAppState,
) -> App<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse<actix_web::body::BoxBody>,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new()
        .app_data(web::Data::new(state))
        // User endpoints
        .route("/users/register", web::post().to(register_user))
        .route("/auth/login", web::post().to(authenticate_user))
        .route("/users/{id}", web::get().to(get_user))
        .route("/users/{id}/password", web::post().to(change_password))
        // OAuth Client endpoints
        .route("/clients/register", web::post().to(register_client))
        .route("/clients/{id}", web::get().to(get_client))
        .route("/clients/{id}", web::delete().to(delete_client))
        // OAuth Authorization endpoints
        .route("/authorize", web::get().to(authorize))
        .route("/authorize/consent", web::post().to(consent))
        // OAuth Token endpoints
        .route("/token", web::post().to(token))
        .route("/revoke", web::post().to(revoke_token))
        .route("/introspect", web::post().to(introspect_token))
}
