//! Top-level wire envelopes — the JSON shapes that cross nub's API
//! boundary on each request, response, or stream chunk.

use serde::{Deserialize, Serialize};

use super::op::Op;
use super::stream::StreamChunk;
use super::types::{
    ContainerCreated, ContainerDetail, ContainerSummary, DockerfileContent, DockerfileSummary, HostInfo, ImageDetail,
    ImageSummary, NetworkDetail, NetworkSummary, SecretSummary, SecretValue, StackCreated, StackDetail, StackSummary,
    VolumeDetail, VolumeSummary, WhoamiInfo,
};

/// Discriminator-tagged envelope for everything on the wire: request,
/// reply, and per-stream chunks.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Frame {
    Request { id: u64, op: Op },
    Response { id: u64, result: OpResult },
    Stream { id: u64, chunk: StreamChunk },
}

/// Reply payload for a unary `Op`. One variant per response shape; the
/// `Err` variant carries a string message rather than a typed code.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum OpResult {
    HostInfo(HostInfo),
    Whoami(WhoamiInfo),
    Containers(Vec<ContainerSummary>),
    ContainerDetail(Box<ContainerDetail>),
    Images(Vec<ImageSummary>),
    Volumes(Vec<VolumeSummary>),
    Networks(Vec<NetworkSummary>),
    ContainerCreated(ContainerCreated),
    ImageDetail(Box<ImageDetail>),
    VolumeDetail(Box<VolumeDetail>),
    NetworkDetail(Box<NetworkDetail>),
    Dockerfiles(Vec<DockerfileSummary>),
    Dockerfile(DockerfileContent),
    Stacks(Vec<StackSummary>),
    StackDetail(Box<StackDetail>),
    StackCreated(StackCreated),
    Secrets(Vec<SecretSummary>),
    Secret(SecretValue),
    Ok,
    StreamStarted,
    Err { message: String },
}
