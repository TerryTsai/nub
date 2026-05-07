mod auth;
mod cli;
mod client;
mod compose;
mod config;
mod ops;
mod proto;
mod server;

use anyhow::{Context, Result};
use clap::Parser;
use std::os::unix::fs::OpenOptionsExt as _;
use std::sync::Arc;
use tokio_rustls::rustls;
use tracing::Level;

use auth::{jwt, AuthState, Issuer};
use cli::{Args, Cmd};

fn main() -> Result<()> {
    let mut args = Args::parse();
    // `arg_required_else_help` ensures bare `nub` printed help and exited
    // before this point, so we always have a subcommand here.
    let cmd = args.cmd.take().expect("clap requires a subcommand");
    if matches!(cmd, Cmd::Run) {
        init_tracing()?;
        return tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?
            .block_on(serve(args));
    }
    // Other CLI subcommands skip tracing init so server-level INFO lines
    // (engine connect, etc.) don't leak into command output.
    cli::dispatch(cmd)
}

// tracing::{info,error} macros expand enough that this small orchestration
// fn crosses cognitive_complexity. The function is sequential setup; allow.
#[allow(clippy::cognitive_complexity)]
async fn serve(args: Args) -> Result<()> {
    let cfg = resolve_config(&args)?;
    let id = cfg.id.clone().unwrap_or_else(cli::hostname);
    let listen = cfg.listen.clone().unwrap_or_else(|| "0.0.0.0:8080".into());
    let tls = resolve_tls(&cfg)?;

    let issuer = Arc::new(resolve_issuer(&cfg)?);
    let admin = ensure_admin_token(&issuer, &id)?;

    println!("issuer key:  ed25519:{}", issuer.public_key_b64());
    println!("admin token: {admin}");
    #[cfg(feature = "embed-ui")]
    {
        let url = cli::connect::url_for_banner(&listen, tls.is_some(), &admin);
        println!("connect:     {url}");
        cli::connect::render_qr(&url);
    }

    let policy = ops::Policy::from_config(&cfg);
    // Re-materialize secret tmpfs files for every stack on disk before
    // we start serving. /run/nub/secrets/ is wiped by reboot; without
    // this, containers with `secrets:` references would fail to start
    // when the engine restart-policy kicks in.
    ops::stacks::rehydrate::rehydrate_all(&policy.stacks_root, &policy.secrets_root).await;

    let app = build_app(policy, id.clone(), Arc::clone(&issuer)).await?;
    let listener = tokio::net::TcpListener::bind(&listen).await?;
    let scheme = if tls.is_some() { "https" } else { "http" };
    tracing::info!("nub {id} listening on {listen} ({scheme})");
    if let Some(tls) = tls {
        if let Err(e) = server::tls::serve(listener, app, tls).await {
            tracing::error!("tls serve failed: {e}");
        }
    } else if let Err(e) = axum::serve(listener, app).await {
        tracing::error!("axum serve failed: {e}");
    }
    Ok(())
}

/// Either load nub's auto-managed keypair from disk (auto-generating
/// on first run) or build a verify-only issuer from the configured
/// `trusted_issuer` public key.
fn resolve_issuer(cfg: &config::Config) -> Result<Issuer> {
    if let Some(b64) = &cfg.trusted_issuer {
        return Issuer::from_public_key_b64(b64);
    }
    Issuer::load_or_generate(&config::default_issuer_key())
}

/// Persist a long-lived admin JWT so it survives restarts. The admin
/// token is just a normal minted token — `sub=admin`, `scope=*`, with a
/// long TTL — that nub mints to itself on first run.
///
/// In external-issuer mode we can't mint, so there's no admin token;
/// the operator brings their own.
fn ensure_admin_token(issuer: &Issuer, host_id: &str) -> Result<String> {
    if !issuer.can_mint() {
        return Ok(String::from("(no auto-admin: trusted_issuer is set; mint your own)"));
    }
    let path = config::default_admin_jwt();
    if path.exists() {
        return std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))
            .map(|s| s.trim().to_string());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    let now = jwt::current_unix_seconds();
    // 10 years. Effectively "until you rotate the issuer key."
    let ten_years = 10 * 365 * 86400;
    let claims = jwt::Claims {
        iss: "nub".into(),
        sub: "admin".into(),
        aud: host_id.to_string(),
        exp: now + ten_years,
        nbf: now,
        iat: now,
        scope: "*".into(),
    };
    let token = jwt::encode(&claims, issuer)?;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&path)
        .with_context(|| format!("creating {}", path.display()))?;
    std::io::Write::write_all(&mut f, token.as_bytes())?;
    Ok(token)
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

async fn build_app(policy: ops::Policy, id: String, issuer: Arc<Issuer>) -> Result<axum::Router> {
    let handler: Arc<dyn ops::OpHandler> = Arc::new(ops::EngineHandler::connect(policy).await?);

    let auth = Arc::new(AuthState { issuer, audience: id });
    let mut app = server::router(handler, auth);
    if let Some(ui) = server::ui::ui_fallback() {
        app = app.merge(ui);
    }
    Ok(app)
}

fn resolve_config(args: &Args) -> Result<config::Config> {
    let mut cfg = config::Config::load(args.config.as_deref())?.unwrap_or_default();
    if let Some(v) = &args.id {
        cfg.id = Some(v.clone());
    }
    if let Some(v) = &args.listen {
        cfg.listen = Some(v.clone());
    }
    if let Some(v) = &args.tls_cert {
        cfg.tls_cert = Some(v.clone());
    }
    if let Some(v) = &args.tls_key {
        cfg.tls_key = Some(v.clone());
    }
    Ok(cfg)
}

fn init_tracing() -> Result<()> {
    // Fixed INFO floor for nub's own logs; we don't need per-module filtering
    // for a daemon with one log namespace, and dropping `env-filter` saves
    // ~180 KB of regex machinery from the binary.
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    Ok(())
}
