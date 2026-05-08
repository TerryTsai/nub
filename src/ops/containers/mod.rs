//! Container ops. One file per Op variant; engine wire shapes
//! consolidated in `wire/`. The `create_build` sibling is the
//! `proto::CreateContainerReq` → engine wire-body translator.

pub(super) mod create;
pub(super) mod exec;
pub(super) mod inspect;
pub(super) mod kill;
pub(super) mod list;
pub(super) mod logs;
pub(super) mod remove;
pub(super) mod restart;
pub(super) mod start;
pub(super) mod stats;
pub(super) mod stop;

mod create_build;
mod wire;
