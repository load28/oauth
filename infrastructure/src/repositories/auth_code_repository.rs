//! SQLite Authorization Code Repository Implementation

use application::errors::RepositoryError;
use application::ports::AuthCodeRepository;
use async_trait::async_trait;
use domain::entities::AuthorizationCode;
use domain::value_objects::{ClientId, CodeChallenge, S256, Scopes, UserId};
use sqlx::SqlitePool;

/// SQLite implementation of AuthCodeRepository
pub struct SqliteAuthCodeRepository {
    pool: SqlitePool,
}

impl SqliteAuthCodeRepository {
    /// Creates a new SqliteAuthCodeRepository
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthCodeRepository for SqliteAuthCodeRepository {
    async fn save(&self, auth_code: &AuthorizationCode) -> Result<(), RepositoryError> {
        let code = auth_code.code();
        let client_id = auth_code.client_id().to_string();
        let user_id = auth_code.user_id().to_string();
        let redirect_uri = auth_code.redirect_uri();
        let scope = auth_code.scope().to_string();
        let code_challenge = auth_code.code_challenge().map(|c| c.as_str());
        let expires_at = auth_code.expires_at().to_rfc3339();
        let created_at = auth_code.created_at().to_rfc3339();
        let used = if auth_code.is_used() { 1 } else { 0 };

        sqlx::query!(
            r#"
            INSERT INTO authorization_codes (code, client_id, user_id, redirect_uri, scope, code_challenge, expires_at, created_at, used)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            code,
            client_id,
            user_id,
            redirect_uri,
            scope,
            code_challenge,
            expires_at,
            created_at,
            used
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                RepositoryError::AlreadyExists
            } else {
                RepositoryError::Database(e.to_string())
            }
        })?;

