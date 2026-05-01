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
    ListContainers {
        all: bool,
    },
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
    InspectContainer {
        id: String,
    },
    ContainerAction {
        id: String,
        action: Action,
    },
    ListImages,
    RemoveImage {
        id: String,
        #[serde(default)]
        force: bool,
    },
    PullImage {
        reference: String,
    },
    ListVolumes,
    RemoveVolume {
        name: String,
        #[serde(default)]
        force: bool,
    },
    ListNetworks,
    RemoveNetwork {
        id: String,
    },
    CreateContainer(Box<CreateContainerReq>),
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
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OpResult {
    HostInfo(HostInfo),
    Containers { items: Vec<ContainerSummary> },
    ContainerDetail(Box<ContainerDetail>),
    Images { items: Vec<ImageSummary> },
    Volumes { items: Vec<VolumeSummary> },
    Networks { items: Vec<NetworkSummary> },
    ContainerCreated(ContainerCreated),
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
    End {
        ok: bool,
        err: Option<String>,
    },
}
