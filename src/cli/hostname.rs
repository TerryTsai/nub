//! Best-effort hostname helper. Used by `nub init` for the default
//! `id`, by `nub url`/`qr` to substitute for unspecified listen
//! addresses, and by `token mint --aud` defaulting.

/// Best-effort hostname. Falls back to "nub" if /etc/hostname is unreadable.
pub fn hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .or_else(|_| std::fs::read_to_string("/proc/sys/kernel/hostname"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "nub".into())
}
