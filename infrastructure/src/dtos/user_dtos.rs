//! User-related Data Transfer Objects

use serde::{Deserialize, Serialize};

/// Request to register a new user
///
/// # Validation
///
/// - `email`: Must be valid email format
/// - `password`: Must be 8-128 characters
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterUserRequest {
    pub email: String,
    pub password: String,
}

/// Response after successful user registration
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterUserResponse {
    pub user_id: String,
    pub email: String,
    pub created_at: String,
}

/// Request to authenticate a user (login)
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticateUserRequest {
    pub email: String,
    pub password: String,
}

/// Response after successful authentication
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticateUserResponse {
    pub user_id: String,
    pub email: String,
}

/// Response for GET /users/{id}
#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserResponse {
    pub user_id: String,
    pub email: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Request to change user password
#[derive(Debug, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

/// Response after successful password change
#[derive(Debug, Serialize, Deserialize)]
pub struct ChangePasswordResponse {
    pub success: bool,
    pub message: String,
}
