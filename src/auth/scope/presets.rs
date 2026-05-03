//! Scope presets — named bundles for common token shapes.
//!
//! These are pure data: each preset is an explicit list of `Scope`
//! variants (or a literal string in the case of `ADMIN_LITERAL`). The
//! mint flow joins these with spaces and embeds the result verbatim in
//! the JWT, so the runtime check never knows what a "preset" is.
//!
//! To audit what a preset grants, read this file. Nothing else.

use super::Scope;

/// Full administrative access. Equivalent to wildcard `*`. Embedded
/// as the single token `"*"` for compactness and to keep the admin
/// token small enough to fit in a connect QR.
pub const ADMIN_LITERAL: &str = "*";

/// Phone UI: everything an operator does day-to-day from a phone.
/// Excludes:
///   - `containers:create`  (phone uses stacks for new workloads)
///   - `images:build`       (build flow lives behind the CLI)
///   - `networks:create`    (managed by stacks)
///   - all `dockerfiles:*`  (CLI authoring)
///   - `secrets:reveal`     (CLI-only by policy)
pub const PHONE: &[Scope] = &[
    Scope::ContainersList,
    Scope::ContainersGet,
    Scope::ContainersLogs,
    Scope::ContainersStats,
    Scope::ContainersAction,
    Scope::ContainersExec,
    Scope::ImagesList,
    Scope::ImagesGet,
    Scope::ImagesPull,
    Scope::ImagesDelete,
    Scope::VolumesList,
    Scope::VolumesGet,
    Scope::VolumesDelete,
    Scope::NetworksList,
    Scope::NetworksGet,
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

/// Read-only across every resource. No state changes; secret values
/// are NOT included (`SecretsReveal` is privileged and admin-only).
pub const READONLY: &[Scope] = &[
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
