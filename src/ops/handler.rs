//! `OpHandler` trait and the engine-backed `EngineHandler` dispatch.
//! Each `Op` variant routes to one per-verb function in an `ops/<family>`
//! module; helpers below wrap the result into a `HandlerOutput` so both
//! transports (HTTP unary, WebSocket streamed) share one envelope.

use std::future::Future;

use anyhow::Result;
use async_trait::async_trait;
use futures::stream::BoxStream;
use tokio::sync::mpsc;

use super::policy::Policy;
use super::{containers, dockerfiles, host, images, networks, secrets, stacks, volumes};
use crate::auth::Claims;
use crate::client::Engine;
use crate::proto::{Op, OpResult, StreamChunk};

pub enum HandlerOutput {
    Unary(OpResult),
    Stream(BoxStream<'static, StreamChunk>),
}

#[async_trait]
pub trait OpHandler: Send + Sync + 'static {
    async fn handle(&self, op: Op, claims: &Claims, input: mpsc::Receiver<StreamChunk>) -> HandlerOutput;
}

/// Shared handle to a boxed `OpHandler` — passed by both transports
/// (HTTP unary, WebSocket framed) and shared between the spawned tasks
/// inside the WebSocket pump.
pub type Shared = std::sync::Arc<dyn OpHandler>;

/// A pre-closed receiver. Use for transports that don't carry client→server
/// stream chunks (e.g. unary HTTP /api/op).
pub fn closed_input() -> mpsc::Receiver<StreamChunk> {
    let (_tx, rx) = mpsc::channel(1);
    rx
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

#[async_trait]
impl OpHandler for EngineHandler {
    #[allow(clippy::too_many_lines)]
    async fn handle(&self, op: Op, claims: &Claims, input: mpsc::Receiver<StreamChunk>) -> HandlerOutput {
        match op {
            // Introspection ops are answered by `auth::introspect` before
            // dispatch ever sees them. Reaching this arm means a transport
            // forgot to call `introspect` — keep it `unreachable!` so the
            // bug surfaces loudly. New introspection ops MUST be added to
            // both `auth::introspect` AND this match.
            Op::Whoami => unreachable!("Whoami handled by auth::introspect"),

            Op::HostInfo => unary(host::run(self).await, OpResult::HostInfo),

            Op::ListContainers { all } => unary(containers::list::run(self, all).await, OpResult::Containers),
            Op::GetContainer { id } => unary(containers::inspect::run(self, id).await, OpResult::ContainerDetail),
            Op::StartContainer { id } => unary(containers::start::run(self, id).await, ok),
            Op::StopContainer { id, timeout } => unary(containers::stop::run(self, id, timeout).await, ok),
            Op::RestartContainer { id, timeout } => unary(containers::restart::run(self, id, timeout).await, ok),
            Op::KillContainer { id, signal } => unary(containers::kill::run(self, id, signal).await, ok),
            Op::RemoveContainer { id, force } => unary(containers::remove::run(self, id, force).await, ok),
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

            Op::ListVolumes => unary(volumes::list::run(self).await, OpResult::Volumes),
            Op::GetVolume { name } => unary(volumes::inspect::run(self, &name).await, OpResult::VolumeDetail),
            Op::CreateVolume {
                name,
                driver,
                labels,
                options,
            } => unary(volumes::create::run(self, name, driver, labels, options).await, ok),
            Op::DeleteVolume { name } => unary(volumes::remove::run(self, name).await, ok),

            Op::ListNetworks => unary(networks::list::run(self).await, OpResult::Networks),
            Op::GetNetwork { id } => unary(networks::inspect::run(self, &id).await, OpResult::NetworkDetail),
            Op::CreateNetwork { name, internal } => unary(
                networks::create::run(self, name, internal, Default::default()).await,
                ok,
            ),
            Op::DeleteNetwork { id } => unary(networks::remove::run(self, id).await, ok),

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

            Op::ListSecrets => unary(secrets::list::run(&self.policy.secrets_root).await, OpResult::Secrets),
            Op::PutSecret { name, value } => unary(
                secrets::put::run(&self.policy.secrets_root, &name, &value).await,
                ok,
            ),
            Op::DeleteSecret { name } => unary(secrets::delete::run(&self.policy.secrets_root, &name).await, ok),
            Op::GetSecret { name } => unary(secrets::get::run(&self.policy.secrets_root, &name).await, OpResult::Secret),
        }
    }
}

fn unary<T>(r: anyhow::Result<T>, into: impl FnOnce(T) -> OpResult) -> HandlerOutput {
    HandlerOutput::Unary(match r {
        Ok(v) => into(v),
        Err(e) => OpResult::Err { message: e.to_string() },
    })
}

fn stream(s: BoxStream<'static, StreamChunk>) -> HandlerOutput {
    HandlerOutput::Stream(s)
}

fn ok(_: ()) -> OpResult {
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
