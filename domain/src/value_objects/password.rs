use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash as Argon2Hash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Plain text password (not hashed)
///
/// # Security
/// - Should only exist temporarily during authentication
/// - Never store or log this value
#[derive(Debug, Clone)]
pub struct PlainPassword(String);

/// Hashed password using Argon2
///
/// # Invariants
/// - Always contains a valid Argon2 hash
/// - Includes salt and parameters
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasswordHash(String);

#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    #[error("Password too short: {0} characters (minimum 8)")]
    TooShort(usize),

    #[error("Password too long: {0} characters (maximum 128)")]
    TooLong(usize),

    #[error("Password cannot be empty")]
    Empty,

    #[error("Hash generation failed: {0}")]
    HashFailed(String),

    #[error("Hash verification failed: {0}")]
    VerificationFailed(String),

    #[error("Invalid hash format")]
    InvalidFormat,
}

impl PlainPassword {
    const MIN_LENGTH: usize = 8;
    const MAX_LENGTH: usize = 128;

    /// Creates a new plain password with validation.
    ///
    /// # Arguments
    /// * `password` - The plain text password
    ///
    /// # Returns
    /// Valid PlainPassword or PasswordError
    ///
    /// # Errors
    /// - `PasswordError::Empty` if password is empty
    /// - `PasswordError::TooShort` if less than 8 characters
    /// - `PasswordError::TooLong` if more than 128 characters
    pub fn new(password: impl Into<String>) -> Result<Self, PasswordError> {
        let password = password.into();

        if password.is_empty() {
            return Err(PasswordError::Empty);
        }

        if password.len() < Self::MIN_LENGTH {
            return Err(PasswordError::TooShort(password.len()));
        }

        if password.len() > Self::MAX_LENGTH {
            return Err(PasswordError::TooLong(password.len()));
        }

        Ok(Self(password))
    }

    /// Returns the password as a byte slice.
    ///
    /// # Security
    /// Only use this for hashing or verification.
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Hashes the password using Argon2.
    ///
    /// # Returns
    /// PasswordHash containing the hashed password
    ///
    /// # Errors
    /// Returns PasswordError if hashing fails
    pub fn hash(&self) -> Result<PasswordHash, PasswordError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let hash = argon2
            .hash_password(self.as_bytes(), &salt)
            .map_err(|e| PasswordError::HashFailed(e.to_string()))?;

        Ok(PasswordHash(hash.to_string()))
    }
}

impl PasswordHash {
    /// Creates a PasswordHash from an existing hash string.
    ///
    /// # Arguments
    /// * `hash` - The hash string (must be valid Argon2 format)
    ///
    /// # Errors
    /// Returns PasswordError::InvalidFormat if not a valid Argon2 hash
    pub fn new(hash: String) -> Result<Self, PasswordError> {
        // Validate that it's a proper Argon2 hash format
        Argon2Hash::new(&hash).map_err(|_| PasswordError::InvalidFormat)?;
        Ok(Self(hash))
    }

    /// Verifies a plain password against this hash.
    ///
    /// # Arguments
    /// * `plain` - The plain password to verify
    ///
    /// # Returns
    /// Ok(true) if password matches, Ok(false) if it doesn't
    ///
    /// # Errors
    /// Returns error if verification process fails
    pub fn verify(&self, plain: &PlainPassword) -> Result<bool, PasswordError> {
        let parsed_hash = Argon2Hash::new(&self.0)
            .map_err(|e| PasswordError::VerificationFailed(e.to_string()))?;

        let argon2 = Argon2::default();

        match argon2.verify_password(plain.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(PasswordError::VerificationFailed(e.to_string())),
        }
    }

    /// Returns the hash as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PasswordHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

// Don't implement Display for PlainPassword to avoid accidental logging
impl Drop for PlainPassword {
    fn drop(&mut self) {
        // Zero out the password in memory (best effort)
        self.0.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_accept_valid_password() {
        let password = PlainPassword::new("SecurePass123");
        assert!(password.is_ok());
    }

    #[test]
    fn should_reject_empty_password() {
        let password = PlainPassword::new("");
        assert!(matches!(password, Err(PasswordError::Empty)));
    }

    #[test]
    fn should_reject_too_short_password() {
        let password = PlainPassword::new("short");
        assert!(matches!(password, Err(PasswordError::TooShort(_))));
    }

    #[test]
    fn should_reject_too_long_password() {
        let password = PlainPassword::new("a".repeat(200));
        assert!(matches!(password, Err(PasswordError::TooLong(_))));
    }

    #[test]
    fn should_hash_password() {
        let plain = PlainPassword::new("SecurePass123").unwrap();
        let hash = plain.hash();
        assert!(hash.is_ok());
        assert!(hash.unwrap().as_str().starts_with("$argon2"));
    }

    #[test]
    fn should_verify_correct_password() {
        let plain = PlainPassword::new("SecurePass123").unwrap();
        let hash = plain.hash().unwrap();

        let verify_plain = PlainPassword::new("SecurePass123").unwrap();
        assert!(hash.verify(&verify_plain).unwrap());
    }

    #[test]
    fn should_reject_incorrect_password() {
        let plain = PlainPassword::new("SecurePass123").unwrap();
        let hash = plain.hash().unwrap();

        let wrong_plain = PlainPassword::new("WrongPassword").unwrap();
        assert!(!hash.verify(&wrong_plain).unwrap());
    }

    #[test]
    fn should_produce_different_hashes_for_same_password() {
        let plain1 = PlainPassword::new("SecurePass123").unwrap();
        let plain2 = PlainPassword::new("SecurePass123").unwrap();

        let hash1 = plain1.hash().unwrap();
        let hash2 = plain2.hash().unwrap();

        // Hashes should be different due to different salts
        assert_ne!(hash1, hash2);

        // But both should verify the same password
        let verify = PlainPassword::new("SecurePass123").unwrap();
        assert!(hash1.verify(&verify).unwrap());
        assert!(hash2.verify(&verify).unwrap());
    }
}
