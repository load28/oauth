# OAuth 2.0 Login Server - API Reference

## 목차
1. [Domain Layer API](#domain-layer-api)
   - [Entities](#entities)
   - [Value Objects](#value-objects)
2. [Application Layer API](#application-layer-api)
   - [Use Cases](#use-cases)
   - [Repositories](#repositories)
3. [Infrastructure Layer API](#infrastructure-layer-api)

---

## Domain Layer API

### Entities

#### 1. User Entity

**파일**: `domain/src/entities/user.rs`

**구조:**
```rust
pub struct User {
    id: UserId,
    email: Email,
    password_hash: PasswordHash,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
```

**생성자:**

##### `User::new`
```rust
pub fn new(
    id: UserId,
    email: Email,
    password_hash: PasswordHash
) -> Self
```
새로운 사용자 엔티티를 생성합니다.

**Parameters:**
- `id`: 고유 사용자 ID (UUID)
- `email`: 검증된 이메일 주소
- `password_hash`: Argon2 해시된 비밀번호

**Returns:** `User` 인스턴스 (created_at, updated_at 자동 설정)

**Example:**
```rust
let user = User::new(
    UserId::new(),
    Email::new("user@example.com")?,
    password.hash()?
);
```

---

##### `User::from_existing`
```rust
pub fn from_existing(
    id: UserId,
    email: Email,
    password_hash: PasswordHash,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
) -> Self
```
데이터베이스에서 읽어온 데이터로 User를 재구성합니다.

**Used by:** Repository 구현체

---

**게터 메서드:**

##### `User::id`
```rust
pub fn id(&self) -> UserId
```
사용자 ID를 반환합니다 (Copy trait 구현).

---

##### `User::email`
```rust
pub fn email(&self) -> &Email
```
이메일 주소 참조를 반환합니다.

---

##### `User::password_hash`
```rust
pub fn password_hash(&self) -> &PasswordHash
```
비밀번호 해시 참조를 반환합니다.

---

##### `User::created_at`
```rust
pub fn created_at(&self) -> DateTime<Utc>
```
생성 시각을 반환합니다 (Copy).

---

##### `User::updated_at`
```rust
pub fn updated_at(&self) -> DateTime<Utc>
```
마지막 수정 시각을 반환합니다 (Copy).

---

**비즈니스 로직 메서드:**

##### `User::change_email`
```rust
pub fn change_email(
    &mut self,
    new_email: Email
) -> Result<(), UserError>
```
이메일 주소를 변경합니다.

**Parameters:**
- `new_email`: 새로운 이메일 (검증됨)

**Returns:**
- `Ok(())`: 성공
- `Err(UserError::SameEmail)`: 동일한 이메일

**Side effects:** `updated_at` 갱신

---

##### `User::change_password`
```rust
pub fn change_password(&mut self, new_password_hash: PasswordHash)
```
비밀번호를 변경합니다.

**Parameters:**
- `new_password_hash`: 새로운 Argon2 해시

**Side effects:** `updated_at` 갱신

---

##### `User::verify_password`
```rust
pub fn verify_password(
    &self,
    plain_password: &PlainPassword
) -> Result<bool, UserError>
```
비밀번호를 검증합니다.

**Parameters:**
- `plain_password`: 평문 비밀번호

**Returns:**
- `Ok(true)`: 비밀번호 일치
- `Ok(false)`: 비밀번호 불일치
- `Err`: 검증 과정에서 에러

**Security:** Constant-time comparison (타이밍 공격 방지)

---

#### 2. OAuthClient Entity

**파일**: `domain/src/entities/oauth_client.rs`

**타입 파라미터:**
- `T`: `Public` 또는 `Confidential` (Marker Type)

**구조:**
```rust
pub struct OAuthClient<T> {
    id: ClientId,
    name: String,
    redirect_uris: Vec<String>,
    allowed_scopes: Scopes,
    secret: Option<ClientSecret>,  // Confidential만 Some
    created_at: DateTime<Utc>,
    _client_type: PhantomData<T>,
}
```

**Public 클라이언트 생성자:**

##### `OAuthClient::<Public>::new`
```rust
impl OAuthClient<Public> {
    pub fn new(
        id: ClientId,
        name: impl Into<String>,
        redirect_uris: Vec<String>,
        allowed_scopes: Scopes,
    ) -> Self
}
```
Public OAuth 클라이언트를 생성합니다 (SPA, 모바일 앱용).

**Invariants:**
- `secret`는 항상 `None`
- PKCE 필수

**Example:**
```rust
let client = OAuthClient::<Public>::new(
    ClientId::new(),
    "My SPA",
    vec!["https://app.example.com/callback".to_string()],
    Scopes::parse("openid profile")?,
);
```

---

**Confidential 클라이언트 생성자:**

##### `OAuthClient::<Confidential>::new`
```rust
impl OAuthClient<Confidential> {
    pub fn new(
        id: ClientId,
        name: impl Into<String>,
        secret: ClientSecret,
        redirect_uris: Vec<String>,
        allowed_scopes: Scopes,
    ) -> Self
}
```
Confidential OAuth 클라이언트를 생성합니다 (서버 사이드 앱용).

**Parameters:**
- `secret`: 클라이언트 시크릿 (32바이트 랜덤)

**Example:**
```rust
let secret = ClientSecret::generate();
let client = OAuthClient::<Confidential>::new(
    ClientId::new(),
    "My Backend",
    secret,
    vec!["https://backend.example.com/callback".to_string()],
    Scopes::parse("openid profile email")?,
);
```

---

**공통 메서드:**

##### `OAuthClient::from_existing`
```rust
pub fn from_existing(
    id: ClientId,
    name: impl Into<String>,
    redirect_uris: Vec<String>,
    allowed_scopes: Scopes,
    secret: Option<ClientSecret>,
    created_at: DateTime<Utc>,
) -> Self
```
데이터베이스에서 복원할 때 사용합니다.

---

##### `OAuthClient::id`
```rust
pub fn id(&self) -> ClientId
```
클라이언트 ID 반환 (Copy).

---

##### `OAuthClient::name`
```rust
pub fn name(&self) -> &str
```
클라이언트 이름 반환.

---

##### `OAuthClient::redirect_uris`
```rust
pub fn redirect_uris(&self) -> &[String]
```
허용된 Redirect URI 목록 반환.

---

##### `OAuthClient::allowed_scopes`
```rust
pub fn allowed_scopes(&self) -> &Scopes
```
허용된 스코프 목록 반환.

---

##### `OAuthClient::created_at`
```rust
pub fn created_at(&self) -> DateTime<Utc>
```
생성 시각 반환.

---

##### `OAuthClient::validate_redirect_uri`
```rust
pub fn validate_redirect_uri(
    &self,
    redirect_uri: &str
) -> Result<(), ClientError>
```
Redirect URI가 등록된 URI 목록에 있는지 확인합니다.

**Parameters:**
- `redirect_uri`: 검증할 URI

**Returns:**
- `Ok(())`: 유효함
- `Err(ClientError::InvalidRedirectUri)`: 등록되지 않은 URI

**Security:** 정확히 일치하는지 확인 (와일드카드 불허)

---

##### `OAuthClient::validate_scopes`
```rust
pub fn validate_scopes(
    &self,
    requested_scopes: &Scopes
) -> Result<(), ClientError>
```
요청된 스코프가 허용된 스코프에 포함되는지 확인합니다.

**Returns:**
- `Ok(())`: 모든 스코프 허용됨
- `Err(ClientError::ScopeNotAllowed)`: 허용되지 않은 스코프 포함

---

##### `OAuthClient::is_confidential`
```rust
pub fn is_confidential(&self) -> bool
```
Confidential 클라이언트 여부 확인.

---

##### `OAuthClient::is_public`
```rust
pub fn is_public(&self) -> bool
```
Public 클라이언트 여부 확인.

---

**Public 전용 메서드:**

##### `OAuthClient::<Public>::requires_pkce`
```rust
impl OAuthClient<Public> {
    pub fn requires_pkce() -> bool { true }
}
```
항상 `true` 반환 (컴파일 타임 상수).

---

**Confidential 전용 메서드:**

##### `OAuthClient::<Confidential>::verify_secret`
```rust
impl OAuthClient<Confidential> {
    pub fn verify_secret(&self, secret: &str) -> Result<(), ClientError>
}
```
클라이언트 시크릿을 검증합니다.

**Parameters:**
- `secret`: 검증할 시크릿

**Returns:**
- `Ok(())`: 시크릿 일치
- `Err(ClientError::InvalidClientSecret)`: 시크릿 불일치

**Security:** Constant-time comparison

---

##### `OAuthClient::<Confidential>::requires_pkce`
```rust
pub fn requires_pkce() -> bool { false }
```
항상 `false` 반환 (PKCE 선택사항).

---

##### `OAuthClient::<Confidential>::secret`
```rust
pub fn secret(&self) -> Option<&ClientSecret>
```
클라이언트 시크릿 참조 반환.

---

#### 3. AuthorizationCode Entity

**파일**: `domain/src/entities/authorization_code.rs`

**구조:**
```rust
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
```

**생성자:**

##### `AuthorizationCode::generate`
```rust
pub fn generate(
    client_id: ClientId,
    user_id: UserId,
    redirect_uri: String,
    scope: Scopes,
    code_challenge: Option<CodeChallenge<S256>>,
) -> Self
```
새로운 인가 코드를 생성합니다.

**Parameters:**
- `client_id`: 요청한 클라이언트 ID
- `user_id`: 인가를 부여한 사용자 ID
- `redirect_uri`: 사용할 리다이렉트 URI
- `scope`: 부여된 스코프
- `code_challenge`: PKCE 챌린지 (Public 클라이언트는 필수)

**Returns:** 새로운 `AuthorizationCode` (32바이트 랜덤, Base64 URL-Safe)

**Expiration:** 10분 후

**Example:**
```rust
let verifier = CodeVerifier::generate();
let challenge = verifier.create_s256_challenge();

let code = AuthorizationCode::generate(
    client_id,
    user_id,
    "https://example.com/callback".to_string(),
    Scopes::parse("openid profile")?,
    Some(challenge),
);
```

---

##### `AuthorizationCode::from_existing`
```rust
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
) -> Self
```
데이터베이스에서 복원.

---

**게터 메서드:**

##### `AuthorizationCode::code`
```rust
pub fn code(&self) -> &str
```
인가 코드 문자열 반환.

---

##### `AuthorizationCode::client_id`
```rust
pub fn client_id(&self) -> ClientId
```
클라이언트 ID 반환.

---

##### `AuthorizationCode::user_id`
```rust
pub fn user_id(&self) -> UserId
```
사용자 ID 반환.

---

##### `AuthorizationCode::redirect_uri`
```rust
pub fn redirect_uri(&self) -> &str
```
리다이렉트 URI 반환.

---

##### `AuthorizationCode::scope`
```rust
pub fn scope(&self) -> &Scopes
```
부여된 스코프 반환.

---

##### `AuthorizationCode::code_challenge`
```rust
pub fn code_challenge(&self) -> Option<&CodeChallenge<S256>>
```
PKCE 챌린지 반환.

---

##### `AuthorizationCode::expires_at`
```rust
pub fn expires_at(&self) -> DateTime<Utc>
```
만료 시각 반환.

---

##### `AuthorizationCode::created_at`
```rust
pub fn created_at(&self) -> DateTime<Utc>
```
생성 시각 반환.

---

##### `AuthorizationCode::is_used`
```rust
pub fn is_used(&self) -> bool
```
사용 여부 반환.

---

##### `AuthorizationCode::is_expired`
```rust
pub fn is_expired(&self) -> bool
```
만료 여부 확인.

---

**비즈니스 로직:**

##### `AuthorizationCode::validate`
```rust
pub fn validate(
    &self,
    client_id: ClientId,
    redirect_uri: &str,
    code_verifier: Option<&CodeVerifier>,
) -> Result<(), AuthCodeError>
```
토큰 교환 시 인가 코드를 검증합니다.

**Validation Steps:**
1. 만료 확인 (`is_expired`)
2. 사용 여부 확인 (`is_used`)
3. 클라이언트 ID 일치
4. Redirect URI 일치
5. PKCE 검증 (챌린지가 있는 경우)

**Parameters:**
- `client_id`: 요청한 클라이언트 ID
- `redirect_uri`: 제공된 리다이렉트 URI
- `code_verifier`: PKCE verifier (챌린지가 있으면 필수)

**Returns:**
- `Ok(())`: 모든 검증 통과
- `Err(AuthCodeError::Expired)`: 만료됨
- `Err(AuthCodeError::AlreadyUsed)`: 이미 사용됨
- `Err(AuthCodeError::InvalidCodeVerifier)`: PKCE 검증 실패
- `Err(AuthCodeError::PkceRequired)`: Verifier 누락

**Security:** 일회용 강제, PKCE 검증

---

##### `AuthorizationCode::mark_as_used`
```rust
pub fn mark_as_used(&mut self)
```
코드를 사용됨으로 표시합니다.

**Side effects:** `used = true`

**Purpose:** 재사용 공격 방지

---

#### 4. AccessToken Entity

**파일**: `domain/src/entities/token.rs`

**구조:**
```rust
pub struct AccessToken {
    token: String,
    client_id: ClientId,
    user_id: UserId,
    scope: Scopes,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}
```

**상수:**
```rust
const DEFAULT_EXPIRATION_HOURS: i64 = 1;
```

**생성자:**

##### `AccessToken::new`
```rust
pub fn new(
    token: String,
    client_id: ClientId,
    user_id: UserId,
    scope: Scopes,
) -> Self
```
기본 만료 시간(1시간)으로 액세스 토큰 생성.

**Example:**
```rust
let token = AccessToken::new(
    "eyJhbGciOiJ...".to_string(),  // JWT
    client_id,
    user_id,
    Scopes::parse("openid profile")?,
);
```

---

##### `AccessToken::with_expiration`
```rust
pub fn with_expiration(
    token: String,
    client_id: ClientId,
    user_id: UserId,
    scope: Scopes,
    expires_in_seconds: i64,
) -> Self
```
커스텀 만료 시간으로 생성.

---

##### `AccessToken::from_existing`
```rust
pub fn from_existing(
    token: String,
    client_id: ClientId,
    user_id: UserId,
    scope: Scopes,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
) -> Self
```
데이터베이스에서 복원.

---

**게터 메서드:**

##### `AccessToken::token`
```rust
pub fn token(&self) -> &str
```
토큰 문자열 반환.

---

##### `AccessToken::client_id`
```rust
pub fn client_id(&self) -> ClientId
```
클라이언트 ID 반환.

---

##### `AccessToken::user_id`
```rust
pub fn user_id(&self) -> UserId
```
사용자 ID 반환.

---

##### `AccessToken::scope`
```rust
pub fn scope(&self) -> &Scopes
```
부여된 스코프 반환.

---

##### `AccessToken::expires_at`
```rust
pub fn expires_at(&self) -> DateTime<Utc>
```
만료 시각 반환.

---

##### `AccessToken::created_at`
```rust
pub fn created_at(&self) -> DateTime<Utc>
```
생성 시각 반환.

---

**검증 메서드:**

##### `AccessToken::is_expired`
```rust
pub fn is_expired(&self) -> bool
```
만료 여부 확인.

---

##### `AccessToken::expires_in`
```rust
pub fn expires_in(&self) -> i64
```
만료까지 남은 시간 (초) 반환.

**Returns:** 0 이상의 정수 (만료된 경우 0)

---

##### `AccessToken::validate`
```rust
pub fn validate(&self) -> Result<(), TokenError>
```
토큰 유효성 검증.

**Returns:**
- `Ok(())`: 유효함
- `Err(TokenError::Expired)`: 만료됨

---

##### `AccessToken::validate_scope`
```rust
pub fn validate_scope(
    &self,
    required_scope: &Scopes
) -> Result<(), TokenError>
```
요구되는 스코프를 가지고 있는지 확인.

**Parameters:**
- `required_scope`: 필요한 스코프

**Returns:**
- `Ok(())`: 모든 스코프 포함
- `Err(TokenError::InsufficientScope)`: 스코프 부족

---

#### 5. RefreshToken Entity

**파일**: `domain/src/entities/token.rs`

**구조:**
```rust
pub struct RefreshToken {
    token: String,
    access_token_id: String,
    client_id: ClientId,
    user_id: UserId,
    scope: Scopes,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    revoked: bool,
}
```

**상수:**
```rust
const DEFAULT_EXPIRATION_DAYS: i64 = 30;
```

**생성자:**

##### `RefreshToken::generate`
```rust
pub fn generate(
    access_token_id: String,
    client_id: ClientId,
    user_id: UserId,
    scope: Scopes,
) -> Self
```
새로운 리프레시 토큰 생성 (32바이트 랜덤).

**Parameters:**
- `access_token_id`: 연결된 액세스 토큰 ID

**Expiration:** 30일 후

**Example:**
```rust
let refresh_token = RefreshToken::generate(
    access_token.token().to_string(),
    client_id,
    user_id,
    scope,
);
```

---

##### `RefreshToken::from_existing`
```rust
pub fn from_existing(
    token: String,
    access_token_id: String,
    client_id: ClientId,
    user_id: UserId,
    scope: Scopes,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    revoked: bool,
) -> Self
```
데이터베이스에서 복원.

---

**게터 메서드:**

##### `RefreshToken::token`
```rust
pub fn token(&self) -> &str
```
토큰 문자열 반환.

---

##### `RefreshToken::access_token_id`
```rust
pub fn access_token_id(&self) -> &str
```
연결된 액세스 토큰 ID 반환.

---

##### `RefreshToken::client_id`
```rust
pub fn client_id(&self) -> ClientId
```
클라이언트 ID 반환.

---

##### `RefreshToken::user_id`
```rust
pub fn user_id(&self) -> UserId
```
사용자 ID 반환.

---

##### `RefreshToken::scope`
```rust
pub fn scope(&self) -> &Scopes
```
부여된 스코프 반환.

---

##### `RefreshToken::expires_at`
```rust
pub fn expires_at(&self) -> DateTime<Utc>
```
만료 시각 반환.

---

##### `RefreshToken::created_at`
```rust
pub fn created_at(&self) -> DateTime<Utc>
```
생성 시각 반환.

---

##### `RefreshToken::is_revoked`
```rust
pub fn is_revoked(&self) -> bool
```
취소 여부 반환.

---

##### `RefreshToken::is_expired`
```rust
pub fn is_expired(&self) -> bool
```
만료 여부 확인.

---

**검증 메서드:**

##### `RefreshToken::validate`
```rust
pub fn validate(&self) -> Result<(), TokenError>
```
리프레시 토큰 유효성 검증.

**Returns:**
- `Ok(())`: 유효함
- `Err(TokenError::Revoked)`: 취소됨
- `Err(TokenError::Expired)`: 만료됨

---

**상태 변경:**

##### `RefreshToken::revoke`
```rust
pub fn revoke(&mut self)
```
토큰을 취소합니다.

**Side effects:** `revoked = true` (영구적, 되돌릴 수 없음)

---

### Value Objects

#### 6. UserId

**파일**: `domain/src/value_objects/user_id.rs`

**구조:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(Uuid);
```

**메서드:**

##### `UserId::new`
```rust
pub fn new() -> Self
```
새로운 UUID v4 생성.

---

##### `UserId::from_uuid`
```rust
pub fn from_uuid(uuid: Uuid) -> Self
```
기존 UUID로부터 생성.

---

##### `UserId::parse`
```rust
pub fn parse(s: &str) -> Result<Self, uuid::Error>
```
문자열 파싱.

**Example:** `UserId::parse("550e8400-e29b-41d4-a716-446655440000")?`

---

##### `UserId::value`
```rust
pub fn value(&self) -> Uuid
```
내부 UUID 반환.

---

**Display 구현:**
```rust
// ToString 사용 가능
let id_string = user_id.to_string();
```

---

#### 7. ClientId

**파일**: `domain/src/value_objects/client_id.rs`

UserId와 동일한 API (UUID v4 기반).

---

#### 8. Email

**파일**: `domain/src/value_objects/email.rs`

**구조:**
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);
```

**메서드:**

##### `Email::new`
```rust
pub fn new(email: impl Into<String>) -> Result<Self, EmailError>
```
이메일 생성 및 검증.

**Validation:**
- 빈 문자열 거부
- 최대 254자 (RFC 5321)
- '@' 기호 포함 필수
- '@' 뒤에 '.' 포함 필수
- '@' 기호 1개만 허용
- 소문자로 정규화

**Returns:**
- `Ok(Email)`: 유효한 이메일
- `Err(EmailError::*)`: 검증 실패

**Example:**
```rust
let email = Email::new("user@example.com")?;
let email = Email::new("USER@EXAMPLE.COM")?;  // "user@example.com"으로 정규화
```

---

##### `Email::new_unchecked`
```rust
pub fn new_unchecked(email: String) -> Self
```
검증 없이 생성 (내부용, 데이터베이스에서 읽을 때).

---

##### `Email::as_str`
```rust
pub fn as_str(&self) -> &str
```
이메일 문자열 참조 반환.

---

#### 9. PlainPassword

**파일**: `domain/src/value_objects/password.rs`

**구조:**
```rust
pub struct PlainPassword(String);
```

**메서드:**

##### `PlainPassword::new`
```rust
pub fn new(password: impl Into<String>) -> Result<Self, PasswordError>
```
평문 비밀번호 생성 및 검증.

**Validation:**
- 최소 8자
- 최대 128자

**Returns:**
- `Ok(PlainPassword)`
- `Err(PasswordError::TooShort | TooLong)`

---

##### `PlainPassword::as_bytes`
```rust
pub fn as_bytes(&self) -> &[u8]
```
비밀번호 바이트 참조 반환.

---

##### `PlainPassword::hash`
```rust
pub fn hash(&self) -> Result<PasswordHash, PasswordError>
```
Argon2id로 비밀번호 해싱.

**Algorithm:** Argon2id (메모리 하드, CPU 하드)
**Salt:** 랜덤 생성 (각 해싱마다 고유)
**Time:** ~100-200ms (보안과 성능 균형)

**Returns:** `PasswordHash` 또는 에러

---

#### 10. PasswordHash

**파일**: `domain/src/value_objects/password.rs`

**구조:**
```rust
pub struct PasswordHash(String);
```

**메서드:**

##### `PasswordHash::new`
```rust
pub fn new(hash: String) -> Result<Self, PasswordError>
```
Argon2 해시 문자열로부터 생성.

**Validation:** Argon2 형식 검증

---

##### `PasswordHash::verify`
```rust
pub fn verify(
    &self,
    plain: &PlainPassword
) -> Result<bool, PasswordError>
```
평문 비밀번호를 검증합니다.

**Parameters:**
- `plain`: 검증할 평문 비밀번호

**Returns:**
- `Ok(true)`: 일치
- `Ok(false)`: 불일치
- `Err`: 검증 과정 에러

**Security:** Constant-time comparison

---

##### `PasswordHash::as_str`
```rust
pub fn as_str(&self) -> &str
```
해시 문자열 반환.

---

#### 11. Scopes

**파일**: `domain/src/value_objects/scope.rs`

**구조:**
```rust
pub struct Scopes(Vec<Scope>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Scope {
    OpenId,
    Profile,
    Email,
    Read,
    Write,
    Custom(String),
}
```

**메서드:**

##### `Scopes::new`
```rust
pub fn new() -> Self
```
빈 스코프 생성.

---

##### `Scopes::parse`
```rust
pub fn parse(scope_string: &str) -> Result<Self, ScopeError>
```
공백 구분 문자열로부터 파싱.

**Example:**
```rust
let scopes = Scopes::parse("openid profile email")?;
```

**Features:**
- 중복 자동 제거
- 추가 공백 무시
- 빈 문자열 거부

---

##### `Scopes::from_vec`
```rust
pub fn from_vec(scopes: Vec<Scope>) -> Result<Self, ScopeError>
```
벡터로부터 생성.

---

##### `Scopes::add`
```rust
pub fn add(&mut self, scope: Scope)
```
스코프 추가 (중복 시 무시).

---

##### `Scopes::contains`
```rust
pub fn contains(&self, scope: &Scope) -> bool
```
특정 스코프 포함 여부.

---

##### `Scopes::contains_all`
```rust
pub fn contains_all(&self, other: &Scopes) -> bool
```
다른 Scopes의 모든 스코프를 포함하는지 확인.

**Use case:** 스코프 검증, 토큰 검증

---

##### `Scopes::len`
```rust
pub fn len(&self) -> usize
```
스코프 개수 반환.

---

##### `Scopes::is_empty`
```rust
pub fn is_empty(&self) -> bool
```
스코프가 없는지 확인.

---

##### `Scopes::iter`
```rust
pub fn iter(&self) -> impl Iterator<Item = &Scope>
```
이터레이터 반환.

---

##### `Scopes::to_vec`
```rust
pub fn to_vec(&self) -> Vec<Scope>
```
벡터로 변환 (복제).

---

**Display 구현:**
```rust
// 공백 구분 문자열로 변환
let scope_string = scopes.to_string();  // "openid profile email"
```

---

#### 12. CodeVerifier (PKCE)

**파일**: `domain/src/value_objects/pkce.rs`

**구조:**
```rust
pub struct CodeVerifier(String);
```

**메서드:**

##### `CodeVerifier::generate`
```rust
pub fn generate() -> Self
```
암호학적으로 안전한 랜덤 verifier 생성 (43-128자).

**Character set:** Unreserved characters (RFC 3986)

---

##### `CodeVerifier::new`
```rust
pub fn new(verifier: impl Into<String>) -> Result<Self, PkceError>
```
문자열로부터 생성 및 검증.

**Validation:**
- 43-128자
- Unreserved characters only: `[A-Z] [a-z] [0-9] - . _ ~`

---

##### `CodeVerifier::as_str`
```rust
pub fn as_str(&self) -> &str
```
verifier 문자열 반환.

---

##### `CodeVerifier::create_s256_challenge`
```rust
pub fn create_s256_challenge(&self) -> CodeChallenge<S256>
```
SHA-256 챌린지 생성.

**Algorithm:** `BASE64URL(SHA256(verifier))`

**Example:**
```rust
let verifier = CodeVerifier::generate();
let challenge = verifier.create_s256_challenge();
```

---

##### `CodeVerifier::create_plain_challenge`
```rust
pub fn create_plain_challenge(&self) -> CodeChallenge<Plain>
```
Plain 챌린지 생성 (비권장).

---

#### 13. CodeChallenge (PKCE)

**파일**: `domain/src/value_objects/pkce.rs`

**구조:**
```rust
pub struct CodeChallenge<M> {
    value: String,
    _method: PhantomData<M>,
}

pub struct S256;   // SHA-256 method
pub struct Plain;  // Plain method (deprecated)
```

**S256 메서드:**

##### `CodeChallenge::<S256>::new_s256`
```rust
pub fn new_s256(challenge: String) -> Self
```
S256 챌린지 생성.

---

##### `CodeChallenge::<S256>::verify`
```rust
pub fn verify(
    &self,
    verifier: &CodeVerifier
) -> Result<bool, PkceError>
```
Verifier를 검증합니다.

**Algorithm:** `SHA256(verifier) == stored_challenge?`

**Returns:**
- `Ok(true)`: 검증 성공
- `Ok(false)`: 검증 실패

---

##### `CodeChallenge::<S256>::method`
```rust
pub fn method(&self) -> CodeChallengeMethod
```
`CodeChallengeMethod::S256` 반환.

---

**Plain 메서드:**

##### `CodeChallenge::<Plain>::new_plain`
```rust
pub fn new_plain(challenge: String) -> Self
```
Plain 챌린지 생성.

---

##### `CodeChallenge::<Plain>::verify`
```rust
pub fn verify(
    &self,
    verifier: &CodeVerifier
) -> Result<bool, PkceError>
```
Plain 검증 (verifier == challenge).

---

**공통 메서드:**

##### `CodeChallenge::as_str`
```rust
pub fn as_str(&self) -> &str
```
챌린지 문자열 반환.

---

##### `CodeChallenge::value`
```rust
pub fn value(&self) -> &str
```
`as_str`과 동일.

---

## Application Layer API

### Use Cases

#### 14. UserUseCase

**파일**: `application/src/ports/input/user_use_case.rs`

**트레이트:**
```rust
#[async_trait]
pub trait UserUseCase: Send + Sync {
    async fn register_user(
        &self,
        input: RegisterUserInput
    ) -> Result<RegisterUserOutput, UseCaseError>;

    async fn authenticate_user(
        &self,
        input: AuthenticateUserInput
    ) -> Result<AuthenticateUserOutput, UseCaseError>;

    async fn get_user(
        &self,
        user_id: UserId
    ) -> Result<User, UseCaseError>;

    async fn change_password(
        &self,
        input: ChangePasswordInput
    ) -> Result<(), UseCaseError>;
}
```

**DTOs:**

```rust
pub struct RegisterUserInput {
    pub email: String,
    pub password: String,
}

pub struct RegisterUserOutput {
    pub user_id: UserId,
    pub email: String,
}

pub struct AuthenticateUserInput {
    pub email: String,
    pub password: String,
}

pub struct AuthenticateUserOutput {
    pub user_id: UserId,
    pub email: String,
}

pub struct ChangePasswordInput {
    pub user_id: UserId,
    pub old_password: String,
    pub new_password: String,
}
```

**구현**: `UserService` (application/src/services/user_service.rs)

---

#### 15. ClientUseCase

**파일**: `application/src/ports/input/client_use_case.rs`

**트레이트:**
```rust
#[async_trait]
pub trait ClientUseCase: Send + Sync {
    async fn register_public_client(
        &self,
        input: RegisterPublicClientInput
    ) -> Result<RegisterPublicClientOutput, UseCaseError>;

    async fn register_confidential_client(
        &self,
        input: RegisterConfidentialClientInput
    ) -> Result<RegisterConfidentialClientOutput, UseCaseError>;

    async fn get_public_client(
        &self,
        client_id: ClientId
    ) -> Result<OAuthClient<Public>, UseCaseError>;

    async fn get_confidential_client(
        &self,
        client_id: ClientId
    ) -> Result<OAuthClient<Confidential>, UseCaseError>;

    async fn delete_client(
        &self,
        client_id: ClientId
    ) -> Result<(), UseCaseError>;
}
```

**DTOs:**

```rust
pub struct RegisterPublicClientInput {
    pub name: String,
    pub redirect_uris: Vec<String>,
    pub allowed_scopes: String,  // Space-separated
}

pub struct RegisterPublicClientOutput {
    pub client_id: ClientId,
    pub name: String,
    pub redirect_uris: Vec<String>,
}

pub struct RegisterConfidentialClientInput {
    pub name: String,
    pub redirect_uris: Vec<String>,
    pub allowed_scopes: String,
}

pub struct RegisterConfidentialClientOutput {
    pub client_id: ClientId,
    pub name: String,
    pub redirect_uris: Vec<String>,
    pub client_secret: String,  // ⚠️ 1회만 표시
}
```

**구현**: `ClientService`

---

#### 16. AuthUseCase

**파일**: `application/src/ports/input/auth_use_case.rs`

**트레이트:**
```rust
#[async_trait]
pub trait AuthUseCase: Send + Sync {
    async fn authorize(
        &self,
        input: AuthorizeInput
    ) -> Result<AuthorizeOutput, UseCaseError>;
}
```

**DTOs:**

```rust
pub struct AuthorizeInput {
    pub client_id: ClientId,
    pub user_id: UserId,
    pub redirect_uri: String,
    pub scope: String,  // Space-separated
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,  // "S256" or "plain"
    pub state: Option<String>,
}

pub struct AuthorizeOutput {
    pub authorization_code: String,
    pub redirect_uri: String,
    pub state: Option<String>,
}
```

**구현**: `AuthService`

---

#### 17. TokenUseCase

**파일**: `application/src/ports/input/token_use_case.rs`

**트레이트:**
```rust
#[async_trait]
pub trait TokenUseCase: Send + Sync {
    async fn exchange_code(
        &self,
        input: ExchangeCodeInput
    ) -> Result<TokenOutput, UseCaseError>;

    async fn refresh_token(
        &self,
        input: RefreshTokenInput
    ) -> Result<TokenOutput, UseCaseError>;

    async fn revoke_token(
        &self,
        input: RevokeTokenInput
    ) -> Result<(), UseCaseError>;

    async fn introspect_token(
        &self,
        input: IntrospectTokenInput
    ) -> Result<IntrospectTokenOutput, UseCaseError>;
}
```

**DTOs:**

```rust
pub struct ExchangeCodeInput {
    pub code: String,
    pub client_id: ClientId,
    pub client_secret: Option<String>,
    pub redirect_uri: String,
    pub code_verifier: Option<String>,
}

pub struct RefreshTokenInput {
    pub refresh_token: String,
    pub client_id: ClientId,
    pub client_secret: Option<String>,
    pub scope: Option<String>,  // Narrower scope
}

pub struct RevokeTokenInput {
    pub token: String,
    pub client_id: ClientId,
    pub client_secret: Option<String>,
    pub token_type_hint: Option<String>,  // "access_token" or "refresh_token"
}

pub struct TokenOutput {
    pub access_token: String,
    pub token_type: String,  // "Bearer"
    pub expires_in: i64,  // Seconds
    pub refresh_token: Option<String>,
    pub scope: String,  // Space-separated
}

pub struct IntrospectTokenInput {
    pub token: String,
    pub client_id: ClientId,
    pub client_secret: Option<String>,
}

pub struct IntrospectTokenOutput {
    pub active: bool,
    pub scope: Option<String>,
    pub client_id: Option<ClientId>,
    pub user_id: Option<UserId>,
    pub exp: Option<i64>,  // Unix timestamp
}
```

**구현**: `TokenService`

---

### Repositories

#### 18. UserRepository

**파일**: `application/src/ports/output/user_repository.rs`

**트레이트:**
```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn save(&self, user: &User) -> Result<(), RepositoryError>;

    async fn find_by_id(&self, id: UserId)
        -> Result<Option<User>, RepositoryError>;

    async fn find_by_email(&self, email: &Email)
        -> Result<Option<User>, RepositoryError>;

    async fn exists_by_email(&self, email: &Email)
        -> Result<bool, RepositoryError>;

    async fn delete(&self, id: UserId) -> Result<(), RepositoryError>;
}
```

**구현**: `SqliteUserRepository` (infrastructure)

---

#### 19. ClientRepository

**파일**: `application/src/ports/output/client_repository.rs`

**트레이트:**
```rust
#[async_trait]
pub trait ClientRepository: Send + Sync {
    async fn save_public(&self, client: &OAuthClient<Public>)
        -> Result<(), RepositoryError>;

    async fn save_confidential(&self, client: &OAuthClient<Confidential>)
        -> Result<(), RepositoryError>;

    async fn find_by_id(&self, id: ClientId)
        -> Result<Option<ClientData>, RepositoryError>;

    async fn delete(&self, id: ClientId) -> Result<(), RepositoryError>;
}

pub struct ClientData {
    pub id: ClientId,
    pub name: String,
    pub redirect_uris: Vec<String>,
    pub allowed_scopes: Scopes,
    pub secret: Option<ClientSecret>,
    pub created_at: DateTime<Utc>,
}

impl ClientData {
    pub fn to_public(self) -> OAuthClient<Public>;
    pub fn to_confidential(self) -> OAuthClient<Confidential>;
    pub fn is_confidential(&self) -> bool;
    pub fn is_public(&self) -> bool;
}
```

**구현**: `SqliteClientRepository`

---

#### 20. AuthCodeRepository

**파일**: `application/src/ports/output/auth_code_repository.rs`

**트레이트:**
```rust
#[async_trait]
pub trait AuthCodeRepository: Send + Sync {
    async fn save(&self, auth_code: &AuthorizationCode)
        -> Result<(), RepositoryError>;

    async fn find_by_code(&self, code: &str)
        -> Result<Option<AuthorizationCode>, RepositoryError>;

    async fn mark_as_used(&self, code: &str) -> Result<(), RepositoryError>;

    async fn delete(&self, code: &str) -> Result<(), RepositoryError>;

    async fn delete_expired(&self) -> Result<usize, RepositoryError>;
}
```

**구현**: `SqliteAuthCodeRepository`

---

#### 21. AccessTokenRepository

**파일**: `application/src/ports/output/token_repository.rs`

**트레이트:**
```rust
#[async_trait]
pub trait AccessTokenRepository: Send + Sync {
    async fn save(&self, token: &AccessToken) -> Result<(), RepositoryError>;

    async fn find_by_token(&self, token: &str)
        -> Result<Option<AccessToken>, RepositoryError>;

    async fn find_by_user_id(&self, user_id: UserId)
        -> Result<Vec<AccessToken>, RepositoryError>;

    async fn revoke(&self, token: &str) -> Result<(), RepositoryError>;

    async fn revoke_all_for_user(&self, user_id: UserId)
        -> Result<usize, RepositoryError>;

    async fn delete_expired(&self) -> Result<usize, RepositoryError>;
}
```

**구현**: `SqliteAccessTokenRepository`

---

#### 22. RefreshTokenRepository

**파일**: `application/src/ports/output/token_repository.rs`

**트레이트:**
```rust
#[async_trait]
pub trait RefreshTokenRepository: Send + Sync {
    async fn save(&self, token: &RefreshToken) -> Result<(), RepositoryError>;

    async fn find_by_token(&self, token: &str)
        -> Result<Option<RefreshToken>, RepositoryError>;

    async fn find_by_user_id(&self, user_id: UserId)
        -> Result<Vec<RefreshToken>, RepositoryError>;

    async fn find_by_access_token_id(&self, access_token_id: &str)
        -> Result<Option<RefreshToken>, RepositoryError>;

    async fn revoke(&self, token: &str) -> Result<(), RepositoryError>;

    async fn revoke_all_for_user(&self, user_id: UserId)
        -> Result<usize, RepositoryError>;

    async fn revoke_all_for_client(&self, client_id: ClientId)
        -> Result<usize, RepositoryError>;

    async fn delete_expired_and_revoked(&self) -> Result<usize, RepositoryError>;
}
```

**구현**: `SqliteRefreshTokenRepository`

---

## Infrastructure Layer API

### 23. Database

**파일**: `infrastructure/src/database.rs`

**구조:**
```rust
#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}
```

**메서드:**

##### `Database::new`
```rust
pub async fn new(database_url: &str) -> Result<Self, sqlx::Error>
```
SQLite 커넥션 풀 생성.

**Parameters:**
- `database_url`: `"sqlite:oauth.db"` 또는 `"sqlite::memory:"`

**Configuration:**
- `create_if_missing = true`
- `foreign_keys = true`
- `max_connections = 5`
- `acquire_timeout = 3s`

---

##### `Database::migrate`
```rust
pub async fn migrate(&self) -> Result<(), sqlx::Error>
```
데이터베이스 마이그레이션 실행.

**Migration files:** `infrastructure/migrations/`

---

##### `Database::pool`
```rust
pub fn pool(&self) -> &SqlitePool
```
커넥션 풀 참조 반환.

---

##### `Database::close`
```rust
pub async fn close(&self)
```
커넥션 풀 종료.

---

### Repository 구현 (공통 패턴)

모든 SQLite Repository 구현체는 동일한 패턴을 따릅니다:

```rust
pub struct SqliteXxxRepository {
    pool: SqlitePool,
}

impl SqliteXxxRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl XxxRepository for SqliteXxxRepository {
    // Trait 메서드 구현
}
```

**에러 매핑:**
- UNIQUE constraint → `RepositoryError::AlreadyExists`
- NOT FOUND (rows_affected = 0) → `RepositoryError::NotFound`
- 기타 SQLx 에러 → `RepositoryError::Database(msg)`

---

## 에러 타입

### Domain Errors

```rust
// domain/src/errors.rs

pub enum UserError {
    EmailAlreadyExists,
    NotFound,
    InvalidCredentials,
    AccountLocked,
    SameEmail,
}

pub enum ClientError {
    InvalidClientId,
    InvalidClientSecret,
    InvalidRedirectUri,
    ScopeNotAllowed,
    NotFound,
}

pub enum AuthCodeError {
    Expired,
    NotFound,
    AlreadyUsed,
    InvalidCodeVerifier,
    PkceRequired,
}

pub enum TokenError {
    Expired,
    Invalid,
    NotFound,
    Revoked,
    InsufficientScope { required: String, actual: String },
}
```

### Application Errors

```rust
// application/src/errors.rs

pub enum UseCaseError {
    Validation(String),
    Unauthorized,
    NotFound,
    AlreadyExists,
    Internal(String),
    Repository(RepositoryError),
}

pub enum RepositoryError {
    NotFound,
    AlreadyExists,
    Database(String),
    Internal(String),
}
```

---

**문서 버전**: 1.0
**최종 수정**: 2025-01-22
