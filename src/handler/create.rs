use crate::proto::*;
use anyhow::{anyhow, Result};
use bollard::container::{Config, CreateContainerOptions};
use bollard::models::{HostConfig, PortBinding, RestartPolicy, RestartPolicyNameEnum};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::DockerHandler;

impl DockerHandler {
    pub(super) async fn create_container(
        &self,
        req: CreateContainerReq,
    ) -> Result<ContainerCreated> {
        validate(&req, &self.policy.allowed_binds)?;
        let create_opts = req.name.as_deref().map(|n| CreateContainerOptions {
            name: n.to_string(),
            platform: None,
        });
        let cfg = build_config(&req);
        let resp = self
            .docker
            .create_container::<String, String>(create_opts, cfg)
            .await?;
        let mut started = false;
        if req.start {
            self.docker
                .start_container::<String>(&resp.id, None)
                .await?;
            started = true;
        }
        Ok(ContainerCreated {
            id: resp.id,
            started,
            warnings: resp.warnings,
        })
    }
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

fn build_config(req: &CreateContainerReq) -> Config<String> {
    Config {
        image: Some(req.image.clone()),
        cmd: nonempty_vec(&req.cmd),
        entrypoint: nonempty_vec(&req.entrypoint),
        env: nonempty_vec(&req.env),
        working_dir: req.working_dir.clone(),
        user: req.user.clone(),
        labels: nonempty_map(&req.labels),
        exposed_ports: build_exposed_ports(&req.ports),
        host_config: Some(HostConfig {
            binds: build_binds(&req.volumes),
            port_bindings: build_port_bindings(&req.ports),
            restart_policy: req.restart.as_ref().map(to_restart_policy),
            memory: req.memory_limit,
            cpu_shares: req.cpu_shares,
            network_mode: req.network.clone(),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn nonempty_vec(v: &[String]) -> Option<Vec<String>> {
    if v.is_empty() {
        None
    } else {
        Some(v.to_vec())
    }
}

fn nonempty_map(m: &HashMap<String, String>) -> Option<HashMap<String, String>> {
    if m.is_empty() {
        None
    } else {
        Some(m.clone())
    }
}

fn build_exposed_ports(ports: &[PortPublish]) -> Option<HashMap<String, HashMap<(), ()>>> {
    if ports.is_empty() {
        return None;
    }
    Some(
        ports
            .iter()
            .map(|p| (normalize_container_port(&p.container), HashMap::new()))
            .collect(),
    )
}

fn build_port_bindings(ports: &[PortPublish]) -> Option<HashMap<String, Option<Vec<PortBinding>>>> {
    if ports.is_empty() {
        return None;
    }
    let mut grouped: HashMap<String, Vec<PortBinding>> = HashMap::new();
    for p in ports {
        let (host_ip, host_port) = parse_host(&p.host);
        grouped
            .entry(normalize_container_port(&p.container))
            .or_default()
            .push(PortBinding {
                host_ip: Some(host_ip),
                host_port: Some(host_port),
            });
    }
    Some(grouped.into_iter().map(|(k, v)| (k, Some(v))).collect())
}

fn normalize_container_port(s: &str) -> String {
    if s.contains('/') {
        s.to_string()
    } else {
        format!("{s}/tcp")
    }
}

fn parse_host(s: &str) -> (String, String) {
    if let Some(idx) = s.rfind(':') {
        (s[..idx].to_string(), s[idx + 1..].to_string())
    } else {
        (String::new(), s.to_string())
    }
}

fn build_binds(volumes: &[VolumeMount]) -> Option<Vec<String>> {
    if volumes.is_empty() {
        return None;
    }
    Some(
        volumes
            .iter()
            .map(|v| {
                let mode = if v.read_only { ":ro" } else { "" };
                format!("{}:{}{}", v.source, v.target, mode)
            })
            .collect(),
    )
}

fn to_restart_policy(spec: &RestartPolicySpec) -> RestartPolicy {
    let (name, retry) = match spec {
        RestartPolicySpec::No => (RestartPolicyNameEnum::NO, None),
        RestartPolicySpec::OnFailure { max_retries } => {
            (RestartPolicyNameEnum::ON_FAILURE, *max_retries)
        }
        RestartPolicySpec::Always => (RestartPolicyNameEnum::ALWAYS, None),
        RestartPolicySpec::UnlessStopped => (RestartPolicyNameEnum::UNLESS_STOPPED, None),
    };
    RestartPolicy {
        name: Some(name),
        maximum_retry_count: retry,
    }
}
