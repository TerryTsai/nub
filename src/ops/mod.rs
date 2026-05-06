//! Op handlers — one file per op family. Each op owns its full pipeline:
//! build the engine path, send, decode JSON or stream, return proto types.
//! No middle layer between proto and the socket.

pub mod configs;
mod containers;
mod dockerfiles;
mod host;
mod images;
mod networks;
pub mod secrets;
mod serde_util;
pub mod stacks;
mod volumes;

use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use futures::stream::BoxStream;
use std::future::Future;
use tokio::sync::mpsc;

use crate::auth::Claims;
use crate::client::Engine;
use crate::proto::*;

// ---- Public surface ------------------------------------------------------

pub enum HandlerOutput {
    Unary(OpResult),
    Stream(BoxStream<'static, StreamChunk>),
}

#[async_trait]
pub trait OpHandler: Send + Sync + 'static {
    async fn handle(&self, op: Op, claims: &Claims, input: mpsc::Receiver<StreamChunk>) -> HandlerOutput;
}

/// A pre-closed receiver. Use for transports that don't carry client→server
/// stream chunks (e.g. unary HTTP /api/op).
pub fn closed_input() -> mpsc::Receiver<StreamChunk> {
    let (_tx, rx) = mpsc::channel(1);
    rx
}

/// Security policy applied at op boundaries. Currently constrains
/// container creation and locates the dockerfiles / stacks / secrets
/// roots.
pub struct Policy {
    /// Host paths permitted as bind-mount sources in CreateContainer.
    /// Empty = no host bind mounts allowed.
    pub allowed_binds: Vec<PathBuf>,
    /// Flat directory holding Dockerfile text files. Always set — the
    /// caller resolves it (config override or XDG default).
    pub dockerfiles_root: PathBuf,
    /// Directory holding compose-stack manifests, one subdir per stack.
    /// Always set — caller resolves config override or XDG default.
    pub stacks_root: PathBuf,
    /// Directory holding age-encrypted secrets and the per-host
    /// encryption identity. Always set — caller resolves config
    /// override or XDG default.
    pub secrets_root: PathBuf,
}

pub struct EngineHandler {
    pub(crate) engine: Engine,
    pub(crate) policy: Policy,
}

impl EngineHandler {
    pub async fn connect(policy: Policy) -> Result<Self> {
        Ok(Self {
            engine: Engine::connect().await?,
            policy,
        })
    }
}

// ---- Dispatch ------------------------------------------------------------

#[async_trait]
impl OpHandler for EngineHandler {
    #[allow(clippy::too_many_lines)]
    async fn handle(&self, op: Op, claims: &Claims, input: mpsc::Receiver<StreamChunk>) -> HandlerOutput {
        match op {
            // Whoami is auth-layer info; transport short-circuits before us.
            Op::Whoami => unreachable!("Whoami handled by transport layer"),

            Op::HostInfo => unary(host::run(self).await, OpResult::HostInfo),

            Op::ListContainers { all } => unary(containers::list::run(self, all).await, OpResult::Containers),
            Op::GetContainer { id } => unary(containers::inspect::run(self, id).await, OpResult::ContainerDetail),
            Op::StartContainer { id } => unary(containers::action::start(self, id).await, ok),
            Op::StopContainer { id, timeout } => unary(containers::action::stop(self, id, timeout).await, ok),
            Op::RestartContainer { id, timeout } => unary(containers::action::restart(self, id, timeout).await, ok),
            Op::KillContainer { id, signal } => unary(containers::action::kill(self, id, signal).await, ok),
            Op::RemoveContainer { id, force } => unary(containers::action::remove(self, id, force).await, ok),
            Op::CreateContainer(req) => unary(containers::create::run(self, *req).await, OpResult::ContainerCreated),
            Op::StreamLogs { id, follow, tail } => stream(containers::logs::run(self, id, follow, tail)),
            Op::StreamStats { id } => stream(containers::stats::run(self, id)),
            Op::Exec { id, cmd, tty } => stream(containers::exec::run(self, id, cmd, tty, input)),

            Op::ListImages => unary(images::list::run(self).await, OpResult::Images),
            Op::GetImage { id } => unary(images::inspect::run(self, id).await, OpResult::ImageDetail),
            Op::DeleteImage { id } => unary(images::remove::run(self, id).await, ok),
            Op::PullImage { reference } => stream(images::pull::run(self, reference)),
            Op::BuildImage {
                dockerfile_content,
                tag,
                build_args,
            } => stream(images::build::run(self, dockerfile_content, tag, build_args)),

            Op::ListVolumes => unary(volumes::list(self).await, OpResult::Volumes),
            Op::GetVolume { name } => unary(volumes::inspect(self, &name).await, OpResult::VolumeDetail),
            Op::CreateVolume {
                name,
                driver,
                labels,
                options,
            } => unary(volumes::create(self, name, driver, labels, options).await, ok),
            Op::DeleteVolume { name } => unary(volumes::remove(self, name).await, ok),

            Op::ListNetworks => unary(networks::list(self).await, OpResult::Networks),
            Op::GetNetwork { id } => unary(networks::inspect(self, &id).await, OpResult::NetworkDetail),
            Op::CreateNetwork { name, internal } => unary(networks::create(self, name, internal).await, ok),
            Op::DeleteNetwork { id } => unary(networks::remove(self, id).await, ok),

            Op::ListDockerfiles => unary(dockerfiles::list(self).await, OpResult::Dockerfiles),
            Op::GetDockerfile { name } => unary(dockerfiles::read(self, &name).await, OpResult::Dockerfile),
            Op::PutDockerfile { name, content } => unary(dockerfiles::write(self, &name, &content).await, ok),
            Op::DeleteDockerfile { name } => unary(dockerfiles::delete(self, &name).await, ok),

            Op::CreateStack { name, yaml } => unary(
                stacks::create::run(self, claims, name, yaml).await,
                OpResult::StackCreated,
            ),
            Op::ListStacks => unary(stacks::list::run(self).await, OpResult::Stacks),
            Op::GetStack { name } => unary(stacks::get::run(self, name).await, OpResult::StackDetail),
            Op::DeleteStack { name } => unary(stacks::delete::run(self, claims, name).await, ok),
            Op::RedeployStack { name } => {
                unary(stacks::redeploy::run(self, claims, name).await, OpResult::StackCreated)
            }
            Op::UpdateStack { name, yaml } => unary(
                stacks::update::run(self, claims, name, yaml).await,
                OpResult::StackCreated,
            ),
            Op::PullStack { name } => unary(stacks::pull::run(self, claims, name).await, OpResult::StackCreated),
            Op::StreamStackLogs { name, follow, tail } => stream(stacks::logs::run(self, name, follow, tail)),

            Op::ListSecrets => unary(secrets::list(&self.policy.secrets_root).await, OpResult::Secrets),
            Op::PutSecret { name, value } => unary(secrets::put(&self.policy.secrets_root, &name, &value).await, ok),
            Op::DeleteSecret { name } => unary(secrets::delete(&self.policy.secrets_root, &name).await, ok),
            Op::GetSecret { name } => unary(secrets::get(&self.policy.secrets_root, &name).await, OpResult::Secret),
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
