# OAuth 2.0 Login Server - Quick Start Guide

## 목차
1. [프로젝트 구조](#프로젝트-구조)
2. [개발 환경 설정](#개발-환경-설정)
3. [코드 탐색 가이드](#코드-탐색-가이드)
4. [테스트 실행](#테스트-실행)
5. [다음 단계](#다음-단계)
6. [일반적인 작업](#일반적인-작업)

---

## 프로젝트 구조

```
login-server/
├── domain/              # Domain Layer (순수 비즈니스 로직)
│   ├── src/
│   │   ├── entities/    # User, OAuthClient, Tokens
│   │   ├── value_objects/ # Email, Password, Scopes, PKCE
│   │   ├── errors.rs    # Domain 에러
│   │   └── lib.rs
│   └── Cargo.toml
│
├── application/         # Application Layer (Use Cases)
│   ├── src/
│   │   ├── ports/       # 인터페이스 정의
│   │   │   ├── input/   # Use Case 트레이트
│   │   │   └── output/  # Repository 트레이트
│   │   ├── services/    # Use Case 구현
│   │   ├── errors.rs    # Application 에러
│   │   └── lib.rs
│   └── Cargo.toml
│
├── infrastructure/      # Infrastructure Layer (외부 연동)
│   ├── src/
│   │   ├── repositories/ # SQLite 구현
│   │   ├── database.rs  # DB 커넥션
│   │   └── lib.rs
│   ├── migrations/      # DB 스키마
│   └── Cargo.toml
│
├── docs/                # 📄 문서
│   ├── ARCHITECTURE.md  # 아키텍처 설계
│   ├── FLOWCHARTS.md    # 플로우차트
│   ├── API_REFERENCE.md # API 레퍼런스
│   └── QUICK_START.md   # 이 문서
│
├── .sqlx/               # SQLx 쿼리 캐시
├── Cargo.toml           # Workspace 설정
└── Cargo.lock
```

---

## 개발 환경 설정

### 필수 요구사항

**Rust 툴체인:**
```bash
# Rust 설치 (rustup)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 버전 확인
rustc --version  # 1.70 이상 권장
cargo --version
```

**SQLx CLI:**
```bash
# SQLx CLI 설치 (마이그레이션 및 쿼리 캐시용)
cargo install sqlx-cli --no-default-features --features sqlite

# 버전 확인
sqlx --version
```

### 프로젝트 클론 및 빌드

```bash
# 프로젝트 디렉토리로 이동
cd /Users/seominyong/Downloads/login-server

# 의존성 다운로드 및 빌드
cargo build --workspace

# 빌드 확인 (Release 모드)
cargo build --workspace --release
```

### 데이터베이스 설정 (개발용)

```bash
# 1. 임시 데이터베이스 생성
cd infrastructure
DATABASE_URL="sqlite:temp.db" sqlx database create

# 2. 마이그레이션 실행
DATABASE_URL="sqlite:temp.db" sqlx migrate run

# 3. SQLx 쿼리 캐시 생성 (오프라인 모드용)
cd ..
DATABASE_URL="sqlite:infrastructure/temp.db" cargo sqlx prepare --workspace

# 4. 임시 DB 삭제
rm infrastructure/temp.db
```

**참고:** SQLx는 컴파일 타임에 쿼리를 검증하므로 쿼리 캐시(`.sqlx/`)가 필요합니다.

---

## 코드 탐색 가이드

### 1. Domain Layer 시작하기

**위치:** `domain/src/`

**추천 순서:**

#### Step 1: Value Objects부터 이해
```bash
# Email 검증 로직
domain/src/value_objects/email.rs

# Password 해싱 및 검증
domain/src/value_objects/password.rs

# Scopes 파싱
domain/src/value_objects/scope.rs

# PKCE (CodeVerifier, CodeChallenge)
domain/src/value_objects/pkce.rs
```

**핵심 개념:**
- Newtype Pattern: 타입 안전성
- 생성자에서 검증 강제
- `Result<T, Error>` 반환

**예제 코드:**
```rust
use domain::value_objects::{Email, PlainPassword, Scopes};

// Email 생성 (검증 포함)
let email = Email::new("user@example.com")?;

// Password 생성 및 해싱
let plain = PlainPassword::new("secret123")?;
let hash = plain.hash()?;

// Scopes 파싱
let scopes = Scopes::parse("openid profile email")?;
```

#### Step 2: Entities 탐색
```bash
# 사용자 엔티티
domain/src/entities/user.rs

# OAuth 클라이언트 (Typestate Pattern)
domain/src/entities/oauth_client.rs

# 인가 코드
domain/src/entities/authorization_code.rs

# 토큰
domain/src/entities/token.rs
```

**핵심 개념:**
- Entity = 식별자 + 비즈니스 로직
- Typestate Pattern (OAuthClient<Public> vs <Confidential>)
- 불변 조건 강제

**예제 코드:**
```rust
use domain::entities::{User, OAuthClient, Public};
use domain::value_objects::{UserId, Email, PasswordHash, ClientId, Scopes};

// User 생성
let user = User::new(
    UserId::new(),
    Email::new("user@example.com")?,
    password_hash,
);

// Public 클라이언트 생성 (Typestate)
let client = OAuthClient::<Public>::new(
    ClientId::new(),
    "My SPA",
    vec!["https://app.example.com/callback".to_string()],
    Scopes::parse("openid profile")?,
);

// 컴파일 타임 타입 체크
assert!(client.requires_pkce());  // Public은 항상 true
```

### 2. Application Layer 살펴보기

**위치:** `application/src/`

#### Step 1: Ports (인터페이스) 이해
```bash
# Input Ports (Use Cases)
application/src/ports/input/user_use_case.rs
application/src/ports/input/client_use_case.rs
application/src/ports/input/auth_use_case.rs
application/src/ports/input/token_use_case.rs

# Output Ports (Repositories)
application/src/ports/output/user_repository.rs
application/src/ports/output/client_repository.rs
application/src/ports/output/auth_code_repository.rs
application/src/ports/output/token_repository.rs
```

**핵심 개념:**
- Input Ports = Use Case 인터페이스
- Output Ports = Repository 인터페이스
- DTOs로 레이어 간 데이터 전달

#### Step 2: Services (구현체) 탐색
```bash
# 서비스 구현
application/src/services/user_service.rs
application/src/services/client_service.rs
application/src/services/auth_service.rs
application/src/services/token_service.rs
```

**예제 코드:**
```rust
use application::services::UserService;
use application::ports::input::{UserUseCase, RegisterUserInput};

// Service 생성 (Dependency Injection)
let user_service = UserService::new(user_repository);

// Use Case 실행
let input = RegisterUserInput {
    email: "user@example.com".to_string(),
    password: "secret123".to_string(),
};

let output = user_service.register_user(input).await?;
println!("User created: {:?}", output.user_id);
```

### 3. Infrastructure Layer 이해하기

**위치:** `infrastructure/src/`

#### Step 1: Database 연결
```bash
# DB 커넥션 풀
infrastructure/src/database.rs

# 마이그레이션 파일
infrastructure/migrations/20250101000000_initial_schema.sql
```

**예제 코드:**
```rust
use infrastructure::Database;

// 데이터베이스 연결
let db = Database::new("sqlite:oauth.db").await?;

// 마이그레이션 실행
db.migrate().await?;

// 커넥션 풀 사용
let pool = db.pool().clone();
```

#### Step 2: Repository 구현
```bash
# Repository 구현체
infrastructure/src/repositories/user_repository.rs
infrastructure/src/repositories/client_repository.rs
infrastructure/src/repositories/auth_code_repository.rs
infrastructure/src/repositories/token_repository.rs
```

**예제 코드:**
```rust
use infrastructure::repositories::SqliteUserRepository;
use application::ports::UserRepository;

// Repository 생성
let user_repo = SqliteUserRepository::new(pool);

// User 저장
user_repo.save(&user).await?;

// User 조회
let found_user = user_repo.find_by_email(&email).await?;
```

---

## 테스트 실행

### 전체 테스트 실행

```bash
# 모든 레이어 테스트 (111개)
cargo test --workspace --lib

# 출력:
# Domain Layer:        72 tests passed
# Application Layer:   22 tests passed
# Infrastructure Layer: 17 tests passed
```

### 레이어별 테스트

```bash
# Domain Layer만 (72 tests)
cargo test --lib -p domain

# Application Layer만 (22 tests)
cargo test --lib -p application

# Infrastructure Layer만 (17 tests)
cargo test --lib -p infrastructure
```

### 특정 테스트 실행

```bash
# 모듈별 필터링
cargo test --lib -p domain user::tests

# 테스트 이름으로 필터링
cargo test --lib should_register_new_user

# 출력 표시 (println! 보기)
cargo test --lib -- --nocapture
```

### 코드 품질 검사

```bash
# Clippy (린터)
cargo clippy --workspace --all-targets -- -D warnings

# 포맷팅 확인
cargo fmt --all -- --check

# 포맷팅 자동 수정
cargo fmt --all
```

### 테스트 커버리지 (선택사항)

```bash
# Tarpaulin 설치
cargo install cargo-tarpaulin

# 커버리지 생성
cargo tarpaulin --workspace --lib --out Html

# 결과: tarpaulin-report.html
```

---

## 다음 단계

### 현재 완료된 항목 ✅

- [x] Domain Layer (100%)
- [x] Application Layer (100%)
- [x] Infrastructure Layer - Repository (100%)
- [x] Database Schema & Migration
- [x] 111개 테스트 (100% 통과)
- [x] SQLx 오프라인 모드 설정
- [x] 문서화 (ARCHITECTURE, FLOWCHARTS, API_REFERENCE)

### 구현 예정 항목 🚧

#### 1단계: HTTP API Layer (우선순위 높음)

```bash
# 생성할 파일들
infrastructure/src/handlers/
├── user_handler.rs      # POST /users/register, POST /auth/login
├── client_handler.rs    # POST /clients/register
├── auth_handler.rs      # GET /authorize, POST /authorize/consent
├── token_handler.rs     # POST /token, POST /revoke, POST /introspect
└── mod.rs

infrastructure/src/main.rs  # Actix-web 서버
```

**구현 가이드:**

```rust
// 예: user_handler.rs
use actix_web::{web, HttpResponse};
use application::ports::input::{UserUseCase, RegisterUserInput};

pub async fn register_user(
    user_service: web::Data<dyn UserUseCase>,
    req: web::Json<RegisterUserInput>,
) -> HttpResponse {
    match user_service.register_user(req.into_inner()).await {
        Ok(output) => HttpResponse::Created().json(output),
        Err(e) => HttpResponse::BadRequest().json(e),
    }
}
```

**라우팅:**

```rust
// main.rs
use actix_web::{App, HttpServer, web};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/users/register", web::post().to(register_user))
            .route("/auth/login", web::post().to(authenticate_user))
            .route("/authorize", web::get().to(authorize_page))
            .route("/token", web::post().to(token_endpoint))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

#### 2단계: JWT 토큰 발급

```bash
# 추가 의존성
[dependencies]
jsonwebtoken = { workspace = true }
```

**구현:**

```rust
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,  // User ID
    aud: String,  // Client ID
    scope: String,
    exp: usize,   // Expiration
    iat: usize,   // Issued at
}

fn create_jwt(user_id: UserId, client_id: ClientId, scope: &Scopes) -> String {
    let claims = Claims {
        sub: user_id.to_string(),
        aud: client_id.to_string(),
        scope: scope.to_string(),
        exp: (Utc::now() + Duration::hours(1)).timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(b"secret")).unwrap()
}
```

#### 3단계: 환경 설정

```bash
# .env 파일 생성
DATABASE_URL=sqlite:oauth.db
JWT_SECRET=your-secret-key-change-in-production
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
LOG_LEVEL=info
```

**사용:**

```rust
use dotenv::dotenv;
use std::env;

fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
}
```

#### 4단계: 로깅

```bash
# 의존성 (이미 추가됨)
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
```

**설정:**

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

// 사용
tracing::info!("Server starting on port 8080");
tracing::debug!("User registered: {}", user_id);
```

#### 5단계: Docker 컨테이너화

```dockerfile
# Dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates
COPY --from=builder /app/target/release/oauth-server /usr/local/bin/
COPY infrastructure/migrations /app/migrations
WORKDIR /app
EXPOSE 8080
CMD ["oauth-server"]
```

```yaml
# docker-compose.yml
version: '3.8'
services:
  oauth-server:
    build: .
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=sqlite:/data/oauth.db
      - JWT_SECRET=${JWT_SECRET}
    volumes:
      - ./data:/data
```

---

## 일반적인 작업

### 새로운 엔티티 추가

1. **Domain Layer에 추가:**
```rust
// domain/src/entities/session.rs
pub struct Session {
    id: SessionId,
    user_id: UserId,
    expires_at: DateTime<Utc>,
}
```

2. **Repository Port 정의:**
```rust
// application/src/ports/output/session_repository.rs
#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn save(&self, session: &Session) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: SessionId) -> Result<Option<Session>, RepositoryError>;
}
```

3. **Repository 구현:**
```rust
// infrastructure/src/repositories/session_repository.rs
pub struct SqliteSessionRepository {
    pool: SqlitePool,
}
```

4. **마이그레이션 추가:**
```sql
-- infrastructure/migrations/20250123000000_add_sessions.sql
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
```

5. **테스트 작성:**
```rust
// infrastructure/src/repositories/session_repository.rs (tests 모듈)
#[tokio::test]
async fn should_save_and_find_session() {
    // ...
}
```

### 새로운 Use Case 추가

1. **Input Port 정의:**
```rust
// application/src/ports/input/session_use_case.rs
#[async_trait]
pub trait SessionUseCase: Send + Sync {
    async fn create_session(&self, user_id: UserId) -> Result<SessionOutput, UseCaseError>;
}
```

2. **Service 구현:**
```rust
// application/src/services/session_service.rs
pub struct SessionService<R: SessionRepository> {
    session_repository: Arc<R>,
}
```

3. **테스트 작성:**
```rust
#[tokio::test]
async fn should_create_session() {
    // Mock repository 사용
}
```

### 데이터베이스 마이그레이션

```bash
# 1. 새로운 마이그레이션 파일 생성
cd infrastructure/migrations
touch 20250123000000_add_feature.sql

# 2. SQL 작성
cat > 20250123000000_add_feature.sql << EOF
-- Add your schema changes here
ALTER TABLE users ADD COLUMN phone TEXT;
EOF

# 3. 마이그레이션 실행
cd ../..
DATABASE_URL="sqlite:infrastructure/temp.db" sqlx migrate run --source infrastructure/migrations

# 4. SQLx 쿼리 캐시 업데이트
DATABASE_URL="sqlite:infrastructure/temp.db" cargo sqlx prepare --workspace

# 5. 테스트
cargo test --workspace --lib
```

### 에러 처리 추가

```rust
// Domain Layer
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Session expired")]
    Expired,
    #[error("Session not found")]
    NotFound,
}

// Application Layer
impl From<SessionError> for UseCaseError {
    fn from(e: SessionError) -> Self {
        match e {
            SessionError::Expired => UseCaseError::Unauthorized,
            SessionError::NotFound => UseCaseError::NotFound,
        }
    }
}
```

---

## 유용한 명령어 모음

```bash
# 🔨 개발
cargo build --workspace                    # 전체 빌드
cargo build --workspace --release          # 릴리스 빌드
cargo watch -x "test --lib"                # 파일 변경 시 자동 테스트

# 🧪 테스트
cargo test --workspace --lib               # 전체 테스트
cargo test --lib -p domain                 # Domain만
cargo test --lib should_register           # 이름으로 필터
cargo test --lib -- --nocapture            # 출력 표시

# ✨ 코드 품질
cargo clippy --workspace --all-targets -- -D warnings  # Lint
cargo fmt --all                            # 포맷팅
cargo fmt --all -- --check                 # 포맷 확인만

# 📦 의존성
cargo tree                                 # 의존성 트리
cargo update                               # 의존성 업데이트
cargo audit                                # 보안 감사

# 🗄️ 데이터베이스
sqlx database create                       # DB 생성
sqlx migrate run                           # 마이그레이션
sqlx migrate revert                        # 마지막 마이그레이션 되돌리기
cargo sqlx prepare --workspace             # 쿼리 캐시 생성

# 📊 분석
cargo bloat --release                      # 바이너리 크기 분석
cargo +nightly udeps --workspace           # 사용하지 않는 의존성 찾기
```

---

## 트러블슈팅

### SQLx 쿼리 에러

**문제:** `error: set DATABASE_URL to use query macros online`

**해결:**
```bash
DATABASE_URL="sqlite:infrastructure/temp.db" sqlx database create
DATABASE_URL="sqlite:infrastructure/temp.db" sqlx migrate run --source infrastructure/migrations
DATABASE_URL="sqlite:infrastructure/temp.db" cargo sqlx prepare --workspace
```

### 외래 키 제약조건 에러

**문제:** `FOREIGN KEY constraint failed`

**해결:** 테스트에서 참조되는 User와 Client를 먼저 생성
```rust
async fn create_test_user(db: &Database) -> UserId {
    // User 생성 후 ID 반환
}

async fn create_test_client(db: &Database) -> ClientId {
    // Client 생성 후 ID 반환
}

// 사용
let user_id = create_test_user(&db).await;
let client_id = create_test_client(&db).await;
let auth_code = AuthorizationCode::generate(client_id, user_id, ...);
```

### 컴파일 에러: trait not in scope

**문제:** `method XYZ not found for this struct`

**해결:** Trait을 import
```rust
use application::ports::UserRepository;  // Trait
use infrastructure::repositories::SqliteUserRepository;  // 구현체
```

---

## 학습 리소스

### 프로젝트 문서
- [ARCHITECTURE.md](ARCHITECTURE.md) - 아키텍처 설계
- [FLOWCHARTS.md](FLOWCHARTS.md) - 플로우차트 및 다이어그램
- [API_REFERENCE.md](API_REFERENCE.md) - API 레퍼런스

### 외부 리소스
- [OAuth 2.0 RFC 6749](https://datatracker.ietf.org/doc/html/rfc6749)
- [PKCE RFC 7636](https://datatracker.ietf.org/doc/html/rfc7636)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Actix-web Documentation](https://actix.rs/)
- [SQLx Documentation](https://docs.rs/sqlx/)

### 추천 학습 순서
1. ✅ Value Objects 이해 (Email, Password)
2. ✅ Entities 탐색 (User, OAuthClient)
3. ✅ Services 로직 분석 (UserService, TokenService)
4. ✅ Repository 패턴 이해
5. 🚧 HTTP Handlers 구현 (다음 단계)
6. 🚧 JWT 토큰 발급
7. 🚧 E2E 테스트 작성

---

## 기여 가이드

### 코드 스타일

- **포맷팅:** `cargo fmt --all` 사용
- **Lint:** `cargo clippy` 경고 없음
- **네이밍:** Rust 컨벤션 준수 (snake_case, CamelCase)
- **문서화:** Public API는 doc comment (`///`) 필수

### 커밋 메시지

```
feat: Add session management
fix: Fix PKCE verification bug
docs: Update API reference
test: Add integration tests for token service
refactor: Extract common validation logic
```

### Pull Request

1. 새로운 브랜치 생성: `git checkout -b feature/session-management`
2. 변경사항 커밋
3. 테스트 통과 확인: `cargo test --workspace --lib`
4. Clippy 확인: `cargo clippy --workspace --all-targets -- -D warnings`
5. PR 생성 및 리뷰 요청

---

## FAQ

### Q: SQLite 대신 PostgreSQL을 사용할 수 있나요?

**A:** 가능합니다. Repository 구현체만 교체하면 됩니다.

```toml
# Cargo.toml
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-native-tls"] }
```

```rust
// 새로운 PostgresUserRepository 구현
pub struct PostgresUserRepository {
    pool: PgPool,
}
```

### Q: JWT 대신 Opaque Token을 사용하나요?

**A:** 현재는 Opaque Token (랜덤 문자열)입니다. JWT 발급은 다음 단계입니다.

### Q: 클라이언트 시크릿은 어떻게 저장하나요?

**A:** 현재는 평문 저장입니다. 프로덕션에서는 해싱 권장 (Argon2 또는 bcrypt).

### Q: HTTPS는 어떻게 설정하나요?

**A:** Actix-web에서 TLS 설정:
```rust
use actix_web::HttpServer;

HttpServer::new(|| App::new())
    .bind_openssl("127.0.0.1:8443", builder)?  // OpenSSL
    .run()
    .await
```

또는 리버스 프록시 (Nginx, Caddy) 사용 권장.

---

**문서 버전**: 1.0
**최종 수정**: 2025-01-22
**유지보수**: Claude Code (Anthropic)

**Happy Coding! 🦀**
