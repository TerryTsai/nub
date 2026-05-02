//! Decoder for `/containers/{id}/json` inspect response. Both engines use
//! the same compat-shaped body.

use std::collections::HashMap;

use serde::Deserialize;

use super::types::{ContainerDetail, MountPoint, NetworkEndpoint, PortMapping};

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct InspectResp {
    #[serde(rename = "Id", default)]
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    image: String,
    #[serde(default)]
    created: String,
    #[serde(default)]
    restart_count: i64,
    #[serde(default)]
    state: State,
    #[serde(default)]
    config: Config,
    #[serde(default)]
    host_config: HostConfig,
    #[serde(default)]
    network_settings: NetSettings,
    #[serde(default)]
    mounts: Vec<Mount>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct State {
    #[serde(default)]
    status: String,
    #[serde(default)]
    running: bool,
    #[serde(default)]
    started_at: String,
    #[serde(default)]
    finished_at: String,
    #[serde(default)]
    exit_code: i64,
    #[serde(default)]
    error: String,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Config {
    #[serde(default)]
    image: String,
    #[serde(default)]
    cmd: Vec<String>,
    #[serde(default)]
    entrypoint: Vec<String>,
    #[serde(default)]
    env: Vec<String>,
    #[serde(default)]
    working_dir: String,
    #[serde(default)]
    user: String,
    #[serde(default)]
    labels: HashMap<String, String>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct HostConfig {
    #[serde(default)]
    network_mode: String,
    #[serde(default)]
    restart_policy: RestartPolicyName,
    #[serde(default)]
    privileged: bool,
    #[serde(default)]
    memory: i64,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RestartPolicyName {
    #[serde(default)]
    name: String,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct NetSettings {
    #[serde(default)]
    networks: HashMap<String, NetEndpoint>,
    #[serde(default)]
    ports: HashMap<String, Option<Vec<PortBinding>>>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct NetEndpoint {
    #[serde(default, rename = "IPAddress")]
    ip_address: String,
    #[serde(default)]
    gateway: String,
    #[serde(default, rename = "MacAddress")]
    mac_address: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PortBinding {
    #[serde(default, rename = "HostIp")]
    host_ip: String,
    #[serde(default, rename = "HostPort")]
    host_port: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Mount {
    #[serde(default, rename = "Type")]
    typ: String,
    #[serde(default)]
    source: String,
    #[serde(default)]
    destination: String,
    #[serde(default)]
    mode: String,
    #[serde(default, rename = "RW")]
    rw: bool,
}

impl InspectResp {
    pub(super) fn into_detail(self) -> ContainerDetail {
        let networks = self.network_settings.networks.into_iter().map(net_endpoint).collect();
        let ports = unwrap_ports(self.network_settings.ports);
        let mounts = self.mounts.into_iter().map(mount_point).collect();
        let image_id = self.image;
        let image = if self.config.image.is_empty() { image_id.clone() } else { self.config.image };
        ContainerDetail {
            id: self.id,
            name: self.name.trim_start_matches('/').to_string(),
            image,
            image_id,
            created: self.created,
            state: self.state.status,
            running: self.state.running,
            started_at: self.state.started_at,
            finished_at: self.state.finished_at,
            exit_code: self.state.exit_code,
            error: self.state.error,
            restart_count: self.restart_count,
            cmd: self.config.cmd,
            entrypoint: self.config.entrypoint,
            env: self.config.env,
            working_dir: self.config.working_dir,
            user: self.config.user,
            labels: self.config.labels,
            network_mode: self.host_config.network_mode,
            restart_policy: self.host_config.restart_policy.name,
            privileged: self.host_config.privileged,
            memory_limit: self.host_config.memory,
            mounts,
            networks,
            ports,
        }
    }
}

fn net_endpoint((k, v): (String, NetEndpoint)) -> (String, NetworkEndpoint) {
    (
        k,
        NetworkEndpoint {
            ip_address: v.ip_address,
            gateway: v.gateway,
            mac_address: v.mac_address,
        },
    )
}

fn unwrap_ports(map: HashMap<String, Option<Vec<PortBinding>>>) -> Vec<PortMapping> {
    let mut out = Vec::new();
    for (cport, bindings) in map {
        for b in bindings.unwrap_or_default() {
            out.push(PortMapping {
                container_port: cport.clone(),
                host_ip: b.host_ip,
                host_port: b.host_port,
            });
        }
    }
    out
}

fn mount_point(m: Mount) -> MountPoint {
    MountPoint {
        kind: m.typ,
        source: m.source,
        destination: m.destination,
        mode: m.mode,
        rw: m.rw,
    }
}
