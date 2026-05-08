//! TOML config struct and loader. Schema is flat, unknown fields
//! rejected — keeps misspellings from silently disabling features.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

use super::discover::auto_paths;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub id: Option<String>,
    pub listen: Option<String>,
    pub tls_cert: Option<PathBuf>,
    pub tls_key: Option<PathBuf>,
    /// Host paths permitted as bind-mount sources in CreateContainer.
    /// Empty (default) = no host bind mounts allowed.
    #[serde(default)]
    pub allowed_binds: Vec<PathBuf>,
    /// Flat directory holding Dockerfile text files. When unset, falls
    /// back to `$XDG_DATA_HOME/nub/dockerfiles` (created on first write).
    pub dockerfiles: Option<PathBuf>,
    /// Directory holding compose-stack manifests. One subdir per stack
    /// containing `compose.yml` and optional `.env`. When unset, falls
    /// back to `$XDG_DATA_HOME/nub/stacks`.
    pub stacks: Option<PathBuf>,
    /// Directory holding age-encrypted secrets and the per-host
    /// encryption identity (`.identity`). When unset, falls back to
    /// `$XDG_DATA_HOME/nub/secrets`.
    pub secrets: Option<PathBuf>,
    /// Base64url-encoded Ed25519 public key. When set, nub validates
    /// presented JWTs against this key only — the operator mints tokens
    /// elsewhere (their laptop, latch, etc.) and nub never holds a
    /// private key. When unset, nub auto-generates and persists its
    /// own keypair at `$XDG_DATA_HOME/nub/issuer.key`.
    pub trusted_issuer: Option<String>,
}

impl Config {
    /// Load config from `explicit` if given, else auto-discover. Returns
    /// `Ok(None)` when no config exists and none was requested — caller can
    /// proceed with CLI-only configuration.
    pub fn load(explicit: Option<&Path>) -> Result<Option<Self>> {
        let path = match explicit {
            Some(p) => Some(p.to_path_buf()),
            None => auto_paths().into_iter().find(|p| p.exists()),
        };
        let Some(path) = path else { return Ok(None) };
        let s = std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        let cfg: Config = toml::from_str(&s).with_context(|| format!("parsing {}", path.display()))?;
        Ok(Some(cfg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_flat_schema() {
        let s = r#"
            id     = "host1"
            listen = "127.0.0.1:8080"
            allowed_binds = ["/data/nub"]
            dockerfiles = "/srv/nub/dockerfiles"
        "#;
        let cfg: Config = toml::from_str(s).unwrap();
        assert_eq!(cfg.id.as_deref(), Some("host1"));
        assert_eq!(cfg.listen.as_deref(), Some("127.0.0.1:8080"));
        assert_eq!(cfg.allowed_binds, vec![PathBuf::from("/data/nub")]);
        assert_eq!(cfg.dockerfiles, Some(PathBuf::from("/srv/nub/dockerfiles")));
        assert_eq!(cfg.trusted_issuer, None);
    }

    #[test]
    fn parses_trusted_issuer() {
        let s = r#"
            id = "host1"
            trusted_issuer = "abcdef"
        "#;
        let cfg: Config = toml::from_str(s).unwrap();
        assert_eq!(cfg.trusted_issuer.as_deref(), Some("abcdef"));
    }

    #[test]
    fn rejects_unknown_field() {
        let s = r#"
            id = "x"
            [[trust]]
            id = "host2"
        "#;
        let err = toml::from_str::<Config>(s).unwrap_err().to_string();
        assert!(err.contains("unknown field"), "got: {err}");
    }
}
