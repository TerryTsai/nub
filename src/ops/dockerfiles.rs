//! CRUD on a flat directory of Dockerfile text files. Filenames are
//! whitelisted, symlinks are rejected (lstat precheck), writes are atomic
//! (tmp + rename). The root is a single directory configured at startup;
//! the API never composes a path beyond `<root>/<name>`.

use std::path::PathBuf;

use anyhow::{anyhow, bail, Context, Result};
use tokio::fs;
use tokio::io::AsyncReadExt as _;

use crate::ops::util::{iso8601_mtime, valid_fs_name};
use crate::ops::EngineHandler;
use crate::proto::{DockerfileContent, DockerfileSummary};

/// Hard cap on a single Dockerfile size. 256 KiB is plenty for hand-written
/// Dockerfiles and stops accidental megabyte pastes.
const MAX_BYTES: u64 = 256 * 1024;

pub(super) async fn list(h: &EngineHandler) -> Result<Vec<DockerfileSummary>> {
    let root = &h.policy.dockerfiles_root;
    let mut entries = match fs::read_dir(root).await {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e).with_context(|| format!("reading {}", root.display())),
    };
    let mut out = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let name = match entry.file_name().into_string() {
            Ok(n) => n,
            Err(_) => continue, // non-UTF-8 names are silently skipped
        };
        if !valid_fs_name(&name) {
            continue;
        }
        // Reject symlinks even in listings — they shouldn't be plantable
        // entry points to files outside the configured root.
        let meta = match entry.metadata().await {
            Ok(m) => m,
            Err(_) => continue,
        };
        if meta.file_type().is_symlink() || !meta.file_type().is_file() {
            continue;
        }
        out.push(DockerfileSummary {
            name,
            size: meta.len(),
            modified_at: iso8601_mtime(meta.modified().ok()),
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

pub(super) async fn read(h: &EngineHandler, name: &str) -> Result<DockerfileContent> {
    let path = entry_path(h, name)?;
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

pub(super) async fn write(h: &EngineHandler, name: &str, content: &str) -> Result<()> {
    if content.len() as u64 > MAX_BYTES {
        bail!("dockerfile content exceeds {} KiB cap", MAX_BYTES / 1024);
    }
    let path = entry_path(h, name)?;
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
    fs::write(&tmp, content)
        .await
        .with_context(|| format!("writing {}", tmp.display()))?;
    fs::rename(&tmp, &path)
        .await
        .with_context(|| format!("renaming {} -> {}", tmp.display(), path.display()))?;
    Ok(())
}

pub(super) async fn delete(h: &EngineHandler, name: &str) -> Result<()> {
    let path = entry_path(h, name)?;
    match fs::remove_file(&path).await {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e).with_context(|| format!("removing {}", path.display())),
    }
}

fn entry_path(h: &EngineHandler, name: &str) -> Result<PathBuf> {
    if !valid_fs_name(name) {
        bail!("invalid dockerfile name: {name:?}");
    }
    Ok(h.policy.dockerfiles_root.join(name))
}
