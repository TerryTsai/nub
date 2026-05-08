//! Remove a secret blob. Idempotent — a missing name succeeds.

use std::path::Path;

use anyhow::Result;

use super::store;

pub async fn run(root: &Path, name: &str) -> Result<()> {
    store::delete(root, name).await
}
