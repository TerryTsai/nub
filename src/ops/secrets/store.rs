//! Filesystem layer for the secrets directory. One flat dir, one
//! `<name>.age` file per secret, plus a hidden `.identity` holding the
//! per-host X25519 key. Atomic writes via `.tmp` + rename. Symlinks
//! rejected on read and write.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{anyhow, bail, Context, Result};
use tokio::fs;

use crate::proto::SecretSummary;

/// Hard cap on a single secret's encrypted blob. 64 KiB covers any
/// realistic credential and stops accidental file-uploads.
const MAX_BYTES: u64 = 64 * 1024;

/// On-disk filename for a named secret.
pub fn entry_path(root: &Path, name: &str) -> Result<PathBuf> {
    if !valid_name(name) {
        bail!("invalid secret name: {name:?}");
    }
    Ok(root.join(format!("{name}.age")))
}

/// Path to the per-host age identity file. Hidden so listings skip it
/// (the name validator rejects leading `.`).
pub fn identity_path(root: &Path) -> PathBuf {
    root.join(".identity")
}

pub async fn list(root: &Path) -> Result<Vec<SecretSummary>> {
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
        if !valid_name(&name) {
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
            modified_at: format_mtime(meta.modified().ok()),
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

pub async fn read_blob(root: &Path, name: &str) -> Result<Vec<u8>> {
    let path = entry_path(root, name)?;
    let lmeta = fs::symlink_metadata(&path)
        .await
        .with_context(|| format!("stat {}", path.display()))?;
    if lmeta.file_type().is_symlink() {
        bail!("refusing to read a symlink at {}", path.display());
    }
    if lmeta.len() > MAX_BYTES {
        bail!("secret {name} is larger than {} KiB", MAX_BYTES / 1024);
    }
    fs::read(&path)
        .await
        .with_context(|| format!("reading {}", path.display()))
}

pub async fn write_blob(root: &Path, name: &str, blob: &[u8]) -> Result<()> {
    if blob.len() as u64 > MAX_BYTES {
        bail!("secret blob exceeds {} KiB cap", MAX_BYTES / 1024);
    }
    let path = entry_path(root, name)?;
    if let Ok(meta) = fs::symlink_metadata(&path).await {
        if meta.file_type().is_symlink() {
            bail!("refusing to overwrite a symlink at {}", path.display());
        }
    }
    let dir = path.parent().ok_or_else(|| anyhow!("invalid secret path"))?;
    fs::create_dir_all(dir).await.ok();
    let tmp = dir.join(format!(".{name}.age.tmp"));
    fs::write(&tmp, blob)
        .await
        .with_context(|| format!("writing {}", tmp.display()))?;
    set_perms_0600(&tmp).await.ok();
    fs::rename(&tmp, &path)
        .await
        .with_context(|| format!("renaming {} -> {}", tmp.display(), path.display()))?;
    Ok(())
}

pub async fn delete(root: &Path, name: &str) -> Result<()> {
    let path = entry_path(root, name)?;
    match fs::remove_file(&path).await {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e).with_context(|| format!("removing {}", path.display())),
    }
}

#[cfg(unix)]
async fn set_perms_0600(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt as _;
    let mut p = fs::metadata(path).await?.permissions();
    p.set_mode(0o600);
    fs::set_permissions(path, p).await?;
    Ok(())
}

#[cfg(not(unix))]
async fn set_perms_0600(_path: &Path) -> Result<()> {
    Ok(())
}

/// Strip the `.age` suffix from an `OsString`, returning the base name.
fn strip_age_suffix(file: &std::ffi::OsString) -> Option<String> {
    let s = file.to_str()?;
    let stripped = s.strip_suffix(".age")?;
    Some(stripped.to_string())
}

/// Same character set as Dockerfiles (no spaces, no path separators).
pub fn valid_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 128 {
        return false;
    }
    if name.starts_with('.') || name.starts_with('-') {
        return false;
    }
    if name == "." || name == ".." {
        return false;
    }
    name.as_bytes()
        .iter()
        .all(|&b| b.is_ascii_alphanumeric() || b == b'.' || b == b'_' || b == b'-')
}

fn format_mtime(t: Option<SystemTime>) -> String {
    let Some(t) = t else { return String::new() };
    let dur = match t.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => d,
        Err(_) => return String::new(),
    };
    iso8601_utc(dur.as_secs() as i64)
}

fn iso8601_utc(unix_secs: i64) -> String {
    let days = unix_secs.div_euclid(86_400);
    let secs_in_day = unix_secs.rem_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    let hh = (secs_in_day / 3600) as u32;
    let mm = ((secs_in_day / 60) % 60) as u32;
    let ss = (secs_in_day % 60) as u32;
    format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_name_basics() {
        assert!(valid_name("db_password"));
        assert!(valid_name("API_KEY"));
        assert!(valid_name("a.b-c_1"));
        assert!(!valid_name(""));
        assert!(!valid_name(".identity"));
        assert!(!valid_name("-leading"));
        assert!(!valid_name("a/b"));
        assert!(!valid_name("a b"));
        assert!(!valid_name(&"x".repeat(129)));
    }
}
