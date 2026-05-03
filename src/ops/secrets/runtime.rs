//! Decryption-to-tmpfs runtime for compose `secrets:`. At deploy time,
//! we decrypt each referenced secret to a file under `/run/nub/secrets/`
//! and inject a read-only bind mount into the container spec. `/run`
//! is a tmpfs on every modern Linux, so plaintext never hits the disk
//! after decryption.
//!
//! Lifecycle: materialized at deploy/redeploy, removed by
//! `cleanup_service` when a stack is torn down. After a host reboot
//! the tmpfs is gone and the operator must redeploy — documented.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::compose::{ServiceSecretRef, StackSpec};
use crate::proto::VolumeMount;

use super::{crypto, store};

/// Root of the tmpfs we mount secret plaintexts into. The container
/// create validator implicitly allows bind sources under this prefix —
/// nub controls these paths and they never persist past reboot.
pub const TMPFS_ROOT: &str = "/run/nub/secrets";

/// Returns the per-service tmpfs subdirectory for a given stack and
/// service. Caller is responsible for creating it.
pub fn service_dir(stack: &str, service: &str) -> PathBuf {
    PathBuf::from(TMPFS_ROOT).join(stack).join(service)
}

/// True if `path` lives under `TMPFS_ROOT`. Used by the bind validator
/// to allow nub-managed mounts without growing `allowed_binds`.
pub fn is_managed_path(path: &Path) -> bool {
    path.starts_with(TMPFS_ROOT)
}

/// Decrypt every secret referenced by `refs` and write the plaintext
/// to the per-service tmpfs dir. Returns the list of bind mounts to
/// inject into the container spec (one per ref).
pub async fn materialize_for_service(
    secrets_root: &Path,
    stack_spec: &StackSpec,
    stack: &str,
    service: &str,
    refs: &[ServiceSecretRef],
) -> Result<Vec<VolumeMount>> {
    if refs.is_empty() {
        return Ok(Vec::new());
    }
    let dir = service_dir(stack, service);
    tokio::fs::create_dir_all(&dir)
        .await
        .with_context(|| format!("creating {}", dir.display()))?;
    set_dir_perms(&dir).await.ok();

    let identity = crypto::load_or_generate_identity(secrets_root).await?;
    let mut mounts = Vec::with_capacity(refs.len());
    for r in refs {
        let lookup = stack_spec
            .secrets
            .iter()
            .find(|s| s.name == r.source)
            .map(|s| s.lookup.clone())
            .unwrap_or_else(|| r.source.clone());
        let blob = store::read_blob(secrets_root, &lookup)
            .await
            .with_context(|| format!("reading nub secret `{lookup}` for service `{service}`"))?;
        let plain = crypto::decrypt(&identity, &blob).with_context(|| format!("decrypting secret `{lookup}`"))?;
        let host_path = dir.join(&r.source);
        write_secret_file(&host_path, &plain).await?;
        mounts.push(VolumeMount {
            source: host_path.display().to_string(),
            target: r.target.clone(),
            read_only: true,
        });
    }
    Ok(mounts)
}

/// Best-effort cleanup of a stack's whole tmpfs subtree.
pub async fn cleanup_stack(stack: &str) {
    let dir = PathBuf::from(TMPFS_ROOT).join(stack);
    let _ = tokio::fs::remove_dir_all(&dir).await;
}

#[cfg(unix)]
async fn write_secret_file(path: &Path, plain: &[u8]) -> Result<()> {
    use std::io::Write as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }
    let path_owned = path.to_path_buf();
    let plain_owned = plain.to_vec();
    tokio::task::spawn_blocking(move || -> Result<()> {
        let _ = std::fs::remove_file(&path_owned); // overwrite-safe
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o400)
            .open(&path_owned)
            .with_context(|| format!("creating {}", path_owned.display()))?;
        f.write_all(&plain_owned)
            .with_context(|| format!("writing {}", path_owned.display()))?;
        Ok(())
    })
    .await
    .map_err(|e| anyhow::anyhow!("join: {e}"))??;
    Ok(())
}

#[cfg(not(unix))]
async fn write_secret_file(path: &Path, plain: &[u8]) -> Result<()> {
    tokio::fs::write(path, plain).await?;
    Ok(())
}

#[cfg(unix)]
async fn set_dir_perms(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt as _;
    let mut p = tokio::fs::metadata(path).await?.permissions();
    p.set_mode(0o700);
    tokio::fs::set_permissions(path, p).await?;
    Ok(())
}

#[cfg(not(unix))]
async fn set_dir_perms(_path: &Path) -> Result<()> {
    Ok(())
}
