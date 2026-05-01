use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub bind: Option<String>,
    pub token: Option<String>,
    pub tls_cert: Option<PathBuf>,
    pub tls_key: Option<PathBuf>,
    #[serde(default)]
    pub allowed_binds: Vec<PathBuf>,
    pub hub: Option<crate::hub_client::Config>,
}

impl Config {
    pub fn load(explicit: Option<&Path>) -> Result<Self> {
        let path = explicit
            .map(|p| p.to_path_buf())
            .or_else(|| {
                ["./nub.toml", "/etc/nub/config.toml"]
                    .into_iter()
                    .map(PathBuf::from)
                    .find(|p| p.exists())
            })
            .ok_or_else(|| anyhow!("no config at ./nub.toml or /etc/nub/config.toml; pass --config"))?;
        let s = std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        let cfg: Config = toml::from_str(&s)?;
        if cfg.bind.is_some() && cfg.token.as_deref().unwrap_or("").is_empty() {
            return Err(anyhow!("`token` required when `bind` is set"));
        }
        if cfg.bind.is_none() && cfg.hub.is_none() {
            return Err(anyhow!("config must set `bind` (standalone), `hub` (fleet), or both"));
        }
        Ok(cfg)
    }
}
