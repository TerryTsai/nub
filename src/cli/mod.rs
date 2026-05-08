//! `nub` CLI — manages the nub daemon itself: config, keys, tokens,
//! secrets, stacks, lifecycle. Container ops live in the embedded
//! web UI (or against the API directly).

pub mod connect;

mod args;
mod bind;
mod completions;
mod config;
mod dispatch;
mod hostname;
mod init;
mod install;
mod key;
mod man;
mod restart;
mod secret;
mod stack;
mod status;
mod token;
mod uninstall;
mod update;

pub use args::{Args, Cmd};
pub use dispatch::dispatch;
pub use hostname::hostname;
