//! `EngineHandler` — Op dispatch against a live engine connection. Each
//! `Op` variant routes to one per-verb function in an `ops/<family>`
//! module; helpers below wrap the result into a `HandlerOutput` so both
//! transports (HTTP unary, WebSocket streamed) share one envelope.

use std::future::Future;

use anyhow::Result;
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

/// Shared handle to the engine-backed dispatch — passed by both
/// transports (HTTP unary, WebSocket framed) and shared between the
/// spawned tasks inside the WebSocket pump.
pub type Shared = std::sync::Arc<EngineHandler>;

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

    #[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
    pub async fn handle(&self, op: Op, claims: &Claims, input: mpsc::Receiver<StreamChunk>) -> HandlerOutput {
        let secrets_root = &self.policy.secrets_root;
        match op {
            // Introspection ops are answered by `auth::introspect` before
            // dispatch ever sees them. Reaching this arm means a transport
            // forgot to call `introspect` — keep it `unreachable!` so the
            // bug surfaces loudly. New introspection ops MUST be added to
            // both `auth::introspect` AND this match.
            Op::Whoami => unreachable!("Whoami handled by auth::introspect"),

            Op::HostInfo => unary(host::info::run(self).await.map(OpResult::HostInfo)),

            Op::ListContainers { all } => unary(containers::list::run(self, all).await.map(OpResult::Containers)),
            Op::GetContainer { id } => unary(containers::inspect::run(self, id).await.map(OpResult::ContainerDetail)),
            Op::StartContainer { id } => unit(containers::start::run(self, id).await),
            Op::StopContainer { id, timeout } => unit(containers::stop::run(self, id, timeout).await),
            Op::RestartContainer { id, timeout } => unit(containers::restart::run(self, id, timeout).await),
            Op::KillContainer { id, signal } => unit(containers::kill::run(self, id, signal).await),
            Op::RemoveContainer { id, force } => unit(containers::remove::run(self, id, force).await),
            Op::CreateContainer(req) => {
                unary(containers::create::run(self, *req).await.map(OpResult::ContainerCreated))
            }
            Op::StreamLogs { id, follow, tail } => stream(containers::logs::run(self, id, follow, tail)),
            Op::StreamStats { id } => stream(containers::stats::run(self, id)),
            Op::Exec { id, cmd, tty } => stream(containers::exec::run(self, id, cmd, tty, input)),

            Op::ListImages => unary(images::list::run(self).await.map(OpResult::Images)),
            Op::GetImage { id } => unary(images::inspect::run(self, id).await.map(OpResult::ImageDetail)),
            Op::DeleteImage { id } => unit(images::remove::run(self, id).await),
            Op::PullImage { reference } => stream(images::pull::run(self, reference)),
            Op::BuildImage {
                dockerfile_content,
                tag,
                build_args,
            } => stream(images::build::run(self, dockerfile_content, tag, build_args)),

            Op::ListVolumes => unary(volumes::list::run(self).await.map(OpResult::Volumes)),
            Op::GetVolume { name } => unary(volumes::inspect::run(self, &name).await.map(OpResult::VolumeDetail)),
            Op::CreateVolume {
                name,
                driver,
                labels,
                options,
            } => unit(volumes::create::run(self, name, driver, labels, options).await),
            Op::DeleteVolume { name } => unit(volumes::remove::run(self, name).await),

            Op::ListNetworks => unary(networks::list::run(self).await.map(OpResult::Networks)),
            Op::GetNetwork { id } => unary(networks::inspect::run(self, &id).await.map(OpResult::NetworkDetail)),
            Op::CreateNetwork { name, internal } => {
                unit(networks::create::run(self, name, internal, Default::default()).await)
            }
            Op::DeleteNetwork { id } => unit(networks::remove::run(self, id).await),

            Op::ListDockerfiles => unary(dockerfiles::list::run(self).await.map(OpResult::Dockerfiles)),
            Op::GetDockerfile { name } => unary(dockerfiles::get::run(self, &name).await.map(OpResult::Dockerfile)),
            Op::PutDockerfile { name, content } => unit(dockerfiles::put::run(self, &name, &content).await),
            Op::DeleteDockerfile { name } => unit(dockerfiles::delete::run(self, &name).await),

            Op::CreateStack { name, yaml } => {
                unary(stacks::create::run(self, claims, name, yaml).await.map(OpResult::StackCreated))
            }
            Op::ListStacks => unary(stacks::list::run(self).await.map(OpResult::Stacks)),
            Op::GetStack { name } => unary(stacks::get::run(self, name).await.map(OpResult::StackDetail)),
            Op::DeleteStack { name } => unit(stacks::delete::run(self, claims, name).await),
            Op::RedeployStack { name } => {
                unary(stacks::redeploy::run(self, claims, name).await.map(OpResult::StackCreated))
            }
            Op::UpdateStack { name, yaml } => {
                unary(stacks::update::run(self, claims, name, yaml).await.map(OpResult::StackCreated))
            }
            Op::PullStack { name } => unary(stacks::pull::run(self, claims, name).await.map(OpResult::StackCreated)),
            Op::StreamStackLogs { name, follow, tail } => stream(stacks::logs::run(self, name, follow, tail)),

            Op::ListSecrets => unary(secrets::list::run(secrets_root).await.map(OpResult::Secrets)),
            Op::PutSecret { name, value } => unit(secrets::put::run(secrets_root, &name, &value).await),
            Op::DeleteSecret { name } => unit(secrets::delete::run(secrets_root, &name).await),
            Op::GetSecret { name } => unary(secrets::get::run(secrets_root, &name).await.map(OpResult::Secret)),
        }
    }
}

fn unary(r: Result<OpResult>) -> HandlerOutput {
    HandlerOutput::Unary(match r {
        Ok(v) => v,
        Err(e) => OpResult::Err { message: e.to_string() },
    })
}

fn unit(r: Result<()>) -> HandlerOutput {
    unary(r.map(|()| OpResult::Ok))
}

fn stream(s: BoxStream<'static, StreamChunk>) -> HandlerOutput {
    HandlerOutput::Stream(s)
}

/// Spawn `produce` and forward its emitted chunks. Appends an `End` chunk
/// when the producer returns. The returned BoxStream owns the receiving end.
pub(crate) fn spawn_chunked<F, Fut>(produce: F) -> BoxStream<'static, StreamChunk>
where
    F: FnOnce(mpsc::Sender<StreamChunk>) -> Fut + Send + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    let (tx, rx) = mpsc::channel::<StreamChunk>(32);
    let inner = tx.clone();
    tokio::spawn(async move {
        let (ok, err) = match produce(inner).await {
            Ok(()) => (true, None),
            Err(e) => (false, Some(e.to_string())),
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
