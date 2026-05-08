//! Delete a Dockerfile entry. Idempotent — a missing name succeeds.

use anyhow::{Context, Result};
use tokio::fs;

use super::store::entry_path;
use crate::ops::EngineHandler;

pub(crate) async fn run(h: &EngineHandler, name: &str) -> Result<()> {
    let path = entry_path(&h.policy.dockerfiles_root, name)?;
    match fs::remove_file(&path).await {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e).with_context(|| format!("removing {}", path.display())),
    }
}
