//! Security policy resolved at startup. The handler reads
//! `Policy.allowed_binds` to gate `CreateContainer` mounts and
//! reads the per-resource roots (`stacks_root`, `secrets_root`,
//! `dockerfiles_root`) for the on-disk storage layers.

use std::path::PathBuf;

use crate::config;

/// Security policy applied at op boundaries. Currently constrains
/// container creation and locates the dockerfiles / stacks / secrets
/// roots.
pub struct Policy {
    /// Host paths permitted as bind-mount sources in CreateContainer.
    /// Empty = no host bind mounts allowed.
    pub allowed_binds: Vec<PathBuf>,
    /// Flat directory holding Dockerfile text files. Always set — the
    /// caller resolves it (config override or XDG default).
    pub dockerfiles_root: PathBuf,
    /// Directory holding compose-stack manifests, one subdir per stack.
    /// Always set — caller resolves config override or XDG default.
    pub stacks_root: PathBuf,
    /// Directory holding age-encrypted secrets and the per-host
    /// encryption identity. Always set — caller resolves config
    /// override or XDG default.
    pub secrets_root: PathBuf,
}

impl Policy {
    /// Resolve a `Policy` from a loaded `Config`, applying XDG defaults
    /// for any unset paths. Used by both `nub run` (server) and the
    /// in-process CLI (`nub stack`, `nub secret`) so the two share one
    /// resolution rule.
    pub fn from_config(cfg: &config::Config) -> Self {
        Self {
            allowed_binds: cfg.allowed_binds.clone(),
            dockerfiles_root: cfg.dockerfiles.clone().unwrap_or_else(config::default_dockerfiles_dir),
            stacks_root: cfg.stacks.clone().unwrap_or_else(config::default_stacks_dir),
            secrets_root: cfg.secrets.clone().unwrap_or_else(config::default_secrets_dir),
        }
    }
}
