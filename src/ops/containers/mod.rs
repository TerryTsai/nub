//! Container ops. One file per verb. The dispatch in `ops::mod` calls the
//! `run()` fn each module exports.

pub(super) mod action;
pub(super) mod create;
pub(super) mod exec;
pub(super) mod inspect;
pub(super) mod list;
pub(super) mod logs;
pub(super) mod stats;
