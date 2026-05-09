//! Remove a secret blob. Idempotent — a missing name succeeds.

use std::path::Path;

use anyhow::{Context, Result};
use tokio::fs;

use super::store::entry_path;

pub async fn run(root: &Path, name: &str) -> Result<()> {
    let path = entry_path(root, name)?;
    match fs::remove_file(&path).await {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e).with_context(|| format!("removing {}", path.display())),
    }
}
