use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Type-safe OAuth Client ID using UUID v4
///
/// # Invariants
/// - Globally unique identifier for OAuth clients
/// - Immutable once created
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClientId(Uuid);

impl ClientId {
    /// Generates a new unique ClientId using UUID v4.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a ClientId from an existing UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Parses a ClientId from a string.
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

impl Default for ClientId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ClientId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for ClientId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<ClientId> for Uuid {
    fn from(id: ClientId) -> Self {
        id.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_generate_unique_client_ids() {
        let id1 = ClientId::new();
        let id2 = ClientId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn should_parse_from_string() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let id = ClientId::parse(uuid_str).unwrap();
        assert_eq!(id.as_str(), uuid_str);
    }
}
