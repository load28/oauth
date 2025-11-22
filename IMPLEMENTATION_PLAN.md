# OAuth 2.0 Server - 상세 구현 계획

## 전체 아키텍처

### Hexagonal Architecture (Ports & Adapters)

```
┌─────────────────────────────────────────────────────────────┐
│                     Infrastructure Layer                     │
│  ┌────────────┐  ┌──────────────┐  ┌──────────────────┐    │
│  │   Actix    │  │   SQLite     │  │   Environment    │    │
│  │  Handlers  │  │ Repositories │  │     Config       │    │
│  └─────┬──────┘  └──────┬───────┘  └────────┬─────────┘    │
│        │                │                    │               │
└────────┼────────────────┼────────────────────┼───────────────┘
         │                │                    │
         ▼                ▼                    ▼
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                         │
│  ┌──────────────────────────────────────────────────────┐   │
│  │                  Input Ports (Use Cases)             │   │
│  │  UserUseCase │ ClientUseCase │ AuthUseCase │ Token  │   │
│  └──────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │                    Services                          │   │
│  │  UserService │ ClientService │ AuthService │ Token  │   │
│  └──────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │                 Output Ports (Repos)                 │   │
│  │  UserRepo │ ClientRepo │ AuthCodeRepo │ TokenRepo   │   │
│  └──────────────────────────────────────────────────────┘   │
└────────────────────────────┬────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                       Domain Layer                           │
│  ┌──────────────────┐  ┌────────────────────────────────┐   │
│  │  Value Objects   │  │         Entities               │   │
│  │  - Email         │  │  - User                        │   │
│  │  - UserId        │  │  - OAuthClient<T>              │   │
│  │  - Password      │  │  - AuthorizationCode           │   │
│  │  - Scopes        │  │  - AccessToken                 │   │
│  │  - PKCE          │  │  - RefreshToken                │   │
│  └──────────────────┘  └────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## 단계별 구현 계획

### Phase 1: Foundation (완료 ✅)

#### 1.1 워크스페이스 설정 ✅
```toml
[workspace]
members = ["domain", "application", "infrastructure"]
```

#### 1.2 Domain Layer ✅
- Value Objects: 타입 안전성, 검증 로직
- Entities: 비즈니스 규칙, 불변성
- 72개 테스트 통과

### Phase 2: Application Layer (진행 중 🔄)

#### 2.1 Ports 정의 ✅
- Input Ports: Use Case 인터페이스
- Output Ports: Repository 인터페이스
- DTOs: 레이어 간 데이터 전달

#### 2.2 Services 구현 (다음 단계 ⏳)

**UserService 구현**
```rust
pub struct UserService<R: UserRepository> {
    user_repository: R,
}

impl<R: UserRepository> UserService<R> {
    pub fn new(user_repository: R) -> Self {
        Self { user_repository }
    }
}

#[async_trait]
impl<R: UserRepository> UserUseCase for UserService<R> {
    async fn register_user(&self, input: RegisterUserInput)
        -> Result<RegisterUserOutput, UseCaseError> {
        // 1. Email 검증
        let email = Email::new(input.email)
            .map_err(|e| UseCaseError::Validation(e.to_string()))?;

        // 2. 중복 확인
        if self.user_repository.exists_by_email(&email).await? {
            return Err(UseCaseError::Domain("Email already exists".into()));
        }

        // 3. 비밀번호 해싱
        let password = PlainPassword::new(input.password)
            .map_err(|e| UseCaseError::Validation(e.to_string()))?;
        let password_hash = password.hash()
            .map_err(|e| UseCaseError::Domain(e.to_string()))?;

        // 4. User 생성
        let user = User::new(UserId::new(), email.clone(), password_hash);

        // 5. 저장
        self.user_repository.save(&user).await?;

        Ok(RegisterUserOutput {
            user_id: user.id(),
            email: email.clone(),
        })
    }
}
```

**ClientService 구현**
```rust
pub struct ClientService<R: ClientRepository> {
    client_repository: R,
}

