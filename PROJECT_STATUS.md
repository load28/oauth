# OAuth 2.0 Authorization Server - 프로젝트 진행 상태

## 프로젝트 개요

Rust로 구현하는 엔터프라이즈급 OAuth 2.0 인증 서버

### 기술 스택
- **언어**: Rust (Edition 2021)
- **아키텍처**: Hexagonal Architecture (Ports & Adapters)
- **웹 프레임워크**: Actix-web
- **데이터베이스**: SQLite + SQLx (compile-time checked queries)
- **OAuth Flow**: Authorization Code Grant with PKCE

### 핵심 설계 원칙
- **Type-Driven Design**: Rust 타입 시스템을 활용한 컴파일 타임 안전성
- **Domain-Driven Design**: 비즈니스 로직을 도메인 레이어에 집중
- **Hexagonal Architecture**: 의존성 방향 제어 (Infrastructure → Application → Domain)

## 프로젝트 구조

```
login-server/
├── domain/           # 도메인 레이어 (비즈니스 로직)
├── application/      # 애플리케이션 레이어 (유스케이스)
└── infrastructure/   # 인프라 레이어 (DB, HTTP)
```

## 진행 상태

### ✅ 완료된 작업

#### 1. Cargo 워크스페이스 설정
- 3개 크레이트로 구성된 모노레포 구조
- 공유 의존성 관리
- 의존성 방향: Infrastructure → Application → Domain

#### 2. Domain Layer 구현 (테스트 72개 통과)

**Value Objects** (40 tests):
- `Email`: RFC 5321 준수, 유효성 검증, 소문자 정규화
- `UserId`, `ClientId`: UUID v4 기반 타입 안전 ID
- `PlainPassword`, `PasswordHash`: Argon2 해싱
- `Scope`, `Scopes`: OAuth 스코프 관리
- `CodeVerifier`, `CodeChallenge<Plain/S256>`: PKCE 구현 (Phantom Types 사용)

**Entities** (32 tests):
- `User`: 비밀번호 검증, 이메일 변경 로직
- `OAuthClient<Public/Confidential>`: Typestate 패턴으로 클라이언트 타입 구분
- `AuthorizationCode`: PKCE 검증, 만료 처리, 단일 사용
- `AccessToken`, `RefreshToken`: 만료 관리, 스코프 검증

**적용된 Rust 타입 패턴**:
- Newtype Pattern: 모든 도메인 개념에 적용
- Phantom Types: PKCE 메소드 추적 (`CodeChallenge<S256>`, `CodeChallenge<Plain>`)
- Typestate Pattern: OAuth 클라이언트 타입 (`OAuthClient<Public>`, `OAuthClient<Confidential>`)
- Validated Constructors: 생성 시 불변성 보장
- thiserror: 타입 안전 에러 처리

#### 3. Application Layer - Ports 정의

**Output Ports (Repository Interfaces)**:
- `UserRepository`: 사용자 영속화 인터페이스
- `ClientRepository`: OAuth 클라이언트 영속화, `ClientData` 타입 제공
- `AuthCodeRepository`: 인증 코드 영속화, 만료된 코드 정리
- `AccessTokenRepository`: 액세스 토큰 관리
- `RefreshTokenRepository`: 리프레시 토큰 관리, 회전 지원

**Input Ports (Use Case Interfaces)**:
- `UserUseCase`: 사용자 등록, 인증, 비밀번호 변경
  - DTOs: `RegisterUserInput/Output`, `AuthenticateUserInput/Output`, `ChangePasswordInput`
- `ClientUseCase`: OAuth 클라이언트 등록 및 관리
  - Public/Confidential 클라이언트 별도 처리
  - DTOs: `RegisterPublicClientInput/Output`, `RegisterConfidentialClientInput/Output`
- `AuthUseCase`: OAuth 인증 플로우
  - DTOs: `AuthorizeInput`, `AuthorizeOutput`
- `TokenUseCase`: 토큰 발급, 갱신, 폐기, 검사
  - DTOs: `ExchangeCodeInput`, `RefreshTokenInput`, `RevokeTokenInput`, `IntrospectTokenInput`
  - 출력: `TokenOutput`, `IntrospectTokenOutput`

**Error Types**:
- `RepositoryError`: 데이터베이스 오류
- `UseCaseError`: 유스케이스 실행 오류

### 🔄 현재 작업 중

#### Application Layer - Use Case Services 구현
- Repository 인터페이스를 사용한 비즈니스 로직 구현
- 도메인 엔티티 조합 및 트랜잭션 관리

**다음 단계**:
1. `UserService`: `UserUseCase` 구현
2. `ClientService`: `ClientUseCase` 구현
3. `AuthService`: `AuthUseCase` 구현
4. `TokenService`: `TokenUseCase` 구현

