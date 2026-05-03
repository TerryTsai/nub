//! Engine wire types for `/containers/create`. PascalCase JSON via serde.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct CreateResp {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(default, rename = "Warnings")]
    pub warnings: Option<Vec<String>>,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(super) struct Body {
    pub image: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cmd: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub entrypoint: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub env: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub labels: HashMap<String, String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub exposed_ports: HashMap<String, EmptyObj>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub healthcheck: Option<HealthcheckWire>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_config: Option<HostConfig>,
}

#[derive(Serialize, Default)]
pub(super) struct EmptyObj {}

#[derive(Serialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(super) struct HostConfig {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub binds: Vec<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub port_bindings: HashMap<String, Vec<PortBindingWire>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart_policy: Option<RestartPolicyWire>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_shares: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_mode: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cap_add: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cap_drop: Vec<String>,
    pub privileged: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub devices: Vec<DeviceWire>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub extra_hosts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub init: Option<bool>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub tmpfs: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shm_size: Option<i64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ulimits: Vec<UlimitWire>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub sysctls: HashMap<String, String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dns: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct PortBindingWire {
    pub host_ip: String,
    pub host_port: String,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct RestartPolicyWire {
    pub name: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum_retry_count: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct HealthcheckWire {
    pub test: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retries: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_period: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct DeviceWire {
    pub path_on_host: String,
    pub path_in_container: String,
    pub cgroup_permissions: String,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct UlimitWire {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub soft: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hard: Option<i64>,
}
