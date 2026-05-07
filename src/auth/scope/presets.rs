//! Scope presets — named bundles for common token shapes.
//!
//! These are pure data: each preset is an explicit list of `Scope`
//! variants (or a literal string in the case of `ADMIN_LITERAL`). The
//! mint flow joins these with spaces and embeds the result verbatim in
//! the JWT, so the runtime check never knows what a "preset" is.
//!
//! To audit what a preset grants, read this file. Nothing else.
//!
//! Presets are general-purpose roles, not device-specific. A token
//! minted with `--preset operator` works the same from a browser, a
//! shell, or a CI runner — the wire surface doesn't care who's holding
//! the token.

use super::Scope;

/// Full administrative access. Equivalent to wildcard `*`. Embedded
/// as the single token `"*"` for compactness and to keep the admin
/// token small enough to fit in a connect QR.
pub const ADMIN_LITERAL: &str = "*";

/// Day-to-day operations: managing running infrastructure. Excludes:
///   - `containers:kill`    (SIGKILL is CLI-only; use stop/restart)
///   - `images:build`       (build flow lives behind the CLI)
///   - all `dockerfiles:*`  (CLI authoring)
///   - `secrets:reveal`     (CLI-only by policy)
pub const OPERATOR: &[Scope] = &[
    Scope::HostInfo,
    Scope::AuthWhoami,
    Scope::ContainersList,
    Scope::ContainersGet,
    Scope::ContainersLogs,
    Scope::ContainersStats,
    Scope::ContainersCreate,
    Scope::ContainersStart,
    Scope::ContainersStop,
    Scope::ContainersRestart,
    Scope::ContainersRemove,
    Scope::ContainersExec,
    Scope::ImagesList,
    Scope::ImagesGet,
    Scope::ImagesPull,
    Scope::ImagesDelete,
    Scope::VolumesList,
    Scope::VolumesGet,
    Scope::VolumesCreate,
    Scope::VolumesDelete,
    Scope::NetworksList,
    Scope::NetworksGet,
    Scope::NetworksCreate,
    Scope::NetworksDelete,
    Scope::StacksList,
    Scope::StacksGet,
    Scope::StacksLogs,
    Scope::StacksCreate,
    Scope::StacksDelete,
    Scope::StacksRedeploy,
    Scope::StacksUpdate,
    Scope::StacksPull,
    Scope::SecretsList,
    Scope::SecretsPut,
    Scope::SecretsDelete,
];

/// Stack delivery from a CI runner: the minimum needed to land a new
/// or updated stack. Includes the read scopes for introspection plus
/// the sub-op scopes a stack op composes (see ops::stacks). Excludes
/// `containers:exec`, `secrets:put`, and `containers:start/stop/restart`
/// as standalone — CI doesn't poke individual containers; it deploys
/// stacks and the stack runtime does the rest.
pub const DEPLOY: &[Scope] = &[
    Scope::HostInfo,
    Scope::AuthWhoami,
    Scope::ContainersList,
    Scope::ContainersGet,
    Scope::ContainersCreate,
    Scope::ContainersStart,
    Scope::ContainersStop,
    Scope::ContainersRemove,
    Scope::ImagesList,
    Scope::ImagesGet,
    Scope::ImagesPull,
    Scope::VolumesList,
    Scope::VolumesGet,
    Scope::VolumesCreate,
    Scope::VolumesDelete,
    Scope::NetworksList,
    Scope::NetworksGet,
    Scope::NetworksCreate,
    Scope::NetworksDelete,
    Scope::StacksList,
    Scope::StacksGet,
    Scope::StacksLogs,
    Scope::StacksCreate,
    Scope::StacksDelete,
    Scope::StacksRedeploy,
    Scope::StacksUpdate,
    Scope::StacksPull,
];

/// Read-only across every resource. No state changes; secret values
/// are NOT included (`SecretsReveal` is privileged and admin-only).
pub const READONLY: &[Scope] = &[
    Scope::HostInfo,
    Scope::AuthWhoami,
    Scope::ContainersList,
    Scope::ContainersGet,
    Scope::ContainersLogs,
    Scope::ContainersStats,
    Scope::ImagesList,
    Scope::ImagesGet,
    Scope::VolumesList,
    Scope::VolumesGet,
    Scope::NetworksList,
    Scope::NetworksGet,
    Scope::DockerfilesList,
    Scope::DockerfilesGet,
    Scope::StacksList,
    Scope::StacksGet,
    Scope::StacksLogs,
    Scope::SecretsList,
];
