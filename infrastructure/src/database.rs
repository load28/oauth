//! Database Connection Pool
//!
//! Manages SQLite database connections using SQLx.

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::ConnectOptions;
use std::str::FromStr;
use std::time::Duration;

/// Database connection pool wrapper
#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Creates a new database connection pool
    ///
    /// # Arguments
    /// * `database_url` - SQLite database URL (e.g., "sqlite:oauth.db")
    ///
    /// # Errors
    /// Returns error if connection fails
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let options = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true)
            .foreign_keys(true)
            .disable_statement_logging();

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect_with(options)
            .await?;

        Ok(Self { pool })
    }

    /// Runs database migrations
    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    /// Returns a reference to the connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Closes the database connection pool
    pub async fn close(&self) {
        self.pool.close().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_create_in_memory_database() {
        let db = Database::new("sqlite::memory:").await;
        assert!(db.is_ok());
    }

    #[tokio::test]
    async fn should_run_migrations() {
        let db = Database::new("sqlite::memory:").await.unwrap();
        let result = db.migrate().await;
        assert!(result.is_ok());
    }
}
