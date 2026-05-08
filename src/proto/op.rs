//! Request enum — every Op nub's wire surface accepts. The discriminator
//! is the `op` tag in JSON. Per-op authorization scope and wire-name
//! mapping live in sibling files.

use serde::{Deserialize, Serialize};

use super::types::CreateContainerReq;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Op {
    HostInfo,
    Whoami,

    ListContainers {
        all: bool,
    },
    GetContainer {
        id: String,
    },
    StartContainer {
        id: String,
    },
    StopContainer {
        id: String,
        #[serde(default)]
        timeout: Option<i64>,
    },
    RestartContainer {
        id: String,
        #[serde(default)]
        timeout: Option<i64>,
    },
    KillContainer {
        id: String,
        #[serde(default)]
        signal: Option<String>,
    },
    RemoveContainer {
        id: String,
        #[serde(default)]
        force: bool,
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
    GetImage {
        id: String,
    },
    DeleteImage {
        id: String,
    },
    PullImage {
        reference: String,
    },
    BuildImage {
        /// Dockerfile contents — caller fetches via GetDockerfile (or supplies
        /// any source). The build handler does not touch the dockerfiles
        /// directory; that's a separate scope.
        dockerfile_content: String,
        /// Tag to apply to the built image, e.g. `nginx:dev`.
        tag: String,
        /// `--build-arg` values. Empty map is fine.
        #[serde(default)]
        build_args: std::collections::HashMap<String, String>,
    },

    ListVolumes,
    GetVolume {
        name: String,
    },
    CreateVolume {
        name: String,
        #[serde(default)]
        driver: Option<String>,
        #[serde(default)]
        labels: std::collections::HashMap<String, String>,
        #[serde(default)]
        options: std::collections::HashMap<String, String>,
    },
    DeleteVolume {
        name: String,
    },

    ListNetworks,
    GetNetwork {
        id: String,
    },
    CreateNetwork {
        name: String,
        /// Block external traffic; only attached containers can reach each
        /// other. Default `false`.
        #[serde(default)]
        internal: bool,
    },
    DeleteNetwork {
        id: String,
    },

    ListDockerfiles,
    GetDockerfile {
        name: String,
    },
    PutDockerfile {
        name: String,
        content: String,
    },
    DeleteDockerfile {
        name: String,
    },

    CreateStack {
        name: String,
        yaml: String,
    },
    ListStacks,
    GetStack {
        name: String,
    },
    DeleteStack {
        name: String,
    },
    RedeployStack {
        name: String,
    },
    UpdateStack {
        name: String,
        yaml: String,
    },
    PullStack {
        name: String,
    },
    StreamStackLogs {
        name: String,
        #[serde(default)]
        follow: bool,
        #[serde(default)]
        tail: Option<u32>,
    },

    ListSecrets,
    PutSecret {
        name: String,
        value: String,
    },
    DeleteSecret {
        name: String,
    },
    /// Privileged: returns the plaintext value of one secret. Requires
    /// `secrets:reveal`, which is intentionally not granted by any preset.
    GetSecret {
        name: String,
    },
}
