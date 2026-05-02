use crate::engine;
use crate::proto::*;
use anyhow::Result;

use super::EngineHandler;

impl EngineHandler {
    pub(super) async fn list_containers(&self, all: bool) -> Result<Vec<ContainerSummary>> {
        let cs = self.engine.list_containers(all).await?;
        Ok(cs.into_iter().map(to_summary).collect())
    }

    pub(super) async fn inspect_container(&self, id: String) -> Result<Box<ContainerDetail>> {
        let d = self.engine.inspect_container(&id).await?;
        Ok(Box::new(to_detail(d)))
    }

    pub(super) async fn container_action(&self, id: String, action: Action) -> Result<()> {
        let mapped = match action {
            Action::Start => engine::ContainerAction::Start,
            Action::Stop { timeout } => engine::ContainerAction::Stop { timeout },
            Action::Restart { timeout } => engine::ContainerAction::Restart { timeout },
            Action::Kill { signal } => engine::ContainerAction::Kill { signal },
            Action::Remove { force, volumes } => engine::ContainerAction::Remove { force, volumes },
        };
        self.engine.container_action(&id, mapped).await?;
        Ok(())
    }
}

fn to_summary(c: engine::ContainerSummary) -> ContainerSummary {
    ContainerSummary {
        id: c.id,
        name: c.name,
        image: c.image,
        state: c.state,
        status: c.status,
        created: c.created,
    }
}

fn to_detail(d: engine::ContainerDetail) -> ContainerDetail {
    ContainerDetail {
        id: d.id,
        name: d.name,
        image: d.image,
        image_id: d.image_id,
        created: d.created,
        state: d.state,
        running: d.running,
        started_at: d.started_at,
        finished_at: d.finished_at,
        exit_code: d.exit_code,
        error: d.error,
        restart_count: d.restart_count,
        cmd: d.cmd,
        entrypoint: d.entrypoint,
        env: d.env,
        working_dir: d.working_dir,
        user: d.user,
        labels: d.labels,
        network_mode: d.network_mode,
        restart_policy: d.restart_policy,
        privileged: d.privileged,
        memory_limit: d.memory_limit,
        mounts: d.mounts.into_iter().map(to_mount).collect(),
        networks: d.networks.into_iter().map(|(k, v)| (k, to_endpoint(v))).collect(),
        ports: d.ports.into_iter().map(to_port).collect(),
    }
}

fn to_mount(m: engine::MountPoint) -> MountPoint {
    MountPoint {
        kind: m.kind,
        source: m.source,
        destination: m.destination,
        mode: m.mode,
        rw: m.rw,
    }
}

fn to_endpoint(e: engine::NetworkEndpoint) -> NetworkEndpoint {
    NetworkEndpoint {
        ip_address: e.ip_address,
        gateway: e.gateway,
        mac_address: e.mac_address,
    }
}

fn to_port(p: engine::PortMapping) -> PortMapping {
    PortMapping {
        container_port: p.container_port,
        host_ip: p.host_ip,
        host_port: p.host_port,
    }
}
