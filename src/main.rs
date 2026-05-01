mod auth;
mod config;
mod handler;
mod http;
mod hub_client;
mod proto;
mod wire;
mod ws;

use anyhow::{anyhow, Result};
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::task::JoinSet;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "nub", about = "Minimal Docker/Podman control plane node")]
struct Args {
    /// Path to TOML config (default: ./nub.toml or /etc/nub/config.toml)
    #[arg(long)]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing()?;
    let cfg = config::Config::load(Args::parse().config.as_deref())?;
    if cfg.tls_cert.is_some() || cfg.tls_key.is_some() {
        tracing::warn!("tls_cert/tls_key set but TLS is not yet wired; serving plaintext");
    }

    let policy = handler::Policy {
        allowed_binds: cfg.allowed_binds,
    };
    let handler: Arc<dyn handler::OpHandler> = Arc::new(handler::DockerHandler::connect(policy)?);

    let mut tasks: JoinSet<()> = JoinSet::new();
    if let Some(bind) = cfg.bind {
        let token = cfg
            .token
            .ok_or_else(|| anyhow!("`token` required when `bind` is set"))?;
        let auth = Arc::new(auth::AuthState { token });
        let app = http::router(handler.clone(), auth);
        let listener = tokio::net::TcpListener::bind(&bind).await?;
        tracing::info!("nub listening on {bind}");
        tasks.spawn(serve_http(listener, app));
    }
    if let Some(hub) = cfg.hub {
        tracing::info!(url = %hub.url, "hub: dialing");
        tasks.spawn(hub_client::run(handler.clone(), hub));
    }
    while tasks.join_next().await.is_some() {}
    Ok(())
}

fn init_tracing() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("nub=info".parse()?))
        .init();
    Ok(())
}

async fn serve_http(listener: tokio::net::TcpListener, app: axum::Router) {
    if let Err(e) = axum::serve(listener, app).await {
        tracing::error!("axum serve failed: {e}");
    }
}
