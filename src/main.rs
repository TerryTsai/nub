mod auth;
mod config;
mod handler;
mod http;
mod hub;
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
#[command(name = "nub", about = "Minimal Docker/Podman control plane: hub + nub")]
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

    // Pure-hub deployments don't talk to a local Docker socket.
    let needs_docker = cfg.bind.is_some() || cfg.nub.is_some();
    let docker: Option<Arc<dyn handler::OpHandler>> = if needs_docker {
        let policy = handler::Policy {
            allowed_binds: cfg.allowed_binds.clone(),
        };
        Some(Arc::new(handler::DockerHandler::connect(policy)?))
    } else {
        None
    };

    let mut tasks: JoinSet<()> = JoinSet::new();
    if let Some(bind) = cfg.bind {
        let token = cfg
            .token
            .ok_or_else(|| anyhow!("`token` required when `bind` is set"))?;
        let auth = Arc::new(auth::AuthState { token });
        let app = http::router(docker.clone().unwrap(), auth);
        let listener = tokio::net::TcpListener::bind(&bind).await?;
        tracing::info!("hubnub listening on {bind}");
        tasks.spawn(serve_http(listener, app));
    }
    if let Some(nub) = cfg.nub {
        tracing::info!(hub = %nub.hub_url, "nub: dialing hub");
        tasks.spawn(hub_client::run(docker.clone().unwrap(), nub));
    }
    if let Some(hub) = cfg.hub {
        tasks.spawn(serve_hub(hub));
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

async fn serve_hub(cfg: hub::Config) {
    if let Err(e) = hub::run(cfg).await {
        tracing::error!("hub failed: {e}");
    }
}
