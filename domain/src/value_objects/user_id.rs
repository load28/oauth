use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Type-safe User ID using UUID v4
///
/// # Invariants
/// - Globally unique identifier
/// - Immutable once created
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId(Uuid);

impl UserId {
    /// Generates a new unique UserId using UUID v4.
    ///
    /// # Examples
    /// ```
    /// use domain::value_objects::UserId;
    ///
    /// let id = UserId::new();
    /// assert_ne!(id, UserId::new());
    /// ```
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a UserId from an existing UUID.
    ///
    /// # Arguments
    /// * `uuid` - The UUID to wrap
    ///
    /// # Examples
    /// ```
    /// use domain::value_objects::UserId;
    /// use uuid::Uuid;
    ///
    /// let uuid = Uuid::new_v4();
    /// let id = UserId::from_uuid(uuid);
    /// ```
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Parses a UserId from a string.
    ///
    /// # Arguments
    /// * `s` - String representation of UUID
    ///
    /// # Errors
    /// Returns error if the string is not a valid UUID
    pub fn parse(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }

    /// Returns the underlying UUID.
    pub fn value(&self) -> Uuid {
        self.0
    }

    /// Returns the UUID as a string.
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for UserId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<UserId> for Uuid {
    fn from(id: UserId) -> Self {
        id.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_generate_unique_ids() {
        let id1 = UserId::new();
        let id2 = UserId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn should_create_from_uuid() {
        let uuid = Uuid::new_v4();
        let id = UserId::from_uuid(uuid);
        assert_eq!(id.value(), uuid);
    }

    #[test]
    fn should_parse_from_string() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let id = UserId::parse(uuid_str).unwrap();
        assert_eq!(id.as_str(), uuid_str);
    }

    #[test]
    fn should_reject_invalid_uuid_string() {
        let result = UserId::parse("not-a-uuid");
        assert!(result.is_err());
    }

    #[test]
    fn should_display_as_string() {
        let id = UserId::new();
        let displayed = format!("{}", id);
        assert_eq!(displayed, id.as_str());
    }
}