### ⏳ 예정된 작업

#### 5. Infrastructure Layer - SQLite 스키마
```sql
-- 예정된 테이블
- users
- oauth_clients
- authorization_codes
- access_tokens
- refresh_tokens
```

#### 6. Infrastructure Layer - Repository 구현
- SQLx를 사용한 컴파일 타임 SQL 검증
- Repository trait 구현
- 트랜잭션 지원

#### 7. Infrastructure Layer - HTTP Handlers
- Actix-web 핸들러 구현
- OAuth 2.0 엔드포인트:
  - `POST /register` - 사용자 등록
  - `POST /login` - 사용자 로그인
  - `GET /authorize` - 인증 요청
  - `POST /authorize` - 인증 승인
  - `POST /token` - 토큰 발급/갱신
  - `POST /revoke` - 토큰 폐기
  - `POST /introspect` - 토큰 검사
  - `POST /clients` - 클라이언트 등록

#### 8. 의존성 주입 및 AppState
- Repository 구현체 주입
- Service 인스턴스 생성
- Actix-web AppState 설정

#### 9. main.rs 서버 진입점
- 서버 초기화
- 데이터베이스 연결
- 라우팅 설정
- 서버 실행

#### 10. 테스트
- 도메인 레이어: 72개 테스트 통과 ✅
- 애플리케이션 레이어: 유스케이스 테스트 (mockall 사용)
- 인프라 레이어: 통합 테스트

#### 11. 환경 설정 및 문서화
- `.env` 파일 템플릿
- `README.md` 작성
- API 문서화

## 기술적 결정 사항

### 1. 타입 안전성
- **Phantom Types for PKCE**: `CodeChallenge<S256>`와 `CodeChallenge<Plain>`으로 메소드를 컴파일 타임에 추적
- **Typestate Pattern**: `OAuthClient<Public>`과 `OAuthClient<Confidential>`로 클라이언트 타입을 구분하여 잘못된 메소드 호출 방지

### 2. 보안
- **Argon2 Password Hashing**: 업계 표준 비밀번호 해싱
- **PKCE 필수**: Public 클라이언트는 PKCE 필수 (RFC 7636)
- **단일 사용 코드**: Authorization Code는 한 번만 사용 가능
- **토큰 만료**: Access Token 1시간, Refresh Token 30일
- **Client Secret**: Confidential 클라이언트만 시크릿 사용

### 3. RFC 준수
- **RFC 6749**: OAuth 2.0 Authorization Framework
- **RFC 7636**: Proof Key for Code Exchange (PKCE)
- **RFC 7009**: Token Revocation
- **RFC 7662**: Token Introspection
- **RFC 5321**: Email Address Validation

### 4. 아키텍처 결정
- **Hexagonal Architecture**: 비즈니스 로직과 인프라 분리
- **Port & Adapter**: async_trait로 비동기 인터페이스 정의
- **Repository Pattern**: 데이터 접근 추상화
- **DTO Pattern**: 레이어 간 데이터 전달

## 컴파일 상태

### Domain Layer
```bash
cargo test -p domain
# 72 tests passed ✅
```

### Application Layer
```bash
cargo check -p application
# 컴파일 오류 수정 중 (unused imports 경고 있음)
```

## 다음 실행 명령어

### 남은 컴파일 오류 수정
```bash
# 1. unused import 제거
# application/src/ports/input/auth_use_case.rs:7 - Scopes 제거
# application/src/ports/input/client_use_case.rs:7 - ClientSecret 제거
# application/src/ports/input/client_use_case.rs:8 - Scopes 제거
# application/src/ports/input/user_use_case.rs:8 - PlainPassword 제거

# 2. 컴파일 확인
cargo check -p application

# 3. 전체 워크스페이스 빌드
cargo build
```

### Use Case Services 구현 시작
```bash
# 1. UserService 구현
touch application/src/services/user_service.rs

# 2. services/mod.rs 업데이트
# 3. UserUseCase trait 구현
# 4. Repository 의존성 주입 (생성자)
```

## 파일 구조