#[async_trait]
impl<R: ClientRepository> ClientUseCase for ClientService<R> {
    async fn register_public_client(&self, input: RegisterPublicClientInput)
        -> Result<RegisterPublicClientOutput, UseCaseError> {
        // 1. Scopes 파싱
        let scopes = Scopes::parse(&input.allowed_scopes)
            .map_err(|e| UseCaseError::Validation(e.to_string()))?;

        // 2. Public Client 생성
        let client = OAuthClient::<Public>::new(
            ClientId::new(),
            input.name.clone(),
            input.redirect_uris.clone(),
            scopes,
        );

        // 3. 저장
        self.client_repository.save_public(&client).await?;

        Ok(RegisterPublicClientOutput {
            client_id: client.id(),
            name: input.name,
            redirect_uris: input.redirect_uris,
        })
    }
}
```

**AuthService 구현**
```rust
pub struct AuthService<CR, AR> {
    client_repository: CR,
    auth_code_repository: AR,
}

#[async_trait]
impl<CR, AR> AuthUseCase for AuthService<CR, AR>
where
    CR: ClientRepository,
    AR: AuthCodeRepository,
{
    async fn authorize(&self, input: AuthorizeInput)
        -> Result<AuthorizeOutput, UseCaseError> {
        // 1. Client 조회
        let client_data = self.client_repository
            .find_by_id(input.client_id)
            .await?
            .ok_or(UseCaseError::NotFound)?;

        // 2. Redirect URI 검증 (타입에 관계없이 공통 로직)
        // 3. Scopes 파싱 및 검증
        // 4. PKCE 검증 (Public 클라이언트는 필수)
        // 5. AuthorizationCode 생성
        // 6. 저장
        // 7. 결과 반환
    }
}
```

**TokenService 구현**
```rust
pub struct TokenService<CR, AR, ATR, RTR> {
    client_repository: CR,
    auth_code_repository: AR,
    access_token_repository: ATR,
    refresh_token_repository: RTR,
}

#[async_trait]
impl<CR, AR, ATR, RTR> TokenUseCase for TokenService<CR, AR, ATR, RTR>
where
    CR: ClientRepository,
    AR: AuthCodeRepository,
    ATR: AccessTokenRepository,
    RTR: RefreshTokenRepository,
{
    async fn exchange_code(&self, input: ExchangeCodeInput)
        -> Result<TokenOutput, UseCaseError> {
        // 1. AuthCode 조회
        // 2. Client 인증 (Confidential은 secret 검증)
        // 3. AuthCode 검증 (PKCE 포함)
        // 4. AccessToken 생성
        // 5. RefreshToken 생성
        // 6. AuthCode 사용 처리
        // 7. 토큰 저장
        // 8. 결과 반환
    }

    async fn refresh_token(&self, input: RefreshTokenInput)
        -> Result<TokenOutput, UseCaseError> {
        // 1. RefreshToken 조회
        // 2. Client 인증
        // 3. RefreshToken 검증
        // 4. 새 AccessToken 생성
        // 5. (선택) RefreshToken 회전
        // 6. 저장
        // 7. 결과 반환
    }
}
```

### Phase 3: Infrastructure Layer - Database (예정 ⏳)

#### 3.1 SQLite 스키마

```sql
-- migrations/001_initial_schema.sql

CREATE TABLE users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX idx_users_email ON users(email);

CREATE TABLE oauth_clients (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    client_type TEXT NOT NULL CHECK(client_type IN ('public', 'confidential')),
    client_secret TEXT,  -- NULL for public clients
    redirect_uris TEXT NOT NULL,  -- JSON array
    allowed_scopes TEXT NOT NULL,  -- Space-separated
    created_at INTEGER NOT NULL
);

