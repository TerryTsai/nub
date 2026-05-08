//! Per-`Op` authorization scope. Every wire op declares one required
//! `Scope`; the auth layer checks token grants against this before
//! dispatching.

use super::Op;
use crate::auth::scope::Scope;

impl Op {
    /// Authorization scope this op requires.
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
