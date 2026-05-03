use serde::{Deserialize, Serialize};

use super::{Action, CreateContainerReq};
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
    GetImage {
        id: String,
    },
    DeleteImage {
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
    GetVolume {
        name: String,
    },
    DeleteVolume {
        name: String,
        #[serde(default)]
        force: bool,
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
    /// Wire-format name (matches the `op` discriminator). Used for
    /// logging and structured errors.
    pub fn name(&self) -> &'static str {
        match self {
            Op::HostInfo => "host_info",
            Op::Whoami => "whoami",
            Op::ListContainers { .. } => "list_containers",
            Op::GetContainer { .. } => "get_container",
            Op::ContainerAction { .. } => "container_action",
            Op::CreateContainer(_) => "create_container",
            Op::StreamLogs { .. } => "stream_logs",
            Op::StreamStats { .. } => "stream_stats",
            Op::Exec { .. } => "exec",
            Op::ListImages => "list_images",
            Op::GetImage { .. } => "get_image",
            Op::DeleteImage { .. } => "delete_image",
            Op::PullImage { .. } => "pull_image",
            Op::BuildImage { .. } => "build_image",
            Op::ListVolumes => "list_volumes",
            Op::GetVolume { .. } => "get_volume",
            Op::DeleteVolume { .. } => "delete_volume",
            Op::ListNetworks => "list_networks",
            Op::GetNetwork { .. } => "get_network",
            Op::CreateNetwork { .. } => "create_network",
            Op::DeleteNetwork { .. } => "delete_network",
            Op::ListDockerfiles => "list_dockerfiles",
            Op::GetDockerfile { .. } => "get_dockerfile",
            Op::PutDockerfile { .. } => "put_dockerfile",
            Op::DeleteDockerfile { .. } => "delete_dockerfile",
            Op::CreateStack { .. } => "create_stack",
            Op::ListStacks => "list_stacks",
            Op::GetStack { .. } => "get_stack",
            Op::DeleteStack { .. } => "delete_stack",
            Op::RedeployStack { .. } => "redeploy_stack",
            Op::UpdateStack { .. } => "update_stack",
            Op::PullStack { .. } => "pull_stack",
            Op::StreamStackLogs { .. } => "stream_stack_logs",
            Op::ListSecrets => "list_secrets",
            Op::PutSecret { .. } => "put_secret",
            Op::DeleteSecret { .. } => "delete_secret",
            Op::GetSecret { .. } => "get_secret",
        }
    }

    /// Authorization scope this op requires, or `None` for introspection
    /// ops that any valid token may invoke.
    pub fn required_scope(&self) -> Option<Scope> {
        match self {
            Op::HostInfo | Op::Whoami => None,

            Op::ListContainers { .. } => Some(Scope::ContainersList),
            Op::GetContainer { .. } => Some(Scope::ContainersGet),
            Op::ContainerAction { .. } => Some(Scope::ContainersAction),
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
