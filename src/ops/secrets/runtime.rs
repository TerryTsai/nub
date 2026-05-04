//! Decryption-to-tmpfs runtime for compose `secrets:`. At deploy time
//! we decrypt each referenced secret to a file under
//! `/tmp/nub-<USER>/secrets/` and inject a read-only bind mount into
//! the container spec.
//!
//! Path choice: `/tmp` (rather than `$XDG_RUNTIME_DIR` or `/run`) is
//! the only location that's both writable for user-systemd nub and
//! traversable by container UIDs after rootless-podman userns
//! mapping. Files are mode 0444 and the parent dirs are 0755 — same
//! posture as docker compose secrets, which is what operators expect.
//! This means any local host user can read materialized plaintext
//! while a stack is up; documented in docs/security.md.
//!
//! Lifecycle: materialized at deploy/redeploy, cleaned up by
//! `cleanup_stack` when a stack is torn down, re-materialized on
//! daemon startup so containers with `restart: always` come back
//! cleanly across reboots.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::compose::{ServiceSecretRef, StackSpec};
use crate::proto::VolumeMount;

use super::{crypto, store};

/// Root of the tmpfs we mount secret plaintexts into. The container
/// create validator implicitly allows bind sources under this prefix —
/// nub controls these paths and clears them on stack delete.
///
/// `/tmp/nub-<USER>/secrets`. The `<USER>` namespace lets multiple
/// nub instances on the same host coexist without colliding. `/tmp` is
/// tmpfs on systemd-default Linux distros (Fedora, Arch, etc.); on
/// Debian/Ubuntu it's typically on disk — the cleanup-on-delete +
/// rehydrate-on-boot behavior keeps stale plaintext bounded either way.
pub fn tmpfs_root() -> PathBuf {
    let user = std::env::var("USER").unwrap_or_else(|_| "user".into());
    PathBuf::from(format!("/tmp/nub-{user}/secrets"))
}

/// Returns the per-service tmpfs subdirectory for a given stack and
/// service. Caller is responsible for creating it.
pub fn service_dir(stack: &str, service: &str) -> PathBuf {
    tmpfs_root().join(stack).join(service)
}

/// True if `path` lives under the tmpfs root. Used by the bind
/// validator to allow nub-managed mounts without growing `allowed_binds`.
pub fn is_managed_path(path: &Path) -> bool {
    path.starts_with(tmpfs_root())
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
    let dir = tmpfs_root().join(stack);
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
        // Overwrite if present so redeploy refreshes the content.
        let _ = std::fs::remove_file(&path_owned);
        // 0444 — world-readable. Matches docker compose's default
        // secret file mode and lets rootless containers (which run as
        // mapped sub-UIDs) read the file regardless of in-container
        // user. See docs/security.md for the threat model.
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o444)
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
    // 0755 — traversable by any UID so rootless containers (with
    // mapped sub-UIDs) can reach the secret files inside. The files
    // themselves are 0444; only nub (writer) and any container that
    // bind-mounts them get to see content.
    p.set_mode(0o755);
    tokio::fs::set_permissions(path, p).await?;
    Ok(())
}

#[cfg(not(unix))]
async fn set_dir_perms(_path: &Path) -> Result<()> {
    Ok(())
}
