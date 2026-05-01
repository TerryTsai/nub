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

use util::unary;

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
            Op::ListContainers { all } => unary(self.list_containers(all).await, |items| {
                OpResult::Containers { items }
            }),
            Op::InspectContainer { id } => unary(self.inspect_container(id).await, |d| {
                OpResult::ContainerDetail(Box::new(d))
            }),
            Op::ContainerAction { id, action } => {
                unary(self.container_action(id, action).await, |()| OpResult::Ok)
            }
            Op::StreamLogs { id, follow, tail } => {
                HandlerOutput::Stream(self.stream_logs(id, follow, tail))
            }
            Op::StreamStats { id } => HandlerOutput::Stream(self.stream_stats(id)),
            Op::Exec { id, cmd, tty } => HandlerOutput::Stream(self.exec(id, cmd, tty, input)),
            Op::ListImages => unary(self.list_images().await, |items| OpResult::Images { items }),
            Op::RemoveImage { id, force } => {
                unary(self.remove_image(id, force).await, |()| OpResult::Ok)
            }
            Op::PullImage { reference } => HandlerOutput::Stream(self.pull_image(reference)),
            Op::ListVolumes => unary(self.list_volumes().await, |items| OpResult::Volumes {
                items,
            }),
            Op::RemoveVolume { name, force } => {
                unary(self.remove_volume(name, force).await, |()| OpResult::Ok)
            }
            Op::ListNetworks => unary(self.list_networks().await, |items| OpResult::Networks {
                items,
            }),
            Op::RemoveNetwork { id } => unary(self.remove_network(id).await, |()| OpResult::Ok),
            Op::CreateContainer(req) => unary(
                self.create_container(*req).await,
                OpResult::ContainerCreated,
            ),
        }
    }
}
