//! Map parsed compose YAML onto nub's existing op shapes. The output is
//! intentionally close to `CreateContainerReq` so the slice-2 stack
//! runtime can call `create_container` with minimal additional work.

use std::collections::HashMap;

use crate::proto::{CreateContainerReq, HealthcheckSpec, PortPublish, RestartPolicySpec, VolumeMount};

use super::spec::{ParseError, ServiceSpec, StackSpec, VolumeSpec};
use super::wire::{Compose, HealthcheckYaml, MapOrList, ServiceYaml, StringOrList};

pub(super) fn transform(raw: Compose) -> Result<StackSpec, ParseError> {
    let mut services = Vec::with_capacity(raw.services.len());
    for (name, svc) in raw.services {
        services.push(transform_service(name, svc)?);
    }
    services.sort_by(|a, b| a.name.cmp(&b.name));

    let mut volumes: Vec<_> = raw
        .volumes
        .into_iter()
        .map(|(name, v)| VolumeSpec { name, external: v.external })
        .collect();
    volumes.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(StackSpec { services, volumes, unsupported: sorted_keys(&raw.extra) })
}

fn transform_service(name: String, svc: ServiceYaml) -> Result<ServiceSpec, ParseError> {
    let image = svc.image.ok_or_else(|| ParseError(format!("service `{name}` has no `image`")))?;
    let unsupported = sorted_keys(&svc.extra);
    let container = CreateContainerReq {
        image,
        name: svc.container_name,
        cmd: svc.command.map(StringOrList::shell_split).unwrap_or_default(),
        entrypoint: svc.entrypoint.map(StringOrList::shell_split).unwrap_or_default(),
        env: svc.environment.map(MapOrList::into_kv_list).unwrap_or_default(),
        working_dir: svc.working_dir,
        user: svc.user,
        labels: svc.labels.map(MapOrList::into_kv_map).unwrap_or_default(),
        ports: parse_ports(&svc.ports, &name)?,
        volumes: parse_volumes(&svc.volumes, &name)?,
        network: svc.network_mode,
        restart: parse_restart(svc.restart.as_deref(), &name)?,
        memory_limit: None,
        cpu_shares: None,
        healthcheck: svc.healthcheck.map(transform_healthcheck).transpose()?,
        cap_add: svc.cap_add,
        cap_drop: svc.cap_drop,
        privileged: svc.privileged,
        devices: vec![],
        extra_hosts: svc.extra_hosts,
        init: svc.init,
        tmpfs: HashMap::new(),
        shm_size: None,
        ulimits: vec![],
        sysctls: HashMap::new(),
        hostname: svc.hostname,
        dns: vec![],
        expose: svc.expose,
        start: false,
    };
    Ok(ServiceSpec { name, container, unsupported })
}

fn parse_ports(items: &[String], svc: &str) -> Result<Vec<PortPublish>, ParseError> {
    items.iter().map(|s| parse_port(s, svc)).collect()
}

fn parse_port(s: &str, svc: &str) -> Result<PortPublish, ParseError> {
    let parts: Vec<&str> = s.split(':').collect();
    match parts.as_slice() {
        [container] => Ok(PortPublish { container: (*container).into(), host: (*container).into() }),
        [host, container] => Ok(PortPublish { container: (*container).into(), host: (*host).into() }),
        [ip, host, container] => {
            Ok(PortPublish { container: (*container).into(), host: format!("{ip}:{host}") })
        }
        _ => Err(ParseError(format!("service `{svc}`: bad port `{s}`"))),
    }
}

fn parse_volumes(items: &[String], svc: &str) -> Result<Vec<VolumeMount>, ParseError> {
    items.iter().map(|s| parse_volume(s, svc)).collect()
}

fn parse_volume(s: &str, svc: &str) -> Result<VolumeMount, ParseError> {
    let parts: Vec<&str> = s.split(':').collect();
    match parts.as_slice() {
        [target] => Ok(VolumeMount { source: String::new(), target: (*target).into(), read_only: false }),
        [source, target] => Ok(VolumeMount { source: (*source).into(), target: (*target).into(), read_only: false }),
        [source, target, mode] => Ok(VolumeMount {
            source: (*source).into(),
            target: (*target).into(),
            read_only: *mode == "ro",
        }),
        _ => Err(ParseError(format!("service `{svc}`: bad volume `{s}`"))),
    }
}

fn parse_restart(s: Option<&str>, svc: &str) -> Result<Option<RestartPolicySpec>, ParseError> {
    let Some(value) = s else {
        return Ok(None);
    };
    let policy = match value {
        "no" => RestartPolicySpec::No,
        "always" => RestartPolicySpec::Always,
        "unless-stopped" => RestartPolicySpec::UnlessStopped,
        v if v == "on-failure" || v.starts_with("on-failure:") => {
            let max_retries = v
                .strip_prefix("on-failure:")
                .map(|n| n.parse::<i64>().map_err(|e| ParseError(format!("service `{svc}`: bad on-failure count: {e}"))))
                .transpose()?;
            RestartPolicySpec::OnFailure { max_retries }
        }
        other => return Err(ParseError(format!("service `{svc}`: unknown restart policy `{other}`"))),
    };
    Ok(Some(policy))
}

fn transform_healthcheck(hc: HealthcheckYaml) -> Result<HealthcheckSpec, ParseError> {
    if hc.disable {
        return Ok(HealthcheckSpec { test: vec!["NONE".into()], ..Default::default() });
    }
    Ok(HealthcheckSpec {
        test: hc.test.map(StringOrList::into_list).unwrap_or_default(),
        interval_ns: hc.interval.as_deref().map(parse_duration_ns).transpose()?,
        timeout_ns: hc.timeout.as_deref().map(parse_duration_ns).transpose()?,
        retries: hc.retries,
        start_period_ns: hc.start_period.as_deref().map(parse_duration_ns).transpose()?,
    })
}

/// Compose-style duration: `1h30m`, `500ms`, `10s`. Output is nanoseconds
/// (the engine wire unit). Bare numbers are interpreted as seconds, matching
/// compose's behavior.
fn parse_duration_ns(s: &str) -> Result<i64, ParseError> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err(ParseError("empty duration".into()));
    }
    if let Ok(n) = trimmed.parse::<i64>() {
        return Ok(n.saturating_mul(1_000_000_000));
    }
    let mut total: i64 = 0;
    let mut num = String::new();
    let mut unit = String::new();
    for ch in trimmed.chars() {
        if ch.is_ascii_digit() {
            if !unit.is_empty() {
                total = total.saturating_add(consume_unit(&num, &unit)?);
                num.clear();
                unit.clear();
            }
            num.push(ch);
        } else {
            unit.push(ch);
        }
    }
    total = total.saturating_add(consume_unit(&num, &unit)?);
    Ok(total)
}

fn consume_unit(num: &str, unit: &str) -> Result<i64, ParseError> {
    let n: i64 = num.parse().map_err(|_| ParseError(format!("bad duration component `{num}{unit}`")))?;
    let mult: i64 = match unit {
        "ns" => 1,
        "us" | "µs" => 1_000,
        "ms" => 1_000_000,
        "s" => 1_000_000_000,
        "m" => 60_000_000_000,
        "h" => 3_600_000_000_000,
        other => return Err(ParseError(format!("unknown duration unit `{other}`"))),
    };
    Ok(n.saturating_mul(mult))
}

fn sorted_keys<V>(m: &HashMap<String, V>) -> Vec<String> {
    let mut k: Vec<String> = m.keys().cloned().collect();
    k.sort();
    k
}
