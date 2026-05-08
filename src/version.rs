//! Single source of truth for the version string nub reports. Combines
//! the Cargo package version with the git-short-hash suffix that
//! `build.rs` exposes via `NUB_VERSION_SUFFIX`.

pub const NUB_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), env!("NUB_VERSION_SUFFIX"));
