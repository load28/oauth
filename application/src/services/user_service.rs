//! User Service Implementation
//!
//! Implements UserUseCase for user management operations.

use crate::errors::UseCaseError;
use crate::ports::{
    AuthenticateUserInput, AuthenticateUserOutput, ChangePasswordInput, RegisterUserInput,
    RegisterUserOutput, UserRepository, UserUseCase,
};
use async_trait::async_trait;
use domain::entities::User;
use domain::value_objects::{Email, PasswordHash, PlainPassword, UserId};
use std::sync::Arc;

/// Service implementation for user management
///
/// # Type Parameters
/// * `R` - The repository implementation (must implement UserRepository)
///
/// # Business Logic
/// - Ensures email uniqueness during registration
/// - Validates credentials during authentication
/// - Enforces password policies through domain entities
pub struct UserService<R: UserRepository> {
    user_repository: Arc<R>,
}

impl<R: UserRepository> UserService<R> {
    /// Creates a new UserService
    ///
    /// # Arguments
    /// * `user_repository` - Repository for persisting users
    pub fn new(user_repository: Arc<R>) -> Self {
        Self { user_repository }
    }
}

#[async_trait]
impl<R: UserRepository> UserUseCase for UserService<R> {
    async fn register_user(
        &self,
        input: RegisterUserInput,
    ) -> Result<RegisterUserOutput, UseCaseError> {
        // 1. Validate and create email
        let email: Email =
            Email::new(input.email).map_err(|e| UseCaseError::Validation(e.to_string()))?;

        // 2. Check if email already exists
        if self.user_repository.exists_by_email(&email).await? {
            return Err(UseCaseError::Domain(format!(
                "Email already in use: {}",
                email.as_str()
            )));
        }

        // 3. Validate and hash password
        let plain_password: PlainPassword = PlainPassword::new(input.password)
            .map_err(|e| UseCaseError::Validation(e.to_string()))?;
        let password_hash: PasswordHash = plain_password
            .hash()
            .map_err(|e| UseCaseError::Internal(e.to_string()))?;

        // 4. Create user entity
        let user: User = User::new(UserId::new(), email.clone(), password_hash);

        // 5. Persist user
        self.user_repository.save(&user).await?;

        // 6. Return output
        Ok(RegisterUserOutput {
            user_id: user.id(),
            email: user.email().clone(),
        })
    }

    async fn authenticate_user(
        &self,
        input: AuthenticateUserInput,
    ) -> Result<AuthenticateUserOutput, UseCaseError> {
        // 1. Validate email
        let email: Email =
            Email::new(input.email).map_err(|e| UseCaseError::Validation(e.to_string()))?;

        // 2. Find user by email
        let user: User = self
            .user_repository
            .find_by_email(&email)
            .await?
            .ok_or(UseCaseError::Unauthorized)?;

        // 3. Validate password
        let plain_password: PlainPassword = PlainPassword::new(input.password)
            .map_err(|e| UseCaseError::Validation(e.to_string()))?;

        if !user
            .verify_password(&plain_password)
            .map_err(|e| UseCaseError::Internal(e.to_string()))?
        {
            return Err(UseCaseError::Unauthorized);
        }

        // 4. Return output
        Ok(AuthenticateUserOutput {
            user_id: user.id(),
            email: user.email().clone(),
        })
    }

    async fn get_user(&self, user_id: UserId) -> Result<User, UseCaseError> {
        self.user_repository
            .find_by_id(user_id)
            .await?
            .ok_or(UseCaseError::NotFound)
    }

