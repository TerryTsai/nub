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
    /// Healthcheck state derived from the engine's free-form Status string:
    /// `"healthy"`, `"unhealthy"`, `"starting"`, or empty (no healthcheck or
    /// not parseable). Both engines format this consistently; we don't pay
    /// the N+1 inspect cost just to get a structured field.
    pub health: String,
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
pub struct ImageSummary {
    pub id: String,
    pub repo_tag: String,
    pub created: i64,
    pub size: i64,
    pub containers: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageDetail {
    pub id: String,
    pub repo_tags: Vec<String>,
    pub repo_digests: Vec<String>,
    pub created: String,
    pub size: i64,
    pub architecture: String,
    pub os: String,
    pub author: String,
    pub comment: String,
    pub cmd: Vec<String>,
    pub entrypoint: Vec<String>,
    pub env: Vec<String>,
    pub working_dir: String,
    pub user: String,
    pub exposed_ports: Vec<String>,
    pub labels: HashMap<String, String>,
    /// Layer count (length of RootFS.Layers).
    pub layers: u32,
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
pub struct VolumeDetail {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
    pub created_at: String,
    pub scope: String,
    pub labels: HashMap<String, String>,
    pub options: HashMap<String, String>,
    /// Number of containers using this volume (from /system/df).
    pub ref_count: i64,
    /// Disk usage bytes (-1 if not reported).
    pub size: i64,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkDetail {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub scope: String,
    pub created: String,
    pub internal: bool,
    /// IPAM subnet/gateway pairs. Most networks have a single entry.
    pub ipam: Vec<IpamConfig>,
    pub containers: Vec<NetworkContainer>,
    pub options: HashMap<String, String>,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IpamConfig {
    pub subnet: String,
    pub gateway: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkContainer {
    pub id: String,
    pub name: String,
    pub ipv4: String,
    pub ipv6: String,
}
