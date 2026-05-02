//! Op handlers — one file per op family. Each op owns its full pipeline:
//! build the engine path, send, decode JSON or stream, return proto types.
//! No middle layer between proto and the socket.

mod containers;
mod host;
mod images;
mod networks;
mod volumes;

use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use futures::stream::BoxStream;
use std::future::Future;
use tokio::sync::mpsc;

use crate::client::Engine;
use crate::proto::*;

// ---- Public surface ------------------------------------------------------

pub enum HandlerOutput {
    Unary(OpResult),
    Stream(BoxStream<'static, StreamChunk>),
}

#[async_trait]
pub trait OpHandler: Send + Sync + 'static {
    async fn handle(&self, op: Op, input: mpsc::Receiver<StreamChunk>) -> HandlerOutput;
}

/// A pre-closed receiver. Use for transports that don't carry client→server
/// stream chunks (e.g. unary HTTP /api/op).
pub fn closed_input() -> mpsc::Receiver<StreamChunk> {
    let (_tx, rx) = mpsc::channel(1);
    rx
}

/// Security policy applied at op boundaries. Currently only constrains
/// container creation.
pub struct Policy {
    /// Host paths permitted as bind-mount sources in CreateContainer.
    /// Empty = no host bind mounts allowed.
    pub allowed_binds: Vec<PathBuf>,
}

pub struct EngineHandler {
    pub(crate) engine: Engine,
    pub(crate) policy: Policy,
}

impl EngineHandler {
    pub async fn connect(policy: Policy) -> Result<Self> {
        Ok(Self { engine: Engine::connect().await?, policy })
    }
}

// ---- Dispatch ------------------------------------------------------------

#[async_trait]
impl OpHandler for EngineHandler {
    async fn handle(&self, op: Op, input: mpsc::Receiver<StreamChunk>) -> HandlerOutput {
        match op {
            // Whoami is auth-layer info; transport short-circuits before us.
            Op::Whoami => unreachable!("Whoami handled by transport layer"),

            Op::HostInfo => unary(host::run(self).await, OpResult::HostInfo),

            Op::ListContainers { all } => {
                unary(containers::list::run(self, all).await, OpResult::Containers)
            }
            Op::InspectContainer { id } => {
                unary(containers::inspect::run(self, id).await, OpResult::ContainerDetail)
            }
            Op::ContainerAction { id, action } => {
                unary(containers::action::run(self, id, action).await, ok)
            }
            Op::CreateContainer(req) => {
                unary(containers::create::run(self, *req).await, OpResult::ContainerCreated)
            }
            Op::StreamLogs { id, follow, tail } => {
                stream(containers::logs::run(self, id, follow, tail))
            }
            Op::StreamStats { id } => stream(containers::stats::run(self, id)),
            Op::Exec { id, cmd, tty } => stream(containers::exec::run(self, id, cmd, tty, input)),

            Op::ListImages => unary(images::list::run(self).await, OpResult::Images),
            Op::RemoveImage { id, force } => unary(images::remove::run(self, id, force).await, ok),
            Op::PullImage { reference } => stream(images::pull::run(self, reference)),

            Op::ListVolumes => unary(volumes::list(self).await, OpResult::Volumes),
            Op::RemoveVolume { name, force } => unary(volumes::remove(self, name, force).await, ok),

            Op::ListNetworks => unary(networks::list(self).await, OpResult::Networks),
            Op::RemoveNetwork { id } => unary(networks::remove(self, id).await, ok),
        }
    }
}

// ---- Result/stream helpers used by the dispatch and op modules -----------

pub(crate) fn unary<T>(r: anyhow::Result<T>, into: impl FnOnce(T) -> OpResult) -> HandlerOutput {
    HandlerOutput::Unary(match r {
        Ok(v) => into(v),
        Err(e) => OpResult::Err { message: e.to_string() },
    })
}

pub(crate) fn stream(s: BoxStream<'static, StreamChunk>) -> HandlerOutput {
    HandlerOutput::Stream(s)
}

pub(crate) fn ok(_: ()) -> OpResult {
    OpResult::Ok
}

/// Spawn `produce` and forward its emitted chunks. Appends an `End` chunk
/// when the producer returns. The returned BoxStream owns the receiving end.
pub(crate) fn spawn_chunked<F, Fut>(produce: F) -> BoxStream<'static, StreamChunk>
where
    F: FnOnce(mpsc::Sender<StreamChunk>) -> Fut + Send + 'static,
    Fut: Future<Output = std::result::Result<(), String>> + Send + 'static,
{
    let (tx, rx) = mpsc::channel::<StreamChunk>(32);
    let inner = tx.clone();
    tokio::spawn(async move {
        let (ok, err) = match produce(inner).await {
            Ok(()) => (true, None),
            Err(e) => (false, Some(e)),
        };
        let _ = tx.send(StreamChunk::End { ok, err }).await;
    });
    Box::pin(futures::stream::unfold(rx, |mut rx| async move {
        rx.recv().await.map(|c| (c, rx))
    }))
}

pub(crate) fn log_chunk(stderr: bool, data: &[u8]) -> StreamChunk {
    StreamChunk::Log {
        stderr,
        data: String::from_utf8_lossy(data).into_owned(),
    }
}
