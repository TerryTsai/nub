//! Public types for the containers API.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ContainerSummary {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: String,
    pub status: String,
    /// ISO 8601 from libpod, or Unix-as-string from compat. Engine-native;
    /// callers parse if they need a structured timestamp.
    pub created: String,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct MountPoint {
    pub kind: String,
    pub source: String,
    pub destination: String,
    pub mode: String,
    pub rw: bool,
}

#[derive(Debug, Clone, Default)]
pub struct NetworkEndpoint {
    pub ip_address: String,
    pub gateway: String,
    pub mac_address: String,
}

#[derive(Debug, Clone)]
pub struct PortMapping {
    pub container_port: String,
    pub host_ip: String,
    pub host_port: String,
}

#[derive(Debug, Clone)]
pub enum ContainerAction {
    Start,
    Stop { timeout: Option<i64> },
    Restart { timeout: Option<i64> },
    Kill { signal: Option<String> },
    Remove { force: bool, volumes: bool },
}

#[derive(Debug, Clone, Default)]
pub struct CreateContainer {
    pub image: String,
    pub name: Option<String>,
    pub cmd: Vec<String>,
    pub entrypoint: Vec<String>,
    pub env: Vec<String>,
    pub working_dir: Option<String>,
    pub user: Option<String>,
    pub labels: HashMap<String, String>,
    pub ports: Vec<PortBinding>,
    pub volumes: Vec<VolumeMount>,
    pub network: Option<String>,
    pub restart: Option<RestartPolicy>,
    pub memory_limit: Option<i64>,
    pub cpu_shares: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct PortBinding {
    pub container: String,
    pub host: String,
}

#[derive(Debug, Clone)]
pub struct VolumeMount {
    pub source: String,
    pub target: String,
    pub read_only: bool,
}

#[derive(Debug, Clone)]
pub enum RestartPolicy {
    No,
    OnFailure { max_retries: Option<i64> },
    Always,
    UnlessStopped,
}

#[derive(Debug, Clone)]
pub struct ContainerCreated {
    pub id: String,
    pub warnings: Vec<String>,
}
