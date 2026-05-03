//! Materialize compose `configs:` content to tmpfs at deploy time.
//! Mirror of `ops::secrets::runtime`, minus the encryption — config
//! content is plaintext, lives inline in the compose YAML, and gets
//! mounted read-only into containers at the requested target path.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::compose::{ServiceConfigRef, StackSpec};
use crate::proto::VolumeMount;

/// Root of the tmpfs we mount config plaintexts into. The container
/// create validator implicitly allows bind sources under this prefix —
/// nub controls these paths and they don't persist past reboot.
pub const TMPFS_ROOT: &str = "/run/nub/configs";

pub fn service_dir(stack: &str, service: &str) -> PathBuf {
    PathBuf::from(TMPFS_ROOT).join(stack).join(service)
}

/// True if `path` lives under `TMPFS_ROOT`. Used by the bind validator
/// to allow nub-managed mounts without growing `allowed_binds`.
pub fn is_managed_path(path: &Path) -> bool {
    path.starts_with(TMPFS_ROOT)
}

/// Write each service-referenced config to the per-service tmpfs dir
/// and return the bind mounts to inject into the container spec.
pub async fn materialize_for_service(
    stack_spec: &StackSpec,
    stack: &str,
    service: &str,
    refs: &[ServiceConfigRef],
) -> Result<Vec<VolumeMount>> {
    if refs.is_empty() {
        return Ok(Vec::new());
    }
    let dir = service_dir(stack, service);
    tokio::fs::create_dir_all(&dir)
        .await
        .with_context(|| format!("creating {}", dir.display()))?;
    set_dir_perms(&dir).await.ok();

    let mut mounts = Vec::with_capacity(refs.len());
    for r in refs {
        let content = stack_spec
            .configs
            .iter()
            .find(|c| c.name == r.source)
            .map(|c| c.content.clone())
            .ok_or_else(|| anyhow::anyhow!("config `{}` referenced by service `{service}` not declared", r.source))?;
        let host_path = dir.join(&r.source);
        write_config_file(&host_path, content.as_bytes()).await?;
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
async fn write_config_file(path: &Path, content: &[u8]) -> Result<()> {
    use std::io::Write as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }
    let path_owned = path.to_path_buf();
    let content_owned = content.to_vec();
    tokio::task::spawn_blocking(move || -> Result<()> {
        // Overwrite if present so redeploy refreshes the content.
        let _ = std::fs::remove_file(&path_owned);
        // 0444 — world-readable. Configs are non-sensitive and need to
        // be reachable from whatever UID the container runs as.
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o444)
            .open(&path_owned)
            .with_context(|| format!("creating {}", path_owned.display()))?;
        f.write_all(&content_owned)
            .with_context(|| format!("writing {}", path_owned.display()))?;
        Ok(())
    })
    .await
    .map_err(|e| anyhow::anyhow!("join: {e}"))??;
    Ok(())
}

#[cfg(not(unix))]
async fn write_config_file(path: &Path, content: &[u8]) -> Result<()> {
    tokio::fs::write(path, content).await?;
    Ok(())
}

#[cfg(unix)]
async fn set_dir_perms(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt as _;
    let mut p = tokio::fs::metadata(path).await?.permissions();
    // 0755: world-readable so any container UID can read its files.
    p.set_mode(0o755);
    tokio::fs::set_permissions(path, p).await?;
    Ok(())
}

#[cfg(not(unix))]
async fn set_dir_perms(_path: &Path) -> Result<()> {
    Ok(())
}
