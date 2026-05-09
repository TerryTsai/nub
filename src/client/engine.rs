//! Engine handle — connect-time socket discovery, podman/docker
//! probe, and the cheap `Engine` type that holds the resolved
//! address. Each call opens a fresh `Conn`; no pooling.

use std::path::PathBuf;

use super::conn::{Address, Conn};
use super::req::Req;

/// Which engine the socket belongs to. Detected at connect time. Affects
/// which paths an op file uses (libpod vs compat).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineKind {
    Docker,
    Podman,
}

/// Connection handle. Cheap to clone — holds only configuration. Each call
/// opens a fresh socket; no pooling.
#[derive(Debug, Clone)]
pub struct Engine {
    address: Address,
    kind: EngineKind,
}

#[derive(Debug)]
pub enum Error {
    /// IO error talking to the engine socket.
    Transport(String),
    /// Engine returned non-2xx. `message` is the engine's own error text
    /// when it provided one, otherwise the raw body.
    Status { code: u16, message: String },
    /// Response wasn't shaped the way we expected (decode failure).
    Decode(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Engine {
    /// Probe the standard socket paths (or `DOCKER_HOST`), then sniff the
    /// engine kind via libpod's `_ping` endpoint.
    pub async fn connect() -> anyhow::Result<Self> {
        let address = resolve_address()?;
        let kind = detect_kind(&address).await;
        tracing::info!("connected to {address:?} as {kind:?}");
        Ok(Self { address, kind })
    }

    pub fn kind(&self) -> EngineKind {
        self.kind
    }

    /// Human-readable address for diagnostics (`unix:///run/.../podman.sock`,
    /// `tcp://localhost:2375`). Used by `nub status`.
    pub fn address_display(&self) -> String {
        format!("{}", self.address)
    }

    /// Open a fresh connection. Each op holds its own — no pooling.
    pub async fn conn(&self) -> Result<Conn> {
        Conn::connect(&self.address).await
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Transport(m) => write!(f, "transport: {m}"),
            Error::Status { code, message } => write!(f, "engine returned {code}: {message}"),
            Error::Decode(m) => write!(f, "decode: {m}"),
        }
    }
}

impl std::error::Error for Error {}

fn resolve_address() -> anyhow::Result<Address> {
    if let Some(host) = std::env::var_os("DOCKER_HOST") {
        return parse_docker_host(&host.to_string_lossy());
    }
    for path in candidate_sockets() {
        if path.exists() {
            return Ok(Address::Unix(path));
        }
    }
    anyhow::bail!(
        "no docker or podman socket found.\n\n\
         if podman is installed, enable its socket (it's daemonless and not started by default):\n  \
         rootless: systemctl --user enable --now podman.socket\n  \
         rootful:  sudo systemctl enable --now podman.socket\n\n\
         if docker is installed, ensure the daemon is running.\n\n\
         override with DOCKER_HOST."
    )
}

fn parse_docker_host(s: &str) -> anyhow::Result<Address> {
    if let Some(rest) = s.strip_prefix("unix://") {
        return Ok(Address::Unix(PathBuf::from(rest)));
    }
    if let Some(rest) = s.strip_prefix("tcp://") {
        return Ok(Address::Tcp(rest.to_string()));
    }
    anyhow::bail!("unsupported DOCKER_HOST scheme: {s}")
}

fn candidate_sockets() -> Vec<PathBuf> {
    let mut out = Vec::with_capacity(4);
    if let Some(xdg) = std::env::var_os("XDG_RUNTIME_DIR") {
        let xdg = PathBuf::from(xdg);
        out.push(xdg.join("docker.sock"));
        out.push(xdg.join("podman/podman.sock"));
    }
    out.push(PathBuf::from("/var/run/docker.sock"));
    out.push(PathBuf::from("/run/podman/podman.sock"));
    out
}

/// 200 from libpod ping = Podman. Anything else (404, error) = Docker. The
/// libpod endpoint is podman-only and unversioned, so the probe is cheap.
async fn detect_kind(address: &Address) -> EngineKind {
    let Ok(mut conn) = Conn::connect(address).await else {
        return EngineKind::Docker;
    };
    match conn.send_unary(Req::get("/libpod/_ping")).await {
        Ok(r) if r.status.is_success() => EngineKind::Podman,
        _ => EngineKind::Docker,
    }
}
