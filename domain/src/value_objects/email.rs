use serde::{Deserialize, Serialize};
use std::fmt;

/// Validated email address
///
/// # Invariants
/// - Must contain '@' and '.'
/// - Maximum length 254 characters (RFC 5321)
/// - Cannot be empty
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Email(String);

#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("Invalid email format: {0}")]
    InvalidFormat(String),

    #[error("Email too long: {0} characters (max 254)")]
    TooLong(usize),

    #[error("Email cannot be empty")]
    Empty,
}

impl Email {
    const MAX_LENGTH: usize = 254;

    /// Creates a new email address with validation.
    ///
    /// # Arguments
    /// * `email` - The email string to validate
    ///
    /// # Returns
    /// Valid Email or EmailError
    ///
    /// # Errors
    /// - `EmailError::Empty` if the email is empty
    /// - `EmailError::TooLong` if longer than 254 characters
    /// - `EmailError::InvalidFormat` if format is invalid
    ///
    /// # Examples
    /// ```
    /// use domain::value_objects::Email;
    ///
    /// let email = Email::new("user@example.com").unwrap();
    /// assert_eq!(email.as_str(), "user@example.com");
    /// ```
    pub fn new(email: impl Into<String>) -> Result<Self, EmailError> {
        let email = email.into();

        if email.is_empty() {
            return Err(EmailError::Empty);
        }

        if email.len() > Self::MAX_LENGTH {
            return Err(EmailError::TooLong(email.len()));
        }

        if !Self::is_valid_format(&email) {
            return Err(EmailError::InvalidFormat(email));
        }

        Ok(Self(email.to_lowercase()))
    }

    /// Creates an Email without validation.
    ///
    /// # Safety
    /// Only use this when the email has already been validated by an external system.
    pub fn new_unchecked(email: String) -> Self {
        Self(email.to_lowercase())
    }

    /// Returns the email as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Basic email format validation.
    ///
    /// Checks for:
    /// - Contains exactly one '@'
    /// - Has text before and after '@'
    /// - Domain part contains at least one '.'
    fn is_valid_format(email: &str) -> bool {
        let parts: Vec<&str> = email.split('@').collect();

        if parts.len() != 2 {
            return false;
        }

        let local = parts[0];
        let domain = parts[1];

        !local.is_empty()
            && !domain.is_empty()
            && domain.contains('.')
            && !domain.starts_with('.')
            && !domain.ends_with('.')
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_accept_valid_email() {
        let email = Email::new("user@example.com");
        assert!(email.is_ok());
        assert_eq!(email.unwrap().as_str(), "user@example.com");
    }

    #[test]
    fn should_normalize_to_lowercase() {
        let email = Email::new("User@Example.COM").unwrap();
        assert_eq!(email.as_str(), "user@example.com");
    }

    #[test]
    fn should_reject_empty_email() {
        let email = Email::new("");
        assert!(matches!(email, Err(EmailError::Empty)));
    }

    #[test]
    fn should_reject_email_without_at() {
        let email = Email::new("userexample.com");
        assert!(matches!(email, Err(EmailError::InvalidFormat(_))));
    }

    #[test]
    fn should_reject_email_without_domain_dot() {
        let email = Email::new("user@example");
        assert!(matches!(email, Err(EmailError::InvalidFormat(_))));
    }

    #[test]
    fn should_reject_email_too_long() {
        let long_email = format!("{}@example.com", "a".repeat(300));
        let email = Email::new(long_email);
        assert!(matches!(email, Err(EmailError::TooLong(_))));
    }

    #[test]
    fn should_reject_multiple_at_signs() {
        let email = Email::new("user@@example.com");
        assert!(matches!(email, Err(EmailError::InvalidFormat(_))));
    }
}
