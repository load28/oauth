//! User Use Case Port
//!
//! Defines operations for user registration and authentication.

use crate::errors::UseCaseError;
use async_trait::async_trait;
use domain::entities::User;
use domain::value_objects::{Email, UserId};

/// Input DTO for user registration
#[derive(Debug, Clone)]
pub struct RegisterUserInput {
    pub email: String,
    pub password: String,
}

/// Output DTO for user registration
#[derive(Debug, Clone)]
pub struct RegisterUserOutput {
    pub user_id: UserId,
    pub email: Email,
}

/// Input DTO for user authentication
#[derive(Debug, Clone)]
pub struct AuthenticateUserInput {
    pub email: String,
    pub password: String,
}

/// Output DTO for user authentication
#[derive(Debug, Clone)]
pub struct AuthenticateUserOutput {
    pub user_id: UserId,
    pub email: Email,
}

/// Input DTO for changing user password
#[derive(Debug, Clone)]
pub struct ChangePasswordInput {
    pub user_id: UserId,
    pub old_password: String,
    pub new_password: String,
}

/// Use case interface for user management
///
/// # Business Rules
/// - Email must be unique
/// - Password must meet security requirements
/// - Authentication requires correct email and password
#[async_trait]
pub trait UserUseCase: Send + Sync {
    /// Registers a new user.
    ///
    /// # Arguments
    /// * `input` - Registration data (email, password)
    ///
    /// # Returns
    /// RegisterUserOutput containing user ID and email
    ///
    /// # Errors
    /// - `UseCaseError::Validation` if email or password is invalid
    /// - `UseCaseError::Domain` if email already exists
    /// - `UseCaseError::Repository` for database errors
    ///
    /// # Business Rules
    /// - Email must be valid and unique
    /// - Password must meet minimum requirements
    async fn register_user(
        &self,
        input: RegisterUserInput,
    ) -> Result<RegisterUserOutput, UseCaseError>;

    /// Authenticates a user with email and password.
    ///
    /// # Arguments
    /// * `input` - Authentication data (email, password)
    ///
    /// # Returns
    /// AuthenticateUserOutput containing user ID and email
    ///
    /// # Errors
    /// - `UseCaseError::Unauthorized` if credentials are invalid
    /// - `UseCaseError::Repository` for database errors
    async fn authenticate_user(
        &self,
        input: AuthenticateUserInput,
    ) -> Result<AuthenticateUserOutput, UseCaseError>;

    /// Gets a user by ID.
    ///
    /// # Arguments
    /// * `user_id` - The user ID to retrieve
    ///
    /// # Returns
    /// The User entity
    ///
    /// # Errors
    /// - `UseCaseError::NotFound` if user doesn't exist
    /// - `UseCaseError::Repository` for database errors
    async fn get_user(&self, user_id: UserId) -> Result<User, UseCaseError>;

    /// Changes a user's password.
    ///
    /// # Arguments
    /// * `input` - Password change data
    ///
    /// # Returns
    /// Ok(()) on success
    ///
    /// # Errors
    /// - `UseCaseError::Unauthorized` if old password is incorrect
    /// - `UseCaseError::NotFound` if user doesn't exist
    /// - `UseCaseError::Validation` if new password is invalid
    /// - `UseCaseError::Repository` for database errors
    async fn change_password(&self, input: ChangePasswordInput) -> Result<(), UseCaseError>;
}
