//! Filesystem layer for the secrets directory. One flat dir, one
//! `<name>.age` file per secret, plus a hidden `.identity` holding the
//! per-host X25519 key. Atomic writes via `.tmp` + rename. Symlinks
//! rejected on read and write.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use tokio::fs;

use crate::ops::names::valid_fs_name;

/// Hard cap on a single secret's encrypted blob. 64 KiB covers any
/// realistic credential and stops accidental file-uploads.
const MAX_BYTES: u64 = 64 * 1024;

/// On-disk filename for a named secret.
pub fn entry_path(root: &Path, name: &str) -> Result<PathBuf> {
    if !valid_fs_name(name) {
        bail!("invalid secret name: {name:?}");
    }
    Ok(root.join(format!("{name}.age")))
}

/// Path to the per-host age identity file. Hidden so listings skip it
/// (the name validator rejects leading `.`).
pub fn identity_path(root: &Path) -> PathBuf {
    root.join(".identity")
}

pub async fn read_blob(root: &Path, name: &str) -> Result<Vec<u8>> {
    let path = entry_path(root, name)?;
    let lmeta = fs::symlink_metadata(&path)
        .await
        .with_context(|| format!("stat {}", path.display()))?;
    if lmeta.file_type().is_symlink() {
        bail!("refusing to read a symlink at {}", path.display());
    }
    if lmeta.len() > MAX_BYTES {
        bail!("secret {name} is larger than {} KiB", MAX_BYTES / 1024);
    }
    fs::read(&path).await.with_context(|| format!("reading {}", path.display()))
}

pub async fn write_blob(root: &Path, name: &str, blob: &[u8]) -> Result<()> {
    if blob.len() as u64 > MAX_BYTES {
        bail!("secret blob exceeds {} KiB cap", MAX_BYTES / 1024);
    }
    let path = entry_path(root, name)?;
    if let Ok(meta) = fs::symlink_metadata(&path).await {
        if meta.file_type().is_symlink() {
            bail!("refusing to overwrite a symlink at {}", path.display());
        }
    }
    let dir = path.parent().ok_or_else(|| anyhow!("invalid secret path"))?;
    fs::create_dir_all(dir).await.ok();
    let tmp = dir.join(format!(".{name}.age.tmp"));
    fs::write(&tmp, blob).await.with_context(|| format!("writing {}", tmp.display()))?;
    set_perms_0600(&tmp).await.ok();
    fs::rename(&tmp, &path)
        .await
        .with_context(|| format!("renaming {} -> {}", tmp.display(), path.display()))?;
    Ok(())
}

#[cfg(unix)]
async fn set_perms_0600(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt as _;
    let mut p = fs::metadata(path).await?.permissions();
    p.set_mode(0o600);
    fs::set_permissions(path, p).await?;
    Ok(())
}

#[cfg(not(unix))]
async fn set_perms_0600(_path: &Path) -> Result<()> {
    Ok(())
}
