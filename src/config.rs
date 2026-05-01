use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub bind: String,
    pub token: String,
    pub tls_cert: Option<PathBuf>,
    pub tls_key: Option<PathBuf>,
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
        let s = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let cfg: Config = toml::from_str(&s)?;
        if cfg.token.is_empty() {
            return Err(anyhow!("token must not be empty"));
        }
        Ok(cfg)
    }
}
