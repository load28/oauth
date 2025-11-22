//! Value Objects for the domain layer
//!
//! All value objects follow the Newtype pattern for type safety.
//! They include validation logic and are immutable once created.

mod client_id;
mod email;
mod password;
mod pkce;
mod scope;
mod user_id;

pub use client_id::ClientId;
pub use email::{Email, EmailError};
pub use password::{PasswordError, PasswordHash, PlainPassword};
pub use pkce::{
    CodeChallenge, CodeChallengeMethod, CodeVerifier, PkceError, Plain, S256,
};
pub use scope::{Scope, ScopeError, Scopes};
pub use user_id::UserId;
