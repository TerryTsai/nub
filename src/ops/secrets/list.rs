//! List the secret names known to nub. Values are never returned by
//! this op — the privileged `get` op (admin-only) is the only path
//! that exposes plaintext over the wire.

use std::path::Path;

use anyhow::{Context, Result};
use tokio::fs;

use crate::ops::names::valid_fs_name;
use crate::ops::time::iso8601_mtime;
use crate::proto::SecretSummary;

pub async fn run(root: &Path) -> Result<Vec<SecretSummary>> {
    let mut entries = match fs::read_dir(root).await {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e).with_context(|| format!("reading {}", root.display())),
    };
    let mut out = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let Some(name) = strip_age_suffix(&entry.file_name()) else {
            continue;
        };
        if !valid_fs_name(&name) {
            continue;
        }
        let meta = match entry.metadata().await {
            Ok(m) => m,
            Err(_) => continue,
        };
        if meta.file_type().is_symlink() || !meta.file_type().is_file() {
            continue;
        }
        out.push(SecretSummary {
            name,
            size: meta.len(),
            modified_at: iso8601_mtime(meta.modified().ok()),
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// Strip the `.age` suffix from an `OsString`, returning the base name.
fn strip_age_suffix(file: &std::ffi::OsString) -> Option<String> {
    let s = file.to_str()?;
    let stripped = s.strip_suffix(".age")?;
    Some(stripped.to_string())
}
