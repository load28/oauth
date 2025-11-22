//! SQLite Client Repository Implementation

use application::errors::RepositoryError;
use application::ports::{ClientData, ClientRepository};
use async_trait::async_trait;
use domain::entities::{ClientSecret, Confidential, OAuthClient, Public};
use domain::value_objects::{ClientId, Scopes};
use sqlx::SqlitePool;

/// SQLite implementation of ClientRepository
pub struct SqliteClientRepository {
    pool: SqlitePool,
}

impl SqliteClientRepository {
    /// Creates a new SqliteClientRepository
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ClientRepository for SqliteClientRepository {
    async fn save_public(&self, client: &OAuthClient<Public>) -> Result<(), RepositoryError> {
        let id = client.id().to_string();
        let name = client.name();
        let redirect_uris =
            serde_json::to_string(&client.redirect_uris()).map_err(|e| {
                RepositoryError::Internal(format!("Failed to serialize redirect URIs: {}", e))
            })?;
        let allowed_scopes = client.allowed_scopes().to_string();
        let created_at = client.created_at().to_rfc3339();

        sqlx::query!(
            r#"
            INSERT INTO oauth_clients (id, client_type, name, redirect_uris, allowed_scopes, client_secret, created_at)
            VALUES (?, 'public', ?, ?, ?, NULL, ?)
            "#,
            id,
            name,
            redirect_uris,
            allowed_scopes,
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

    async fn save_confidential(
        &self,
        client: &OAuthClient<Confidential>,
    ) -> Result<(), RepositoryError> {
        let id = client.id().to_string();
        let name = client.name();
        let redirect_uris =
            serde_json::to_string(&client.redirect_uris()).map_err(|e| {
                RepositoryError::Internal(format!("Failed to serialize redirect URIs: {}", e))
            })?;
        let allowed_scopes = client.allowed_scopes().to_string();
        let client_secret = client.secret().map(|s| s.as_str());
        let created_at = client.created_at().to_rfc3339();

        sqlx::query!(
            r#"
            INSERT INTO oauth_clients (id, client_type, name, redirect_uris, allowed_scopes, client_secret, created_at)
            VALUES (?, 'confidential', ?, ?, ?, ?, ?)
            "#,
            id,
            name,
            redirect_uris,
            allowed_scopes,
            client_secret,
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

    async fn find_by_id(&self, id: ClientId) -> Result<Option<ClientData>, RepositoryError> {
        let id_str = id.to_string();

        let row = sqlx::query!(
            r#"
            SELECT id, client_type, name, redirect_uris, allowed_scopes, client_secret, created_at
            FROM oauth_clients
            WHERE id = ?
            "#,
            id_str
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let client = row
            .map(|r| {
                let id = ClientId::parse(&r.id)
                    .map_err(|e| RepositoryError::Internal(format!("Invalid client ID: {}", e)))?;

                let redirect_uris: Vec<String> = serde_json::from_str(&r.redirect_uris)
                    .map_err(|e| {
                        RepositoryError::Internal(format!(
                            "Failed to deserialize redirect URIs: {}",
                            e
                        ))
                    })?;

                let allowed_scopes = Scopes::parse(&r.allowed_scopes)
                    .map_err(|e| RepositoryError::Internal(e.to_string()))?;

                let secret = r.client_secret.map(ClientSecret::new);

                let created_at = chrono::DateTime::parse_from_rfc3339(&r.created_at)
                    .map_err(|e| RepositoryError::Internal(e.to_string()))?
                    .with_timezone(&chrono::Utc);

                Ok(ClientData {
                    id,
                    name: r.name,
                    redirect_uris,
                    allowed_scopes,
                    secret,
                    created_at,
                })
            })
            .transpose()?;

        Ok(client)
    }

    async fn delete(&self, id: ClientId) -> Result<(), RepositoryError> {
        let id_str = id.to_string();

        let result = sqlx::query!(
            r#"
            DELETE FROM oauth_clients
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

    async fn setup_test_db() -> (Database, SqliteClientRepository) {
        let db = Database::new("sqlite::memory:").await.unwrap();
        db.migrate().await.unwrap();
        let repo = SqliteClientRepository::new(db.pool().clone());
        (db, repo)
    }

    #[tokio::test]
    async fn should_save_and_find_public_client() {
        let (_db, repo) = setup_test_db().await;

        let client = OAuthClient::<Public>::new(
            ClientId::new(),
            "Test Public Client",
            vec!["http://localhost:3000/callback".to_string()],
            Scopes::parse("openid profile").unwrap(),
        );

        // Save client
        repo.save_public(&client).await.unwrap();

        // Find client
        let found = repo.find_by_id(client.id()).await.unwrap();
        assert!(found.is_some());

        let found_data = found.unwrap();
        assert!(found_data.is_public());
        assert_eq!(found_data.name, "Test Public Client");
    }

    #[tokio::test]
    async fn should_save_and_find_confidential_client() {
        let (_db, repo) = setup_test_db().await;

        let secret = ClientSecret::generate();
        let client = OAuthClient::<Confidential>::new(
            ClientId::new(),
            "Test Confidential Client",
            secret.clone(),
            vec!["http://localhost:3000/callback".to_string()],
            Scopes::parse("openid profile email").unwrap(),
        );

        // Save client
        repo.save_confidential(&client).await.unwrap();

        // Find client
        let found = repo.find_by_id(client.id()).await.unwrap();
        assert!(found.is_some());

        let found_data = found.unwrap();
        assert!(found_data.is_confidential());
        assert_eq!(found_data.name, "Test Confidential Client");
        assert!(found_data.secret.is_some());
    }

    #[tokio::test]
    async fn should_delete_client() {
        let (_db, repo) = setup_test_db().await;

        let client = OAuthClient::<Public>::new(
            ClientId::new(),
            "Test Client",
            vec!["http://localhost:3000/callback".to_string()],
            Scopes::parse("openid").unwrap(),
        );

        repo.save_public(&client).await.unwrap();
        repo.delete(client.id()).await.unwrap();

        let found = repo.find_by_id(client.id()).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn should_return_error_for_duplicate_client() {
        let (_db, repo) = setup_test_db().await;

        let client_id = ClientId::new();
        let client1 = OAuthClient::<Public>::new(
            client_id,
            "Test Client 1",
            vec!["http://localhost:3000/callback".to_string()],
            Scopes::parse("openid").unwrap(),
        );

        repo.save_public(&client1).await.unwrap();

        let client2 = OAuthClient::<Public>::new(
            client_id,
            "Test Client 2",
            vec!["http://localhost:4000/callback".to_string()],
            Scopes::parse("profile").unwrap(),
        );

        let result = repo.save_public(&client2).await;
        assert!(matches!(result, Err(RepositoryError::AlreadyExists)));
    }
}
