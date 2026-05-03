use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
