//! `nub restart` — restart the nub systemd unit. Auto-detects whether
//! it's a user-level or system-level install by checking which unit
//! file exists. Refuses cleanly if neither does.

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config;

#[derive(Clone, Copy)]
enum Scope {
    User,
    System,
}

pub fn run() -> Result<()> {
    let scope = detect_scope()?;
    let mut cmd = Command::new("systemctl");
    if matches!(scope, Scope::User) {
        cmd.arg("--user");
    }
    let status = cmd.args(["restart", "nub"]).status()?;
    if !status.success() {
        return Err(anyhow!("systemctl restart nub failed"));
    }
    println!("restarted nub ({})", scope_label(scope));
    Ok(())
}

fn detect_scope() -> Result<Scope> {
    let user_unit: PathBuf = config::xdg_config_home().join("systemd/user/nub.service");
    if user_unit.exists() {
        return Ok(Scope::User);
    }
    let system_unit = Path::new("/etc/systemd/system/nub.service");
    if system_unit.exists() {
        return Ok(Scope::System);
    }
    Err(anyhow!("no nub systemd unit found; run `nub install systemd` first"))
}

fn scope_label(scope: Scope) -> &'static str {
    match scope {
        Scope::User => "user",
        Scope::System => "system",
    }
}
