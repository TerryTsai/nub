//! Atomic write of a Dockerfile. tmp file + rename so the entry is
//! never half-written. Refuses to overwrite a symlink target.

use anyhow::{anyhow, bail, Context, Result};
use tokio::fs;

use super::store::{entry_path, MAX_BYTES};
use crate::ops::EngineHandler;

pub(crate) async fn run(h: &EngineHandler, name: &str, content: &str) -> Result<()> {
    if content.len() as u64 > MAX_BYTES {
        bail!("dockerfile content exceeds {} KiB cap", MAX_BYTES / 1024);
    }
    let path = entry_path(&h.policy.dockerfiles_root, name)?;
    // Reject overwriting through a symlink: a previous attacker could have
    // dropped one in. `entry_path` already validated the name, but the
    // file at `<root>/<name>` itself might be a symlink from a manual
    // intervention. lstat returns the link's metadata; check before rename.
    if let Ok(meta) = fs::symlink_metadata(&path).await {
        if meta.file_type().is_symlink() {
            bail!("refusing to overwrite a symlink at {}", path.display());
        }
    }
    let dir = path.parent().ok_or_else(|| anyhow!("invalid dockerfile path"))?;
    fs::create_dir_all(dir).await.ok();
    let tmp = dir.join(format!(".{name}.tmp"));
    fs::write(&tmp, content).await.with_context(|| format!("writing {}", tmp.display()))?;
    fs::rename(&tmp, &path).await.with_context(|| format!("renaming {} -> {}", tmp.display(), path.display()))?;
    Ok(())
}
