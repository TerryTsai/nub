//! `nub bind` — manage `allowed_binds` in `nub.toml`. Edits the file in
//! place via `toml_edit` so comments and field order survive the
//! round-trip. Requires nub restart for the change to take effect
//! (Policy is read once at boot).

use anyhow::{anyhow, bail, Context, Result};
use clap::Subcommand;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config;

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

pub fn run(action: BindCmd) -> Result<()> {
    match action {
        BindCmd::List => list(),
        BindCmd::Allow { path } => add(path),
        BindCmd::Deny { path } => remove(path),
    }
}

fn list() -> Result<()> {
    let Some(config_path) = locate_config_optional() else {
        println!("(no nub.toml found — run `nub init` first)");
        return Ok(());
    };
    let raw = fs::read_to_string(&config_path).with_context(|| format!("reading {}", config_path.display()))?;
    let doc: toml_edit::DocumentMut = raw.parse().with_context(|| format!("parsing {}", config_path.display()))?;
    let current = read_array(&doc);
    if current.is_empty() {
        println!("(allowlist empty — bind mounts are denied)");
        println!("config: {}", config_path.display());
        return Ok(());
    }
    for p in &current {
        println!("{p}");
    }
    println!("config: {}", config_path.display());
    Ok(())
}

fn locate_config_optional() -> Option<PathBuf> {
    config::locate_path()
}

fn add(path: String) -> Result<()> {
    let resolved = resolve(&path)?;
    let config_path = locate_config()?;
    let added = update(&config_path, |list| {
        let s = resolved.display().to_string();
        if list.iter().any(|x| x == &s) {
            return false;
        }
        list.push(s);
        true
    })?;
    if added {
        println!("allowed: {}", resolved.display());
    } else {
        println!("already allowed: {}", resolved.display());
    }
    println!("wrote {}", config_path.display());
    println!("restart nub for the change to take effect.");
    Ok(())
}

fn remove(path: String) -> Result<()> {
    let resolved = resolve(&path)?;
    let config_path = locate_config()?;
    let removed = update(&config_path, |list| {
        let s = resolved.display().to_string();
        let before = list.len();
        list.retain(|x| x != &s);
        before != list.len()
    })?;
    if removed {
        println!("denied: {}", resolved.display());
    } else {
        println!("not in allowlist: {}", resolved.display());
    }
    println!("wrote {}", config_path.display());
    println!("restart nub for the change to take effect.");
    Ok(())
}

fn resolve(p: &str) -> Result<PathBuf> {
    let path = Path::new(p);
    if !path.exists() {
        bail!("path `{p}` does not exist");
    }
    path.canonicalize().with_context(|| format!("canonicalizing {p}"))
}

/// Find an existing nub.toml. Refuses to invent one — running `allow`
/// before `init` is almost always a mistake (the resulting sparse
/// config has no id/bind/etc. and would surprise on next restart).
fn locate_config() -> Result<PathBuf> {
    locate_config_optional().ok_or_else(|| anyhow!("no nub.toml found; run `nub init` first"))
}

fn update(path: &Path, mutate: impl FnOnce(&mut Vec<String>) -> bool) -> Result<bool> {
    let raw = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let mut doc: toml_edit::DocumentMut = raw.parse().with_context(|| format!("parsing {}", path.display()))?;
    let mut current = read_array(&doc);
    let changed = mutate(&mut current);
    if !changed {
        return Ok(false);
    }
    write_array(&mut doc, &current);
    fs::write(path, doc.to_string()).with_context(|| format!("writing {}", path.display()))?;
    Ok(true)
}

fn read_array(doc: &toml_edit::DocumentMut) -> Vec<String> {
    doc.get("allowed_binds")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|i| i.as_str().map(String::from)).collect())
        .unwrap_or_default()
}

fn write_array(doc: &mut toml_edit::DocumentMut, paths: &[String]) {
    let mut arr = toml_edit::Array::new();
    for p in paths {
        arr.push(p.as_str());
    }
    doc["allowed_binds"] = toml_edit::value(arr);
}
