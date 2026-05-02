//! Direct TLS via file-based PEM cert + key. No ACME, no self-signed
//! convenience, no hot-reload — those are out of scope on purpose. The
//! cert and key are loaded once at startup; rotate by restarting nub.

use std::path::Path;
use std::sync::Arc;

use anyhow::{anyhow, bail, Context, Result};
use axum::Router;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use hyper_util::service::TowerToHyperService;
use tokio::net::TcpListener;
use tokio_rustls::rustls::pki_types::PrivateKeyDer;
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;

/// Build a rustls `ServerConfig` from PEM files. Fails loudly on missing
/// files, empty cert chains, missing keys, or mismatched cert/key pairs.
pub fn load_config(cert: &Path, key: &Path) -> Result<Arc<ServerConfig>> {
    install_crypto_provider_once();
    let cert_bytes = std::fs::read(cert).with_context(|| format!("reading TLS cert {}", cert.display()))?;
    let key_bytes = std::fs::read(key).with_context(|| format!("reading TLS key {}", key.display()))?;
    let certs = rustls_pemfile::certs(&mut cert_bytes.as_slice())
        .collect::<std::io::Result<Vec<_>>>()
        .with_context(|| format!("parsing certs from {}", cert.display()))?;
    if certs.is_empty() {
        bail!("no certificates found in {}", cert.display());
    }
    let key: PrivateKeyDer<'static> = rustls_pemfile::private_key(&mut key_bytes.as_slice())
        .with_context(|| format!("parsing private key from {}", key.display()))?
        .ok_or_else(|| anyhow!("no private key found in {}", key.display()))?;
    let cfg = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| anyhow!("building TLS config: {e}"))?;
    Ok(Arc::new(cfg))
}

/// Accept loop that handshakes each connection with TLS, then hands the
/// stream to hyper-util's auto builder so the existing axum router handles
/// HTTP/1.1 + WebSocket upgrades the same way `axum::serve` does.
pub async fn serve(listener: TcpListener, app: Router, tls: Arc<ServerConfig>) -> Result<()> {
    let acceptor = TlsAcceptor::from(tls);
    loop {
        let (stream, peer) = match listener.accept().await {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!("accept failed: {e}");
                continue;
            }
        };
        let acceptor = acceptor.clone();
        let app = app.clone();
        tokio::spawn(async move {
            let stream = match acceptor.accept(stream).await {
                Ok(s) => s,
                Err(e) => {
                    tracing::debug!("tls handshake failed for {peer}: {e}");
                    return;
                }
            };
            let svc = TowerToHyperService::new(app);
            if let Err(e) = Builder::new(TokioExecutor::new())
                .serve_connection_with_upgrades(TokioIo::new(stream), svc)
                .await
            {
                tracing::debug!("conn {peer} closed: {e}");
            }
        });
    }
}

/// rustls 0.23 requires a crypto provider be installed before use. We pick
/// `ring` for static-musl friendliness (pure Rust + asm; no CMake). Idempotent —
/// subsequent calls noop, so callers don't have to coordinate.
fn install_crypto_provider_once() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let _ = tokio_rustls::rustls::crypto::ring::default_provider().install_default();
    });
}
