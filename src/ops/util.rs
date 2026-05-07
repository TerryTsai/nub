//! Cross-resource ops helpers: serde adapters, ISO 8601 formatting,
//! the shared FS-name validator, and the deploy-time tmpfs primitives
//! used by both `ops::secrets::runtime` and `ops::configs::runtime`.
//! Stack names use a stricter validator (lowercase + 63-char cap) —
//! that one stays in `ops::stacks::store`.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::Deserialize;

/// Deserialize, treating JSON `null` as the type's `Default`. Pair with
/// `#[serde(default)]` so missing fields also default. Use on `HashMap`
/// or `Vec` fields where the engine may emit `null`.
pub(super) fn null_to_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

/// FS-name validator shared by Dockerfile and secret names. Letters,
/// digits, `.`, `_`, `-`. Bans path separators, traversal, leading
/// dot/hyphen, embedded NUL. Capped at 128 chars.
pub(super) fn valid_fs_name(name: &str) -> bool {
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

/// RFC 3339 / ISO 8601 in UTC. Howard Hinnant's civil-from-days; ample
/// range for filesystem mtimes; no chrono dep.
pub(super) fn iso8601_utc(unix_secs: i64) -> String {
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

/// SystemTime → ISO 8601 string, or empty when the FS doesn't expose mtime.
pub(super) fn iso8601_mtime(t: Option<SystemTime>) -> String {
    let Some(t) = t else { return String::new() };
    let Ok(dur) = t.duration_since(SystemTime::UNIX_EPOCH) else {
        return String::new();
    };
    iso8601_utc(dur.as_secs() as i64)
}

/// Root of the per-host tmpfs namespace nub uses for materializing
/// secrets and configs at deploy time. `<USER>` lets multiple nub
/// instances on the same host coexist. `kind` (`"secrets"` /
/// `"configs"`) gives each resource family its own subtree so the
/// per-runtime `is_managed_path` checks can distinguish them.
///
/// `/tmp` (rather than `$XDG_RUNTIME_DIR` or `/run`) is the only
/// location that's both writable for user-systemd nub and traversable
/// by container UIDs after rootless-podman userns mapping. Files end
/// up mode 0444 and parent dirs 0755 — same posture as docker compose
/// secrets, which is what operators expect. See docs/security.md.
pub(super) fn tmpfs_root(kind: &str) -> PathBuf {
    let user = std::env::var("USER").unwrap_or_else(|_| "user".into());
    PathBuf::from(format!("/tmp/nub-{user}/{kind}"))
}

/// Per-service tmpfs subdirectory for a stack/service.
pub(super) fn service_dir(kind: &str, stack: &str, service: &str) -> PathBuf {
    tmpfs_root(kind).join(stack).join(service)
}

/// Best-effort cleanup of one stack's tmpfs subtree.
pub(super) async fn cleanup_stack_dir(kind: &str, stack: &str) {
    let dir = tmpfs_root(kind).join(stack);
    let _ = tokio::fs::remove_dir_all(&dir).await;
}

/// Write `content` to `path` as a 0444, world-readable file. Overwrites
/// any prior entry so redeploy refreshes content. World-readable so
/// rootless containers (mapped sub-UIDs) can read regardless of the
/// in-container user. Files are read-only by design — bind-mounted
/// into the container, never written from inside.
#[cfg(unix)]
pub(super) async fn write_world_readable(path: &Path, content: &[u8]) -> anyhow::Result<()> {
    use anyhow::Context as _;
    use std::io::Write as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }
    let path_owned = path.to_path_buf();
    let content_owned = content.to_vec();
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let _ = std::fs::remove_file(&path_owned);
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o444)
            .open(&path_owned)
            .with_context(|| format!("creating {}", path_owned.display()))?;
        f.write_all(&content_owned)
            .with_context(|| format!("writing {}", path_owned.display()))?;
        Ok(())
    })
    .await
    .map_err(|e| anyhow::anyhow!("join: {e}"))??;
    Ok(())
}

#[cfg(not(unix))]
pub(super) async fn write_world_readable(path: &Path, content: &[u8]) -> anyhow::Result<()> {
    tokio::fs::write(path, content).await?;
    Ok(())
}

/// Make `path` traversable (0755) so any container UID can reach the
/// materialized files inside.
#[cfg(unix)]
pub(super) async fn ensure_dir_traversable(path: &Path) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt as _;
    let mut p = tokio::fs::metadata(path).await?.permissions();
    p.set_mode(0o755);
    tokio::fs::set_permissions(path, p).await?;
    Ok(())
}

#[cfg(not(unix))]
pub(super) async fn ensure_dir_traversable(_path: &Path) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso8601_known_values() {
        assert_eq!(iso8601_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso8601_utc(1_704_067_200), "2024-01-01T00:00:00Z");
        assert_eq!(iso8601_utc(1_777_766_400), "2026-05-03T00:00:00Z");
        assert_eq!(iso8601_utc(1_777_823_730), "2026-05-03T15:55:30Z");
    }

    #[test]
    fn valid_fs_name_accepts_typical_shapes() {
        assert!(valid_fs_name("nginx"));
        assert!(valid_fs_name("nginx.Dockerfile"));
        assert!(valid_fs_name("foo_bar-1"));
        assert!(valid_fs_name("db_password"));
        assert!(valid_fs_name("API_KEY"));
        assert!(valid_fs_name("a.b-c_1"));
        assert!(valid_fs_name("1nginx"));
        assert!(valid_fs_name("9up"));
        assert!(valid_fs_name(&"a".repeat(128)));
    }

    #[test]
    fn valid_fs_name_rejects_traversal_and_specials() {
        assert!(!valid_fs_name(""));
        assert!(!valid_fs_name("."));
        assert!(!valid_fs_name(".."));
        assert!(!valid_fs_name(".hidden"));
        assert!(!valid_fs_name(".identity"));
        assert!(!valid_fs_name("-leading"));
        assert!(!valid_fs_name("a/b"));
        assert!(!valid_fs_name("a\0b"));
        assert!(!valid_fs_name("x y"));
        assert!(!valid_fs_name(&"a".repeat(129)));
    }

    #[test]
    fn valid_fs_name_rejects_unicode() {
        assert!(!valid_fs_name("café"));
        assert!(!valid_fs_name("nginx™"));
    }
}
