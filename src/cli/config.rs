//! `nub config show` — print the resolved config plus the paths nub
//! reads/writes. No mutation today; `nub bind` covers the only field
//! worth scripted edits.

use anyhow::Result;
use std::path::PathBuf;

use crate::config;

use super::ConfigCmd;

pub fn run(action: ConfigCmd) -> Result<()> {
    match action {
        ConfigCmd::Show => show(),
    }
}

fn show() -> Result<()> {
    let candidates = [
        config::xdg_config_home().join("nub/nub.toml"),
        PathBuf::from("./nub.toml"),
        PathBuf::from("/etc/nub/config.toml"),
    ];
    if let Some(path) = candidates.into_iter().find(|p| p.exists()) {
        println!("# {}", path.display());
        let content = std::fs::read_to_string(&path)?;
        print!("{content}");
        if !content.ends_with('\n') {
            println!();
        }
    } else {
        println!("# (no config file; using compiled defaults)");
        println!("id = \"{}\"", super::hostname());
        println!("bind = \"0.0.0.0:8080\"");
    }
    println!();
    println!("# Resolved paths:");
    println!("#   config dir   {}", config::xdg_config_home().join("nub").display());
    println!("#   data dir     {}", config::xdg_data_home().join("nub").display());
    println!("#   issuer key   {}", config::default_issuer_key().display());
    println!("#   admin token  {}", config::default_admin_jwt().display());
    println!("#   dockerfiles  {}", config::default_dockerfiles_dir().display());
    println!("#   stacks       {}", config::default_stacks_dir().display());
    Ok(())
}
