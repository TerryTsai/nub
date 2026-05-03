//! `nub url` / `nub qr` — emit the connect URL stored on disk so a
//! phone can pair without parsing a 270-char JWT out of journalctl.
//! Also home of the helpers the boot banner uses to render the same
//! URL and QR — single source of truth.

use anyhow::{anyhow, Context, Result};

use crate::config::{self, Config};

pub fn print_url() -> Result<()> {
    println!("{}", url_from_disk()?);
    Ok(())
}

pub fn print_qr() -> Result<()> {
    let url = url_from_disk()?;
    println!("{url}");
    render_qr(&url);
    Ok(())
}

/// Read the persisted admin token + the saved (or default) bind config
/// and assemble the URL the boot banner emits. Returns a friendly error
/// if nub has never been run on this host (admin.jwt absent).
pub(super) fn url_from_disk() -> Result<String> {
    let admin_path = config::default_admin_jwt();
    if !admin_path.exists() {
        return Err(anyhow!(
            "no admin token at {}; run `nub` once to generate it",
            admin_path.display()
        ));
    }
    let admin = std::fs::read_to_string(&admin_path)
        .with_context(|| format!("reading {}", admin_path.display()))?
        .trim()
        .to_string();
    if admin.is_empty() {
        return Err(anyhow!("admin token at {} is empty", admin_path.display()));
    }
    let cfg = locate_config()?.unwrap_or_default();
    Ok(build_url(&cfg, &admin))
}

fn locate_config() -> Result<Option<Config>> {
    Config::load(None)
}

fn build_url(cfg: &Config, token: &str) -> String {
    let bind = cfg.bind.clone().unwrap_or_else(|| "0.0.0.0:8080".into());
    let scheme = if tls_enabled(cfg) { "https" } else { "http" };
    format!("{scheme}://{}/add#t={token}", display_authority(&bind))
}

fn tls_enabled(cfg: &Config) -> bool {
    cfg.tls_cert.is_some() && cfg.tls_key.is_some()
}

/// Compose the URL the boot banner shows. Used by main.rs.
#[cfg(feature = "embed-ui")]
pub fn url_for_banner(bind: &str, tls: bool, token: &str) -> String {
    let scheme = if tls { "https" } else { "http" };
    format!("{scheme}://{}/add#t={token}", display_authority(bind))
}

/// Substitute hostname for unspecified bind addresses so the printed URL
/// is usable on the LAN. Specific binds pass through unchanged.
pub fn display_authority(bind: &str) -> String {
    let (host, port) = bind.rsplit_once(':').unwrap_or((bind, ""));
    let host = match host.trim_matches(['[', ']']) {
        "0.0.0.0" | "::" | "" => super::hostname(),
        h => h.to_string(),
    };
    if port.is_empty() {
        host
    } else {
        format!("{host}:{port}")
    }
}

/// Render the connect URL as a terminal QR. Phones scan with their
/// camera and land directly on /add#t=…, sidestepping the need to type
/// or paste a 270-char JWT. Colors are inverted for dark terminals
/// (the common case for ssh / journalctl).
#[cfg(feature = "embed-ui")]
pub fn render_qr(url: &str) {
    use qrcode::render::unicode::Dense1x2;
    use qrcode::{EcLevel, QrCode};
    match QrCode::with_error_correction_level(url.as_bytes(), EcLevel::L) {
        Ok(code) => {
            let image = code
                .render::<Dense1x2>()
                .dark_color(Dense1x2::Light)
                .light_color(Dense1x2::Dark)
                .build();
            println!("\n{image}");
        }
        Err(e) => tracing::warn!("could not render connect QR: {e}"),
    }
}

/// Stub used when `embed-ui` is off — nub-without-UI has no /add path,
/// so the QR would point nowhere. Tell the user instead of silently
/// emitting nothing.
#[cfg(not(feature = "embed-ui"))]
pub fn render_qr(_url: &str) {
    eprintln!("warning: this nub build has no embedded UI; the QR/URL would point at a non-existent /add path.");
}
