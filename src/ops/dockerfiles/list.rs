//! List the Dockerfiles known to nub. Symlinks and non-files are
//! filtered out — only regular files with whitelisted names appear.

use anyhow::{Context, Result};
use tokio::fs;

use crate::ops::names::valid_fs_name;
use crate::ops::time::iso8601_mtime;
use crate::ops::EngineHandler;
use crate::proto::DockerfileSummary;

pub(crate) async fn run(h: &EngineHandler) -> Result<Vec<DockerfileSummary>> {
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
