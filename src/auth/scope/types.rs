//! Scope-grammar types — the `Scope` enum (one variant per
//! network-exposed proto Op), the canonical `ALL` list, and the
//! parse/display impls.

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_scope_has_unique_string() {
        let mut seen = HashSet::new();
        for &s in Scope::ALL {
            assert!(seen.insert(s.as_str()), "duplicate scope string: {}", s.as_str());
        }
    }

    #[test]
    fn every_scope_is_resource_colon_action() {
        for &s in Scope::ALL {
            let str_form = s.as_str();
            assert!(str_form.contains(':'), "scope `{str_form}` missing `:`");
            assert!(
                !str_form.starts_with(':') && !str_form.ends_with(':'),
                "scope `{str_form}` has empty resource or action"
            );
            let r = s.resource();
            assert!(str_form.starts_with(r));
            assert_eq!(str_form.as_bytes()[r.len()], b':');
        }
    }

    #[test]
    fn from_str_roundtrip() {
        for &s in Scope::ALL {
            let parsed: Scope = s.as_str().parse().unwrap();
            assert_eq!(parsed, s);
        }
    }

    #[test]
    fn from_str_rejects_unknown() {
        assert!("nonsense".parse::<Scope>().is_err());
        assert!("containers:nope".parse::<Scope>().is_err());
        assert!("".parse::<Scope>().is_err());
    }
}
