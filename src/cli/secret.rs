//! `nub secret` — manage age-encrypted secrets in the host secrets dir.
//! Operates on the local filesystem directly: encrypt/decrypt happen in
//! this process using the per-host identity. The daemon doesn't have to
//! be running.

use std::io::{IsTerminal as _, Read as _};

use anyhow::{Context, Result};

use crate::config;
use crate::ops::secrets;

use super::SecretCmd;

pub fn run(action: SecretCmd) -> Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
    runtime.block_on(dispatch(action))
}

async fn dispatch(action: SecretCmd) -> Result<()> {
    let root = secrets_root();
    match action {
        SecretCmd::Put { name, from_file } => put(&root, name, from_file).await,
        SecretCmd::List => list(&root).await,
        SecretCmd::Rm { name } => rm(&root, name).await,
        SecretCmd::Get { name } => get(&root, name).await,
    }
}

async fn put(root: &std::path::Path, name: String, from_file: Option<String>) -> Result<()> {
    let value = match from_file {
        Some(path) => std::fs::read_to_string(&path).with_context(|| format!("reading {path}"))?,
        None => read_stdin_value()?,
    };
    let value = value.strip_suffix('\n').unwrap_or(&value).to_string();
    secrets::put(root, &name, &value).await?;
    println!("stored {name}");
    Ok(())
}

async fn list(root: &std::path::Path) -> Result<()> {
    let items = secrets::list(root).await?;
    if items.is_empty() {
        println!("(no secrets)");
        return Ok(());
    }
    let name_width = items.iter().map(|s| s.name.len()).max().unwrap_or(0).max(4);
    for s in items {
        println!("{:<name_width$}  {:>6}B  {}", s.name, s.size, s.modified_at);
    }
    Ok(())
}

async fn rm(root: &std::path::Path, name: String) -> Result<()> {
    secrets::delete(root, &name).await?;
    println!("removed {name}");
    Ok(())
}

async fn get(root: &std::path::Path, name: String) -> Result<()> {
    let v = secrets::get(root, &name).await?;
    print!("{}", v.value);
    if !v.value.ends_with('\n') {
        println!();
    }
    Ok(())
}

fn read_stdin_value() -> Result<String> {
    let mut stdin = std::io::stdin();
    if stdin.is_terminal() {
        eprintln!("(reading secret from stdin; finish with Ctrl-D)");
    }
    let mut buf = String::new();
    stdin.read_to_string(&mut buf).context("reading stdin")?;
    Ok(buf)
}

fn secrets_root() -> std::path::PathBuf {
    let cfg = config::Config::load(None).ok().flatten().unwrap_or_default();
    cfg.secrets.unwrap_or_else(config::default_secrets_dir)
}