    async fn change_password(&self, input: ChangePasswordInput) -> Result<(), UseCaseError> {
        // 1. Find user
        let mut user: User = self
            .user_repository
            .find_by_id(input.user_id)
            .await?
            .ok_or(UseCaseError::NotFound)?;

        // 2. Validate old password
        let old_password: PlainPassword = PlainPassword::new(input.old_password)
            .map_err(|e| UseCaseError::Validation(e.to_string()))?;

        // Verify old password
        if !user
            .verify_password(&old_password)
            .map_err(|e| UseCaseError::Internal(e.to_string()))?
        {
            return Err(UseCaseError::Unauthorized);
        }

        // 3. Validate and hash new password
        let new_password: PlainPassword = PlainPassword::new(input.new_password)
            .map_err(|e| UseCaseError::Validation(e.to_string()))?;
        let new_password_hash: PasswordHash = new_password
            .hash()
            .map_err(|e| UseCaseError::Internal(e.to_string()))?;

        // 4. Change password
        user.change_password(new_password_hash);

        // 5. Persist changes
        self.user_repository.save(&user).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::RepositoryError;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// Mock repository for testing
    struct MockUserRepository {
        users: Mutex<HashMap<UserId, User>>,
        users_by_email: Mutex<HashMap<String, User>>,
    }

    impl MockUserRepository {
        fn new() -> Self {
            Self {
                users: Mutex::new(HashMap::new()),
                users_by_email: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn save(&self, user: &User) -> Result<(), RepositoryError> {
            let mut users = self.users.lock().unwrap();
            let mut users_by_email = self.users_by_email.lock().unwrap();

            users.insert(user.id(), user.clone());
            users_by_email.insert(user.email().as_str().to_string(), user.clone());

            Ok(())
        }

        async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepositoryError> {
            let users = self.users.lock().unwrap();
            Ok(users.get(&id).cloned())
        }

        async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepositoryError> {
            let users_by_email = self.users_by_email.lock().unwrap();
            Ok(users_by_email.get(email.as_str()).cloned())
        }

        async fn exists_by_email(&self, email: &Email) -> Result<bool, RepositoryError> {
            let users_by_email = self.users_by_email.lock().unwrap();
            Ok(users_by_email.contains_key(email.as_str()))
        }

        async fn delete(&self, id: UserId) -> Result<(), RepositoryError> {
            let mut users = self.users.lock().unwrap();
            let mut users_by_email = self.users_by_email.lock().unwrap();

            if let Some(user) = users.remove(&id) {
                users_by_email.remove(user.email().as_str());
                Ok(())
            } else {
                Err(RepositoryError::NotFound)
            }
        }
    }

    #[tokio::test]
    async fn should_register_new_user() {
        let repo = Arc::new(MockUserRepository::new());
        let service = UserService::new(repo);

        let input = RegisterUserInput {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let result = service.register_user(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.email.as_str(), "test@example.com");
    }

    #[tokio::test]
    async fn should_reject_duplicate_email() {
        let repo = Arc::new(MockUserRepository::new());
        let service = UserService::new(repo);

        let input = RegisterUserInput {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        service.register_user(input.clone()).await.unwrap();
        let result = service.register_user(input).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UseCaseError::Domain(_)));
    }

    #[tokio::test]
    async fn should_authenticate_user_with_correct_credentials() {
        let repo = Arc::new(MockUserRepository::new());
        let service = UserService::new(repo);

        let register_input = RegisterUserInput {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };
        service.register_user(register_input).await.unwrap();

        let auth_input = AuthenticateUserInput {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let result = service.authenticate_user(auth_input).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_reject_authentication_with_wrong_password() {
        let repo = Arc::new(MockUserRepository::new());
        let service = UserService::new(repo);

        let register_input = RegisterUserInput {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };
        service.register_user(register_input).await.unwrap();

        let auth_input = AuthenticateUserInput {
            email: "test@example.com".to_string(),
            password: "wrongpassword".to_string(),
        };

        let result = service.authenticate_user(auth_input).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UseCaseError::Unauthorized));
    }

    #[tokio::test]
    async fn should_change_password() {
        let repo = Arc::new(MockUserRepository::new());
        let service = UserService::new(repo);

        let register_input = RegisterUserInput {
            email: "test@example.com".to_string(),
            password: "oldpassword".to_string(),
        };
        let user_output = service.register_user(register_input).await.unwrap();

        let change_input = ChangePasswordInput {
            user_id: user_output.user_id,
            old_password: "oldpassword".to_string(),
            new_password: "newpassword".to_string(),
        };

        let result = service.change_password(change_input).await;
        assert!(result.is_ok());

        // Verify new password works
        let auth_input = AuthenticateUserInput {
            email: "test@example.com".to_string(),
            password: "newpassword".to_string(),
        };
        assert!(service.authenticate_user(auth_input).await.is_ok());
    }

    #[tokio::test]
    async fn should_reject_password_change_with_wrong_old_password() {
        let repo = Arc::new(MockUserRepository::new());
        let service = UserService::new(repo);

        let register_input = RegisterUserInput {
            email: "test@example.com".to_string(),
            password: "oldpassword".to_string(),
        };
        let user_output = service.register_user(register_input).await.unwrap();

        let change_input = ChangePasswordInput {
            user_id: user_output.user_id,
            old_password: "wrongpassword".to_string(),
            new_password: "newpassword".to_string(),
        };

        let result = service.change_password(change_input).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UseCaseError::Unauthorized));
    }

    #[tokio::test]
    async fn should_get_user_by_id() {
        let repo = Arc::new(MockUserRepository::new());
        let service = UserService::new(repo);

        let register_input = RegisterUserInput {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };
        let user_output = service.register_user(register_input).await.unwrap();

        let result = service.get_user(user_output.user_id).await;
        assert!(result.is_ok());

        let user = result.unwrap();
        assert_eq!(user.email().as_str(), "test@example.com");
    }

    #[tokio::test]
    async fn should_return_not_found_for_nonexistent_user() {
        let repo = Arc::new(MockUserRepository::new());
        let service = UserService::new(repo);

        let result = service.get_user(UserId::new()).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UseCaseError::NotFound));
    }
}
