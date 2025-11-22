//! Configuration Management
//!
//! Type-safe configuration loading from environment variables following
//! 12-Factor App principles.

use std::env;
use std::fmt;

/// Configuration error types
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingVariable(String),

    #[error("Invalid value for {variable}: {reason}")]
    InvalidValue { variable: String, reason: String },

    #[error("Failed to parse {variable}: {source}")]
    ParseError {
        variable: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Environment file error: {0}")]
    EnvFileError(String),
}

/// Main application configuration
///
/// All configuration is loaded from environment variables following
/// the 12-Factor App methodology for strict separation of config from code.
#[derive(Debug, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub server: ServerConfig,
    pub cors: CorsConfig,
    pub token: TokenConfig,
}

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Database connection URL (e.g., "sqlite:oauth.db")
    pub url: String,
}

/// JWT (JSON Web Token) configuration
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// Secret key for JWT signing and verification
    /// SECURITY: Must be at least 32 characters in production
    secret: String,
}

impl JwtConfig {
    /// Get JWT secret as bytes for signing
    pub fn secret_bytes(&self) -> &[u8] {
        self.secret.as_bytes()
    }

    /// Get JWT secret as string
    pub fn secret(&self) -> &str {
        &self.secret
    }
}

// Prevent secret from being printed in debug output
impl fmt::Display for JwtConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JwtConfig {{ secret: [REDACTED] }}")
    }
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Server bind address (e.g., "127.0.0.1")
    pub host: String,

    /// Server port number
    pub port: u16,
}

impl ServerConfig {
    /// Get full server address (host:port)
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// CORS (Cross-Origin Resource Sharing) configuration
#[derive(Debug, Clone)]
pub struct CorsConfig {
    /// List of allowed origins for CORS
    pub allowed_origins: Vec<String>,
}

/// Token expiration configuration
#[derive(Debug, Clone)]
pub struct TokenConfig {
    /// Access token expiration time in seconds
    pub access_token_expiration: u64,

    /// Refresh token expiration time in seconds
    pub refresh_token_expiration: u64,
}

impl Config {
    /// Load configuration from environment variables
    ///
    /// Reads from .env file in development and falls back to system environment variables.
    /// Follows 12-Factor App principle: system env vars take precedence over .env file.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if:
    /// - Required environment variables are missing
    /// - Values fail validation (e.g., JWT secret too short)
    /// - Values cannot be parsed (e.g., invalid port number)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use infrastructure::Config;
    ///
    /// let config = Config::from_env().expect("Failed to load configuration");
    /// println!("Server will run on {}", config.server.address());
    /// ```
    pub fn from_env() -> Result<Self, ConfigError> {
        // Load .env file if it exists (development/testing)
        // This never overwrites existing environment variables
        dotenv::dotenv().ok();

        let database = Self::load_database_config()?;
        let jwt = Self::load_jwt_config()?;
        let server = Self::load_server_config()?;
        let cors = Self::load_cors_config()?;
        let token = Self::load_token_config()?;

        Ok(Self {
            database,
            jwt,
            server,
            cors,
            token,
        })
    }

    fn load_database_config() -> Result<DatabaseConfig, ConfigError> {
        let url = env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingVariable("DATABASE_URL".to_string()))?;

        if url.is_empty() {
            return Err(ConfigError::InvalidValue {
                variable: "DATABASE_URL".to_string(),
                reason: "cannot be empty".to_string(),
            });
        }

        Ok(DatabaseConfig { url })
    }

    fn load_jwt_config() -> Result<JwtConfig, ConfigError> {
        let secret = env::var("JWT_SECRET")
            .map_err(|_| ConfigError::MissingVariable("JWT_SECRET".to_string()))?;

        // SECURITY: Enforce minimum secret length (RFC 9700 OAuth 2.0 Security Best Practices)
        const MIN_SECRET_LENGTH: usize = 32;
        if secret.len() < MIN_SECRET_LENGTH {
            return Err(ConfigError::InvalidValue {
                variable: "JWT_SECRET".to_string(),
                reason: format!(
                    "must be at least {} characters (current: {})",
                    MIN_SECRET_LENGTH,
                    secret.len()
                ),
            });
        }

        Ok(JwtConfig { secret })
    }

