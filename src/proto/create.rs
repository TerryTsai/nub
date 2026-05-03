use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    #[serde(default)]
    pub start: bool,
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
    pub started: bool,
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
