//! Decrypt and return the plaintext value of one secret. Caller must
//! have already gated this on `secrets:reveal` (admin-only).

use std::path::Path;

use anyhow::Result;

use super::{crypto, store};
use crate::proto::SecretValue;

pub async fn run(root: &Path, name: &str) -> Result<SecretValue> {
    let id = crypto::load_or_generate_identity(root).await?;
    let blob = store::read_blob(root, name).await?;
    let plain = crypto::decrypt(&id, &blob)?;
    let value = String::from_utf8(plain).map_err(|_| anyhow::anyhow!("secret `{name}` is not valid UTF-8"))?;
    Ok(SecretValue {
        name: name.to_string(),
        value,
    })
}
