mod auth;
mod config;
mod handler;
mod http;
mod proto;
mod ui;
mod wire;
mod ws;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use config::TrustEntry;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "nub", about = "Minimal Docker/Podman control plane")]
struct Args {
    /// Path to TOML config (default: ./nub.toml or /etc/nub/config.toml; optional)
    #[arg(long)]
    config: Option<PathBuf>,
    /// This binary's identifier
    #[arg(long)]
    id: Option<String>,
    /// Address to listen on (e.g. 127.0.0.1:8080)
    #[arg(long)]
    bind: Option<String>,
    /// TLS certificate path
    #[arg(long)]
    tls_cert: Option<PathBuf>,
    /// TLS private key path
    #[arg(long)]
    tls_key: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing()?;
    let args = Args::parse();
    let mut cfg = config::Config::load(args.config.as_deref())?.unwrap_or_default();

    // CLI overrides file.
    if let Some(v) = args.id {
        cfg.id = Some(v);
    }
    if let Some(v) = args.bind {
        cfg.bind = Some(v);
    }
    if let Some(v) = args.tls_cert {
        cfg.tls_cert = Some(v);
    }
    if let Some(v) = args.tls_key {
        cfg.tls_key = Some(v);
    }

    let id = cfg.id.ok_or_else(|| anyhow!("`id` required (config or --id)"))?;
    let bind = cfg.bind.ok_or_else(|| anyhow!("`bind` required (config or --bind)"))?;
    if cfg.tls_cert.is_some() || cfg.tls_key.is_some() {
        tracing::warn!("tls_cert/tls_key set but TLS is not yet wired; serving plaintext");
    }

    let policy = handler::Policy {
        allowed_binds: cfg.engine.allowed_binds,
    };
    let handler: Arc<dyn handler::OpHandler> = Arc::new(handler::DockerHandler::connect(policy)?);

    let admin = admin_entry()?;
    println!(
        "admin token: {}   (regenerates each restart, allows everything)",
        admin.token
    );
    let mut trust = vec![admin];
    trust.extend(cfg.trust);
    let auth = Arc::new(auth::AuthState { trust });
    let mut app = http::router(handler, auth);
    if let Some(ui) = ui::ui_fallback() {
        app = app.merge(ui);
    }

    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!("nub {id} listening on {bind}");
    if let Err(e) = axum::serve(listener, app).await {
        tracing::error!("axum serve failed: {e}");
    }
    Ok(())
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
