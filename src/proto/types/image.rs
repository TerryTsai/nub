use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
