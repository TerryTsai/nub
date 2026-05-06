mod create;
mod op;
mod op_strings;
mod stream;
mod types;

use serde::{Deserialize, Serialize};

pub use create::*;
pub use op::Op;
pub use stream::*;
pub use types::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Frame {
    Request { id: u64, op: Op },
    Response { id: u64, result: OpResult },
    Stream { id: u64, chunk: StreamChunk },
}

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
