//! Authorization scopes — one per network-exposed proto Op.
//!
//! Grammar: `<resource>:<action>`. A token's `scope` claim is a
//! space-separated list of these strings, with two wildcard forms:
//!
//!   - `*`              all scopes
//!   - `<resource>:*`   all actions on a single resource
//!
//! Each `Op` declares exactly one required `Scope`. The check is
//! trivially auditable: equality on three short strings.
//!
//! Presets (`presets::ADMIN_LITERAL`, `presets::OPERATOR`,
//! `presets::DEPLOY`, `presets::READONLY`) are CLI sugar — the mint
//! flow expands them into explicit scope lists embedded in the JWT,
//! so the runtime check never knows about presets. To audit a preset,
//! read `presets.rs`.

pub mod presets;

mod name;
mod types;
mod validate;

pub use types::Scope;
pub use validate::{granted_allows, join_scopes, validate_string};
