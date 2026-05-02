use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct WhoamiInfo {
    pub id: String,
    pub allowed: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HostInfo {
    pub engine: String,
    pub version: String,
    pub os: String,
    pub arch: String,
    pub kernel: String,
    pub cpus: u64,
    pub mem_total: u64,
    pub containers_running: u64,
    pub containers_total: u64,
    pub images: u64,
}

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
pub struct ImageSummary {
    pub id: String,
    pub repo_tag: String,
    pub created: i64,
    pub size: i64,
    pub containers: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VolumeSummary {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
    pub created_at: String,
    pub scope: String,
    /// True if at least one container (running or stopped) mounts this volume.
    pub in_use: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DockerfileSummary {
    pub name: String,
    /// File size in bytes.
    pub size: u64,
    /// ISO 8601 mtime, or empty when the FS doesn't expose one.
    pub modified_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DockerfileContent {
    pub name: String,
    pub content: String,
    pub size: u64,
    pub modified_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkSummary {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub scope: String,
    pub created: String,
    pub internal: bool,
    /// True if at least one container (running or stopped) is attached.
    pub in_use: bool,
}
