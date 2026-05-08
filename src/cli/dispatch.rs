//! Top-level CLI dispatch — `Cmd` variant → per-subcommand `run`.
//! `Cmd::Run` is handled by `main.rs` (it owns the tokio runtime); every
//! other variant routes through here.

use anyhow::Result;

use super::args::Cmd;
use super::completions;
use super::{bind, config, connect, init, install, key, man, restart, secret, stack, status, token, uninstall, update};

pub fn dispatch(cmd: Cmd) -> Result<()> {
    match cmd {
        Cmd::Init { path, force } => init::run(path, force),
        Cmd::Install { target } => install::run(target),
        Cmd::Uninstall { yes } => uninstall::run(yes),
        Cmd::Run => unreachable!("`nub run` is handled in main()"),
        Cmd::Restart => restart::run(),
        Cmd::Update { check, version } => update::run(check, version),
        Cmd::Status => status::run(),
        Cmd::Config { action } => config::run(action),
        Cmd::Url => connect::print_url(),
        Cmd::Qr => connect::print_qr(),
        Cmd::Bind { action } => bind::run(action),
        Cmd::Key { action } => key::run(action),
        Cmd::Token { action } => token::run(action),
        Cmd::Stack { action } => stack::run(action),
        Cmd::Secret { action } => secret::run(action),
        Cmd::Completions { shell } => completions::run(shell),
        Cmd::Man => man::run(),
    }
}
