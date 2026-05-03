//! `nub uninstall` — wipe `$XDG_CONFIG_HOME/nub` and `$XDG_DATA_HOME/nub`.
//! The binary itself isn't touched; users who installed via `install.sh`
//! can `rm /usr/local/bin/nub` afterward.

use anyhow::{Context, Result};
use std::fs;
use std::io::{BufRead, Write};
use std::path::Path;

pub fn run(yes: bool) -> Result<()> {
    let cfg = crate::config::xdg_config_home().join("nub");
    let data = crate::config::xdg_data_home().join("nub");
    let targets: [&Path; 2] = [cfg.as_path(), data.as_path()];

    let existing: Vec<&Path> = targets.iter().copied().filter(|p| p.exists()).collect();
    if existing.is_empty() {
        println!(
            "nothing to remove (neither {} nor {} exist)",
            cfg.display(),
            data.display()
        );
        return Ok(());
    }

    println!("the following will be deleted:");
    for p in &existing {
        println!("  {}", p.display());
    }
    if !yes {
        print!("proceed? [y/N] ");
        std::io::stdout().flush().ok();
        let mut answer = String::new();
        std::io::stdin().lock().read_line(&mut answer)?;
        if !answer.trim().eq_ignore_ascii_case("y") {
            println!("cancelled");
            return Ok(());
        }
    }
    for p in &existing {
        fs::remove_dir_all(p).with_context(|| format!("removing {}", p.display()))?;
        println!("removed {}", p.display());
    }
    Ok(())
}
