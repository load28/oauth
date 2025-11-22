//! SQLite Token Repository Implementations

use application::errors::RepositoryError;
use application::ports::{AccessTokenRepository, RefreshTokenRepository};
use async_trait::async_trait;
use domain::entities::{AccessToken, RefreshToken};
use domain::value_objects::{ClientId, Scopes, UserId};
use sqlx::SqlitePool;

/// SQLite implementation of AccessTokenRepository
pub struct SqliteAccessTokenRepository {
    pool: SqlitePool,
}

impl SqliteAccessTokenRepository {
    /// Creates a new SqliteAccessTokenRepository
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccessTokenRepository for SqliteAccessTokenRepository {
    async fn save(&self, token: &AccessToken) -> Result<(), RepositoryError> {
        let token_str = token.token();
        let client_id = token.client_id().to_string();
        let user_id = token.user_id().to_string();
        let scope = token.scope().to_string();
        let expires_at = token.expires_at().to_rfc3339();
        let created_at = token.created_at().to_rfc3339();

        sqlx::query!(
            r#"
            INSERT INTO access_tokens (token, client_id, user_id, scope, expires_at, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            token_str,
            client_id,
            user_id,
            scope,
            expires_at,
            created_at
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

    async fn find_by_token(&self, token: &str) -> Result<Option<AccessToken>, RepositoryError> {
        let row = sqlx::query!(
            r#"
            SELECT token, client_id, user_id, scope, expires_at, created_at
            FROM access_tokens
            WHERE token = ?
            "#,
            token
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let access_token = row
            .map(|r| {
                let client_id = ClientId::parse(&r.client_id)
                    .map_err(|e| RepositoryError::Internal(format!("Invalid client ID: {}", e)))?;

                let user_id = UserId::parse(&r.user_id)
                    .map_err(|e| RepositoryError::Internal(format!("Invalid user ID: {}", e)))?;

                let scope = Scopes::parse(&r.scope)
                    .map_err(|e| RepositoryError::Internal(e.to_string()))?;

                let expires_at = chrono::DateTime::parse_from_rfc3339(&r.expires_at)
                    .map_err(|e| RepositoryError::Internal(e.to_string()))?
                    .with_timezone(&chrono::Utc);

                let created_at = chrono::DateTime::parse_from_rfc3339(&r.created_at)
                    .map_err(|e| RepositoryError::Internal(e.to_string()))?
                    .with_timezone(&chrono::Utc);

                Ok(AccessToken::from_existing(
                    r.token,
                    client_id,
                    user_id,
                    scope,
                    expires_at,
                    created_at,
                ))
            })
            .transpose()?;

        Ok(access_token)
    }

    async fn find_by_user_id(
        &self,
        user_id: UserId,
    ) -> Result<Vec<AccessToken>, RepositoryError> {
        let user_id_str = user_id.to_string();

        let rows = sqlx::query!(
            r#"
            SELECT token, client_id, user_id, scope, expires_at, created_at
            FROM access_tokens
            WHERE user_id = ?
            "#,
            user_id_str
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let tokens: Result<Vec<_>, _> = rows
            .into_iter()
            .map(|r| {
                let client_id = ClientId::parse(&r.client_id)
                    .map_err(|e| RepositoryError::Internal(format!("Invalid client ID: {}", e)))?;

                let user_id = UserId::parse(&r.user_id)
                    .map_err(|e| RepositoryError::Internal(format!("Invalid user ID: {}", e)))?;

                let scope = Scopes::parse(&r.scope)
                    .map_err(|e| RepositoryError::Internal(e.to_string()))?;

                let expires_at = chrono::DateTime::parse_from_rfc3339(&r.expires_at)
                    .map_err(|e| RepositoryError::Internal(e.to_string()))?
                    .with_timezone(&chrono::Utc);

                let created_at = chrono::DateTime::parse_from_rfc3339(&r.created_at)
                    .map_err(|e| RepositoryError::Internal(e.to_string()))?
                    .with_timezone(&chrono::Utc);

                Ok(AccessToken::from_existing(
                    r.token,
                    client_id,
                    user_id,
                    scope,
                    expires_at,
                    created_at,
                ))
            })
            .collect();

        tokens
    }

    async fn revoke(&self, token: &str) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM access_tokens
            WHERE token = ?
            "#,
            token
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    async fn revoke_all_for_user(&self, user_id: UserId) -> Result<usize, RepositoryError> {
        let user_id_str = user_id.to_string();

        let result = sqlx::query!(
            r#"
            DELETE FROM access_tokens
            WHERE user_id = ?
            "#,
            user_id_str
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result.rows_affected() as usize)
    }

    async fn delete_expired(&self) -> Result<usize, RepositoryError> {
        let now = chrono::Utc::now().to_rfc3339();

        let result = sqlx::query!(
            r#"
            DELETE FROM access_tokens
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

/// SQLite implementation of RefreshTokenRepository
pub struct SqliteRefreshTokenRepository {
    pool: SqlitePool,
}

impl SqliteRefreshTokenRepository {
    /// Creates a new SqliteRefreshTokenRepository
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RefreshTokenRepository for SqliteRefreshTokenRepository {
    async fn save(&self, token: &RefreshToken) -> Result<(), RepositoryError> {
        let token_str = token.token();
        let access_token_id = token.access_token_id();
        let client_id = token.client_id().to_string();
        let user_id = token.user_id().to_string();
        let scope = token.scope().to_string();
        let expires_at = token.expires_at().to_rfc3339();
        let created_at = token.created_at().to_rfc3339();
        let revoked = if token.is_revoked() { 1 } else { 0 };

        sqlx::query!(
            r#"
            INSERT INTO refresh_tokens (token, access_token_id, client_id, user_id, scope, expires_at, created_at, revoked)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            token_str,
            access_token_id,
            client_id,
            user_id,
            scope,
            expires_at,
            created_at,
            revoked
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

    async fn find_by_token(&self, token: &str) -> Result<Option<RefreshToken>, RepositoryError> {
        let row = sqlx::query!(
            r#"
            SELECT token, access_token_id, client_id, user_id, scope, expires_at, created_at, revoked
            FROM refresh_tokens
            WHERE token = ?
            "#,
            token
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let refresh_token = row
            .map(|r| {
                let client_id = ClientId::parse(&r.client_id)
                    .map_err(|e| RepositoryError::Internal(format!("Invalid client ID: {}", e)))?;

                let user_id = UserId::parse(&r.user_id)
                    .map_err(|e| RepositoryError::Internal(format!("Invalid user ID: {}", e)))?;

                let scope = Scopes::parse(&r.scope)
                    .map_err(|e| RepositoryError::Internal(e.to_string()))?;

                let expires_at = chrono::DateTime::parse_from_rfc3339(&r.expires_at)
                    .map_err(|e| RepositoryError::Internal(e.to_string()))?
                    .with_timezone(&chrono::Utc);

                let created_at = chrono::DateTime::parse_from_rfc3339(&r.created_at)
                    .map_err(|e| RepositoryError::Internal(e.to_string()))?
                    .with_timezone(&chrono::Utc);

                let revoked = r.revoked != 0;

                Ok(RefreshToken::from_existing(
                    r.token,
                    r.access_token_id,
                    client_id,
                    user_id,
                    scope,
                    expires_at,
                    created_at,
                    revoked,
                ))
            })
            .transpose()?;

        Ok(refresh_token)
    }

    async fn find_by_user_id(
        &self,
        user_id: UserId,
    ) -> Result<Vec<RefreshToken>, RepositoryError> {
        let user_id_str = user_id.to_string();

        let rows = sqlx::query!(
            r#"
            SELECT token, access_token_id, client_id, user_id, scope, expires_at, created_at, revoked
            FROM refresh_tokens
            WHERE user_id = ?
            "#,
            user_id_str
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let tokens: Result<Vec<_>, _> = rows
            .into_iter()
            .map(|r| {
                let client_id = ClientId::parse(&r.client_id)
                    .map_err(|e| RepositoryError::Internal(format!("Invalid client ID: {}", e)))?;

                let user_id = UserId::parse(&r.user_id)
                    .map_err(|e| RepositoryError::Internal(format!("Invalid user ID: {}", e)))?;

                let scope = Scopes::parse(&r.scope)
                    .map_err(|e| RepositoryError::Internal(e.to_string()))?;

                let expires_at = chrono::DateTime::parse_from_rfc3339(&r.expires_at)
                    .map_err(|e| RepositoryError::Internal(e.to_string()))?
                    .with_timezone(&chrono::Utc);

                let created_at = chrono::DateTime::parse_from_rfc3339(&r.created_at)
                    .map_err(|e| RepositoryError::Internal(e.to_string()))?
                    .with_timezone(&chrono::Utc);

                let revoked = r.revoked != 0;

                Ok(RefreshToken::from_existing(
                    r.token,
                    r.access_token_id,
                    client_id,
                    user_id,
                    scope,
                    expires_at,
                    created_at,
                    revoked,
                ))
            })
            .collect();

        tokens
    }

    async fn find_by_access_token_id(
        &self,
        access_token_id: &str,
    ) -> Result<Option<RefreshToken>, RepositoryError> {
        let row = sqlx::query!(
            r#"
            SELECT token, access_token_id, client_id, user_id, scope, expires_at, created_at, revoked
            FROM refresh_tokens
            WHERE access_token_id = ?
            "#,
            access_token_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let refresh_token = row
            .map(|r| {
                let client_id = ClientId::parse(&r.client_id)
                    .map_err(|e| RepositoryError::Internal(format!("Invalid client ID: {}", e)))?;

                let user_id = UserId::parse(&r.user_id)
                    .map_err(|e| RepositoryError::Internal(format!("Invalid user ID: {}", e)))?;

                let scope = Scopes::parse(&r.scope)
                    .map_err(|e| RepositoryError::Internal(e.to_string()))?;

                let expires_at = chrono::DateTime::parse_from_rfc3339(&r.expires_at)
                    .map_err(|e| RepositoryError::Internal(e.to_string()))?
                    .with_timezone(&chrono::Utc);

                let created_at = chrono::DateTime::parse_from_rfc3339(&r.created_at)
                    .map_err(|e| RepositoryError::Internal(e.to_string()))?
                    .with_timezone(&chrono::Utc);

                let revoked = r.revoked != 0;

                Ok(RefreshToken::from_existing(
                    r.token,
                    r.access_token_id,
                    client_id,
                    user_id,
                    scope,
                    expires_at,
                    created_at,
                    revoked,
                ))
            })
            .transpose()?;

        Ok(refresh_token)
    }

    async fn revoke(&self, token: &str) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"
            UPDATE refresh_tokens
            SET revoked = 1
            WHERE token = ?
            "#,
            token
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    async fn revoke_all_for_user(&self, user_id: UserId) -> Result<usize, RepositoryError> {
        let user_id_str = user_id.to_string();

        let result = sqlx::query!(
            r#"
            UPDATE refresh_tokens
            SET revoked = 1
            WHERE user_id = ?
            "#,
            user_id_str
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result.rows_affected() as usize)
    }

    async fn revoke_all_for_client(&self, client_id: ClientId) -> Result<usize, RepositoryError> {
        let client_id_str = client_id.to_string();

        let result = sqlx::query!(
            r#"
            UPDATE refresh_tokens
            SET revoked = 1
            WHERE client_id = ?
            "#,
            client_id_str
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result.rows_affected() as usize)
    }

    async fn delete_expired_and_revoked(&self) -> Result<usize, RepositoryError> {
        let now = chrono::Utc::now().to_rfc3339();

        let result = sqlx::query!(
            r#"
            DELETE FROM refresh_tokens
            WHERE expires_at < ? OR revoked = 1
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
    use domain::value_objects::{Email, PlainPassword};

    async fn setup_test_db() -> (
        Database,
        SqliteAccessTokenRepository,
        SqliteRefreshTokenRepository,
    ) {
        let db = Database::new("sqlite::memory:").await.unwrap();
        db.migrate().await.unwrap();
        let access_repo = SqliteAccessTokenRepository::new(db.pool().clone());
        let refresh_repo = SqliteRefreshTokenRepository::new(db.pool().clone());
        (db, access_repo, refresh_repo)
    }

    async fn create_test_user(db: &Database) -> UserId {
        let user_repo = SqliteUserRepository::new(db.pool().clone());
        let email = Email::new("test@example.com").unwrap();
        let password = PlainPassword::new("password123").unwrap();
        let password_hash = password.hash().unwrap();
        let user = User::new(UserId::new(), email, password_hash);
        user_repo.save(&user).await.unwrap();
        user.id()
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
    async fn should_save_and_find_access_token() {
        let (db, access_repo, _refresh_repo) = setup_test_db().await;

        let user_id = create_test_user(&db).await;
        let client_id = create_test_client(&db).await;

        let token = AccessToken::new(
            "test_access_token".to_string(),
            client_id,
            user_id,
            Scopes::parse("openid profile").unwrap(),
        );

        access_repo.save(&token).await.unwrap();

        let found = access_repo.find_by_token(token.token()).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().token(), token.token());
    }

    #[tokio::test]
    async fn should_save_and_find_refresh_token() {
        let (db, _access_repo, refresh_repo) = setup_test_db().await;

        let user_id = create_test_user(&db).await;
        let client_id = create_test_client(&db).await;

        let token = RefreshToken::generate(
            "access_token_id".to_string(),
            client_id,
            user_id,
            Scopes::parse("openid profile").unwrap(),
        );

        refresh_repo.save(&token).await.unwrap();

        let found = refresh_repo.find_by_token(token.token()).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().token(), token.token());
    }

    #[tokio::test]
    async fn should_revoke_refresh_token() {
        let (db, _access_repo, refresh_repo) = setup_test_db().await;

        let user_id = create_test_user(&db).await;
        let client_id = create_test_client(&db).await;

        let token = RefreshToken::generate(
            "access_token_id".to_string(),
            client_id,
            user_id,
            Scopes::parse("openid").unwrap(),
        );

        refresh_repo.save(&token).await.unwrap();
        refresh_repo.revoke(token.token()).await.unwrap();

        let found = refresh_repo.find_by_token(token.token()).await.unwrap();
        assert!(found.unwrap().is_revoked());
    }

    #[tokio::test]
    async fn should_delete_expired_tokens() {
        let (db, access_repo, refresh_repo) = setup_test_db().await;

        let user_id = create_test_user(&db).await;
        let client_id = create_test_client(&db).await;

        // Create an expired access token
        let token_str = "expired_access_token";
        let client_id_str = client_id.to_string();
        let user_id_str = user_id.to_string();
        let scope = "openid";
        let expires_at = (chrono::Utc::now() - chrono::Duration::hours(1)).to_rfc3339();
        let created_at = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO access_tokens (token, client_id, user_id, scope, expires_at, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(token_str)
        .bind(&client_id_str)
        .bind(&user_id_str)
        .bind(scope)
        .bind(&expires_at)
        .bind(&created_at)
        .execute(db.pool())
        .await
        .unwrap();

        // Create an expired refresh token
        let refresh_token_str = "expired_refresh_token";
        let access_token_id = "some_access_token";
        let revoked: i64 = 0;

        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (token, access_token_id, client_id, user_id, scope, expires_at, created_at, revoked)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(refresh_token_str)
        .bind(access_token_id)
        .bind(&client_id_str)
        .bind(&user_id_str)
        .bind(scope)
        .bind(&expires_at)
        .bind(&created_at)
        .bind(revoked)
        .execute(db.pool())
        .await
        .unwrap();

        // Delete expired tokens
        let deleted_access = access_repo.delete_expired().await.unwrap();
        assert_eq!(deleted_access, 1);

        let deleted_refresh = refresh_repo.delete_expired_and_revoked().await.unwrap();
        assert_eq!(deleted_refresh, 1);
    }
}
