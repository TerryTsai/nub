//! `clap`-derived command-line surface — the help-template strings, the
//! top-level `Args` struct, and the `Cmd` discriminator. Subcommand
//! enums live next to their per-verb files.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use super::bind::BindCmd;
use super::config::ConfigCmd;
use super::install::InstallTarget;
use super::key::KeyCmd;
use super::secret::SecretCmd;
use super::stack::StackCmd;
use super::token::TokenCmd;
use crate::version::NUB_VERSION;

const ABOUT: &str = "A control plane for one container host.";

const LONG_ABOUT: &str = "\
nub re-shapes Docker or Podman as the API you wish it had: a smaller
engine surface, scoped auth, encrypted secrets, and a compose-compatible
deploy layer. `nub run` starts the daemon foreground; in production
`nub install systemd` puts it under systemd. The CLI manages nub itself —
config, keys, tokens, secrets, stacks, and lifecycle. Container ops
happen through the embedded UI or the API directly.";

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

Lifecycle
  run                   Start the daemon in the foreground
  restart               Restart the systemd unit
  update                Pull the latest release and restart

Status
  status                Daemon, engine, listen state
  config show           Print effective config

Connect
  url                   Print connect URL
  qr                    Print connect URL as a QR

Management
  bind   list|allow|deny           Manage the bind-mount allowlist
  key    gen|rotate                Manage the Ed25519 issuer keypair
  token  mint|scopes               Mint JWTs and inspect scope vocabulary
  stack  deploy|ls|rm|redeploy|logs  Manage compose-shaped stacks
  secret put|list|rm|get           Manage age-encrypted secrets

Tools
  completions <SHELL>   Shell completion script
  man                   Man page (groff source)
";

const EXAMPLES: &str = "\
Examples:
  nub run                      Start the daemon (foreground)
  nub init                     Generate config
  nub install systemd          Install + enable the unit
  nub status                   Health check
  nub url                      Print connect URL
  nub bind allow /data         Permit a bind-mount source
  nub stack deploy app app.yml Deploy a stack from YAML
  echo s3cr3t | nub secret put db_password   Store an encrypted secret
  nub update                   Pull latest release and restart

For per-command help: nub <COMMAND> --help
For tab completion: nub completions zsh > ~/.zfunc/_nub
";

#[derive(Parser)]
#[command(
    name = "nub",
    about = ABOUT,
    long_about = LONG_ABOUT,
    version = NUB_VERSION,
    help_template = HELP_TEMPLATE,
    before_help = COMMANDS,
    after_help = EXAMPLES,
    arg_required_else_help = true,
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
    pub listen: Option<String>,
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
    /// Start the daemon in the foreground.
    Run,
    /// Restart the systemd unit (auto-detect user vs system).
    Restart,
    /// Pull the latest release and restart.
    Update {
        /// Just print what's available, don't change anything.
        #[arg(long)]
        check: bool,
        /// Pin to a specific version (e.g. v0.0.20). Default: latest.
        #[arg(long, value_name = "TAG")]
        version: Option<String>,
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
    /// Manage encrypted secrets.
    Secret {
        #[command(subcommand)]
        action: SecretCmd,
    },
    /// Print shell completion script.
    Completions {
        #[arg(value_name = "SHELL")]
        shell: clap_complete::Shell,
    },
    /// Print man page (groff source).
    Man,
}
