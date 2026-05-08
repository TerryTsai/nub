//! Network ops. One file per Op variant; engine wire shapes
//! consolidated in `wire.rs`.

pub(super) mod create;
pub(super) mod inspect;
pub(super) mod list;
pub(super) mod remove;

mod wire;
