//! Engine wire shapes for container ops. One file per verb-file that
//! consumes them; `mod.rs` is glue.

pub(super) mod create;
pub(super) mod inspect;
pub(super) mod list;
