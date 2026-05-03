//! `nub uninstall` — wipe `$XDG_CONFIG_HOME/nub`, `$XDG_DATA_HOME/nub`,
//! and any nub systemd unit nub itself wrote. The binary isn't touched;
//! users who installed via `install.sh` can `rm $(which nub)` afterward.

use anyhow::{Context, Result};
use std::fs;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run(yes: bool) -> Result<()> {
    let cfg = crate::config::xdg_config_home().join("nub");
    let data = crate::config::xdg_data_home().join("nub");
    let user_unit = crate::config::xdg_config_home().join("systemd/user/nub.service");
    let system_unit = PathBuf::from("/etc/systemd/system/nub.service");

    let dirs: Vec<&Path> = [cfg.as_path(), data.as_path()]
        .into_iter()
        .filter(|p| p.exists())
        .collect();
    let units: Vec<&Path> = [user_unit.as_path(), system_unit.as_path()]
        .into_iter()
        .filter(|p| p.exists())
        .collect();

    if dirs.is_empty() && units.is_empty() {
        println!("nothing to remove (no nub config, data, or systemd unit found)");
        return Ok(());
    }

    println!("the following will be deleted:");
    for p in &dirs {
        println!("  {}", p.display());
    }
    for p in &units {
        println!("  {} (will also stop+disable the unit)", p.display());
    }
    if !yes && !confirm()? {
        println!("cancelled");
        return Ok(());
    }

    // Disable+stop units before deleting the files, so systemd's view
    // doesn't outlive the on-disk state. Best-effort: missing systemctl
    // or non-zero exits get a warning, not a hard failure.
    for unit in &units {
        teardown_unit(unit);
    }
    for unit in &units {
        if let Err(e) = fs::remove_file(unit) {
            eprintln!("warning: removing {}: {e}", unit.display());
        } else {
            println!("removed {}", unit.display());
        }
    }
    if !units.is_empty() {
        reload_daemon();
    }
    for p in &dirs {
        fs::remove_dir_all(p).with_context(|| format!("removing {}", p.display()))?;
        println!("removed {}", p.display());
    }
    Ok(())
}

fn confirm() -> Result<bool> {
    print!("proceed? [y/N] ");
    std::io::stdout().flush().ok();
    let mut answer = String::new();
    std::io::stdin().lock().read_line(&mut answer)?;
    Ok(answer.trim().eq_ignore_ascii_case("y"))
}

fn teardown_unit(path: &Path) {
    let is_system = path.starts_with("/etc/systemd/system");
    let scope = if is_system { "--system" } else { "--user" };
    // disable --now stops + disables in one shot.
    let _ = Command::new("systemctl")
        .args([scope, "disable", "--now", "nub"])
        .status();
}

fn reload_daemon() {
    let _ = Command::new("systemctl").args(["--user", "daemon-reload"]).status();
    let _ = Command::new("systemctl").args(["--system", "daemon-reload"]).status();
}
