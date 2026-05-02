//! `POST /containers/create`. Compat shape works on both Docker and Podman.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::types::{ContainerCreated, CreateContainer, PortBinding, RestartPolicy, VolumeMount};
use crate::engine::{http::Query, Engine, Req, Result};

pub(super) async fn run(engine: &Engine, spec: CreateContainer) -> Result<ContainerCreated> {
    let mut q = Query::new();
    if let Some(name) = &spec.name {
        q.push("name", name);
    }
    let body = build_body(&spec);
    let path = format!("/containers/create{}", q.finish());
    let mut conn = engine.conn().await?;
    let raw: CreateResp = conn
        .send_unary(Req::post(path).json(&body)?.build()?)
        .await?
        .json()?;
    Ok(ContainerCreated {
        id: raw.id,
        warnings: raw.warnings.unwrap_or_default(),
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct CreateResp {
    #[serde(rename = "Id")]
    id: String,
    #[serde(default, rename = "Warnings")]
    warnings: Option<Vec<String>>,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "PascalCase")]
struct Body {
    image: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    cmd: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    entrypoint: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    env: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    working_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    labels: HashMap<String, String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    exposed_ports: HashMap<String, EmptyObj>,
    #[serde(skip_serializing_if = "Option::is_none")]
    host_config: Option<HostConfig>,
}

#[derive(Serialize, Default)]
struct EmptyObj {}

#[derive(Serialize, Default)]
#[serde(rename_all = "PascalCase")]
struct HostConfig {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    binds: Vec<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    port_bindings: HashMap<String, Vec<PortBindingWire>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    restart_policy: Option<RestartPolicyWire>,
    #[serde(skip_serializing_if = "Option::is_none")]
    memory: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cpu_shares: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    network_mode: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct PortBindingWire {
    host_ip: String,
    host_port: String,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct RestartPolicyWire {
    name: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    maximum_retry_count: Option<i64>,
}

fn build_body(spec: &CreateContainer) -> Body {
    let (exposed, port_bindings) = build_ports(&spec.ports);
    let binds = build_binds(&spec.volumes);
    let host_config = HostConfig {
        binds,
        port_bindings,
        restart_policy: spec.restart.as_ref().map(restart_policy_wire),
        memory: spec.memory_limit,
        cpu_shares: spec.cpu_shares,
        network_mode: spec.network.clone(),
    };
    Body {
        image: spec.image.clone(),
        cmd: spec.cmd.clone(),
        entrypoint: spec.entrypoint.clone(),
        env: spec.env.clone(),
        working_dir: spec.working_dir.clone(),
        user: spec.user.clone(),
        labels: spec.labels.clone(),
        exposed_ports: exposed,
        host_config: Some(host_config),
    }
}

fn build_ports(ports: &[PortBinding]) -> (HashMap<String, EmptyObj>, HashMap<String, Vec<PortBindingWire>>) {
    let mut exposed = HashMap::new();
    let mut bindings: HashMap<String, Vec<PortBindingWire>> = HashMap::new();
    for p in ports {
        let cp = normalize_container_port(&p.container);
        exposed.insert(cp.clone(), EmptyObj {});
        let (host_ip, host_port) = parse_host(&p.host);
        bindings.entry(cp).or_default().push(PortBindingWire { host_ip, host_port });
    }
    (exposed, bindings)
}

fn build_binds(volumes: &[VolumeMount]) -> Vec<String> {
    volumes
        .iter()
        .filter(|v| is_host_path(&v.source))
        .map(|v| {
            let mode = if v.read_only { ":ro" } else { "" };
            format!("{}:{}{}", v.source, v.target, mode)
        })
        .collect()
}

fn normalize_container_port(s: &str) -> String {
    if s.contains('/') { s.to_string() } else { format!("{s}/tcp") }
}

fn parse_host(s: &str) -> (String, String) {
    match s.rfind(':') {
        Some(i) => (s[..i].to_string(), s[i + 1..].to_string()),
        None => (String::new(), s.to_string()),
    }
}

fn is_host_path(s: &str) -> bool {
    s.starts_with('/') || s.starts_with("./") || s.starts_with("../")
}

fn restart_policy_wire(spec: &RestartPolicy) -> RestartPolicyWire {
    match spec {
        RestartPolicy::No => RestartPolicyWire { name: "no", maximum_retry_count: None },
        RestartPolicy::OnFailure { max_retries } => RestartPolicyWire {
            name: "on-failure",
            maximum_retry_count: *max_retries,
        },
        RestartPolicy::Always => RestartPolicyWire { name: "always", maximum_retry_count: None },
        RestartPolicy::UnlessStopped => RestartPolicyWire {
            name: "unless-stopped",
            maximum_retry_count: None,
        },
    }
}
