mod container_streams;
mod containers;
mod create;
mod host;
mod images;
mod networks;
mod util;
mod volumes;

use crate::proto::*;
use anyhow::Result;
use async_trait::async_trait;
use bollard::Docker;
use futures::stream::BoxStream;
use std::path::PathBuf;
use tokio::sync::mpsc;

use util::{ok, stream, unary};

pub enum HandlerOutput {
    Unary(OpResult),
    Stream(BoxStream<'static, StreamChunk>),
}

#[async_trait]
pub trait OpHandler: Send + Sync + 'static {
    async fn handle(&self, op: Op, input: mpsc::Receiver<StreamChunk>) -> HandlerOutput;
}

/// A receiver pre-closed at construction. Use for transports that don't support
/// client-to-server stream chunks (e.g. unary HTTP /op).
pub fn closed_input() -> mpsc::Receiver<StreamChunk> {
    let (_tx, rx) = mpsc::channel::<StreamChunk>(1);
    rx
}

/// Security policy applied at handler boundaries. Currently only constrains
/// container creation; future fields go here.
pub struct Policy {
    /// Host paths permitted as bind-mount sources in CreateContainer.
    /// Empty = no host bind mounts allowed.
    pub allowed_binds: Vec<PathBuf>,
}

pub struct DockerHandler {
    docker: Docker,
    policy: Policy,
}

impl DockerHandler {
    pub fn connect(policy: Policy) -> Result<Self> {
        Ok(Self {
            docker: Docker::connect_with_local_defaults()?,
            policy,
        })
    }
}

#[async_trait]
impl OpHandler for DockerHandler {
    async fn handle(&self, op: Op, input: mpsc::Receiver<StreamChunk>) -> HandlerOutput {
        match op {
            Op::HostInfo => unary(self.host_info().await, OpResult::HostInfo),
            // Whoami is auth-layer info; transport short-circuits before reaching here.
            Op::Whoami => unreachable!("Whoami handled by transport layer"),

            Op::ListContainers { all } => unary(self.list_containers(all).await, OpResult::Containers),
            Op::InspectContainer { id } => unary(self.inspect_container(id).await, OpResult::ContainerDetail),
            Op::ContainerAction { id, action } => unary(self.container_action(id, action).await, ok),
            Op::CreateContainer(req) => unary(self.create_container(*req).await, OpResult::ContainerCreated),
            Op::StreamLogs { id, follow, tail } => stream(self.stream_logs(id, follow, tail)),
            Op::StreamStats { id } => stream(self.stream_stats(id)),
            Op::Exec { id, cmd, tty } => stream(self.exec(id, cmd, tty, input)),

            Op::ListImages => unary(self.list_images().await, OpResult::Images),
            Op::RemoveImage { id, force } => unary(self.remove_image(id, force).await, ok),
            Op::PullImage { reference } => stream(self.pull_image(reference)),

            Op::ListVolumes => unary(self.list_volumes().await, OpResult::Volumes),
            Op::RemoveVolume { name, force } => unary(self.remove_volume(name, force).await, ok),

            Op::ListNetworks => unary(self.list_networks().await, OpResult::Networks),
            Op::RemoveNetwork { id } => unary(self.remove_network(id).await, ok),
        }
    }
}
