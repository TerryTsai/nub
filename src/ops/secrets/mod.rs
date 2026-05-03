//! Secrets — age-encrypted blobs in a flat directory, one file per
//! secret, plus a hidden per-host X25519 identity. The op layer is
//! resource-shaped: list, put, delete, get. CLI/HTTP both call into
//! these helpers; auth (scope check) happens above us.

mod crypto;
pub mod runtime;
mod store;

use std::path::Path;

use anyhow::Result;

use crate::proto::{SecretSummary, SecretValue};

pub async fn list(root: &Path) -> Result<Vec<SecretSummary>> {
    store::list(root).await
}

pub async fn put(root: &Path, name: &str, value: &str) -> Result<()> {
    let id = crypto::load_or_generate_identity(root).await?;
    let blob = crypto::encrypt(&id, value.as_bytes())?;
    store::write_blob(root, name, &blob).await
}

pub async fn delete(root: &Path, name: &str) -> Result<()> {
    store::delete(root, name).await
}

/// Decrypt and return the plaintext. Caller must have already gated
/// this on `secrets:reveal` (admin-only).
pub async fn get(root: &Path, name: &str) -> Result<SecretValue> {
    let id = crypto::load_or_generate_identity(root).await?;
    let blob = store::read_blob(root, name).await?;
    let plain = crypto::decrypt(&id, &blob)?;
    let value = String::from_utf8(plain).map_err(|_| anyhow::anyhow!("secret `{name}` is not valid UTF-8"))?;
    Ok(SecretValue {
        name: name.to_string(),
        value,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn tmp() -> TempDir {
        TempDir::new().expect("tempdir")
    }

    #[tokio::test]
    async fn put_list_get_roundtrip() {
        let dir = tmp();
        put(dir.path(), "db_password", "hunter2").await.unwrap();
        let listing = list(dir.path()).await.unwrap();
        assert_eq!(listing.len(), 1);
        assert_eq!(listing[0].name, "db_password");
        assert!(listing[0].size > 0);

        let got = get(dir.path(), "db_password").await.unwrap();
        assert_eq!(got.name, "db_password");
        assert_eq!(got.value, "hunter2");
    }

    #[tokio::test]
    async fn put_overwrites() {
        let dir = tmp();
        put(dir.path(), "k", "first").await.unwrap();
        put(dir.path(), "k", "second").await.unwrap();
        let got = get(dir.path(), "k").await.unwrap();
        assert_eq!(got.value, "second");
    }

    #[tokio::test]
    async fn delete_removes() {
        let dir = tmp();
        put(dir.path(), "ephem", "x").await.unwrap();
        delete(dir.path(), "ephem").await.unwrap();
        let listing = list(dir.path()).await.unwrap();
        assert!(listing.is_empty());
    }

    #[tokio::test]
    async fn delete_missing_is_ok() {
        let dir = tmp();
        delete(dir.path(), "never_existed").await.unwrap();
    }

    #[tokio::test]
    async fn list_skips_identity_file() {
        let dir = tmp();
        put(dir.path(), "a", "v").await.unwrap();
        // .identity should now exist but never appear in listings.
        assert!(dir.path().join(".identity").exists());
        let listing = list(dir.path()).await.unwrap();
        assert_eq!(listing.len(), 1);
        assert_eq!(listing[0].name, "a");
    }

    #[tokio::test]
    async fn rejects_bad_names() {
        let dir = tmp();
        assert!(put(dir.path(), "../etc", "x").await.is_err());
        assert!(put(dir.path(), ".hidden", "x").await.is_err());
        assert!(put(dir.path(), "with space", "x").await.is_err());
        assert!(put(dir.path(), "", "x").await.is_err());
    }
}
