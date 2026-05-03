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
