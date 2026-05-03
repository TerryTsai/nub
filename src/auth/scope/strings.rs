//! Wire-string mapping for `Scope`. Split out so the central enum file
//! stays under the project's per-file line limit.

use super::Scope;

impl Scope {
    /// Wire string for this scope (`<resource>:<action>`).
    pub fn as_str(self) -> &'static str {
        match self {
            Scope::ContainersList => "containers:list",
            Scope::ContainersGet => "containers:get",
            Scope::ContainersLogs => "containers:logs",
            Scope::ContainersStats => "containers:stats",
            Scope::ContainersCreate => "containers:create",
            Scope::ContainersAction => "containers:action",
            Scope::ContainersExec => "containers:exec",

            Scope::ImagesList => "images:list",
            Scope::ImagesGet => "images:get",
            Scope::ImagesPull => "images:pull",
            Scope::ImagesBuild => "images:build",
            Scope::ImagesDelete => "images:delete",

            Scope::VolumesList => "volumes:list",
            Scope::VolumesGet => "volumes:get",
            Scope::VolumesDelete => "volumes:delete",

            Scope::NetworksList => "networks:list",
            Scope::NetworksGet => "networks:get",
            Scope::NetworksCreate => "networks:create",
            Scope::NetworksDelete => "networks:delete",

            Scope::DockerfilesList => "dockerfiles:list",
            Scope::DockerfilesGet => "dockerfiles:get",
            Scope::DockerfilesPut => "dockerfiles:put",
            Scope::DockerfilesDelete => "dockerfiles:delete",

            Scope::StacksList => "stacks:list",
            Scope::StacksGet => "stacks:get",
            Scope::StacksLogs => "stacks:logs",
            Scope::StacksCreate => "stacks:create",
            Scope::StacksDelete => "stacks:delete",
            Scope::StacksRedeploy => "stacks:redeploy",
            Scope::StacksUpdate => "stacks:update",
            Scope::StacksPull => "stacks:pull",

            Scope::SecretsList => "secrets:list",
            Scope::SecretsPut => "secrets:put",
            Scope::SecretsDelete => "secrets:delete",
            Scope::SecretsReveal => "secrets:reveal",
        }
    }

    /// Resource portion of `<resource>:<action>`, used for the
    /// `<resource>:*` wildcard match.
    pub fn resource(self) -> &'static str {
        let s = self.as_str();
        let idx = s.as_bytes().iter().position(|&b| b == b':').unwrap();
        &s[..idx]
    }
}
