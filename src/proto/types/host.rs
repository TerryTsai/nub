use serde::{Deserialize, Serialize};

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
