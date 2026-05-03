//! `nub` subcommands. Each command's implementation lives in its own
//! sibling module; this file only owns the `Cmd` enum (the clap surface)
//! and the dispatch table.

pub mod init;
pub mod keygen;
pub mod mint;
pub mod systemd;
pub mod uninstall;

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Cmd {
    /// Generate a starter config file. Default: $XDG_CONFIG_HOME/nub/nub.toml.
    Init {
        /// Where to write. Use `-` for stdout.
        path: Option<String>,
        /// Overwrite if file exists.
        #[arg(long)]
        force: bool,
    },
    /// Remove nub's config and data directories ($XDG_CONFIG_HOME/nub
    /// and $XDG_DATA_HOME/nub). The binary itself stays put.
    Uninstall {
        /// Skip the confirmation prompt.
        #[arg(long)]
        yes: bool,
    },
    /// Print a systemd unit file for nub on stdout. Default is `--user`
    /// (drop into ~/.config/systemd/user); `--system` for /etc/systemd/system.
    SystemdUnit {
        /// User-level unit (default).
        #[arg(long, conflicts_with = "system")]
        user: bool,
        /// System-level unit (runs as root unless edited).
        #[arg(long, conflicts_with = "user")]
        system: bool,
    },
    /// Generate (or rotate) the Ed25519 issuer keypair nub uses to sign
    /// tokens. With no flags, prints the existing public key.
    Keygen {
        /// Replace the existing key. Invalidates ALL previously-issued tokens.
        #[arg(long)]
        rotate: bool,
    },
    /// Mint a JWT signed by nub's issuer key. Prints the token on stdout.
    Mint {
        /// Subject claim — the identity this token represents (e.g. `phone-1`).
        #[arg(long)]
        sub: String,
        /// Space-separated op names, or `*` for all ops.
        #[arg(long, default_value = "*")]
        scope: String,
        /// TTL: e.g. `90d`, `1y`, `12h`. Default: 90 days.
        #[arg(long, default_value = "90d")]
        expires: String,
        /// Audience claim — the host id this token is for.
        /// Defaults to this machine's hostname.
        #[arg(long)]
        aud: Option<String>,
    },
}

pub fn dispatch(cmd: Cmd) -> Result<()> {
    match cmd {
        Cmd::Init { path, force } => init::run(path, force),
        Cmd::Uninstall { yes } => uninstall::run(yes),
        Cmd::SystemdUnit { system, user: _ } => systemd::print_unit(!system),
        Cmd::Keygen { rotate } => keygen::run(rotate),
        Cmd::Mint {
            sub,
            scope,
            expires,
            aud,
        } => mint::run(sub, scope, expires, aud),
    }
}

/// Best-effort hostname. Falls back to "nub" if /etc/hostname is unreadable.
/// Used both by `nub init`'s template substitution and by the running
/// binary when no `--id` is set.
pub fn hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .or_else(|_| std::fs::read_to_string("/proc/sys/kernel/hostname"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "nub".into())
}