        Ok(())
    }

    async fn find_by_code(&self, code: &str) -> Result<Option<AuthorizationCode>, RepositoryError> {
        let row = sqlx::query!(
            r#"
            SELECT code, client_id, user_id, redirect_uri, scope, code_challenge, expires_at, created_at, used
            FROM authorization_codes
            WHERE code = ?
            "#,
            code
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let auth_code = row
            .map(|r| {
                let client_id = ClientId::parse(&r.client_id)
                    .map_err(|e| RepositoryError::Internal(format!("Invalid client ID: {}", e)))?;

                let user_id = UserId::parse(&r.user_id)
                    .map_err(|e| RepositoryError::Internal(format!("Invalid user ID: {}", e)))?;

                let scope = Scopes::parse(&r.scope)
                    .map_err(|e| RepositoryError::Internal(e.to_string()))?;

                let code_challenge = r.code_challenge.map(CodeChallenge::<S256>::new_s256);

                let expires_at = chrono::DateTime::parse_from_rfc3339(&r.expires_at)
                    .map_err(|e| RepositoryError::Internal(e.to_string()))?
                    .with_timezone(&chrono::Utc);

                let created_at = chrono::DateTime::parse_from_rfc3339(&r.created_at)
                    .map_err(|e| RepositoryError::Internal(e.to_string()))?
                    .with_timezone(&chrono::Utc);

                let used = r.used != 0;

                Ok(AuthorizationCode::from_existing(
                    r.code,
                    client_id,
                    user_id,
                    r.redirect_uri,
                    scope,
                    code_challenge,
                    expires_at,
                    created_at,
                    used,
                ))
            })
            .transpose()?;

        Ok(auth_code)
    }

    async fn mark_as_used(&self, code: &str) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"
            UPDATE authorization_codes
            SET used = 1
            WHERE code = ?
            "#,
            code
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    async fn delete(&self, code: &str) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"
            DELETE FROM authorization_codes
            WHERE code = ?
            "#,
            code
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        // Deletion is idempotent - don't error if code doesn't exist
        Ok(())
    }

    async fn delete_expired(&self) -> Result<usize, RepositoryError> {
        let now = chrono::Utc::now().to_rfc3339();

        let result = sqlx::query!(
            r#"
            DELETE FROM authorization_codes
            WHERE expires_at < ?
            "#,
            now
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result.rows_affected() as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use application::ports::{ClientRepository, UserRepository};
    use crate::repositories::{SqliteClientRepository, SqliteUserRepository};
    use crate::Database;
    use domain::entities::{OAuthClient, Public, User};
    use domain::value_objects::{CodeVerifier, Email, PlainPassword};

    async fn setup_test_db() -> (Database, SqliteAuthCodeRepository) {
        let db = Database::new("sqlite::memory:").await.unwrap();
        db.migrate().await.unwrap();
        let repo = SqliteAuthCodeRepository::new(db.pool().clone());
        (db, repo)
    }

    async fn create_test_user(db: &Database) -> (UserId, Email) {
        let user_repo = SqliteUserRepository::new(db.pool().clone());
        let email = Email::new("test@example.com").unwrap();
        let password = PlainPassword::new("password123").unwrap();
        let password_hash = password.hash().unwrap();
        let user = User::new(UserId::new(), email.clone(), password_hash);
        user_repo.save(&user).await.unwrap();
        (user.id(), email)
    }

    async fn create_test_client(db: &Database) -> ClientId {
        let client_repo = SqliteClientRepository::new(db.pool().clone());
        let client = OAuthClient::<Public>::new(
            ClientId::new(),
            "Test Client",
            vec!["http://localhost:3000/callback".to_string()],
            Scopes::parse("openid profile email").unwrap(),
        );
        client_repo.save_public(&client).await.unwrap();
        client.id()
    }

    #[tokio::test]
    async fn should_save_and_find_auth_code() {
        let (db, repo) = setup_test_db().await;

        // Create test user and client first
        let (user_id, _) = create_test_user(&db).await;
        let client_id = create_test_client(&db).await;

        let verifier = CodeVerifier::generate();
        let challenge = verifier.create_s256_challenge();

        let auth_code = AuthorizationCode::generate(
            client_id,
            user_id,
            "http://localhost:3000/callback".to_string(),
            Scopes::parse("openid profile").unwrap(),
            Some(challenge),
        );

        // Save auth code
        repo.save(&auth_code).await.unwrap();

        // Find auth code
        let found = repo.find_by_code(auth_code.code()).await.unwrap();
        assert!(found.is_some());

        let found_code = found.unwrap();
        assert_eq!(found_code.code(), auth_code.code());
        assert_eq!(found_code.client_id(), auth_code.client_id());
        assert!(!found_code.is_used());
    }

    #[tokio::test]
    async fn should_mark_as_used() {
        let (db, repo) = setup_test_db().await;

        // Create test user and client first
        let (user_id, _) = create_test_user(&db).await;
        let client_id = create_test_client(&db).await;

        let auth_code = AuthorizationCode::generate(
            client_id,
            user_id,
            "http://localhost:3000/callback".to_string(),
            Scopes::parse("openid").unwrap(),
            None,
        );

        repo.save(&auth_code).await.unwrap();
        repo.mark_as_used(auth_code.code()).await.unwrap();

        let found = repo.find_by_code(auth_code.code()).await.unwrap();
        assert!(found.unwrap().is_used());
    }

    #[tokio::test]
    async fn should_delete_auth_code() {
        let (db, repo) = setup_test_db().await;

        // Create test user and client first
        let (user_id, _) = create_test_user(&db).await;
        let client_id = create_test_client(&db).await;

        let auth_code = AuthorizationCode::generate(
            client_id,
            user_id,
            "http://localhost:3000/callback".to_string(),
            Scopes::parse("openid").unwrap(),
            None,
        );

        repo.save(&auth_code).await.unwrap();
        repo.delete(auth_code.code()).await.unwrap();

        let found = repo.find_by_code(auth_code.code()).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn should_delete_expired_codes() {
        let (db, repo) = setup_test_db().await;

        // Create test user and client first
        let (user_id, _) = create_test_user(&db).await;
        let client_id = create_test_client(&db).await;

        // Create an expired auth code by manipulating the database directly
        let code = "expired_code";
        let client_id_str = client_id.to_string();
        let user_id_str = user_id.to_string();
        let redirect_uri = "http://localhost:3000/callback";
        let scope = "openid";
        let expires_at = (chrono::Utc::now() - chrono::Duration::hours(1)).to_rfc3339();
        let created_at = chrono::Utc::now().to_rfc3339();
        let used: i64 = 0;

        sqlx::query(
            r#"
            INSERT INTO authorization_codes (code, client_id, user_id, redirect_uri, scope, code_challenge, expires_at, created_at, used)
            VALUES (?, ?, ?, ?, ?, NULL, ?, ?, ?)
            "#,
        )
        .bind(code)
        .bind(client_id_str)
        .bind(user_id_str)
        .bind(redirect_uri)
        .bind(scope)
        .bind(expires_at)
        .bind(created_at)
        .bind(used)
        .execute(db.pool())
        .await
        .unwrap();

        // Delete expired codes
        let deleted = repo.delete_expired().await.unwrap();
        assert_eq!(deleted, 1);

        // Verify it's gone
        let found = repo.find_by_code(code).await.unwrap();
        assert!(found.is_none());
    }
}
