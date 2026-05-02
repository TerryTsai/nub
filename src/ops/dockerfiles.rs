//! CRUD on a flat directory of Dockerfile text files. Filenames are
//! whitelisted, symlinks are rejected (lstat precheck), writes are atomic
//! (tmp + rename). The root is a single directory configured at startup;
//! the API never composes a path beyond `<root>/<name>`.

use std::path::PathBuf;
use std::time::SystemTime;

use anyhow::{anyhow, bail, Context, Result};
use tokio::fs;
use tokio::io::AsyncReadExt as _;

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
        if !valid_name(&name) {
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
            modified_at: format_mtime(meta.modified().ok()),
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
        modified_at: format_mtime(lmeta.modified().ok()),
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
    if !valid_name(name) {
        bail!("invalid dockerfile name: {name:?}");
    }
    Ok(h.policy.dockerfiles_root.join(name))
}

/// Allow a deliberately small character set: letters, digits, `.`, `_`, `-`.
/// Bans path separators, parent-dir traversal, leading dot/hyphen, and any
/// embedded NULs. Length capped to keep filesystems happy.
pub(super) fn valid_name(name: &str) -> bool {
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
    let secs = dur.as_secs() as i64;
    // RFC 3339 in UTC. No chrono dep — build it by hand.
    iso8601_utc(secs)
}

fn iso8601_utc(unix_secs: i64) -> String {
    // Civil-from-days from Howard Hinnant's algorithm. Good for any year
    // we care about; ample range for filesystem mtimes.
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
    fn valid_name_accepts_simple() {
        assert!(valid_name("nginx"));
        assert!(valid_name("nginx.Dockerfile"));
        assert!(valid_name("foo_bar-1"));
    }

    #[test]
    fn valid_name_rejects_traversal_and_specials() {
        assert!(!valid_name(""));
        assert!(!valid_name("."));
        assert!(!valid_name(".."));
        assert!(!valid_name(".hidden"));
        assert!(!valid_name("-leading"));
        assert!(!valid_name("a/b"));
        assert!(!valid_name("a\0b"));
        assert!(!valid_name("x y"));
        assert!(!valid_name(&"a".repeat(129)));
    }

    #[test]
    fn valid_name_accepts_leading_digit_and_max_length() {
        assert!(valid_name("1nginx"));
        assert!(valid_name("9up"));
        // Exact boundary at 128 chars.
        assert!(valid_name(&"a".repeat(128)));
    }

    #[test]
    fn valid_name_rejects_unicode() {
        assert!(!valid_name("café"));
        assert!(!valid_name("nginx™"));
    }

    #[test]
    fn iso8601_known_values() {
        assert_eq!(iso8601_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso8601_utc(1_704_067_200), "2024-01-01T00:00:00Z");
    }
}
