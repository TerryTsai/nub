//! Authorization scopes — one per network-exposed proto Op.
//!
//! Grammar: `<resource>:<action>`. A token's `scope` claim is a
//! space-separated list of these strings, with two wildcard forms:
//!
//!   - `*`              all scopes
//!   - `<resource>:*`   all actions on a single resource
//!
//! Each `Op` declares exactly one required `Scope` (or `None` for
//! introspection ops like `whoami`/`host_info`). The check is trivially
//! auditable: equality on three short strings.
//!
//! Presets (`presets::ADMIN_LITERAL`, `presets::PHONE`, `presets::READONLY`)
//! are CLI sugar — the mint flow expands them into explicit scope lists
//! embedded in the JWT, so the runtime check never knows about presets.
//! To audit a preset, read `presets.rs`.

pub mod presets;
mod strings;
#[cfg(test)]
mod tests;

use std::fmt;
use std::str::FromStr;

/// Every scope nub recognizes. One variant per network-exposed op.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Scope {
    HostInfo,
    AuthWhoami,

    ContainersList,
    ContainersGet,
    ContainersLogs,
    ContainersStats,
    ContainersCreate,
    ContainersStart,
    ContainersStop,
    ContainersRestart,
    ContainersKill,
    ContainersRemove,
    ContainersExec,

    ImagesList,
    ImagesGet,
    ImagesPull,
    ImagesBuild,
    ImagesDelete,

    VolumesList,
    VolumesGet,
    VolumesCreate,
    VolumesDelete,

    NetworksList,
    NetworksGet,
    NetworksCreate,
    NetworksDelete,

    DockerfilesList,
    DockerfilesGet,
    DockerfilesPut,
    DockerfilesDelete,

    StacksList,
    StacksGet,
    StacksLogs,
    StacksCreate,
    StacksDelete,
    StacksRedeploy,
    StacksUpdate,
    StacksPull,

    SecretsList,
    SecretsPut,
    SecretsDelete,
    /// Privileged: read a secret's plaintext value back. CLI-only by
    /// convention — no preset grants it; only `*` (admin) authorizes.
    SecretsReveal,
}

impl Scope {
    /// Every scope, in declaration order. Used for validation, listing,
    /// and completions; not for runtime checks.
    pub const ALL: &'static [Scope] = &[
        Scope::HostInfo,
        Scope::AuthWhoami,
        Scope::ContainersList,
        Scope::ContainersGet,
        Scope::ContainersLogs,
        Scope::ContainersStats,
        Scope::ContainersCreate,
        Scope::ContainersStart,
        Scope::ContainersStop,
        Scope::ContainersRestart,
        Scope::ContainersKill,
        Scope::ContainersRemove,
        Scope::ContainersExec,
        Scope::ImagesList,
        Scope::ImagesGet,
        Scope::ImagesPull,
        Scope::ImagesBuild,
        Scope::ImagesDelete,
        Scope::VolumesList,
        Scope::VolumesGet,
        Scope::VolumesCreate,
        Scope::VolumesDelete,
        Scope::NetworksList,
        Scope::NetworksGet,
        Scope::NetworksCreate,
        Scope::NetworksDelete,
        Scope::DockerfilesList,
        Scope::DockerfilesGet,
        Scope::DockerfilesPut,
        Scope::DockerfilesDelete,
        Scope::StacksList,
        Scope::StacksGet,
        Scope::StacksLogs,
        Scope::StacksCreate,
        Scope::StacksDelete,
        Scope::StacksRedeploy,
        Scope::StacksUpdate,
        Scope::StacksPull,
        Scope::SecretsList,
        Scope::SecretsPut,
        Scope::SecretsDelete,
        Scope::SecretsReveal,
    ];
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Scope {
    type Err = ParseScopeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for &scope in Scope::ALL {
            if scope.as_str() == s {
                return Ok(scope);
            }
        }
        Err(ParseScopeError(s.to_string()))
    }
}

#[derive(Debug)]
pub struct ParseScopeError(pub String);

impl fmt::Display for ParseScopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown scope `{}`", self.0)
    }
}

impl std::error::Error for ParseScopeError {}

/// Resources that admit `<resource>:*` wildcards. Kept in sync with
/// `Scope::ALL` by `tests::resources_match_scopes`.
pub const RESOURCES: &[&str] = &[
    "host",
    "auth",
    "containers",
    "images",
    "volumes",
    "networks",
    "dockerfiles",
    "stacks",
    "secrets",
];

/// Validate one scope token (one word from a JWT `scope` claim or
/// `--scope` arg). Accepts `*`, `<resource>:*`, or any concrete scope.
pub fn validate_token(tok: &str) -> Result<(), ParseScopeError> {
    if tok == "*" {
        return Ok(());
    }
    if tok.matches(':').count() != 1 {
        return Err(ParseScopeError(tok.to_string()));
    }
    let (res, action) = tok.split_once(':').unwrap();
    if res.is_empty() || action.is_empty() {
        return Err(ParseScopeError(tok.to_string()));
    }
    if action == "*" {
        if RESOURCES.contains(&res) {
            return Ok(());
        }
        return Err(ParseScopeError(tok.to_string()));
    }
    tok.parse::<Scope>().map(|_| ())
}

/// Validate a whole space- or comma-separated scope string. Returns
/// the list of unknown tokens if any are bad.
pub fn validate_string(s: &str) -> Result<(), Vec<String>> {
    let bad: Vec<String> = s
        .split_ascii_whitespace()
        .filter(|t| validate_token(t).is_err())
        .map(str::to_string)
        .collect();
    if bad.is_empty() {
        Ok(())
    } else {
        Err(bad)
    }
}

/// Does any token in the granted scope string authorize `needed`?
pub fn granted_allows(granted: &str, needed: Scope) -> bool {
    let needed_str = needed.as_str();
    let needed_resource = needed.resource();
    granted.split_ascii_whitespace().any(|tok| {
        if tok == "*" || tok == needed_str {
            return true;
        }
        match tok.split_once(':') {
            Some((res, "*")) => res == needed_resource,
            _ => false,
        }
    })
}

/// Render a `&[Scope]` as a single space-separated string suitable for
/// embedding in a JWT `scope` claim.
pub fn join_scopes(scopes: &[Scope]) -> String {
    let mut out = String::new();
    for (i, s) in scopes.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        out.push_str(s.as_str());
    }
    out
}
