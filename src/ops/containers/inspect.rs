//! `docker container inspect` — `GET /containers/{id}/json`. Compat shape;
//! both engines return the same body for this endpoint.

use std::collections::HashMap;

use anyhow::Result;
use serde::Deserialize;

use crate::client::Req;
use crate::ops::EngineHandler;
use crate::proto::{ContainerDetail, MountPoint, NetworkEndpoint, PortMapping};

pub(crate) async fn run(h: &EngineHandler, id: String) -> Result<Box<ContainerDetail>> {
    let path = format!("/containers/{id}/json");
    let raw: RawInspect = h
        .engine
        .conn()
        .await?
        .send_unary(Req::get(path).build()?)
        .await?
        .json()?;
    Ok(Box::new(raw.into_detail()))
}

// ---- Wire decoder --------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawInspect {
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
    state: RawState,
    #[serde(default)]
    config: RawConfig,
    #[serde(default)]
    host_config: RawHostConfig,
    #[serde(default)]
    network_settings: RawNetSettings,
    #[serde(default)]
    mounts: Vec<RawMount>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawState {
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
    #[serde(default)]
    health: Option<RawHealth>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawHealth {
    #[serde(default)]
    status: String,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawConfig {
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
    #[serde(default, deserialize_with = "crate::ops::serde_helpers::null_to_default")]
    labels: HashMap<String, String>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawHostConfig {
    #[serde(default)]
    network_mode: String,
    #[serde(default)]
    restart_policy: RawRestartPolicy,
    #[serde(default)]
    privileged: bool,
    #[serde(default)]
    memory: i64,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawRestartPolicy {
    #[serde(default)]
    name: String,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawNetSettings {
    #[serde(default, deserialize_with = "crate::ops::serde_helpers::null_to_default")]
    networks: HashMap<String, RawNetEndpoint>,
    #[serde(default, deserialize_with = "crate::ops::serde_helpers::null_to_default")]
    ports: HashMap<String, Option<Vec<RawPortBinding>>>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawNetEndpoint {
    #[serde(default, rename = "IPAddress")]
    ip_address: String,
    #[serde(default)]
    gateway: String,
    #[serde(default, rename = "MacAddress")]
    mac_address: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawPortBinding {
    #[serde(default, rename = "HostIp")]
    host_ip: String,
    #[serde(default, rename = "HostPort")]
    host_port: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawMount {
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

// ---- Translate to proto::ContainerDetail ---------------------------------

impl RawInspect {
    fn into_detail(self) -> ContainerDetail {
        let networks = self.network_settings.networks.into_iter().map(net_endpoint).collect();
        let ports = unwrap_ports(self.network_settings.ports);
        let mounts = self.mounts.into_iter().map(mount).collect();
        let image_id = self.image;
        let image = if self.config.image.is_empty() {
            image_id.clone()
        } else {
            self.config.image
        };
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
            health: self.state.health.map(|h| h.status).unwrap_or_default(),
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

fn net_endpoint((k, v): (String, RawNetEndpoint)) -> (String, NetworkEndpoint) {
    (
        k,
        NetworkEndpoint {
            ip_address: v.ip_address,
            gateway: v.gateway,
            mac_address: v.mac_address,
        },
    )
}

fn unwrap_ports(map: HashMap<String, Option<Vec<RawPortBinding>>>) -> Vec<PortMapping> {
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

fn mount(m: RawMount) -> MountPoint {
    MountPoint {
        kind: m.typ,
        source: m.source,
        destination: m.destination,
        mode: m.mode,
        rw: m.rw,
    }
}
