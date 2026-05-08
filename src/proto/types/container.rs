//! Container wire types — request shapes (`CreateContainerReq` and its
//! field types) and response shapes (`ContainerSummary`,
//! `ContainerDetail`).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerSummary {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: String,
    pub status: String,
    /// ISO 8601 string from libpod, or Unix-as-string from compat. Wire-stable
    /// across both engines without forcing nub to do timestamp math.
    pub created: String,
    /// Last exit code. 0 by default — also when the container hasn't exited or
    /// when the engine doesn't include the field on the list response.
    pub exit_code: i32,
    /// Healthcheck state derived from the engine's free-form Status string:
    /// `"healthy"`, `"unhealthy"`, `"starting"`, or empty (no healthcheck or
    /// not parseable). Both engines format this consistently; we don't pay
    /// the N+1 inspect cost just to get a structured field.
    pub health: String,
    /// User-declared labels plus engine-managed ones. Used to surface
    /// stack membership (`nub.stack=<name>`) in the UI without an extra
    /// inspect call per container.
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerDetail {
    pub id: String,
    pub name: String,
    pub image: String,
    pub image_id: String,
    pub created: String,
    pub state: String,
    pub running: bool,
    pub started_at: String,
    pub finished_at: String,
    pub exit_code: i64,
    pub error: String,
    pub restart_count: i64,
    /// Healthcheck state from inspect's structured `State.Health.Status`.
    /// Empty when no healthcheck is configured.
    pub health: String,
    pub cmd: Vec<String>,
    pub entrypoint: Vec<String>,
    pub env: Vec<String>,
    pub working_dir: String,
    pub user: String,
    pub labels: HashMap<String, String>,
    pub network_mode: String,
    pub restart_policy: String,
    pub privileged: bool,
    pub memory_limit: i64,
    pub mounts: Vec<MountPoint>,
    pub networks: HashMap<String, NetworkEndpoint>,
    pub ports: Vec<PortMapping>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MountPoint {
    pub kind: String,
    pub source: String,
    pub destination: String,
    pub mode: String,
    pub rw: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkEndpoint {
    pub ip_address: String,
    pub gateway: String,
    pub mac_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortMapping {
    pub container_port: String,
    pub host_ip: String,
    pub host_port: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateContainerReq {
    pub image: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub cmd: Vec<String>,
    #[serde(default)]
    pub entrypoint: Vec<String>,
    #[serde(default)]
    pub env: Vec<String>,
    #[serde(default)]
    pub working_dir: Option<String>,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub labels: HashMap<String, String>,
    #[serde(default)]
    pub ports: Vec<PortPublish>,
    #[serde(default)]
    pub volumes: Vec<VolumeMount>,
    #[serde(default)]
    pub network: Option<String>,
    #[serde(default)]
    pub restart: Option<RestartPolicySpec>,
    #[serde(default)]
    pub memory_limit: Option<i64>,
    #[serde(default)]
    pub cpu_shares: Option<i64>,
    #[serde(default)]
    pub healthcheck: Option<HealthcheckSpec>,
    #[serde(default)]
    pub cap_add: Vec<String>,
    #[serde(default)]
    pub cap_drop: Vec<String>,
    #[serde(default)]
    pub privileged: bool,
    #[serde(default)]
    pub devices: Vec<DeviceMapping>,
    #[serde(default)]
    pub extra_hosts: Vec<String>,
    #[serde(default)]
    pub init: Option<bool>,
    #[serde(default)]
    pub tmpfs: HashMap<String, String>,
    #[serde(default)]
    pub shm_size: Option<i64>,
    #[serde(default)]
    pub ulimits: Vec<UlimitSpec>,
    #[serde(default)]
    pub sysctls: HashMap<String, String>,
    #[serde(default)]
    pub hostname: Option<String>,
    #[serde(default)]
    pub dns: Vec<String>,
    #[serde(default)]
    pub expose: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortPublish {
    /// Container port. "80" or "80/tcp"; "/tcp" assumed if no protocol.
    pub container: String,
    /// Host binding. "8080" (any iface) or "127.0.0.1:8080".
    pub host: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VolumeMount {
    /// Named volume identifier OR host path (validated against allowlist).
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub read_only: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RestartPolicySpec {
    No,
    OnFailure {
        #[serde(default)]
        max_retries: Option<i64>,
    },
    Always,
    UnlessStopped,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerCreated {
    pub id: String,
    pub warnings: Vec<String>,
}

/// Healthcheck spec with durations already normalized to nanoseconds —
/// the engine wire format. Compose's `1m30s` style strings are parsed
/// at the YAML layer, not here.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct HealthcheckSpec {
    /// First element is the engine command type (`CMD`, `CMD-SHELL`,
    /// `NONE`, `INHERIT`); rest are arguments.
    pub test: Vec<String>,
    #[serde(default)]
    pub interval_ns: Option<i64>,
    #[serde(default)]
    pub timeout_ns: Option<i64>,
    #[serde(default)]
    pub retries: Option<i64>,
    #[serde(default)]
    pub start_period_ns: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceMapping {
    /// Host path, e.g. `/dev/dri/renderD128`.
    pub host: String,
    /// Container path. Empty string means "same as host".
    #[serde(default)]
    pub container: String,
    /// cgroup permissions string, e.g. `rwm`. Defaults to `rwm` if absent.
    #[serde(default)]
    pub permissions: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UlimitSpec {
    pub name: String,
    #[serde(default)]
    pub soft: Option<i64>,
    #[serde(default)]
    pub hard: Option<i64>,
}