```
login-server/
├── Cargo.toml                    # 워크스페이스 설정
├── PROJECT_STATUS.md             # 이 문서
│
├── domain/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── errors.rs             # 도메인 에러
│       ├── value_objects/
│       │   ├── mod.rs
│       │   ├── email.rs          # Email 값 객체
│       │   ├── user_id.rs        # UserId (UUID)
│       │   ├── client_id.rs      # ClientId (UUID)
│       │   ├── password.rs       # PlainPassword, PasswordHash
│       │   ├── scope.rs          # Scope, Scopes
│       │   └── pkce.rs           # CodeVerifier, CodeChallenge<M>
│       └── entities/
│           ├── mod.rs
│           ├── user.rs           # User 엔티티
│           ├── oauth_client.rs   # OAuthClient<T>
│           ├── authorization_code.rs  # AuthorizationCode
│           └── token.rs          # AccessToken, RefreshToken
│
├── application/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── errors.rs             # RepositoryError, UseCaseError
│       ├── ports/
│       │   ├── mod.rs
│       │   ├── input/
│       │   │   ├── mod.rs
│       │   │   ├── user_use_case.rs      # UserUseCase trait + DTOs
│       │   │   ├── client_use_case.rs    # ClientUseCase trait + DTOs
│       │   │   ├── auth_use_case.rs      # AuthUseCase trait + DTOs
│       │   │   └── token_use_case.rs     # TokenUseCase trait + DTOs
│       │   └── output/
│       │       ├── mod.rs
│       │       ├── user_repository.rs    # UserRepository trait
│       │       ├── client_repository.rs  # ClientRepository trait
│       │       ├── auth_code_repository.rs  # AuthCodeRepository trait
│       │       └── token_repository.rs   # AccessTokenRepository, RefreshTokenRepository
│       └── services/
│           └── mod.rs            # 서비스 구현 예정
│
└── infrastructure/
    ├── Cargo.toml
    └── src/
        └── lib.rs                # 아직 구현 안됨
```

## 주요 타입 시그니처

### Domain Layer

```rust
// Value Objects
pub struct Email(String);
pub struct UserId(Uuid);
pub struct ClientId(Uuid);
pub struct PlainPassword(String);
pub struct PasswordHash(String);
pub struct CodeVerifier(String);
pub struct CodeChallenge<M> { value: String, _method: PhantomData<M> }

// Entities
pub struct User { id: UserId, email: Email, password_hash: PasswordHash, ... }
pub struct OAuthClient<T> { id: ClientId, secret: Option<ClientSecret>, ... }
pub struct AuthorizationCode { code: String, code_challenge: Option<CodeChallenge<S256>>, ... }
pub struct AccessToken { token: String, expires_at: DateTime<Utc>, ... }
pub struct RefreshToken { token: String, revoked: bool, ... }
```

### Application Layer - Repository Traits

```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn save(&self, user: &User) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepositoryError>;
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepositoryError>;
}

#[async_trait]
pub trait ClientRepository: Send + Sync {
    async fn save_public(&self, client: &OAuthClient<Public>) -> Result<(), RepositoryError>;
    async fn save_confidential(&self, client: &OAuthClient<Confidential>) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: ClientId) -> Result<Option<ClientData>, RepositoryError>;
}
```

### Application Layer - Use Case Traits

```rust
#[async_trait]
pub trait UserUseCase: Send + Sync {
    async fn register_user(&self, input: RegisterUserInput) -> Result<RegisterUserOutput, UseCaseError>;
    async fn authenticate_user(&self, input: AuthenticateUserInput) -> Result<AuthenticateUserOutput, UseCaseError>;
}

#[async_trait]
pub trait TokenUseCase: Send + Sync {
    async fn exchange_code(&self, input: ExchangeCodeInput) -> Result<TokenOutput, UseCaseError>;
    async fn refresh_token(&self, input: RefreshTokenInput) -> Result<TokenOutput, UseCaseError>;
    async fn revoke_token(&self, input: RevokeTokenInput) -> Result<(), UseCaseError>;
}
```

## 참고 문서

### 프로젝트 스킬 문서
- `rust-type-programming/type-safety.md`: Newtype, Phantom Types
- `rust-type-programming/type-driven-design.md`: Typestate Pattern
- `rust-type-programming/error-handling.md`: thiserror 사용법
- `rust-type-programming/hexagonal-architecture.md`: 아키텍처 가이드
- `technical-documentation-standards`: 기술 결정 기준

### OAuth 2.0 RFC
- RFC 6749: OAuth 2.0 Authorization Framework
- RFC 7636: PKCE
- RFC 7009: Token Revocation
- RFC 7662: Token Introspection

## 작업 우선순위

1. **즉시**: Application Layer 컴파일 오류 수정 (unused imports)
2. **다음**: Use Case Services 구현 (UserService부터 시작)
3. **그 다음**: SQLite 스키마 설계 및 마이그레이션
4. **이후**: Repository 구현 (SQLx)
5. **마지막**: HTTP Handlers 및 서버 실행

---

**마지막 업데이트**: 2025-11-21
**테스트 상태**: Domain 72/72 통과
**컴파일 상태**: Application - 경고만 있음 (unused imports)
