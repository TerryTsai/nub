use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub id: Option<String>,
    pub bind: Option<String>,
    pub tls_cert: Option<PathBuf>,
    pub tls_key: Option<PathBuf>,
    #[serde(default)]
    pub engine: Engine,
    #[serde(default)]
    pub trust: Vec<TrustEntry>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Engine {
    #[serde(default)]
    pub allowed_binds: Vec<PathBuf>,
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
            None => ["./nub.toml", "/etc/nub/config.toml"]
                .into_iter()
                .map(PathBuf::from)
                .find(|p| p.exists()),
        };
        let Some(path) = path else { return Ok(None) };
        let s = std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        let cfg: Config = toml::from_str(&s).with_context(|| format!("parsing {}", path.display()))?;
        Ok(Some(cfg))
    }
}
