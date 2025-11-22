use crate::errors::UserError;
use crate::value_objects::{Email, PasswordHash, PlainPassword, UserId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// User entity
///
/// # Invariants
/// - UserId is unique and immutable
/// - Email is validated and unique
/// - Password is always hashed
/// - Created timestamp is immutable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: UserId,
    email: Email,
    password_hash: PasswordHash,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl User {
    /// Creates a new User with hashed password.
    ///
    /// # Arguments
    /// * `id` - Unique user identifier
    /// * `email` - Validated email address
    /// * `password_hash` - Pre-hashed password
    ///
    /// # Examples
    /// ```
    /// use domain::entities::User;
    /// use domain::value_objects::{UserId, Email, PlainPassword};
    ///
    /// let id = UserId::new();
    /// let email = Email::new("user@example.com").unwrap();
    /// let plain = PlainPassword::new("SecurePass123").unwrap();
    /// let hash = plain.hash().unwrap();
    ///
    /// let user = User::new(id, email, hash);
    /// ```
    pub fn new(id: UserId, email: Email, password_hash: PasswordHash) -> Self {
        let now = Utc::now();
        Self {
            id,
            email,
            password_hash,
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates a User from existing data (e.g., from database).
    ///
    /// # Arguments
    /// * `id` - User ID
    /// * `email` - Email address
    /// * `password_hash` - Password hash
    /// * `created_at` - Creation timestamp
    /// * `updated_at` - Last update timestamp
    pub fn from_existing(
        id: UserId,
        email: Email,
        password_hash: PasswordHash,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            email,
            password_hash,
            created_at,
            updated_at,
        }
    }

    /// Returns the user ID.
    pub fn id(&self) -> UserId {
        self.id
    }

    /// Returns the email address.
    pub fn email(&self) -> &Email {
        &self.email
    }

    /// Returns the password hash.
    pub fn password_hash(&self) -> &PasswordHash {
        &self.password_hash
    }

    /// Returns the creation timestamp.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Returns the last update timestamp.
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Changes the user's email address.
    ///
    /// # Arguments
    /// * `new_email` - The new email address
    ///
    /// # Returns
    /// Ok(()) or UserError
    ///
    /// # Errors
    /// - `UserError::SameEmail` if the new email is the same as current
    ///
    /// # Business Rules
    /// - Email must be different from current
    /// - Updates the updated_at timestamp
    pub fn change_email(&mut self, new_email: Email) -> Result<(), UserError> {
        if self.email == new_email {
            return Err(UserError::SameEmail);
        }

        self.email = new_email;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Changes the user's password.
    ///
    /// # Arguments
    /// * `new_password_hash` - The new password hash
    ///
    /// # Business Rules
    /// - Password must be pre-hashed
    /// - Updates the updated_at timestamp
    pub fn change_password(&mut self, new_password_hash: PasswordHash) {
        self.password_hash = new_password_hash;
        self.updated_at = Utc::now();
    }

    /// Verifies a plain password against the stored hash.
    ///
    /// # Arguments
    /// * `plain_password` - The plain password to verify
    ///
    /// # Returns
    /// Ok(true) if password matches, Ok(false) if it doesn't
    ///
    /// # Errors
    /// Returns error if verification process fails
    ///
    /// # Business Rules
    /// - Does not modify any state
    /// - Uses constant-time comparison (via Argon2)
    pub fn verify_password(&self, plain_password: &PlainPassword) -> Result<bool, UserError> {
        self.password_hash
            .verify(plain_password)
            .map_err(|_| UserError::InvalidCredentials)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_user() -> User {
        let id = UserId::new();
        let email = Email::new("test@example.com").unwrap();
        let plain = PlainPassword::new("SecurePass123").unwrap();
        let hash = plain.hash().unwrap();
        User::new(id, email, hash)
    }

    #[test]
    fn should_create_new_user() {
        let user = create_test_user();
        assert_eq!(user.email().as_str(), "test@example.com");
        assert!(user.created_at() <= Utc::now());
        assert_eq!(user.created_at(), user.updated_at());
    }

    #[test]
    fn should_change_email() {
        let mut user = create_test_user();
        let old_updated_at = user.updated_at();

        std::thread::sleep(std::time::Duration::from_millis(10));

        let new_email = Email::new("newemail@example.com").unwrap();
        let result = user.change_email(new_email.clone());

        assert!(result.is_ok());
        assert_eq!(user.email(), &new_email);
        assert!(user.updated_at() > old_updated_at);
    }

    #[test]
    fn should_reject_same_email() {
        let mut user = create_test_user();
        let same_email = Email::new("test@example.com").unwrap();

        let result = user.change_email(same_email);
        assert!(matches!(result, Err(UserError::SameEmail)));
    }

    #[test]
    fn should_change_password() {
        let mut user = create_test_user();
        let old_updated_at = user.updated_at();

        std::thread::sleep(std::time::Duration::from_millis(10));

        let new_plain = PlainPassword::new("NewSecurePass456").unwrap();
        let new_hash = new_plain.hash().unwrap();
        user.change_password(new_hash);

        assert!(user.updated_at() > old_updated_at);
    }

    #[test]
    fn should_verify_correct_password() {
        let plain = PlainPassword::new("SecurePass123").unwrap();
        let hash = plain.hash().unwrap();
        let user = User::new(UserId::new(), Email::new("test@example.com").unwrap(), hash);

        let verify_plain = PlainPassword::new("SecurePass123").unwrap();
        assert!(user.verify_password(&verify_plain).unwrap());
    }

    #[test]
    fn should_reject_incorrect_password() {
        let plain = PlainPassword::new("SecurePass123").unwrap();
        let hash = plain.hash().unwrap();
        let user = User::new(UserId::new(), Email::new("test@example.com").unwrap(), hash);

        let wrong_plain = PlainPassword::new("WrongPassword").unwrap();
        assert!(!user.verify_password(&wrong_plain).unwrap());
    }

    #[test]
    fn should_create_from_existing() {
        let id = UserId::new();
        let email = Email::new("test@example.com").unwrap();
        let plain = PlainPassword::new("SecurePass123").unwrap();
        let hash = plain.hash().unwrap();
        let created = Utc::now();
        let updated = Utc::now();

        let user = User::from_existing(id, email, hash, created, updated);

        assert_eq!(user.id(), id);
        assert_eq!(user.created_at(), created);
        assert_eq!(user.updated_at(), updated);
    }
}
