//! `create_stack` — parse compose YAML, save to disk, then provision the
//! network, named volumes, and one container per service. Failure
//! mid-deploy leaves whatever was created intact; the caller is expected
//! to follow up with `delete_stack` for cleanup. Compose itself behaves
//! the same way and we don't try to be smarter.

use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Result};

use crate::compose;
use crate::ops::configs;
use crate::ops::containers;
use crate::ops::secrets;
use crate::ops::EngineHandler;
use crate::proto::{CreateContainerReq, StackCreated, VolumeMount};

use super::engine;
use super::labels::{container_name, network_name, stack_labels, volume_name, STACK_LABEL};
use super::store;

pub(crate) async fn run(h: &EngineHandler, name: String, yaml: String) -> Result<StackCreated> {
    store::validate_name(&name)?;
    if store::exists(&h.policy.stacks_root, &name) {
        return Err(anyhow!("stack `{name}` already exists; use redeploy or update"));
    }
    let spec = compose::parse(&yaml, &HashMap::new()).map_err(|e| anyhow!("compose: {e}"))?;
    if spec.services.is_empty() {
        return Err(anyhow!("stack `{name}` has no services"));
    }
    store::write_yaml(&h.policy.stacks_root, &name, &yaml)?;
    deploy_from_spec(h, &name, spec).await.map(|ids| StackCreated {
        name,
        container_ids: ids,
    })
}

/// Provision engine resources from an already-parsed spec. Used by both
/// `create_stack` and `redeploy_stack`. Caller is responsible for the
/// on-disk manifest and for tearing down any prior resources.
pub(super) async fn deploy_from_spec(h: &EngineHandler, name: &str, spec: compose::StackSpec) -> Result<Vec<String>> {
    let stack_label_only = label_only(name);
    engine::create_network(h, &network_name(name), stack_label_only.clone()).await?;

    let declared_volumes: HashSet<String> = spec.volumes.iter().map(|v| v.name.clone()).collect();
    for v in &spec.volumes {
        if v.external {
            continue;
        }
        engine::create_volume(h, &volume_name(name, &v.name), stack_label_only.clone()).await?;
    }

    let mut ids = Vec::with_capacity(spec.services.len());
    let mut spec = spec;
    let services = std::mem::take(&mut spec.services);
    for svc in services {
        let id = create_service(h, name, svc, &spec, &declared_volumes).await?;
        ids.push(id);
    }
    Ok(ids)
}

fn label_only(name: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert(STACK_LABEL.to_string(), name.to_string());
    m
}

async fn create_service(
    h: &EngineHandler,
    stack: &str,
    svc: compose::ServiceSpec,
    stack_spec: &compose::StackSpec,
    declared_volumes: &HashSet<String>,
) -> Result<String> {
    let secret_mounts =
        secrets::runtime::materialize_for_service(&h.policy.secrets_root, stack_spec, stack, &svc.name, &svc.secrets)
            .await?;
    let config_mounts = configs::runtime::materialize_for_service(stack_spec, stack, &svc.name, &svc.configs).await?;
    let extra_mounts = secret_mounts.into_iter().chain(config_mounts).collect();
    let req = build_request(stack, svc, declared_volumes, extra_mounts);
    let created = containers::create::run(h, req).await?;
    Ok(created.id)
}

fn build_request(
    stack: &str,
    svc: compose::ServiceSpec,
    declared_volumes: &HashSet<String>,
    extra_mounts: Vec<VolumeMount>,
) -> CreateContainerReq {
    let mut req = svc.container;
    req.name = Some(container_name(stack, &svc.name, req.name.as_deref()));
    if req.network.is_none() {
        req.network = Some(network_name(stack));
    }
    req.volumes = req
        .volumes
        .into_iter()
        .map(|v| rewrite_volume(stack, v, declared_volumes))
        .chain(extra_mounts)
        .collect();
    merge_labels(&mut req.labels, stack, &svc.name);
    req.start = true;
    req
}

fn rewrite_volume(stack: &str, mut v: VolumeMount, declared_volumes: &HashSet<String>) -> VolumeMount {
    if declared_volumes.contains(&v.source) {
        v.source = volume_name(stack, &v.source);
    }
    v
}

fn merge_labels(labels: &mut HashMap<String, String>, stack: &str, service: &str) {
    for (k, v) in stack_labels(stack, service) {
        labels.insert(k, v);
    }
}
