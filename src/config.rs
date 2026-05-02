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
    pub bind: Option<String>,
    pub tls_cert: Option<PathBuf>,
    pub tls_key: Option<PathBuf>,
    /// Host paths permitted as bind-mount sources in CreateContainer.
    /// Empty (default) = no host bind mounts allowed.
    #[serde(default)]
    pub allowed_binds: Vec<PathBuf>,
    /// Flat directory holding Dockerfile text files. When unset, falls
    /// back to `$XDG_DATA_HOME/nub/dockerfiles` (created on first write).
    pub dockerfiles: Option<PathBuf>,
    #[serde(default)]
    pub trust: Vec<TrustEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrustEntry {
    pub id: String,
    pub token: String,
    pub allowed: Vec<String>,
}

impl TrustEntry {
    pub fn allows(&self, op_name: &str) -> bool {
        self.allowed.iter().any(|s| s == "*" || s == op_name)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_flat_schema() {
        let s = r#"
            id   = "host1"
            bind = "127.0.0.1:8080"
            allowed_binds = ["/data/nub"]
            dockerfiles = "/srv/nub/dockerfiles"
        "#;
        let cfg: Config = toml::from_str(s).unwrap();
        assert_eq!(cfg.id.as_deref(), Some("host1"));
        assert_eq!(cfg.bind.as_deref(), Some("127.0.0.1:8080"));
        assert_eq!(cfg.allowed_binds, vec![PathBuf::from("/data/nub")]);
        assert_eq!(cfg.dockerfiles, Some(PathBuf::from("/srv/nub/dockerfiles")));
    }

    #[test]
    fn rejects_unknown_field() {
        // The pre-0.0.10 `[engine]` and `[dockerfiles]` sections are now
        // unknown keys; `deny_unknown_fields` makes that a load-time error
        // rather than a silently-ignored stale section.
        let s = r#"
            id = "x"
            [engine]
            allowed_binds = []
        "#;
        let err = toml::from_str::<Config>(s).unwrap_err().to_string();
        assert!(err.contains("unknown field"), "got: {err}");
    }

    #[test]
    fn trust_allows_star_and_explicit() {
        let t = TrustEntry {
            id: "phone".into(),
            token: "x".into(),
            allowed: vec!["host_info".into(), "list_containers".into()],
        };
        assert!(t.allows("host_info"));
        assert!(t.allows("list_containers"));
        assert!(!t.allows("create_container"));

        let admin = TrustEntry {
            id: "admin".into(),
            token: "x".into(),
            allowed: vec!["*".into()],
        };
        assert!(admin.allows("anything"));
    }
}
