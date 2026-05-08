//! Auto-discover the nub.toml location. Search order: XDG config home,
//! `./nub.toml`, `/etc/nub/config.toml`. Used by `Config::load` and by
//! every CLI subcommand that needs to find the active config.

use std::path::PathBuf;

use super::paths::xdg_config_home;

/// Candidate paths nub checks for `nub.toml`, in priority order.
pub fn auto_paths() -> Vec<PathBuf> {
    vec![
        xdg_config_home().join("nub/nub.toml"),
        PathBuf::from("./nub.toml"),
        PathBuf::from("/etc/nub/config.toml"),
    ]
}

/// First existing config path, if any. Returns `None` when nub has
/// never been configured on this host.
pub fn locate_path() -> Option<PathBuf> {
    auto_paths().into_iter().find(|p| p.exists())
}
