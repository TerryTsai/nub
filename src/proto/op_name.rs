//! Wire-name mapping for `Op`. Each variant maps to its `op` tag in
//! JSON; used for logging and structured errors.

use super::Op;

impl Op {
    /// Wire-format name (matches the `op` discriminator).
    pub fn name(&self) -> &'static str {
        match self {
            Op::HostInfo => "host_info",
            Op::Whoami => "whoami",
            Op::ListContainers { .. } => "list_containers",
            Op::GetContainer { .. } => "get_container",
            Op::StartContainer { .. } => "start_container",
            Op::StopContainer { .. } => "stop_container",
            Op::RestartContainer { .. } => "restart_container",
            Op::KillContainer { .. } => "kill_container",
            Op::RemoveContainer { .. } => "remove_container",
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
            Op::CreateVolume { .. } => "create_volume",
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
}
