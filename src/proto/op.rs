use serde::{Deserialize, Serialize};

use super::CreateContainerReq;
use crate::auth::scope::Scope;

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

impl Op {
    /// Authorization scope this op requires, or `None` for introspection
    /// ops that any valid token may invoke.
    pub fn required_scope(&self) -> Option<Scope> {
        match self {
            Op::HostInfo => Some(Scope::HostInfo),
            Op::Whoami => Some(Scope::AuthWhoami),

            Op::ListContainers { .. } => Some(Scope::ContainersList),
            Op::GetContainer { .. } => Some(Scope::ContainersGet),
            Op::StartContainer { .. } => Some(Scope::ContainersStart),
            Op::StopContainer { .. } => Some(Scope::ContainersStop),
            Op::RestartContainer { .. } => Some(Scope::ContainersRestart),
            Op::KillContainer { .. } => Some(Scope::ContainersKill),
            Op::RemoveContainer { .. } => Some(Scope::ContainersRemove),
            Op::CreateContainer(_) => Some(Scope::ContainersCreate),
            Op::StreamLogs { .. } => Some(Scope::ContainersLogs),
            Op::StreamStats { .. } => Some(Scope::ContainersStats),
            Op::Exec { .. } => Some(Scope::ContainersExec),

            Op::ListImages => Some(Scope::ImagesList),
            Op::GetImage { .. } => Some(Scope::ImagesGet),
            Op::DeleteImage { .. } => Some(Scope::ImagesDelete),
            Op::PullImage { .. } => Some(Scope::ImagesPull),
            Op::BuildImage { .. } => Some(Scope::ImagesBuild),

            Op::ListVolumes => Some(Scope::VolumesList),
            Op::GetVolume { .. } => Some(Scope::VolumesGet),
            Op::CreateVolume { .. } => Some(Scope::VolumesCreate),
            Op::DeleteVolume { .. } => Some(Scope::VolumesDelete),

            Op::ListNetworks => Some(Scope::NetworksList),
            Op::GetNetwork { .. } => Some(Scope::NetworksGet),
            Op::CreateNetwork { .. } => Some(Scope::NetworksCreate),
            Op::DeleteNetwork { .. } => Some(Scope::NetworksDelete),

            Op::ListDockerfiles => Some(Scope::DockerfilesList),
            Op::GetDockerfile { .. } => Some(Scope::DockerfilesGet),
            Op::PutDockerfile { .. } => Some(Scope::DockerfilesPut),
            Op::DeleteDockerfile { .. } => Some(Scope::DockerfilesDelete),

            Op::CreateStack { .. } => Some(Scope::StacksCreate),
            Op::ListStacks => Some(Scope::StacksList),
            Op::GetStack { .. } => Some(Scope::StacksGet),
            Op::DeleteStack { .. } => Some(Scope::StacksDelete),
            Op::RedeployStack { .. } => Some(Scope::StacksRedeploy),
            Op::UpdateStack { .. } => Some(Scope::StacksUpdate),
            Op::PullStack { .. } => Some(Scope::StacksPull),
            Op::StreamStackLogs { .. } => Some(Scope::StacksLogs),

            Op::ListSecrets => Some(Scope::SecretsList),
            Op::PutSecret { .. } => Some(Scope::SecretsPut),
            Op::DeleteSecret { .. } => Some(Scope::SecretsDelete),
            Op::GetSecret { .. } => Some(Scope::SecretsReveal),
        }
    }
}
