mod create;
mod types;

use serde::{Deserialize, Serialize};

pub use create::*;
pub use types::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Frame {
    Request { id: u64, op: Op },
    Response { id: u64, result: OpResult },
    Stream { id: u64, chunk: StreamChunk },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Op {
    HostInfo,
    Whoami,

    ListContainers {
        all: bool,
    },
    InspectContainer {
        id: String,
    },
    ContainerAction {
        id: String,
        action: Action,
    },
    CreateContainer(Box<CreateContainerReq>),
    StreamLogs {
        id: String,
        #[serde(default)]
        follow: bool,
        #[serde(default)]
        tail: Option<u32>,
    },
    StreamStats {
        id: String,
    },
    Exec {
        id: String,
        cmd: Vec<String>,
        #[serde(default)]
        tty: bool,
    },

    ListImages,
    InspectImage {
        id: String,
    },
    RemoveImage {
        id: String,
        #[serde(default)]
        force: bool,
    },
    PullImage {
        reference: String,
    },
    BuildImage {
        /// Filename inside the configured dockerfiles directory.
        dockerfile: String,
        /// Tag to apply to the built image, e.g. `nginx:dev`.
        tag: String,
        /// `--build-arg` values. Empty map is fine.
        #[serde(default)]
        build_args: std::collections::HashMap<String, String>,
    },

    ListVolumes,
    InspectVolume {
        name: String,
    },
    RemoveVolume {
        name: String,
        #[serde(default)]
        force: bool,
    },

    ListNetworks,
    InspectNetwork {
        id: String,
    },
    RemoveNetwork {
        id: String,
    },

    ListDockerfiles,
    ReadDockerfile {
        name: String,
    },
    WriteDockerfile {
        name: String,
        content: String,
    },
    DeleteDockerfile {
        name: String,
    },
}

impl Op {
    /// Wire-format name (matches the `op` discriminator), used for permission checks.
    pub fn name(&self) -> &'static str {
        match self {
            Op::HostInfo => "host_info",
            Op::Whoami => "whoami",
            Op::ListContainers { .. } => "list_containers",
            Op::InspectContainer { .. } => "inspect_container",
            Op::ContainerAction { .. } => "container_action",
            Op::CreateContainer(_) => "create_container",
            Op::StreamLogs { .. } => "stream_logs",
            Op::StreamStats { .. } => "stream_stats",
            Op::Exec { .. } => "exec",
            Op::ListImages => "list_images",
            Op::InspectImage { .. } => "inspect_image",
            Op::RemoveImage { .. } => "remove_image",
            Op::PullImage { .. } => "pull_image",
            Op::BuildImage { .. } => "build_image",
            Op::ListVolumes => "list_volumes",
            Op::InspectVolume { .. } => "inspect_volume",
            Op::RemoveVolume { .. } => "remove_volume",
            Op::ListNetworks => "list_networks",
            Op::InspectNetwork { .. } => "inspect_network",
            Op::RemoveNetwork { .. } => "remove_network",
            Op::ListDockerfiles => "list_dockerfiles",
            Op::ReadDockerfile { .. } => "read_dockerfile",
            Op::WriteDockerfile { .. } => "write_dockerfile",
            Op::DeleteDockerfile { .. } => "delete_dockerfile",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    Start,
    Stop {
        #[serde(default)]
        timeout: Option<i64>,
    },
    Restart {
        #[serde(default)]
        timeout: Option<i64>,
    },
    Kill {
        #[serde(default)]
        signal: Option<String>,
    },
    Remove {
        #[serde(default)]
        force: bool,
        #[serde(default)]
        volumes: bool,
    },
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
    Ok,
    StreamStarted,
    Err { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamChunk {
    Log {
        stderr: bool,
        data: String,
    },
    Stats {
        cpu_pct: f64,
        mem_used: u64,
        mem_limit: u64,
        net_rx: u64,
        net_tx: u64,
    },
    Lagging {
        dropped: u32,
    },
    Stdin {
        data: String,
    },
    StdinClose,
    PullProgress {
        id: String,
        status: String,
        current: u64,
        total: u64,
    },
    BuildProgress {
        /// Engine output line. Newline-terminated; UI concatenates as-is.
        stream: String,
        /// Final image ID (sha256:…) on the last `aux` chunk, otherwise None.
        image_id: Option<String>,
    },
    End {
        ok: bool,
        err: Option<String>,
    },
}
