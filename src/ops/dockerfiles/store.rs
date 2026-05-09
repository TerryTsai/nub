//! Filesystem layer for the Dockerfiles directory. The root is a single
//! directory configured at startup; the API never composes a path beyond
//! `<root>/<name>`. Symlinks are rejected on every entry.

use std::path::{Path, PathBuf};

use anyhow::{ensure, Result};

use crate::ops::names::valid_fs_name;

/// Hard cap on a single Dockerfile size. 256 KiB is plenty for hand-written
/// Dockerfiles and stops accidental megabyte pastes.
pub(super) const MAX_BYTES: u64 = 256 * 1024;

pub(super) fn entry_path(root: &Path, name: &str) -> Result<PathBuf> {
    ensure!(valid_fs_name(name), "invalid dockerfile name: {name:?}");
    Ok(root.join(name))
}
