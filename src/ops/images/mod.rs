//! Image ops. One file per Op variant; engine wire shapes consolidated
//! in `wire.rs`. The `tar` sibling is the build-context helper used by
//! `build`.

pub(super) mod build;
pub(super) mod inspect;
pub(super) mod list;
pub(super) mod pull;
pub(super) mod remove;

mod tar;
mod wire;
