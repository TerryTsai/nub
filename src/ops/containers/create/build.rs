//! Translate `CreateContainerReq` into the engine wire body.

use std::collections::HashMap;

use crate::proto::{
    CreateContainerReq, DeviceMapping, HealthcheckSpec, PortPublish, RestartPolicySpec, UlimitSpec, VolumeMount,
};

use super::is_host_path;
use super::wire::{
    Body, DeviceWire, EmptyObj, HealthcheckWire, HostConfig, PortBindingWire, RestartPolicyWire, UlimitWire,
};

pub(super) fn body(req: &CreateContainerReq) -> Body {
    let exposed_ports = build_exposed(&req.ports, &req.expose);
    let port_bindings = build_port_bindings(&req.ports);
    let host_config = build_host_config(req, port_bindings);
    Body {
        image: req.image.clone(),
        cmd: req.cmd.clone(),
        entrypoint: req.entrypoint.clone(),
        env: req.env.clone(),
        working_dir: req.working_dir.clone(),
        user: req.user.clone(),
        labels: req.labels.clone(),
        exposed_ports,
        hostname: req.hostname.clone(),
        healthcheck: req.healthcheck.as_ref().map(healthcheck_wire),
        host_config: Some(host_config),
    }
}

fn build_host_config(req: &CreateContainerReq, port_bindings: HashMap<String, Vec<PortBindingWire>>) -> HostConfig {
    HostConfig {
        binds: build_binds(&req.volumes),
        port_bindings,
        restart_policy: req.restart.as_ref().map(restart_policy_wire),
        memory: req.memory_limit,
        cpu_shares: req.cpu_shares,
        network_mode: req.network.clone(),
        cap_add: req.cap_add.clone(),
        cap_drop: req.cap_drop.clone(),
        privileged: req.privileged,
        devices: build_devices(&req.devices),
        extra_hosts: req.extra_hosts.clone(),
        init: req.init,
        tmpfs: req.tmpfs.clone(),
        shm_size: req.shm_size,
        ulimits: build_ulimits(&req.ulimits),
        sysctls: req.sysctls.clone(),
        dns: req.dns.clone(),
    }
}

fn build_port_bindings(ports: &[PortPublish]) -> HashMap<String, Vec<PortBindingWire>> {
    let mut bindings: HashMap<String, Vec<PortBindingWire>> = HashMap::new();
    for p in ports {
        let cp = normalize_container_port(&p.container);
        let (host_ip, host_port) = parse_host(&p.host);
        bindings
            .entry(cp)
            .or_default()
            .push(PortBindingWire { host_ip, host_port });
    }
    bindings
}

fn build_exposed(ports: &[PortPublish], expose: &[String]) -> HashMap<String, EmptyObj> {
    let mut exposed = HashMap::new();
    for p in ports {
        exposed.insert(normalize_container_port(&p.container), EmptyObj {});
    }
    for e in expose {
        exposed.insert(normalize_container_port(e), EmptyObj {});
    }
    exposed
}

fn build_devices(devs: &[DeviceMapping]) -> Vec<DeviceWire> {
    devs.iter()
        .map(|d| DeviceWire {
            path_on_host: d.host.clone(),
            path_in_container: if d.container.is_empty() {
                d.host.clone()
            } else {
                d.container.clone()
            },
            cgroup_permissions: d.permissions.clone().unwrap_or_else(|| "rwm".into()),
        })
        .collect()
}

fn build_ulimits(ulimits: &[UlimitSpec]) -> Vec<UlimitWire> {
    ulimits
        .iter()
        .map(|u| UlimitWire {
            name: u.name.clone(),
            soft: u.soft,
            hard: u.hard,
        })
        .collect()
}

fn healthcheck_wire(spec: &HealthcheckSpec) -> HealthcheckWire {
    HealthcheckWire {
        test: spec.test.clone(),
        interval: spec.interval_ns,
        timeout: spec.timeout_ns,
        retries: spec.retries,
        start_period: spec.start_period_ns,
    }
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