    fn load_server_config() -> Result<ServerConfig, ConfigError> {
        let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

        let port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(|e| ConfigError::ParseError {
                variable: "SERVER_PORT".to_string(),
                source: Box::new(e),
            })?;

        Ok(ServerConfig { host, port })
    }

    fn load_cors_config() -> Result<CorsConfig, ConfigError> {
        let origins_str = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());

        let allowed_origins = origins_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(CorsConfig { allowed_origins })
    }

    fn load_token_config() -> Result<TokenConfig, ConfigError> {
        let access_token_expiration = env::var("ACCESS_TOKEN_EXPIRATION")
            .unwrap_or_else(|_| "3600".to_string())
            .parse::<u64>()
            .map_err(|e| ConfigError::ParseError {
                variable: "ACCESS_TOKEN_EXPIRATION".to_string(),
                source: Box::new(e),
            })?;

        let refresh_token_expiration = env::var("REFRESH_TOKEN_EXPIRATION")
            .unwrap_or_else(|_| "2592000".to_string())
            .parse::<u64>()
            .map_err(|e| ConfigError::ParseError {
                variable: "REFRESH_TOKEN_EXPIRATION".to_string(),
                source: Box::new(e),
            })?;

        Ok(TokenConfig {
            access_token_expiration,
            refresh_token_expiration,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Ensure tests run serially to avoid environment variable conflicts
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn setup_test_env() {
        // Clear all config-related environment variables
        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("JWT_SECRET");
        std::env::remove_var("SERVER_HOST");
        std::env::remove_var("SERVER_PORT");
        std::env::remove_var("CORS_ALLOWED_ORIGINS");
        std::env::remove_var("ACCESS_TOKEN_EXPIRATION");
        std::env::remove_var("REFRESH_TOKEN_EXPIRATION");
        std::env::remove_var("RUST_LOG");
    }

    #[test]
    fn should_reject_short_jwt_secret() {
        let _lock = TEST_MUTEX.lock().unwrap();
        setup_test_env();

        std::env::set_var("JWT_SECRET", "too-short");
        std::env::set_var("DATABASE_URL", "sqlite::memory:");

        let result = Config::from_env();
        assert!(result.is_err());

        if let Err(ConfigError::InvalidValue { variable, reason }) = result {
            assert_eq!(variable, "JWT_SECRET");
            assert!(reason.contains("at least 32 characters"));
        } else {
            panic!("Expected InvalidValue error");
        }
    }

    #[test]
    fn should_parse_valid_configuration() {
        let _lock = TEST_MUTEX.lock().unwrap();
        setup_test_env();

        std::env::set_var("DATABASE_URL", "sqlite:test.db");
        std::env::set_var("JWT_SECRET", "this-is-a-valid-secret-with-32-chars-minimum");
        std::env::set_var("SERVER_HOST", "0.0.0.0");
        std::env::set_var("SERVER_PORT", "9090");
        std::env::set_var("CORS_ALLOWED_ORIGINS", "http://example.com,http://test.com");
        std::env::set_var("ACCESS_TOKEN_EXPIRATION", "7200");
        std::env::set_var("REFRESH_TOKEN_EXPIRATION", "604800");

        let config = Config::from_env().expect("Should load valid config");

        assert_eq!(config.database.url, "sqlite:test.db");
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 9090);
        assert_eq!(config.server.address(), "0.0.0.0:9090");
        assert_eq!(config.cors.allowed_origins.len(), 2);
        assert_eq!(config.token.access_token_expiration, 7200);
        assert_eq!(config.token.refresh_token_expiration, 604800);
    }

    #[test]
    fn should_use_default_values_for_optional_vars() {
        let _lock = TEST_MUTEX.lock().unwrap();
        setup_test_env();

        std::env::set_var("DATABASE_URL", "sqlite::memory:");
        std::env::set_var("JWT_SECRET", "valid-secret-key-with-minimum-32-characters");

        let config = Config::from_env().expect("Should use defaults");

        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert!(!config.cors.allowed_origins.is_empty());
    }
}
