//! Internal helpers shared across the engine submodules.

/// Trim docker/podman ids to the conventional 12-char short form, stripping
/// any `sha256:` prefix.
pub(crate) fn short_id(id: &str) -> String {
    id.strip_prefix("sha256:").unwrap_or(id).chars().take(12).collect()
}
