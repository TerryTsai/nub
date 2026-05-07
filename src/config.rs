use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

fn auto_paths() -> Vec<PathBuf> {
    vec![
        xdg_config_home().join("nub/nub.toml"),
        PathBuf::from("./nub.toml"),
        PathBuf::from("/etc/nub/config.toml"),
    ]
}

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

/// `$XDG_CONFIG_HOME` if set, else `$HOME/.config`, else `./.config`.
pub fn xdg_config_home() -> PathBuf {
    if let Some(x) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(x);
    }
    if let Some(h) = std::env::var_os("HOME") {
        return PathBuf::from(h).join(".config");
    }
    PathBuf::from(".config")
}

/// `$XDG_DATA_HOME` if set, else `$HOME/.local/share`, else `./.local/share`.
pub fn xdg_data_home() -> PathBuf {
    if let Some(x) = std::env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(x);
    }
    if let Some(h) = std::env::var_os("HOME") {
        return PathBuf::from(h).join(".local/share");
    }
    PathBuf::from(".local/share")
}

/// Default dockerfiles directory: `$XDG_DATA_HOME/nub/dockerfiles`.
pub fn default_dockerfiles_dir() -> PathBuf {
    xdg_data_home().join("nub/dockerfiles")
}

/// Default stacks directory: `$XDG_DATA_HOME/nub/stacks`. One subdir
/// per stack with the YAML manifest inside.
pub fn default_stacks_dir() -> PathBuf {
    xdg_data_home().join("nub/stacks")
}

/// Default secrets directory: `$XDG_DATA_HOME/nub/secrets`. Holds the
/// age `.identity` file plus one `<name>.age` blob per secret.
pub fn default_secrets_dir() -> PathBuf {
    xdg_data_home().join("nub/secrets")
}

/// Default issuer key path: `$XDG_DATA_HOME/nub/issuer.key`. PKCS#8
/// binary; written mode 600 by `Issuer::load_or_generate`.
pub fn default_issuer_key() -> PathBuf {
    xdg_data_home().join("nub/issuer.key")
}

/// Default admin JWT path: `$XDG_DATA_HOME/nub/admin.jwt`. Persisted
/// once at first run; re-printed on subsequent starts so paired
/// clients (browsers, scripts, anything holding the connect URL)
/// survive restart.
pub fn default_admin_jwt() -> PathBuf {
    xdg_data_home().join("nub/admin.jwt")
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
