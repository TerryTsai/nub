//! Materialize compose `configs:` content to tmpfs at deploy time.
//! Mirror of `ops::secrets::runtime` with no encryption — config
//! content is plaintext, lives inline in the compose YAML, and gets
//! mounted read-only into containers at the requested target path.
//!
//! Path posture (tmpfs choice, file modes) lives on the shared helpers
//! in `ops::util`.

use std::path::Path;

use anyhow::{Context, Result};

use crate::compose::{ServiceConfigRef, StackSpec};
use crate::ops::util;
use crate::proto::VolumeMount;

const KIND: &str = "configs";

/// True if `path` lives under the configs tmpfs root. Used by the bind
/// validator to allow nub-managed mounts without growing `allowed_binds`.
pub fn is_managed_path(path: &Path) -> bool {
    path.starts_with(util::tmpfs_root(KIND))
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
    let dir = util::service_dir(KIND, stack, service);
    tokio::fs::create_dir_all(&dir)
        .await
        .with_context(|| format!("creating {}", dir.display()))?;
    util::ensure_dir_traversable(&dir).await.ok();

    let mut mounts = Vec::with_capacity(refs.len());
    for r in refs {
        let content = stack_spec
            .configs
            .iter()
            .find(|c| c.name == r.source)
            .map(|c| c.content.clone())
            .ok_or_else(|| anyhow::anyhow!("config `{}` referenced by service `{service}` not declared", r.source))?;
        let host_path = dir.join(&r.source);
        util::write_world_readable(&host_path, content.as_bytes()).await?;
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
    util::cleanup_stack_dir(KIND, stack).await;
}
