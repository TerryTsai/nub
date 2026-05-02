mod auth;
mod config;
mod engine;
mod handler;
mod http;
mod init;
mod proto;
mod ui;
mod wire;
mod ws;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use config::TrustEntry;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
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
    cmd: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Generate a starter config file. Default: $XDG_CONFIG_HOME/nub/nub.toml.
    Init {
        /// Where to write. Use `-` for stdout.
        path: Option<String>,
        /// Overwrite if file exists.
        #[arg(long)]
        force: bool,
    },
}

fn main() -> Result<()> {
    init_tracing()?;
    let args = Args::parse();
    if let Some(Cmd::Init { path, force }) = args.cmd {
        return init::run(path, force);
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
    let id = cfg.id.clone().unwrap_or_else(init::hostname);
    let bind = cfg.bind.clone().unwrap_or_else(|| "0.0.0.0:8080".into());
    let admin = admin_entry()?;
    println!(
        "admin token: {}   (regenerates each restart, allows everything)",
        admin.token
    );
    if cfg!(feature = "embed-ui") {
        println!("connect:     http://{}/add#t={}", display_authority(&bind), admin.token);
    }
    let app = build_app(cfg, admin).await?;
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!("nub {id} listening on {bind}");
    if let Err(e) = axum::serve(listener, app).await {
        tracing::error!("axum serve failed: {e}");
    }
    Ok(())
}

async fn build_app(cfg: config::Config, admin: TrustEntry) -> Result<axum::Router> {
    if cfg.tls_cert.is_some() || cfg.tls_key.is_some() {
        tracing::warn!("tls_cert/tls_key set but TLS is not yet wired; serving plaintext");
    }
    let policy = handler::Policy {
        allowed_binds: cfg.engine.allowed_binds,
    };
    let handler: Arc<dyn handler::OpHandler> = Arc::new(handler::EngineHandler::connect(policy).await?);

    let mut trust = vec![admin];
    trust.extend(cfg.trust);
    let auth = Arc::new(auth::AuthState { trust });
    let mut app = http::router(handler, auth);
    if let Some(ui) = ui::ui_fallback() {
        app = app.merge(ui);
    }
    Ok(app)
}

// Substitute hostname for unspecified bind addresses so the printed URL is
// usable on the LAN. Specific binds pass through unchanged.
fn display_authority(bind: &str) -> String {
    let (host, port) = bind.rsplit_once(':').unwrap_or((bind, ""));
    let host = match host.trim_matches(['[', ']']) {
        "0.0.0.0" | "::" | "" => init::hostname(),
        h => h.to_string(),
    };
    if port.is_empty() { host } else { format!("{host}:{port}") }
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
