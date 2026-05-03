mod auth;
mod cli;
mod client;
mod config;
mod ops;
mod proto;
mod server;

use anyhow::{Context, Result};
use clap::Parser;
use config::TrustEntry;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use tokio_rustls::rustls;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "nub", about = "Minimal Docker/Podman control plane")]
struct Args {
    /// Path to TOML config (default: $XDG_CONFIG_HOME/nub/nub.toml, ./nub.toml, /etc/nub/config.toml)
    #[arg(long)]
    config: Option<PathBuf>,
    /// This binary's identifier (default: /etc/hostname or "nub")
    #[arg(long)]
    id: Option<String>,
    /// Address to listen on (default: 0.0.0.0:8080)
    #[arg(long)]
    bind: Option<String>,
    /// TLS certificate path
    #[arg(long)]
    tls_cert: Option<PathBuf>,
    /// TLS private key path
    #[arg(long)]
    tls_key: Option<PathBuf>,

    #[command(subcommand)]
    cmd: Option<cli::Cmd>,
}

fn main() -> Result<()> {
    init_tracing()?;
    let args = Args::parse();
    if let Some(cmd) = args.cmd {
        return cli::dispatch(cmd);
    }
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(serve(args))
}

// tracing::{info,error} macros expand enough that this small orchestration
// fn crosses cognitive_complexity. The function is sequential setup; allow.
#[allow(clippy::cognitive_complexity)]
async fn serve(args: Args) -> Result<()> {
    let cfg = resolve_config(&args)?;
    let id = cfg.id.clone().unwrap_or_else(cli::hostname);
    let bind = cfg.bind.clone().unwrap_or_else(|| "0.0.0.0:8080".into());
    let tls = resolve_tls(&cfg)?;
    let admin = admin_entry()?;
    println!(
        "admin token: {}   (regenerates each restart, allows everything)",
        admin.token
    );
    if cfg!(feature = "embed-ui") {
        let scheme = if tls.is_some() { "https" } else { "http" };
        println!(
            "connect:     {scheme}://{}/add#t={}",
            display_authority(&bind),
            admin.token
        );
    }
    let app = build_app(cfg, admin).await?;
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    let scheme = if tls.is_some() { "https" } else { "http" };
    tracing::info!("nub {id} listening on {bind} ({scheme})");
    if let Some(tls) = tls {
        if let Err(e) = server::tls::serve(listener, app, tls).await {
            tracing::error!("tls serve failed: {e}");
        }
    } else if let Err(e) = axum::serve(listener, app).await {
        tracing::error!("axum serve failed: {e}");
    }
    Ok(())
}

/// Loads the TLS config when both cert and key paths are set. Half-set
/// is a misconfiguration — fail loudly rather than silently drop to
/// plaintext, since users would think TLS was on.
fn resolve_tls(cfg: &config::Config) -> Result<Option<Arc<rustls::ServerConfig>>> {
    match (&cfg.tls_cert, &cfg.tls_key) {
        (Some(cert), Some(key)) => Ok(Some(server::tls::load_config(cert, key)?)),
        (None, None) => Ok(None),
        _ => Err(anyhow::anyhow!(
            "tls_cert and tls_key must be set together (got one of two)"
        )),
    }
}

async fn build_app(cfg: config::Config, admin: TrustEntry) -> Result<axum::Router> {
    let policy = ops::Policy {
        allowed_binds: cfg.allowed_binds,
        dockerfiles_root: cfg.dockerfiles.unwrap_or_else(config::default_dockerfiles_dir),
    };
    let handler: Arc<dyn ops::OpHandler> = Arc::new(ops::EngineHandler::connect(policy).await?);

    let mut trust = vec![admin];
    trust.extend(cfg.trust);
    let auth = Arc::new(auth::AuthState { trust });
    let mut app = server::router(handler, auth);
    if let Some(ui) = server::ui::ui_fallback() {
        app = app.merge(ui);
    }
    Ok(app)
}

// Substitute hostname for unspecified bind addresses so the printed URL is
// usable on the LAN. Specific binds pass through unchanged.
fn display_authority(bind: &str) -> String {
    let (host, port) = bind.rsplit_once(':').unwrap_or((bind, ""));
    let host = match host.trim_matches(['[', ']']) {
        "0.0.0.0" | "::" | "" => cli::hostname(),
        h => h.to_string(),
    };
    if port.is_empty() {
        host
    } else {
        format!("{host}:{port}")
    }
}

fn resolve_config(args: &Args) -> Result<config::Config> {
    let mut cfg = config::Config::load(args.config.as_deref())?.unwrap_or_default();
    if let Some(v) = &args.id {
        cfg.id = Some(v.clone());
    }
    if let Some(v) = &args.bind {
        cfg.bind = Some(v.clone());
    }
    if let Some(v) = &args.tls_cert {
        cfg.tls_cert = Some(v.clone());
    }
    if let Some(v) = &args.tls_key {
        cfg.tls_key = Some(v.clone());
    }
    Ok(cfg)
}

fn admin_entry() -> Result<TrustEntry> {
    let mut buf = [0u8; 16];
    std::fs::File::open("/dev/urandom")
        .context("opening /dev/urandom")?
        .read_exact(&mut buf)
        .context("reading /dev/urandom")?;
    let token = buf.iter().map(|b| format!("{b:02x}")).collect();
    Ok(TrustEntry {
        id: "admin".into(),
        token,
        allowed: vec!["*".into()],
    })
}

fn init_tracing() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("nub=info".parse()?))
        .init();
    Ok(())
}
