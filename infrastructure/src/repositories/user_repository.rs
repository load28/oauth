//! SQLite User Repository Implementation

use application::errors::RepositoryError;
use application::ports::UserRepository;
use async_trait::async_trait;
use domain::entities::User;
use domain::value_objects::{Email, PasswordHash, UserId};
use sqlx::SqlitePool;

/// SQLite implementation of UserRepository
pub struct SqliteUserRepository {
    pool: SqlitePool,
}

impl SqliteUserRepository {
    /// Creates a new SqliteUserRepository
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn save(&self, user: &User) -> Result<(), RepositoryError> {
        let id = user.id().to_string();
        let email = user.email().as_str();
        let password_hash = user.password_hash().as_str();
        let created_at = user.created_at().to_rfc3339();
        let updated_at = user.updated_at().to_rfc3339();

        sqlx::query!(
            r#"
            INSERT INTO users (id, email, password_hash, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                email = excluded.email,
                password_hash = excluded.password_hash,
                updated_at = excluded.updated_at
            "#,
            id,
            email,
            password_hash,
            created_at,
            updated_at
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepositoryError> {
        let id_str = id.to_string();

        let row = sqlx::query!(
            r#"
            SELECT id, email, password_hash, created_at, updated_at
            FROM users
            WHERE id = ?
            "#,
            id_str
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let user = row
            .map(|r| {
                let user_id = UserId::parse(&r.id).map_err(|e| {
                    RepositoryError::Internal(format!("Invalid user ID: {}", e))
                })?;

                let email =
                    Email::new(&r.email).map_err(|e| RepositoryError::Internal(e.to_string()))?;

                let password_hash = PasswordHash::new(r.password_hash)
                    .map_err(|e| RepositoryError::Internal(e.to_string()))?;

                let created_at = chrono::DateTime::parse_from_rfc3339(&r.created_at)
                    .map_err(|e| RepositoryError::Internal(e.to_string()))?
                    .with_timezone(&chrono::Utc);

                let updated_at = chrono::DateTime::parse_from_rfc3339(&r.updated_at)
                    .map_err(|e| RepositoryError::Internal(e.to_string()))?
                    .with_timezone(&chrono::Utc);

                Ok(User::from_existing(
                    user_id,
                    email,
                    password_hash,
                    created_at,
                    updated_at,
                ))
            })
            .transpose()?;

        Ok(user)
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepositoryError> {
        let email_str = email.as_str();

        let row = sqlx::query!(
            r#"
            SELECT id, email, password_hash, created_at, updated_at
            FROM users
            WHERE email = ?
            "#,
            email_str
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let user = row
            .map(|r| {
                let user_id = UserId::parse(&r.id).map_err(|e| {
                    RepositoryError::Internal(format!("Invalid user ID: {}", e))
                })?;

                let email =
                    Email::new(&r.email).map_err(|e| RepositoryError::Internal(e.to_string()))?;

                let password_hash = PasswordHash::new(r.password_hash)
                    .map_err(|e| RepositoryError::Internal(e.to_string()))?;

                let created_at = chrono::DateTime::parse_from_rfc3339(&r.created_at)
                    .map_err(|e| RepositoryError::Internal(e.to_string()))?
                    .with_timezone(&chrono::Utc);

                let updated_at = chrono::DateTime::parse_from_rfc3339(&r.updated_at)
                    .map_err(|e| RepositoryError::Internal(e.to_string()))?
                    .with_timezone(&chrono::Utc);

                Ok(User::from_existing(
                    user_id,
                    email,
                    password_hash,
                    created_at,
                    updated_at,
                ))
            })
            .transpose()?;

        Ok(user)
    }

    async fn exists_by_email(&self, email: &Email) -> Result<bool, RepositoryError> {
        let email_str = email.as_str();

        let count: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as count
            FROM users
            WHERE email = ?
            "#,
            email_str
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(count > 0)
    }

    async fn delete(&self, id: UserId) -> Result<(), RepositoryError> {
        let id_str = id.to_string();

        let result = sqlx::query!(
            r#"
            DELETE FROM users
            WHERE id = ?
            "#,
            id_str
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Database;
    use domain::value_objects::PlainPassword;

    async fn setup_test_db() -> (Database, SqliteUserRepository) {
        let db = Database::new("sqlite::memory:").await.unwrap();
        db.migrate().await.unwrap();
        let repo = SqliteUserRepository::new(db.pool().clone());
        (db, repo)
    }

    #[tokio::test]
    async fn should_save_and_find_user() {
        let (_db, repo) = setup_test_db().await;

        let email = Email::new("test@example.com").unwrap();
        let password = PlainPassword::new("password123").unwrap();
        let password_hash = password.hash().unwrap();
        let user = User::new(UserId::new(), email.clone(), password_hash);

        // Save user
        repo.save(&user).await.unwrap();

        // Find by ID
        let found = repo.find_by_id(user.id()).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().email(), &email);

        // Find by email
        let found = repo.find_by_email(&email).await.unwrap();
        assert!(found.is_some());
    }

    #[tokio::test]
    async fn should_check_email_exists() {
        let (_db, repo) = setup_test_db().await;

        let email = Email::new("test@example.com").unwrap();
        assert!(!repo.exists_by_email(&email).await.unwrap());

        let password = PlainPassword::new("password123").unwrap();
        let password_hash = password.hash().unwrap();
        let user = User::new(UserId::new(), email.clone(), password_hash);
        repo.save(&user).await.unwrap();

        assert!(repo.exists_by_email(&email).await.unwrap());
    }

    #[tokio::test]
    async fn should_delete_user() {
        let (_db, repo) = setup_test_db().await;

        let email = Email::new("test@example.com").unwrap();
        let password = PlainPassword::new("password123").unwrap();
        let password_hash = password.hash().unwrap();
        let user = User::new(UserId::new(), email, password_hash);

        repo.save(&user).await.unwrap();
        repo.delete(user.id()).await.unwrap();

        let found = repo.find_by_id(user.id()).await.unwrap();
        assert!(found.is_none());
    }
}
