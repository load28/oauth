use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use std::marker::PhantomData;

/// PKCE Code Challenge Method
///
/// # Standards
/// RFC 7636 - Proof Key for Code Exchange
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CodeChallengeMethod {
    /// Plain text (not recommended, only for legacy clients)
    Plain,
    /// SHA-256 hash (recommended)
    S256,
}

impl CodeChallengeMethod {
    pub fn as_str(&self) -> &str {
        match self {
            CodeChallengeMethod::Plain => "plain",
            CodeChallengeMethod::S256 => "S256",
        }
    }
}

impl fmt::Display for CodeChallengeMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for CodeChallengeMethod {
    type Err = PkceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "plain" => Ok(CodeChallengeMethod::Plain),
            "S256" => Ok(CodeChallengeMethod::S256),
            _ => Err(PkceError::InvalidMethod(s.to_string())),
        }
    }
}

/// Marker type for Plain method
#[derive(Debug, Clone, Copy)]
pub struct Plain;

/// Marker type for S256 method
#[derive(Debug, Clone, Copy)]
pub struct S256;

/// PKCE Code Verifier
///
/// # Invariants
/// - Length between 43 and 128 characters
/// - Contains only unreserved characters (RFC 3986)
///
/// # Standards
/// RFC 7636 Section 4.1
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeVerifier(String);

/// PKCE Code Challenge
///
/// Uses Phantom Type to track the challenge method at compile time.
///
/// # Type Parameters
/// * `M` - The challenge method marker type (Plain or S256)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeChallenge<M> {
    value: String,
    _method: PhantomData<M>,
}

#[derive(Debug, thiserror::Error)]
pub enum PkceError {
    #[error("Code verifier too short: {0} characters (minimum 43)")]
    VerifierTooShort(usize),

    #[error("Code verifier too long: {0} characters (maximum 128)")]
    VerifierTooLong(usize),

    #[error("Code verifier contains invalid characters")]
    InvalidVerifierCharacters,

    #[error("Invalid challenge method: {0}")]
    InvalidMethod(String),

    #[error("Code challenge verification failed")]
    VerificationFailed,

    #[error("Cannot verify: challenge method mismatch")]
    MethodMismatch,
}

impl CodeVerifier {
    const MIN_LENGTH: usize = 43;
    const MAX_LENGTH: usize = 128;

    /// Generates a new cryptographically secure code verifier.
    ///
    /// # Examples
    /// ```
    /// use domain::value_objects::CodeVerifier;
    ///
    /// let verifier = CodeVerifier::generate();
    /// assert!(verifier.as_str().len() >= 43);
    /// ```
    pub fn generate() -> Self {
        use rand::Rng;

        let random_bytes: Vec<u8> = (0..32).map(|_| rand::rng().random()).collect();
        let verifier = URL_SAFE_NO_PAD.encode(&random_bytes);

        // This should always be valid since we control the generation
        Self::new(verifier).expect("Generated verifier should be valid")
    }

    /// Creates a CodeVerifier from a string with validation.
    ///
    /// # Arguments
    /// * `verifier` - The code verifier string
    ///
    /// # Errors
    /// - `PkceError::VerifierTooShort` if less than 43 characters
    /// - `PkceError::VerifierTooLong` if more than 128 characters
    /// - `PkceError::InvalidVerifierCharacters` if contains invalid characters
    pub fn new(verifier: impl Into<String>) -> Result<Self, PkceError> {
        let verifier = verifier.into();

        if verifier.len() < Self::MIN_LENGTH {
            return Err(PkceError::VerifierTooShort(verifier.len()));
        }

        if verifier.len() > Self::MAX_LENGTH {
            return Err(PkceError::VerifierTooLong(verifier.len()));
        }

        // RFC 7636: unreserved characters [A-Z] / [a-z] / [0-9] / "-" / "." / "_" / "~"
        if !verifier
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_' | '~'))
        {
            return Err(PkceError::InvalidVerifierCharacters);
        }

        Ok(Self(verifier))
    }

    /// Returns the verifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Creates a SHA-256 code challenge from this verifier.
    ///
    /// # Returns
    /// CodeChallenge with S256 method
    pub fn create_s256_challenge(&self) -> CodeChallenge<S256> {
        let mut hasher = Sha256::new();
        hasher.update(self.0.as_bytes());
        let hash = hasher.finalize();
        let challenge = URL_SAFE_NO_PAD.encode(hash);

        CodeChallenge {
            value: challenge,
            _method: PhantomData,
        }
    }

