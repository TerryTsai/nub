//! Sub-command enums kept separate from `mod.rs` to keep that file
//! focused on the top-level surface (Args, Cmd, dispatch).

use clap::Subcommand;

#[derive(Subcommand)]
pub enum InstallTarget {
    /// Install a systemd unit. User-level by default; `--system` for /etc.
    Systemd {
        /// User-level unit (default).
        #[arg(long, conflicts_with = "system")]
        user: bool,
        /// System-level unit (requires root).
        #[arg(long, conflicts_with = "user")]
        system: bool,
        /// Print the unit text instead of installing.
        #[arg(long)]
        print: bool,
    },
}

#[derive(Subcommand)]
pub enum ConfigCmd {
    /// Print effective config (defaults + file + flags).
    Show,
}

#[derive(Subcommand)]
pub enum BindCmd {
    /// List the current allowlist.
    List,
    /// Add a path to the allowlist. Path must exist; canonicalized before write.
    Allow {
        #[arg(value_name = "PATH")]
        path: String,
    },
    /// Remove a path from the allowlist.
    Deny {
        #[arg(value_name = "PATH")]
        path: String,
    },
}

#[derive(Subcommand)]
pub enum KeyCmd {
    /// Generate the keypair if missing; print the public key either way.
    Gen,
    /// Replace the keypair. Invalidates ALL previously-issued tokens.
    Rotate,
}

#[derive(Subcommand)]
pub enum TokenCmd {
    /// Mint a JWT signed by nub's issuer key.
    Mint {
        /// Subject claim — the identity this token represents.
        #[arg(long, value_name = "ID")]
        sub: String,
        /// Space-separated op names, or `*` for all ops.
        #[arg(long, value_name = "OPS", default_value = "*")]
        scope: String,
        /// TTL: e.g. `90d`, `1y`, `12h`.
        #[arg(long, value_name = "DUR", default_value = "90d")]
        expires: String,
        /// Audience — the host id this token is for.
        #[arg(long, value_name = "HOST")]
        aud: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum StackCmd {
    /// Deploy a compose file as a stack.
    Deploy {
        /// Stack name. Lowercase alphanumeric, dash, underscore.
        #[arg(value_name = "NAME")]
        name: String,
        /// Path to compose file. Use `-` for stdin.
        #[arg(value_name = "FILE")]
        file: String,
    },
}
