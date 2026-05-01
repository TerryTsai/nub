mod auth;
mod config;
mod handler;
mod http;
mod proto;
mod ws;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
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
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("nub=info".parse()?))
        .init();

    let args = Args::parse();
    let cfg = config::Config::load(args.config.as_deref())?;

    if cfg.tls_cert.is_some() || cfg.tls_key.is_some() {
        tracing::warn!(
            "tls_cert/tls_key set in config but TLS is not yet wired; serving plaintext"
        );
    }

    let policy = handler::Policy {
        allowed_binds: cfg.allowed_binds.clone(),
    };
    let handler = Arc::new(handler::DockerHandler::connect(policy)?);
    let auth = Arc::new(auth::AuthState {
        token: cfg.token.clone(),
    });
    let app = http::router(handler, auth);

    let listener = tokio::net::TcpListener::bind(&cfg.bind).await?;
    tracing::info!("nub listening on {}", cfg.bind);
    axum::serve(listener, app).await?;
    Ok(())
}
