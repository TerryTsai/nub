//! Decryption-to-tmpfs runtime for compose `secrets:`. At deploy time
//! we decrypt each referenced secret to a file under
//! `/tmp/nub-<USER>/secrets/` and inject a read-only bind mount into
//! the container spec.
//!
//! Path posture (tmpfs choice, file modes, threat model) lives on the
//! shared helpers in `ops::tmpfs`. See docs/security.md.
//!
//! Lifecycle: materialized at deploy/redeploy, cleaned up by
//! `cleanup_stack` when a stack is torn down, re-materialized on
//! daemon startup so containers with `restart: always` come back
//! cleanly across reboots.

use std::path::Path;

use anyhow::{Context, Result};

use crate::compose::{ServiceSecretRef, StackSpec};
use crate::ops::tmpfs;
use crate::proto::VolumeMount;

use super::{crypto, store};

const KIND: &str = "secrets";

/// True if `path` lives under the secrets tmpfs root. Used by the bind
/// validator to allow nub-managed mounts without growing `allowed_binds`.
pub fn is_managed_path(path: &Path) -> bool {
    path.starts_with(tmpfs::tmpfs_root(KIND))
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
    let dir = tmpfs::service_dir(KIND, stack, service);
    tokio::fs::create_dir_all(&dir).await.with_context(|| format!("creating {}", dir.display()))?;
    tmpfs::ensure_dir_traversable(&dir).await.ok();

    let identity = crypto::load_or_generate_identity(secrets_root).await?;
    let mut mounts = Vec::with_capacity(refs.len());
    for r in refs {
        let lookup = stack_spec
            .secrets
            .iter()
            .find(|s| s.name == r.source)
            .map_or_else(|| r.source.clone(), |s| s.lookup.clone());
        let blob = store::read_blob(secrets_root, &lookup)
            .await
            .with_context(|| format!("reading nub secret `{lookup}` for service `{service}`"))?;
        let plain = crypto::decrypt(&identity, &blob).with_context(|| format!("decrypting secret `{lookup}`"))?;
        let host_path = dir.join(&r.source);
        tmpfs::write_world_readable(&host_path, &plain).await?;
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
    tmpfs::cleanup_stack_dir(KIND, stack).await;
}
