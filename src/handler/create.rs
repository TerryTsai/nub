use crate::engine;
use crate::proto::*;
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

use super::EngineHandler;

impl EngineHandler {
    pub(super) async fn create_container(&self, req: CreateContainerReq) -> Result<ContainerCreated> {
        validate(&req, &self.policy.allowed_binds)?;
        let spec = to_engine_spec(&req);
        let resp = self.engine.create_container(spec).await?;
        let mut started = false;
        if req.start {
            self.engine
                .container_action(&resp.id, engine::ContainerAction::Start)
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

fn to_engine_spec(req: &CreateContainerReq) -> engine::CreateContainer {
    engine::CreateContainer {
        image: req.image.clone(),
        name: req.name.clone(),
        cmd: req.cmd.clone(),
        entrypoint: req.entrypoint.clone(),
        env: req.env.clone(),
        working_dir: req.working_dir.clone(),
        user: req.user.clone(),
        labels: req.labels.clone(),
        ports: req
            .ports
            .iter()
            .map(|p| engine::PortBinding {
                container: p.container.clone(),
                host: p.host.clone(),
            })
            .collect(),
        volumes: req
            .volumes
            .iter()
            .map(|v| engine::VolumeMount {
                source: v.source.clone(),
                target: v.target.clone(),
                read_only: v.read_only,
            })
            .collect(),
        network: req.network.clone(),
        restart: req.restart.as_ref().map(restart_policy),
        memory_limit: req.memory_limit,
        cpu_shares: req.cpu_shares,
    }
}

fn restart_policy(spec: &RestartPolicySpec) -> engine::RestartPolicy {
    match spec {
        RestartPolicySpec::No => engine::RestartPolicy::No,
        RestartPolicySpec::OnFailure { max_retries } => engine::RestartPolicy::OnFailure {
            max_retries: *max_retries,
        },
        RestartPolicySpec::Always => engine::RestartPolicy::Always,
        RestartPolicySpec::UnlessStopped => engine::RestartPolicy::UnlessStopped,
    }
}
