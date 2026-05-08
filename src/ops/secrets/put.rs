//! Encrypt and store a secret value. Loads or generates the per-host
//! age identity, encrypts, then writes the blob via the store layer.

use std::path::Path;

use anyhow::Result;

use super::{crypto, store};

pub async fn run(root: &Path, name: &str, value: &str) -> Result<()> {
    let id = crypto::load_or_generate_identity(root).await?;
    let blob = crypto::encrypt(&id, value.as_bytes())?;
    store::write_blob(root, name, &blob).await
}
