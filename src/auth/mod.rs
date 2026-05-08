//! Authentication: Ed25519-signed JWT bearer tokens. nub trusts a single
//! issuer key (auto-generated and persisted, or pinned externally via
//! `trusted_issuer` in config). Authorization derives from the token's
//! `scope` claim.

pub mod jwt;
pub mod scope;

mod issuer;
mod middleware;

pub use issuer::Issuer;
pub use jwt::Claims;
pub use middleware::{introspect, require_token, AuthState};
