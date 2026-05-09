//! `nub status` — one-shot snapshot of nub's state on this host. No
//! daemon-to-CLI RPC; everything reads from the filesystem, the engine
//! socket, and `systemctl is-active`.

use anyhow::Result;
use std::path::Path;
use std::process::Command;

use crate::config::{self, Config};

use super::connect;

pub fn run() -> Result<()> {
    println!("nub {}", crate::version::NUB_VERSION);

    if let Some(p) = config::locate_path() {
        println!("  config        {} (loaded)", p.display());
    } else {
        println!("  config        (none — using defaults)");
    }
    let cfg = Config::load(None)?.unwrap_or_default();
    let listen = cfg.listen.clone().unwrap_or_else(|| "0.0.0.0:8080".into());
    let tls = cfg.tls_cert.is_some() && cfg.tls_key.is_some();
    println!("  listen        {listen}");
    println!("  tls           {}", if tls { "on" } else { "off" });
    println!("  data dir      {}", config::xdg_data_home().join("nub").display());
    print_engine();
    print_systemd();
    print_connect_url();
    print_counts(&cfg);
    Ok(())
}

fn print_engine() {
    let runtime = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
        Ok(rt) => rt,
        Err(e) => {
            println!("  engine        (probe failed: {e})");
            return;
        }
    };
    match runtime.block_on(crate::client::Engine::connect()) {
        Ok(eng) => println!("  engine        {} ({})", eng.kind(), eng.address_display()),
        Err(e) => println!("  engine        unreachable ({e})"),
    }
}

fn print_systemd() {
    if let Some((scope, state)) = systemd_state() {
        println!("  systemd       {state} ({scope} unit)");
    } else {
        println!("  systemd       (no nub unit installed)");
    }
}

fn systemd_state() -> Option<(&'static str, String)> {
    let user_unit = config::xdg_config_home().join("systemd/user/nub.service");
    if user_unit.exists() {
        return Some(("user", is_active_state(true)));
    }
    let system_unit = Path::new("/etc/systemd/system/nub.service");
    if system_unit.exists() {
        return Some(("system", is_active_state(false)));
    }
    None
}

fn is_active_state(user: bool) -> String {
    let mut cmd = Command::new("systemctl");
    if user {
        cmd.arg("--user");
    }
    cmd.args(["is-active", "nub"]);
    match cmd.output() {
        Ok(out) => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        Err(_) => "unknown".into(),
    }
}

fn print_connect_url() {
    match connect::url_from_disk() {
        Ok(url) => println!("  connect       {url}"),
        Err(_) => println!("  connect       (no admin token yet — run nub once)"),
    }
}

fn print_counts(cfg: &Config) {
    println!("  bind paths    {} allowed", cfg.allowed_binds.len());
    let stacks_root = cfg.stacks.clone().unwrap_or_else(config::default_stacks_dir);
    println!("  stacks        {}", count_stacks(&stacks_root));
}

fn count_stacks(root: &Path) -> usize {
    if !root.exists() {
        return 0;
    }
    let Ok(entries) = std::fs::read_dir(root) else { return 0 };
    entries.flatten().filter(|e| e.path().join("compose.yml").exists()).count()
}
