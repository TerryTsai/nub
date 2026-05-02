//! `docker container create` — `POST /containers/create`. Optionally starts
//! the container after create (the `start: true` field on the request).
//! Compat path works on both engines.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::client::{Query, Req};
use crate::ops::EngineHandler;
use crate::proto::{ContainerCreated, CreateContainerReq, PortPublish, RestartPolicySpec, VolumeMount};

pub(crate) async fn run(h: &EngineHandler, req: CreateContainerReq) -> Result<ContainerCreated> {
    validate(&req, &h.policy.allowed_binds)?;
    let body = build_body(&req);
    let path = format!("/containers/create{}", create_query(req.name.as_deref()));
    let resp: CreateResp = h
        .engine
        .conn()
        .await?
        .send_unary(Req::post(path).json(&body)?.build()?)
        .await?
        .json()?;

    let mut started = false;
    if req.start {
        h.engine
            .conn()
            .await?
            .send_unary(Req::post(format!("/containers/{}/start", resp.id)).build()?)
            .await?
            .ok()?;
        started = true;
    }
    Ok(ContainerCreated {
        id: resp.id,
        started,
        warnings: resp.warnings.unwrap_or_default(),
    })
}

fn create_query(name: Option<&str>) -> String {
    let mut q = Query::new();
    if let Some(n) = name {
        q.push("name", n);
    }
    q.finish()
}

fn validate(req: &CreateContainerReq, allowed_binds: &[PathBuf]) -> Result<()> {
    if let Some(net) = &req.network {
        if net == "host" || net.starts_with("container:") {
            return Err(anyhow!("network mode '{net}' not allowed"));
        }
    }
    for v in &req.volumes {
        if !is_host_path(&v.source) {
            continue;
        }
        let src = Path::new(&v.source);
        if !allowed_binds.iter().any(|p| src.starts_with(p)) {
            return Err(anyhow!("bind source '{}' not in allowed_binds", v.source));
        }
    }
    Ok(())
}

fn is_host_path(s: &str) -> bool {
    s.starts_with('/') || s.starts_with("./") || s.starts_with("../")
}

// ---- Wire body shaping ---------------------------------------------------

fn build_body(req: &CreateContainerReq) -> Body {
    let (exposed, port_bindings) = build_ports(&req.ports);
    let host_config = HostConfig {
        binds: build_binds(&req.volumes),
        port_bindings,
        restart_policy: req.restart.as_ref().map(restart_policy_wire),
        memory: req.memory_limit,
        cpu_shares: req.cpu_shares,
        network_mode: req.network.clone(),
    };
    Body {
        image: req.image.clone(),
        cmd: req.cmd.clone(),
        entrypoint: req.entrypoint.clone(),
        env: req.env.clone(),
        working_dir: req.working_dir.clone(),
        user: req.user.clone(),
        labels: req.labels.clone(),
        exposed_ports: exposed,
        host_config: Some(host_config),
    }
}

fn build_ports(ports: &[PortPublish]) -> (HashMap<String, EmptyObj>, HashMap<String, Vec<PortBindingWire>>) {
    let mut exposed = HashMap::new();
    let mut bindings: HashMap<String, Vec<PortBindingWire>> = HashMap::new();
    for p in ports {
        let cp = normalize_container_port(&p.container);
        exposed.insert(cp.clone(), EmptyObj {});
        let (host_ip, host_port) = parse_host(&p.host);
        bindings
            .entry(cp)
            .or_default()
            .push(PortBindingWire { host_ip, host_port });
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
    if s.contains('/') {
        s.to_string()
    } else {
        format!("{s}/tcp")
    }
}

fn parse_host(s: &str) -> (String, String) {
    match s.rfind(':') {
        Some(i) => (s[..i].to_string(), s[i + 1..].to_string()),
        None => (String::new(), s.to_string()),
    }
}

fn restart_policy_wire(spec: &RestartPolicySpec) -> RestartPolicyWire {
    match spec {
        RestartPolicySpec::No => RestartPolicyWire {
            name: "no",
            maximum_retry_count: None,
        },
        RestartPolicySpec::OnFailure { max_retries } => RestartPolicyWire {
            name: "on-failure",
            maximum_retry_count: *max_retries,
        },
        RestartPolicySpec::Always => RestartPolicyWire {
            name: "always",
            maximum_retry_count: None,
        },
        RestartPolicySpec::UnlessStopped => RestartPolicyWire {
            name: "unless-stopped",
            maximum_retry_count: None,
        },
    }
}

// ---- Wire types ----------------------------------------------------------

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
