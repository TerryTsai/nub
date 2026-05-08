//! Read a Dockerfile's content. Refuses symlinks and oversize files.

use anyhow::{bail, Context, Result};
use tokio::fs;
use tokio::io::AsyncReadExt as _;

use super::store::{entry_path, MAX_BYTES};
use crate::ops::time::iso8601_mtime;
use crate::ops::EngineHandler;
use crate::proto::DockerfileContent;

pub(crate) async fn run(h: &EngineHandler, name: &str) -> Result<DockerfileContent> {
    let path = entry_path(&h.policy.dockerfiles_root, name)?;
    let lmeta = fs::symlink_metadata(&path)
        .await
        .with_context(|| format!("stat {}", path.display()))?;
    if lmeta.file_type().is_symlink() {
        bail!("refusing to read a symlink at {}", path.display());
    }
    if lmeta.len() > MAX_BYTES {
        bail!("dockerfile {name} is larger than {} KiB", MAX_BYTES / 1024);
    }
    let mut f = fs::File::open(&path)
        .await
        .with_context(|| format!("opening {}", path.display()))?;
    let mut content = String::with_capacity(lmeta.len() as usize);
    f.read_to_string(&mut content)
        .await
        .with_context(|| format!("reading {}", path.display()))?;
    Ok(DockerfileContent {
        name: name.to_string(),
        content,
        size: lmeta.len(),
        modified_at: iso8601_mtime(lmeta.modified().ok()),
    })
}