CREATE TABLE authorization_codes (
    code TEXT PRIMARY KEY,
    client_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    redirect_uri TEXT NOT NULL,
    scope TEXT NOT NULL,
    code_challenge TEXT,
    code_challenge_method TEXT CHECK(code_challenge_method IN ('plain', 'S256')),
    expires_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    used INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (client_id) REFERENCES oauth_clients(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_auth_codes_expires ON authorization_codes(expires_at);
CREATE INDEX idx_auth_codes_client ON authorization_codes(client_id);

CREATE TABLE access_tokens (
    token TEXT PRIMARY KEY,
    client_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    scope TEXT NOT NULL,
    expires_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (client_id) REFERENCES oauth_clients(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_access_tokens_user ON access_tokens(user_id);
CREATE INDEX idx_access_tokens_expires ON access_tokens(expires_at);

CREATE TABLE refresh_tokens (
    token TEXT PRIMARY KEY,
    access_token_id TEXT NOT NULL,
    client_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    scope TEXT NOT NULL,
    expires_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    revoked INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (client_id) REFERENCES oauth_clients(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_refresh_tokens_user ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_client ON refresh_tokens(client_id);
CREATE INDEX idx_refresh_tokens_access ON refresh_tokens(access_token_id);
```

#### 3.2 Repository 구현 (SQLx)

```rust
// infrastructure/src/persistence/user_repository_impl.rs

use application::ports::output::UserRepository;
use async_trait::async_trait;
use sqlx::SqlitePool;

pub struct SqliteUserRepository {
    pool: SqlitePool,
}

impl SqliteUserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn save(&self, user: &User) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"
            INSERT INTO users (id, email, password_hash, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                email = excluded.email,
                password_hash = excluded.password_hash,
                updated_at = excluded.updated_at
            "#,
            user.id().to_string(),
            user.email().as_str(),
            user.password_hash().as_str(),
            user.created_at().timestamp(),
            user.updated_at().timestamp(),
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepositoryError> {
        let row = sqlx::query!(
            r#"SELECT id, email, password_hash, created_at, updated_at
               FROM users WHERE email = ?"#,
            email.as_str()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match row {
            Some(r) => {
                let user = User::from_existing(
                    UserId::parse(&r.id).map_err(|e| RepositoryError::Internal(e.to_string()))?,
                    Email::new(r.email).map_err(|e| RepositoryError::Internal(e.to_string()))?,
                    PasswordHash::from_hash(r.password_hash).map_err(|e| RepositoryError::Internal(e.to_string()))?,
                    DateTime::from_timestamp(r.created_at, 0).unwrap(),
                    DateTime::from_timestamp(r.updated_at, 0).unwrap(),
                );
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }
}
```

### Phase 4: Infrastructure Layer - HTTP (예정 ⏳)

#### 4.1 Actix-web 핸들러

```rust
// infrastructure/src/http/handlers/user_handlers.rs

use actix_web::{web, HttpResponse};
use application::ports::input::{UserUseCase, RegisterUserInput};

#[derive(Deserialize)]
pub struct RegisterRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    user_id: String,
    email: String,
}

pub async fn register_user<U: UserUseCase>(
    user_service: web::Data<U>,
    req: web::Json<RegisterRequest>,
) -> Result<HttpResponse, Error> {
    let input = RegisterUserInput {
        email: req.email.clone(),
        password: req.password.clone(),
    };

    match user_service.register_user(input).await {
        Ok(output) => Ok(HttpResponse::Created().json(RegisterResponse {
            user_id: output.user_id.to_string(),
            email: output.email.to_string(),
        })),
        Err(e) => Ok(HttpResponse::BadRequest().json(ErrorResponse {
            error: e.to_string(),
        })),
    }
}
```

#### 4.2 OAuth Endpoints

```rust
// POST /authorize - 인증 요청 승인
pub async fn authorize<A: AuthUseCase>(
    auth_service: web::Data<A>,
    req: web::Json<AuthorizeRequest>,
) -> Result<HttpResponse, Error>;

// POST /token - 토큰 발급/갱신
pub async fn token<T: TokenUseCase>(
    token_service: web::Data<T>,
    req: web::Form<TokenRequest>,
) -> Result<HttpResponse, Error>;

// POST /revoke - 토큰 폐기
pub async fn revoke<T: TokenUseCase>(
    token_service: web::Data<T>,
    req: web::Form<RevokeRequest>,
) -> Result<HttpResponse, Error>;

// POST /introspect - 토큰 검사
pub async fn introspect<T: TokenUseCase>(
    token_service: web::Data<T>,
    req: web::Form<IntrospectRequest>,
) -> Result<HttpResponse, Error>;
```

#### 4.3 라우팅

```rust
// infrastructure/src/http/routes.rs

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // User endpoints
        .route("/register", web::post().to(handlers::register_user))
        .route("/login", web::post().to(handlers::login_user))

        // OAuth endpoints
        .route("/authorize", web::get().to(handlers::authorize_page))
        .route("/authorize", web::post().to(handlers::authorize))
        .route("/token", web::post().to(handlers::token))
        .route("/revoke", web::post().to(handlers::revoke))
        .route("/introspect", web::post().to(handlers::introspect))

        // Client registration
        .route("/clients", web::post().to(handlers::register_client));
}
```

### Phase 5: Integration (예정 ⏳)

#### 5.1 의존성 주입

```rust
// infrastructure/src/lib.rs

pub struct AppState {
    pub user_service: Arc<dyn UserUseCase>,
    pub client_service: Arc<dyn ClientUseCase>,
    pub auth_service: Arc<dyn AuthUseCase>,
    pub token_service: Arc<dyn TokenUseCase>,
}

pub async fn create_app_state(pool: SqlitePool) -> AppState {
    // Repositories
    let user_repo = Arc::new(SqliteUserRepository::new(pool.clone()));
    let client_repo = Arc::new(SqliteClientRepository::new(pool.clone()));
    let auth_code_repo = Arc::new(SqliteAuthCodeRepository::new(pool.clone()));
    let access_token_repo = Arc::new(SqliteAccessTokenRepository::new(pool.clone()));
    let refresh_token_repo = Arc::new(SqliteRefreshTokenRepository::new(pool.clone()));

    // Services
    let user_service = Arc::new(UserService::new(user_repo)) as Arc<dyn UserUseCase>;
    let client_service = Arc::new(ClientService::new(client_repo.clone())) as Arc<dyn ClientUseCase>;
    let auth_service = Arc::new(AuthService::new(client_repo.clone(), auth_code_repo)) as Arc<dyn AuthUseCase>;
    let token_service = Arc::new(TokenService::new(
        client_repo,
        auth_code_repo,
        access_token_repo,
        refresh_token_repo,
    )) as Arc<dyn TokenUseCase>;

    AppState {
        user_service,
        client_service,
        auth_service,
        token_service,
    }
}
```

#### 5.2 main.rs

```rust
// infrastructure/src/main.rs

use actix_web::{web, App, HttpServer};
use sqlx::sqlite::SqlitePoolOptions;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment
    dotenv::dotenv().ok();
    env_logger::init();

    // Database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:oauth.db".to_string());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Create app state
    let app_state = create_app_state(pool).await;

    // Start server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.user_service.clone()))
            .app_data(web::Data::new(app_state.client_service.clone()))
            .app_data(web::Data::new(app_state.auth_service.clone()))
            .app_data(web::Data::new(app_state.token_service.clone()))
            .configure(routes::configure_routes)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## 테스트 전략

### Unit Tests
- Domain: 모든 엔티티 및 값 객체 (✅ 72개 통과)
- Application: 서비스 로직 (mockall로 Repository 모킹)

### Integration Tests
- Infrastructure: 실제 SQLite DB 사용
- HTTP: Actix-web TestRequest

### E2E Tests
- 전체 OAuth 플로우 시나리오

## 환경 설정

### .env
```env
DATABASE_URL=sqlite:oauth.db
RUST_LOG=info
SERVER_ADDRESS=127.0.0.1:8080
```

### Cargo.toml (infrastructure)
```toml
[dependencies]
domain = { path = "../domain" }
application = { path = "../application" }

# Web framework
actix-web = "4"
actix-rt = "2"

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Environment
dotenv = "0.15"
env_logger = "0.11"

# Async
tokio = { version = "1", features = ["full"] }
```

## 다음 단계 체크리스트

- [ ] Application 컴파일 오류 수정 (unused imports)
- [ ] UserService 구현
- [ ] ClientService 구현
- [ ] AuthService 구현
- [ ] TokenService 구현
- [ ] SQLite 스키마 작성
- [ ] UserRepository 구현
- [ ] ClientRepository 구현
- [ ] AuthCodeRepository 구현
- [ ] TokenRepository 구현
- [ ] HTTP Handlers 구현
- [ ] 라우팅 설정
- [ ] main.rs 작성
- [ ] 통합 테스트
- [ ] README 작성

---

**문서 버전**: 1.0
**마지막 업데이트**: 2025-11-21
