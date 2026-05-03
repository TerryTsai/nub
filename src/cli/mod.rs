//! `nub` CLI — manages the nub daemon. Container ops live on the phone
//! UI; this CLI is for setup, auth, and inspection of nub itself.

pub mod bind;
mod cmds;
pub mod completions;
pub mod config;
pub mod connect;
pub mod init;
pub mod install;
pub mod key;
pub mod man;
pub mod stack;
pub mod status;
pub mod token;
pub mod uninstall;

pub use cmds::{BindCmd, ConfigCmd, InstallTarget, KeyCmd, StackCmd, TokenCmd};

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), env!("NUB_VERSION_SUFFIX"));

const ABOUT: &str = "Minimal Docker/Podman control plane.";

const LONG_ABOUT: &str = "\
nub is an agent-shape daemon: it runs on one host, manages that host's
container engine, and exposes a phone-first UI. Run `nub` with no args
to start the server. The CLI manages nub itself — config, keys, tokens,
the bind allowlist, and a few one-shot helpers. Container ops happen on
the phone (or via podman/docker directly).";

const HELP_TEMPLATE: &str = "\
{about-with-newline}
{usage-heading} {usage}

{before-help}\
Options:
{options}
{after-help}";

const COMMANDS: &str = "\
Setup
  init                  Generate a starter nub.toml
  install systemd       Install + enable the systemd unit
  uninstall             Remove config, data, and systemd unit

Status
  status                Daemon, engine, listen state
  config show           Print effective config

Connect
  url                   Print connect URL
  qr                    Print connect URL as a QR

Management
  bind list|allow|deny  Manage the bind-mount allowlist
  key  gen|rotate       Manage the Ed25519 issuer keypair
  token mint            Mint a JWT
  stack deploy          Deploy a compose file as a stack

Tools
  completions <SHELL>   Shell completion script
  man                   Man page (groff source)
";

const EXAMPLES: &str = "\
Examples:
  nub                          Start the server
  nub init                     Generate config
  nub install systemd          Install + start systemd unit
  nub status                   Health check
  nub url                      Print connect URL
  nub bind allow /data         Permit /data as a bind-mount source
  nub stack deploy app app.yml Deploy a stack from a YAML file

For per-command help: nub <COMMAND> --help
For tab completion: nub completions zsh > ~/.zfunc/_nub
";

#[derive(Parser)]
#[command(
    name = "nub",
    about = ABOUT,
    long_about = LONG_ABOUT,
    version = VERSION,
    help_template = HELP_TEMPLATE,
    before_help = COMMANDS,
    after_help = EXAMPLES,
)]
pub struct Args {
    /// Path to TOML config (default: $XDG_CONFIG_HOME/nub/nub.toml,
    /// ./nub.toml, /etc/nub/config.toml).
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,
    /// Identifier this binary advertises (default: /etc/hostname or "nub").
    #[arg(long, value_name = "ID")]
    pub id: Option<String>,
    /// Address to listen on.
    #[arg(long, value_name = "ADDR")]
    pub bind: Option<String>,
    /// TLS certificate path (PEM). Pair with --tls-key.
    #[arg(long, value_name = "PATH")]
    pub tls_cert: Option<PathBuf>,
    /// TLS private key path (PEM). Pair with --tls-cert.
    #[arg(long, value_name = "PATH")]
    pub tls_key: Option<PathBuf>,

    #[command(subcommand)]
    pub cmd: Option<Cmd>,
}

#[derive(Subcommand)]
pub enum Cmd {
    /// Generate a starter nub.toml.
    Init {
        /// Where to write. Use `-` for stdout.
        #[arg(value_name = "PATH")]
        path: Option<String>,
        /// Overwrite if file exists.
        #[arg(long)]
        force: bool,
    },
    /// Install systemd unit, daemon-reload, and start nub.
    Install {
        #[command(subcommand)]
        target: InstallTarget,
    },
    /// Remove nub's config, data, and any systemd unit it wrote.
    Uninstall {
        /// Skip the confirmation prompt.
        #[arg(long)]
        yes: bool,
    },
    /// Daemon, engine, and listen state at a glance.
    Status,
    /// Inspect or manage configuration.
    Config {
        #[command(subcommand)]
        action: ConfigCmd,
    },
    /// Print connect URL (uses the persisted admin token).
    Url,
    /// Print connect URL as a scannable QR.
    Qr,
    /// Manage the bind-mount allowlist.
    Bind {
        #[command(subcommand)]
        action: BindCmd,
    },
    /// Manage the Ed25519 issuer keypair.
    Key {
        #[command(subcommand)]
        action: KeyCmd,
    },
    /// Manage tokens.
    Token {
        #[command(subcommand)]
        action: TokenCmd,
    },
    /// Manage stacks.
    Stack {
        #[command(subcommand)]
        action: StackCmd,
    },
    /// Print shell completion script.
    Completions {
        #[arg(value_name = "SHELL")]
        shell: clap_complete::Shell,
    },
    /// Print man page (groff source).
    Man,
}

pub fn dispatch(cmd: Cmd) -> Result<()> {
    match cmd {
        Cmd::Init { path, force } => init::run(path, force),
        Cmd::Install { target } => install::run(target),
        Cmd::Uninstall { yes } => uninstall::run(yes),
        Cmd::Status => status::run(),
        Cmd::Config { action } => config::run(action),
        Cmd::Url => connect::print_url(),
        Cmd::Qr => connect::print_qr(),
        Cmd::Bind { action } => bind::run(action),
        Cmd::Key { action } => key::run(action),
        Cmd::Token { action } => token::run(action),
        Cmd::Stack { action } => stack::run(action),
        Cmd::Completions { shell } => completions::run(shell),
        Cmd::Man => man::run(),
    }
}

/// Best-effort hostname. Falls back to "nub" if /etc/hostname is unreadable.
pub fn hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .or_else(|_| std::fs::read_to_string("/proc/sys/kernel/hostname"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "nub".into())
}
