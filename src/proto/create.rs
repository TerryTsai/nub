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
