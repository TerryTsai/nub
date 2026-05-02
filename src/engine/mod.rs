//! Standalone container engine client. Talks HTTP/1.1 directly to a Docker
//! or Podman socket — no bollard. Each operation is a typed method returning
//! either a value or an `impl Stream`. Engine differences (compat vs libpod
//! paths, response shape variations) are handled internally; callers see one
//! API.

mod containers;
mod exec;
mod host;
mod http;
mod images;
mod logs;
mod networks;
mod stats;
mod util;
mod volumes;

use std::path::PathBuf;

use http::Address;

// Public types — callers reference these by name (handler/* and downstream).
// Types that are only ever return-positioned (HostInfo, ContainerCreated,
// PullProgress, LogChunk, Stats, ExecStream) live in their submodules and
// don't need to be re-exported here.
pub use containers::{
    ContainerAction, ContainerDetail, ContainerSummary, CreateContainer, MountPoint,
    NetworkEndpoint, PortBinding, PortMapping, RestartPolicy, VolumeMount,
};
pub use exec::{ExecOptions, ExecOutput, ExecReader, ExecWriter};
pub use images::ImageSummary;
pub use logs::LogsOptions;
pub use networks::NetworkSummary;
pub use volumes::VolumeSummary;

/// Which engine we're talking to. Affects which HTTP paths we hit and how a
/// few response payloads are decoded. Detected at connect time via libpod's
/// `_ping` endpoint — Podman 200s, Docker 404s.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineKind {
    Docker,
    Podman,
}

/// Connection handle. Cheap to clone — holds only configuration. Each method
/// opens a fresh socket connection internally; no pooling.
#[derive(Debug, Clone)]
pub struct Engine {
    address: Address,
    kind: EngineKind,
}

#[derive(Debug)]
pub enum Error {
    /// Network / IO error talking to the engine socket.
    Transport(String),
    /// Engine returned a non-2xx status. `message` is the engine's own error
    /// text when it provided one, otherwise the raw body.
    Status { code: u16, message: String },
    /// Response wasn't shaped the way we expected (decoding failure).
    Decode(String),
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

pub type Result<T> = std::result::Result<T, Error>;

impl Engine {
    /// Connect to the first available engine socket. Honors `DOCKER_HOST` if
    /// set; otherwise probes the standard rootless then rootful paths.
    pub async fn connect() -> anyhow::Result<Self> {
        let address = resolve_address()?;
        let kind = detect_kind(&address).await;
        tracing::info!("engine: connected to {address:?} as {kind:?}");
        Ok(Self { address, kind })
    }

    pub fn kind(&self) -> EngineKind {
        self.kind
    }

    /// Open a fresh connection. Internal helper used by every operation.
    pub(crate) async fn conn(&self) -> Result<Conn> {
        Conn::connect(&self.address).await
    }
}

fn resolve_address() -> anyhow::Result<Address> {
    if let Some(host) = std::env::var_os("DOCKER_HOST") {
        let s = host.to_string_lossy().into_owned();
        if let Some(rest) = s.strip_prefix("unix://") {
            return Ok(Address::Unix(PathBuf::from(rest)));
        }
        if let Some(rest) = s.strip_prefix("tcp://") {
            return Ok(Address::Tcp(rest.to_string()));
        }
        anyhow::bail!("unsupported DOCKER_HOST scheme: {s}");
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

/// Hit the libpod-only `_ping` endpoint. 200 = Podman, anything else = Docker
/// (or older Podman before the endpoint existed, in which case compat paths
/// still work).
async fn detect_kind(address: &Address) -> EngineKind {
    let Ok(mut conn) = Conn::connect(address).await else {
        return EngineKind::Docker;
    };
    let Ok(req) = http::Req::get("/libpod/_ping").build() else {
        return EngineKind::Docker;
    };
    match conn.send_unary(req).await {
        Ok(r) if r.status.is_success() => EngineKind::Podman,
        _ => EngineKind::Docker,
    }
}

// Internal re-exports for sibling modules.
pub(crate) use http::{Conn, Query, Req};
