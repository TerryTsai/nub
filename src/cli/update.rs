//! `nub update` — fetch the latest GitHub release, swap the binary
//! atomically, and restart the systemd unit. Shells out to `curl` and
//! `tar` (already required by install.sh) to avoid pulling HTTPS/TLS
//! client crates into the binary.

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::restart;

const REPO: &str = "TerryTsai/nub";

pub fn run(check: bool, pinned: Option<String>) -> Result<()> {
    need("curl")?;
    need("tar")?;

    let target = detect_target()?;
    let current = env!("CARGO_PKG_VERSION");
    let latest_tag = match pinned {
        Some(v) => normalize_tag(&v),
        None => fetch_latest_tag()?,
    };
    let latest = latest_tag.trim_start_matches('v');

    println!("current: {current}");
    println!("latest:  {latest}");

    if check {
        if current == latest {
            println!("up to date");
        } else {
            println!("run `nub update` to upgrade");
        }
        return Ok(());
    }

    if current == latest {
        println!("up to date — nothing to do");
        return Ok(());
    }

    let target_path = std::env::current_exe().context("locating current nub binary")?;
    check_writable_dir(&target_path)?;

    let tmp = make_tempdir()?;
    let url = format!("https://github.com/{REPO}/releases/download/{latest_tag}/nub-{latest_tag}-{target}.tar.gz");
    println!("downloading {url}");
    download_and_extract(&url, &tmp)?;

    let new_binary = tmp.join("nub");
    if !new_binary.exists() {
        return Err(anyhow!("downloaded archive doesn't contain a `nub` binary"));
    }

    println!("swapping {}", target_path.display());
    swap_binary(&new_binary, &target_path)?;
    let _ = std::fs::remove_dir_all(&tmp);

    println!("installed nub {latest}");
    match restart::run() {
        Ok(()) => {}
        Err(_) => {
            println!("(no systemd unit installed yet — run `nub install systemd` to set one up,");
            println!(" or start the new binary manually with `nub run`)");
        }
    }
    Ok(())
}

fn need(bin: &str) -> Result<()> {
    if Command::new(bin).arg("--version").output().is_err() {
        return Err(anyhow!("`{bin}` not found in PATH (required for nub update)"));
    }
    Ok(())
}

fn normalize_tag(s: &str) -> String {
    if s.starts_with('v') {
        s.to_string()
    } else {
        format!("v{s}")
    }
}

fn detect_target() -> Result<&'static str> {
    let arch = std::env::consts::ARCH;
    match arch {
        "x86_64" => Ok("x86_64-unknown-linux-musl"),
        "aarch64" => Ok("aarch64-unknown-linux-musl"),
        a => Err(anyhow!("unsupported architecture: {a}")),
    }
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
}

fn fetch_latest_tag() -> Result<String> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let out = Command::new("curl")
        .args(["-fsSL", "-H", "Accept: application/vnd.github+json", &url])
        .output()
        .context("running curl")?;
    if !out.status.success() {
        return Err(anyhow!("curl failed: {}", String::from_utf8_lossy(&out.stderr)));
    }
    let r: Release = serde_json::from_slice(&out.stdout).context("parsing release info")?;
    Ok(r.tag_name)
}

fn make_tempdir() -> Result<PathBuf> {
    let p = std::env::temp_dir().join(format!("nub-update-{}", std::process::id()));
    std::fs::create_dir_all(&p).with_context(|| format!("creating {}", p.display()))?;
    Ok(p)
}

fn download_and_extract(url: &str, dir: &Path) -> Result<()> {
    let archive = dir.join("nub.tar.gz");
    let status =
        Command::new("curl").args(["-fsSL", "-o", archive.to_str().unwrap(), url]).status().context("running curl")?;
    if !status.success() {
        return Err(anyhow!("download failed (curl exit {:?})", status.code()));
    }
    let status = Command::new("tar")
        .args(["-xzf", archive.to_str().unwrap(), "-C", dir.to_str().unwrap()])
        .status()
        .context("running tar")?;
    if !status.success() {
        return Err(anyhow!("tar extract failed (exit {:?})", status.code()));
    }
    Ok(())
}

fn swap_binary(src: &Path, dst: &Path) -> Result<()> {
    let staging = dst.with_extension("new");
    std::fs::copy(src, &staging).with_context(|| format!("copying to {}", staging.display()))?;
    set_executable(&staging)?;
    std::fs::rename(&staging, dst).with_context(|| format!("renaming {} -> {}", staging.display(), dst.display()))?;
    Ok(())
}

fn set_executable(p: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(p)?.permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(p, perms)?;
    Ok(())
}

/// Probe writability before downloading anything — saves the user from
/// "downloaded 5MB then permission denied at the last step."
fn check_writable_dir(binary: &Path) -> Result<()> {
    let parent = binary.parent().ok_or_else(|| anyhow!("can't find parent of {}", binary.display()))?;
    let probe = parent.join(format!(".nub-update-probe-{}", std::process::id()));
    match std::fs::File::create(&probe) {
        Ok(_) => {
            let _ = std::fs::remove_file(&probe);
            Ok(())
        }
        Err(_) => Err(anyhow!(
            "can't write to {} (binary lives in {}). Re-run with sudo if installed system-wide.",
            parent.display(),
            binary.display()
        )),
    }
}
