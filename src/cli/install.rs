//! `nub install systemd` — write the unit file, daemon-reload, enable
//! and start. `ExecStart` is set to the binary's `current_exe()` so the
//! unit always points at the nub that emitted it.

use anyhow::{anyhow, ensure, Context, Result};
use clap::Subcommand;
use std::path::{Path, PathBuf};
use std::process::Command;

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

pub fn run(target: InstallTarget) -> Result<()> {
    match target {
        InstallTarget::Systemd { user, system, print } => systemd(user, system, print),
    }
}

fn systemd(user_flag: bool, system_flag: bool, print: bool) -> Result<()> {
    let scope = pick_scope(user_flag, system_flag)?;
    let unit = render_unit(scope);
    if print {
        print!("{unit}");
        return Ok(());
    }
    let path = unit_path(scope);
    if scope == Scope::System && !is_root() {
        return Err(anyhow!("system-level install needs root; run with sudo or pass --user"));
    }
    write_unit(&path, &unit)?;
    println!("wrote {}", path.display());
    daemon_reload(scope);
    enable_now(scope)?;
    println!("started nub via systemd ({scope}). manage with:");
    print_manage_hints(scope);
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Scope {
    User,
    System,
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User => write!(f, "user"),
            Self::System => write!(f, "system"),
        }
    }
}

fn pick_scope(user: bool, system: bool) -> Result<Scope> {
    match (user, system) {
        (true, true) => unreachable!("clap conflicts_with prevents this"),
        (false, true) => Ok(Scope::System),
        (true, false) | (false, false) => {
            // Default to user when neither flag is set, matching how
            // most homelab installs prefer to avoid root.
            Ok(Scope::User)
        }
    }
}

fn unit_path(scope: Scope) -> PathBuf {
    match scope {
        Scope::User => crate::config::xdg_config_home().join("systemd/user/nub.service"),
        Scope::System => PathBuf::from("/etc/systemd/system/nub.service"),
    }
}

fn render_unit(scope: Scope) -> String {
    let exec = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(String::from))
        .unwrap_or_else(|| "/usr/local/bin/nub".to_string());
    match scope {
        Scope::User => user_unit(&exec),
        Scope::System => system_unit(&exec),
    }
}

fn write_unit(path: &Path, unit: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    std::fs::write(path, unit).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

fn daemon_reload(scope: Scope) {
    let mut cmd = Command::new("systemctl");
    if scope == Scope::User {
        cmd.arg("--user");
    }
    let _ = cmd.arg("daemon-reload").status();
}

fn enable_now(scope: Scope) -> Result<()> {
    let mut cmd = Command::new("systemctl");
    if scope == Scope::User {
        cmd.arg("--user");
    }
    let status = cmd.args(["enable", "--now", "nub"]).status().context("running systemctl enable --now nub")?;
    ensure!(
        status.success(),
        "systemctl enable --now nub failed (exit {:?})",
        status.code()
    );
    Ok(())
}

fn print_manage_hints(scope: Scope) {
    let prefix = match scope {
        Scope::User => "systemctl --user",
        Scope::System => "sudo systemctl",
    };
    let journal_prefix = match scope {
        Scope::User => "journalctl --user -u nub",
        Scope::System => "sudo journalctl -u nub",
    };
    println!("  {prefix} status nub");
    println!("  {prefix} restart nub");
    println!("  {journal_prefix} -f");
}

fn is_root() -> bool {
    std::env::var_os("USER").map(|u| u == "root").unwrap_or(false)
        || std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("Uid:"))
                    .and_then(|l| l.split_whitespace().nth(1))
                    .and_then(|n| n.parse::<u32>().ok())
            })
            .map(|uid| uid == 0)
            .unwrap_or(false)
}

fn user_unit(exec: &str) -> String {
    format!(
        r#"[Unit]
Description=nub Docker/Podman control plane
After=default.target

[Service]
Type=simple
ExecStart={exec} run
Restart=on-failure
RestartSec=2s

[Install]
WantedBy=default.target
"#
    )
}

fn system_unit(exec: &str) -> String {
    format!(
        r#"[Unit]
Description=nub Docker/Podman control plane
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart={exec} run
Restart=on-failure
RestartSec=2s

[Install]
WantedBy=multi-user.target
"#
    )
}
