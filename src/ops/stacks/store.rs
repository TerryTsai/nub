//! Filesystem layer for stack manifests. Each stack lives at
//! `<stacks_root>/<name>/compose.yml`. The directory is the source of
//! truth for "does this stack exist"; engine state may diverge after
//! crashes or external mutations.

use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::ops::time::iso8601_mtime;

/// Validate a user-supplied stack name. lowercase, `[a-z0-9_-]`, max 63
/// chars (Docker label limit). Reject path-traversal characters loudly
/// rather than risk writing outside `stacks_root`.
pub fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() || name.len() > 63 {
        return Err(anyhow!("stack name must be 1..=63 chars"));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
    {
        return Err(anyhow!("stack name must be lowercase alphanumeric, `-`, or `_`"));
    }
    Ok(())
}

pub fn stack_dir(root: &Path, name: &str) -> PathBuf {
    root.join(name)
}

pub fn yaml_path(root: &Path, name: &str) -> PathBuf {
    stack_dir(root, name).join("compose.yml")
}

pub fn exists(root: &Path, name: &str) -> bool {
    yaml_path(root, name).exists()
}

pub fn write_yaml(root: &Path, name: &str, yaml: &str) -> Result<()> {
    validate_name(name)?;
    let dir = stack_dir(root, name);
    fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    let path = yaml_path(root, name);
    fs::write(&path, yaml).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

pub fn read_yaml(root: &Path, name: &str) -> Result<String> {
    validate_name(name)?;
    let path = yaml_path(root, name);
    fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))
}

pub fn delete_dir(root: &Path, name: &str) -> Result<()> {
    validate_name(name)?;
    let dir = stack_dir(root, name);
    if !dir.exists() {
        return Ok(());
    }
    fs::remove_dir_all(&dir).with_context(|| format!("removing {}", dir.display()))?;
    Ok(())
}

/// Modified-at as ISO 8601 UTC, or empty if the FS doesn't expose mtime.
pub fn modified_at(root: &Path, name: &str) -> String {
    let path = yaml_path(root, name);
    let meta = match fs::metadata(&path) {
        Ok(m) => m,
        Err(_) => return String::new(),
    };
    iso8601_mtime(meta.modified().ok())
}

/// List stack names — directories under `root` that contain a `compose.yml`.
/// Returns alphabetically sorted.
pub fn list_names(root: &Path) -> Result<Vec<String>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in fs::read_dir(root).with_context(|| format!("reading {}", root.display()))? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = match entry.file_name().into_string() {
            Ok(s) => s,
            Err(_) => continue,
        };
        if validate_name(&name).is_err() {
            continue;
        }
        if !yaml_path(root, &name).exists() {
            continue;
        }
        names.push(name);
    }
    names.sort();
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_bad_names() {
        assert!(validate_name("").is_err());
        assert!(validate_name("UPPER").is_err());
        assert!(validate_name("with space").is_err());
        assert!(validate_name("../escape").is_err());
        assert!(validate_name(&"x".repeat(64)).is_err());
    }

    #[test]
    fn accepts_good_names() {
        assert!(validate_name("foo").is_ok());
        assert!(validate_name("foo-bar_2").is_ok());
        assert!(validate_name("a").is_ok());
    }
}
