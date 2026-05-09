//! Map parsed compose YAML onto nub's existing op shapes. The output is
//! intentionally close to `CreateContainerReq` so the slice-2 stack
//! runtime can call `create_container` with minimal additional work.

use std::collections::{HashMap, HashSet};

use crate::proto::{CreateContainerReq, HealthcheckSpec, PortPublish, RestartPolicySpec, VolumeMount};

use super::configs as configs_xform;
use super::duration::parse_ns;
use super::secrets::{transform_service_refs, transform_top_level};
use super::types::{ParseError, ServiceSpec, StackSpec, VolumeSpec};
use super::wire::{Compose, HealthcheckYaml, MapOrList, ServiceYaml, StringOrList};

pub(super) fn transform(raw: Compose) -> Result<StackSpec, ParseError> {
    let secrets = transform_top_level(raw.secrets)?;
    let secret_names: HashSet<String> = secrets.iter().map(|s| s.name.clone()).collect();
    let configs = configs_xform::transform_top_level(raw.configs)?;
    let config_names: HashSet<String> = configs.iter().map(|c| c.name.clone()).collect();

    let mut services = Vec::with_capacity(raw.services.len());
    for (name, svc) in raw.services {
        services.push(transform_service(name, svc, &secret_names, &config_names)?);
    }
    services.sort_by(|a, b| a.name.cmp(&b.name));

    let mut volumes: Vec<_> = raw
        .volumes
        .into_iter()
        .map(|(name, v)| VolumeSpec {
            name,
            external: v.external,
        })
        .collect();
    volumes.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(StackSpec {
        services,
        volumes,
        secrets,
        configs,
        unsupported: sorted_keys(&raw.extra),
    })
}

fn transform_service(
    name: String,
    svc: ServiceYaml,
    declared_secrets: &HashSet<String>,
    declared_configs: &HashSet<String>,
) -> Result<ServiceSpec, ParseError> {
    let image = svc.image.ok_or_else(|| ParseError(format!("service `{name}` has no `image`")))?;
    let container = CreateContainerReq {
        image,
        name: svc.container_name,
        cmd: opt(svc.command, StringOrList::shell_split),
        entrypoint: opt(svc.entrypoint, StringOrList::shell_split),
        env: opt(svc.environment, MapOrList::into_kv_list),
        labels: opt(svc.labels, MapOrList::into_kv_map),
        working_dir: svc.working_dir,
        user: svc.user,
        ports: parse_ports(&svc.ports, &name)?,
        volumes: parse_volumes(&svc.volumes, &name)?,
        network: svc.network_mode,
        restart: parse_restart(svc.restart.as_deref(), &name)?,
        healthcheck: svc.healthcheck.map(transform_healthcheck).transpose()?,
        cap_add: svc.cap_add,
        cap_drop: svc.cap_drop,
        privileged: svc.privileged,
        extra_hosts: svc.extra_hosts,
        init: svc.init,
        hostname: svc.hostname,
        expose: svc.expose,
        ..Default::default()
    };
    Ok(ServiceSpec {
        secrets: transform_service_refs(&name, svc.secrets, declared_secrets)?,
        configs: configs_xform::transform_service_refs(&name, svc.configs, declared_configs)?,
        unsupported: sorted_keys(&svc.extra),
        name,
        container,
    })
}

/// `Option<T>` → `U` via `f`, defaulting to `U::default()` when `None`.
/// Compresses the `.map(f).unwrap_or_default()` pattern that appears
/// many times in the service-field initializer above.
fn opt<T, U: Default>(o: Option<T>, f: impl FnOnce(T) -> U) -> U {
    o.map(f).unwrap_or_default()
}

fn parse_ports(items: &[String], svc: &str) -> Result<Vec<PortPublish>, ParseError> {
    items.iter().map(|s| parse_port(s, svc)).collect()
}

fn parse_port(s: &str, svc: &str) -> Result<PortPublish, ParseError> {
    let parts: Vec<&str> = s.split(':').collect();
    match parts.as_slice() {
        [container] => Ok(PortPublish {
            container: (*container).into(),
            host: (*container).into(),
        }),
        [host, container] => Ok(PortPublish {
            container: (*container).into(),
            host: (*host).into(),
        }),
        [ip, host, container] => Ok(PortPublish {
            container: (*container).into(),
            host: format!("{ip}:{host}"),
        }),
        _ => Err(ParseError(format!("service `{svc}`: bad port `{s}`"))),
    }
}

fn parse_volumes(items: &[String], svc: &str) -> Result<Vec<VolumeMount>, ParseError> {
    items.iter().map(|s| parse_volume(s, svc)).collect()
}

fn parse_volume(s: &str, svc: &str) -> Result<VolumeMount, ParseError> {
    let parts: Vec<&str> = s.split(':').collect();
    match parts.as_slice() {
        [target] => Ok(VolumeMount {
            source: String::new(),
            target: (*target).into(),
            read_only: false,
        }),
        [source, target] => Ok(VolumeMount {
            source: (*source).into(),
            target: (*target).into(),
            read_only: false,
        }),
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
                .map(|n| {
                    n.parse::<i64>().map_err(|e| ParseError(format!("service `{svc}`: bad on-failure count: {e}")))
                })
                .transpose()?;
            RestartPolicySpec::OnFailure { max_retries }
        }
        other => return Err(ParseError(format!("service `{svc}`: unknown restart policy `{other}`"))),
    };
    Ok(Some(policy))
}

fn transform_healthcheck(hc: HealthcheckYaml) -> Result<HealthcheckSpec, ParseError> {
    if hc.disable {
        return Ok(HealthcheckSpec {
            test: vec!["NONE".into()],
            ..Default::default()
        });
    }
    Ok(HealthcheckSpec {
        test: opt(hc.test, StringOrList::into_list),
        interval_ns: hc.interval.as_deref().map(parse_ns).transpose()?,
        timeout_ns: hc.timeout.as_deref().map(parse_ns).transpose()?,
        retries: hc.retries,
        start_period_ns: hc.start_period.as_deref().map(parse_ns).transpose()?,
    })
}

fn sorted_keys<V>(m: &HashMap<String, V>) -> Vec<String> {
    let mut k: Vec<String> = m.keys().cloned().collect();
    k.sort();
    k
}
