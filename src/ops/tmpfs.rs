//! Deploy-time tmpfs primitives shared by `ops::secrets::runtime` and
//! `ops::configs::runtime`. nub materializes secrets and configs to a
//! per-host, per-user tmpfs directory and bind-mounts them read-only
//! into containers.

use std::path::{Path, PathBuf};

/// Root of the per-host tmpfs namespace nub uses for materializing
/// secrets and configs at deploy time. `<USER>` lets multiple nub
/// instances on the same host coexist. `kind` (`"secrets"` /
/// `"configs"`) gives each resource family its own subtree so the
/// per-runtime `is_managed_path` checks can distinguish them.
///
/// `/tmp` (rather than `$XDG_RUNTIME_DIR` or `/run`) is the only
/// location that's both writable for user-systemd nub and traversable
/// by container UIDs after rootless-podman userns mapping. Files end
/// up mode 0444 and parent dirs 0755 — same posture as docker compose
/// secrets, which is what operators expect. See docs/security.md.
pub(super) fn tmpfs_root(kind: &str) -> PathBuf {
    let user = std::env::var("USER").unwrap_or_else(|_| "user".into());
    PathBuf::from(format!("/tmp/nub-{user}/{kind}"))
}

/// Per-service tmpfs subdirectory for a stack/service.
pub(super) fn service_dir(kind: &str, stack: &str, service: &str) -> PathBuf {
    tmpfs_root(kind).join(stack).join(service)
}

/// Best-effort cleanup of one stack's tmpfs subtree.
pub(super) async fn cleanup_stack_dir(kind: &str, stack: &str) {
    let dir = tmpfs_root(kind).join(stack);
    let _ = tokio::fs::remove_dir_all(&dir).await;
}

/// Write `content` to `path` as a 0444, world-readable file. Overwrites
/// any prior entry so redeploy refreshes content. World-readable so
/// rootless containers (mapped sub-UIDs) can read regardless of the
/// in-container user. Files are read-only by design — bind-mounted
/// into the container, never written from inside.
#[cfg(unix)]
pub(super) async fn write_world_readable(path: &Path, content: &[u8]) -> anyhow::Result<()> {
    use anyhow::Context as _;
    use std::io::Write as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }
    let path_owned = path.to_path_buf();
    let content_owned = content.to_vec();
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let _ = std::fs::remove_file(&path_owned);
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
pub(super) async fn write_world_readable(path: &Path, content: &[u8]) -> anyhow::Result<()> {
    tokio::fs::write(path, content).await?;
    Ok(())
}

/// Make `path` traversable (0755) so any container UID can reach the
/// materialized files inside.
#[cfg(unix)]
pub(super) async fn ensure_dir_traversable(path: &Path) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt as _;
    let mut p = tokio::fs::metadata(path).await?.permissions();
    p.set_mode(0o755);
    tokio::fs::set_permissions(path, p).await?;
    Ok(())
}

#[cfg(not(unix))]
pub(super) async fn ensure_dir_traversable(_path: &Path) -> anyhow::Result<()> {
    Ok(())
}
