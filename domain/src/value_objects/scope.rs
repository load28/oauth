use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

/// OAuth 2.0 Scope
///
/// Represents a single permission scope
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    /// OpenID Connect scope - required for OIDC
    OpenId,
    /// User profile information
    Profile,
    /// User email address
    Email,
    /// Read user data
    Read,
    /// Write user data
    Write,
    /// Custom scope
    Custom(String),
}

impl Scope {
    /// Returns the string representation of the scope.
    pub fn as_str(&self) -> &str {
        match self {
            Scope::OpenId => "openid",
            Scope::Profile => "profile",
            Scope::Email => "email",
            Scope::Read => "read",
            Scope::Write => "write",
            Scope::Custom(s) => s.as_str(),
        }
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Scope {
    type Err = ScopeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "openid" => Ok(Scope::OpenId),
            "profile" => Ok(Scope::Profile),
            "email" => Ok(Scope::Email),
            "read" => Ok(Scope::Read),
            "write" => Ok(Scope::Write),
            custom if !custom.is_empty() => Ok(Scope::Custom(custom.to_string())),
            _ => Err(ScopeError::InvalidScope(s.to_string())),
        }
    }
}

/// A collection of OAuth scopes
///
/// # Invariants
/// - No duplicate scopes
/// - At least one scope must be present for valid operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scopes {
    scopes: HashSet<Scope>,
}

#[derive(Debug, thiserror::Error)]
pub enum ScopeError {
    #[error("Invalid scope: {0}")]
    InvalidScope(String),

    #[error("Empty scope set")]
    Empty,

    #[error("Invalid scope string format: {0}")]
    InvalidFormat(String),
}

impl Scopes {
    /// Creates a new empty Scopes collection.
    pub fn new() -> Self {
        Self {
            scopes: HashSet::new(),
        }
    }

    /// Creates Scopes from a space-separated string.
    ///
    /// # Arguments
    /// * `scope_string` - Space-separated scope values (e.g., "openid profile email")
    ///
    /// # Returns
    /// Scopes or ScopeError
    ///
    /// # Errors
    /// - `ScopeError::Empty` if the string is empty
    /// - `ScopeError::InvalidScope` if any scope is invalid
    ///
    /// # Examples
    /// ```
    /// use domain::value_objects::Scopes;
    ///
    /// let scopes = Scopes::parse("openid profile email").unwrap();
    /// assert_eq!(scopes.to_string(), "email openid profile");
    /// ```
    pub fn parse(scope_string: &str) -> Result<Self, ScopeError> {
        if scope_string.trim().is_empty() {
            return Err(ScopeError::Empty);
        }

        let scopes: Result<HashSet<Scope>, ScopeError> = scope_string
            .split_whitespace()
            .map(|s| s.parse::<Scope>())
            .collect();

        Ok(Self { scopes: scopes? })
    }

    /// Creates Scopes from a vector of Scope enums.
    pub fn from_vec(scopes: Vec<Scope>) -> Result<Self, ScopeError> {
        if scopes.is_empty() {
            return Err(ScopeError::Empty);
        }

        Ok(Self {
            scopes: scopes.into_iter().collect(),
        })
    }

    /// Adds a scope to the collection.
    pub fn add(&mut self, scope: Scope) {
        self.scopes.insert(scope);
    }

    /// Checks if a specific scope is present.
    pub fn contains(&self, scope: &Scope) -> bool {
        self.scopes.contains(scope)
    }

    /// Checks if this contains all scopes from another Scopes.
    pub fn contains_all(&self, other: &Scopes) -> bool {
        other.scopes.iter().all(|s| self.scopes.contains(s))
    }

    /// Returns the number of scopes.
    pub fn len(&self) -> usize {
        self.scopes.len()
    }

    /// Checks if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.scopes.is_empty()
    }

    /// Returns an iterator over the scopes.
    pub fn iter(&self) -> impl Iterator<Item = &Scope> {
        self.scopes.iter()
    }

    /// Returns the scopes as a Vec.
    pub fn to_vec(&self) -> Vec<Scope> {
        self.scopes.iter().cloned().collect()
    }
}

impl Default for Scopes {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Scopes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut scopes: Vec<&str> = self.scopes.iter().map(|s| s.as_str()).collect();
        scopes.sort();
        write!(f, "{}", scopes.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_single_scope() {
        let scope: Scope = "openid".parse().unwrap();
        assert_eq!(scope, Scope::OpenId);
    }

    #[test]
    fn should_parse_scope_string() {
        let scopes = Scopes::parse("openid profile email").unwrap();
        assert_eq!(scopes.len(), 3);
        assert!(scopes.contains(&Scope::OpenId));
        assert!(scopes.contains(&Scope::Profile));
        assert!(scopes.contains(&Scope::Email));
    }

    #[test]
    fn should_handle_extra_whitespace() {
        let scopes = Scopes::parse("  openid   profile  ").unwrap();
        assert_eq!(scopes.len(), 2);
    }

    #[test]
    fn should_reject_empty_scope_string() {
        let scopes = Scopes::parse("");
        assert!(matches!(scopes, Err(ScopeError::Empty)));
    }

    #[test]
    fn should_remove_duplicates() {
        let scopes = Scopes::parse("openid openid profile").unwrap();
        assert_eq!(scopes.len(), 2);
    }

    #[test]
    fn should_convert_to_string() {
        let scopes = Scopes::parse("profile openid email").unwrap();
        // Should be sorted
        assert_eq!(scopes.to_string(), "email openid profile");
    }

    #[test]
    fn should_check_contains_all() {
        let scopes1 = Scopes::parse("openid profile email").unwrap();
        let scopes2 = Scopes::parse("openid profile").unwrap();

        assert!(scopes1.contains_all(&scopes2));
        assert!(!scopes2.contains_all(&scopes1));
    }

    #[test]
    fn should_support_custom_scopes() {
        let scopes = Scopes::parse("openid custom:scope").unwrap();
        assert_eq!(scopes.len(), 2);
        assert!(scopes.contains(&Scope::OpenId));
    }
}