    /// Creates a plain code challenge from this verifier.
    ///
    /// # Security
    /// Plain method is not recommended. Use S256 instead.
    pub fn create_plain_challenge(&self) -> CodeChallenge<Plain> {
        CodeChallenge {
            value: self.0.clone(),
            _method: PhantomData,
        }
    }
}

impl<M> CodeChallenge<M> {
    /// Returns the challenge as a string slice.
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Returns the challenge value.
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl CodeChallenge<S256> {
    /// Creates a CodeChallenge from a string (S256 method).
    pub fn new_s256(challenge: String) -> Self {
        Self {
            value: challenge,
            _method: PhantomData,
        }
    }

    /// Verifies a code verifier against this S256 challenge.
    ///
    /// # Arguments
    /// * `verifier` - The code verifier to check
    ///
    /// # Returns
    /// Ok(true) if verification succeeds
    ///
    /// # Errors
    /// Returns error if verification fails
    pub fn verify(&self, verifier: &CodeVerifier) -> Result<bool, PkceError> {
        let computed_challenge = verifier.create_s256_challenge();
        Ok(self.value == computed_challenge.value)
    }

    /// Returns the method type.
    pub fn method(&self) -> CodeChallengeMethod {
        CodeChallengeMethod::S256
    }
}

impl CodeChallenge<Plain> {
    /// Creates a CodeChallenge from a string (Plain method).
    pub fn new_plain(challenge: String) -> Self {
        Self {
            value: challenge,
            _method: PhantomData,
        }
    }

    /// Verifies a code verifier against this Plain challenge.
    pub fn verify(&self, verifier: &CodeVerifier) -> Result<bool, PkceError> {
        Ok(self.value == verifier.as_str())
    }

    /// Returns the method type.
    pub fn method(&self) -> CodeChallengeMethod {
        CodeChallengeMethod::Plain
    }
}

impl fmt::Display for CodeVerifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<M> fmt::Display for CodeChallenge<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_generate_valid_verifier() {
        let verifier = CodeVerifier::generate();
        assert!(verifier.as_str().len() >= CodeVerifier::MIN_LENGTH);
        assert!(verifier.as_str().len() <= CodeVerifier::MAX_LENGTH);
    }

    #[test]
    fn should_reject_short_verifier() {
        let short = "a".repeat(40);
        let verifier = CodeVerifier::new(short);
        assert!(matches!(verifier, Err(PkceError::VerifierTooShort(_))));
    }

    #[test]
    fn should_reject_long_verifier() {
        let long = "a".repeat(150);
        let verifier = CodeVerifier::new(long);
        assert!(matches!(verifier, Err(PkceError::VerifierTooLong(_))));
    }

    #[test]
    fn should_reject_invalid_characters() {
        let invalid = format!("{}@#$%", "a".repeat(40));
        let verifier = CodeVerifier::new(invalid);
        assert!(matches!(
            verifier,
            Err(PkceError::InvalidVerifierCharacters)
        ));
    }

    #[test]
    fn should_create_s256_challenge() {
        let verifier = CodeVerifier::generate();
        let challenge = verifier.create_s256_challenge();
        assert!(!challenge.as_str().is_empty());
    }

    #[test]
    fn should_verify_s256_challenge() {
        let verifier = CodeVerifier::generate();
        let challenge = verifier.create_s256_challenge();

        assert!(challenge.verify(&verifier).unwrap());
    }

    #[test]
    fn should_fail_verification_with_wrong_verifier() {
        let verifier1 = CodeVerifier::generate();
        let challenge = verifier1.create_s256_challenge();

        let verifier2 = CodeVerifier::generate();
        assert!(!challenge.verify(&verifier2).unwrap());
    }

    #[test]
    fn should_create_plain_challenge() {
        let verifier = CodeVerifier::generate();
        let challenge = verifier.create_plain_challenge();
        assert_eq!(challenge.as_str(), verifier.as_str());
    }

    #[test]
    fn should_verify_plain_challenge() {
        let verifier = CodeVerifier::generate();
        let challenge = verifier.create_plain_challenge();
        assert!(challenge.verify(&verifier).unwrap());
    }

    #[test]
    fn should_have_different_hashes_for_different_verifiers() {
        let verifier1 = CodeVerifier::generate();
        let verifier2 = CodeVerifier::generate();

        let challenge1 = verifier1.create_s256_challenge();
        let challenge2 = verifier2.create_s256_challenge();

        assert_ne!(challenge1.as_str(), challenge2.as_str());
    }
}
