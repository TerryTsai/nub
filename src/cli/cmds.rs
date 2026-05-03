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
        /// Explicit scope list (space- or comma-separated). Each entry is
        /// a `<resource>:<action>` scope, `*`, or `<resource>:*`. Mutually
        /// exclusive with `--preset`. See `nub token scopes`.
        #[arg(long, value_name = "SCOPES", conflicts_with = "preset")]
        scope: Option<String>,
        /// Named preset: `admin`, `phone`, or `readonly`. Expanded to an
        /// explicit scope list at mint time.
        #[arg(long, value_name = "NAME", conflicts_with = "scope")]
        preset: Option<String>,
        /// TTL: e.g. `90d`, `1y`, `12h`.
        #[arg(long, value_name = "DUR", default_value = "90d")]
        expires: String,
        /// Audience — the host id this token is for.
        #[arg(long, value_name = "HOST")]
        aud: Option<String>,
    },
    /// List every recognized scope, plus the named presets.
    Scopes,
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
    /// List stacks on this host.
    Ls,
    /// Tear down a stack: stop and remove its containers, drop the
    /// stack network, delete the manifest. Named volumes are preserved.
    Rm {
        #[arg(value_name = "NAME")]
        name: String,
        /// Skip the confirmation prompt.
        #[arg(long)]
        yes: bool,
    },
    /// Redeploy a stack from its stored manifest. Always-recreate.
    Redeploy {
        #[arg(value_name = "NAME")]
        name: String,
    },
    /// Stream interleaved logs from every container in a stack.
    Logs {
        #[arg(value_name = "NAME")]
        name: String,
        /// Keep streaming after current logs flush.
        #[arg(long, short)]
        follow: bool,
        /// Lines of history to replay. Default: 100.
        #[arg(long, value_name = "N")]
        tail: Option<u32>,
    },
}

#[derive(Subcommand)]
pub enum SecretCmd {
    /// Store a secret value. Reads from stdin by default — pipe or
    /// type-then-Ctrl-D — to keep the value out of shell history.
    Put {
        /// Secret name. Letters, digits, dot, underscore, dash.
        #[arg(value_name = "NAME")]
        name: String,
        /// Read the value from this file instead of stdin.
        #[arg(long, value_name = "PATH")]
        from_file: Option<String>,
    },
    /// List secret names + sizes. Values are never printed.
    List,
    /// Delete a secret. Idempotent — succeeds if the name doesn't exist.
    Rm {
        #[arg(value_name = "NAME")]
        name: String,
    },
    /// Print a secret's plaintext value to stdout. Admin-only by policy.
    Get {
        #[arg(value_name = "NAME")]
        name: String,
    },
}
