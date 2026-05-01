use crate::proto::*;
use anyhow::Result;
use bollard::models::{
    ContainerInspectResponse, ContainerSummary as RawSummary, EndpointSettings as RawEndpoint,
    MountPoint as RawMount, PortBinding,
};

use super::util::short_id;
use super::DockerHandler;

impl DockerHandler {
    pub(super) async fn list_containers(&self, all: bool) -> Result<Vec<ContainerSummary>> {
        use bollard::container::ListContainersOptions;
        let opts = ListContainersOptions::<String> {
            all,
            ..Default::default()
        };
        let cs = self.docker.list_containers(Some(opts)).await?;
        Ok(cs.into_iter().map(summarize).collect())
    }

    pub(super) async fn inspect_container(&self, id: String) -> Result<ContainerDetail> {
        let r = self.docker.inspect_container(&id, None).await?;
        Ok(to_detail(r))
    }

    pub(super) async fn container_action(&self, id: String, action: Action) -> Result<()> {
        use bollard::container::{
            KillContainerOptions, RemoveContainerOptions, RestartContainerOptions,
            StopContainerOptions,
        };
        match action {
            Action::Start => {
                self.docker.start_container::<String>(&id, None).await?;
            }
            Action::Stop { timeout } => {
                let opts = timeout.map(|t| StopContainerOptions { t });
                self.docker.stop_container(&id, opts).await?;
            }
            Action::Restart { timeout } => {
                let opts = timeout.map(|t| RestartContainerOptions { t: t as isize });
                self.docker.restart_container(&id, opts).await?;
            }
            Action::Kill { signal } => {
                let opts = signal.map(|s| KillContainerOptions { signal: s });
                self.docker.kill_container(&id, opts).await?;
            }
            Action::Remove { force, volumes } => {
                let opts = RemoveContainerOptions {
                    force,
                    v: volumes,
                    link: false,
                };
                self.docker.remove_container(&id, Some(opts)).await?;
            }
        }
        Ok(())
    }
}

fn summarize(c: RawSummary) -> ContainerSummary {
    ContainerSummary {
        id: short_id(&c.id.unwrap_or_default()),
        name: c
            .names
            .and_then(|n| n.into_iter().next())
            .map(|n| n.trim_start_matches('/').to_string())
            .unwrap_or_default(),
        image: c.image.unwrap_or_default(),
        state: c.state.unwrap_or_default(),
        status: c.status.unwrap_or_default(),
        created: c.created.unwrap_or(0),
    }
}

fn to_detail(r: ContainerInspectResponse) -> ContainerDetail {
    let state = r.state.unwrap_or_default();
    let config = r.config.unwrap_or_default();
    let host = r.host_config.unwrap_or_default();
    let net = r.network_settings.unwrap_or_default();
    ContainerDetail {
        id: r.id.unwrap_or_default(),
        name: r
            .name
            .map(|n| n.trim_start_matches('/').to_string())
            .unwrap_or_default(),
        image: config.image.clone().unwrap_or_default(),
        image_id: r.image.unwrap_or_default(),
        created: r.created.unwrap_or_default(),
        state: state.status.map(|s| s.to_string()).unwrap_or_default(),
        running: state.running.unwrap_or(false),
        started_at: state.started_at.unwrap_or_default(),
        finished_at: state.finished_at.unwrap_or_default(),
        exit_code: state.exit_code.unwrap_or(0),
        error: state.error.unwrap_or_default(),
        restart_count: r.restart_count.unwrap_or(0),
        cmd: config.cmd.unwrap_or_default(),
        entrypoint: config.entrypoint.unwrap_or_default(),
        env: config.env.unwrap_or_default(),
        working_dir: config.working_dir.unwrap_or_default(),
        user: config.user.unwrap_or_default(),
        labels: config.labels.unwrap_or_default(),
        network_mode: host.network_mode.unwrap_or_default(),
        restart_policy: host
            .restart_policy
            .and_then(|p| p.name.map(|n| n.to_string()))
            .unwrap_or_default(),
        privileged: host.privileged.unwrap_or(false),
        memory_limit: host.memory.unwrap_or(0),
        mounts: r
            .mounts
            .unwrap_or_default()
            .into_iter()
            .map(to_mount)
            .collect(),
        networks: net
            .networks
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k, to_endpoint(v)))
            .collect(),
        ports: ports_from_map(net.ports),
    }
}

fn to_mount(m: RawMount) -> MountPoint {
    MountPoint {
        kind: m.typ.map(|t| t.to_string()).unwrap_or_default(),
        source: m.source.unwrap_or_default(),
        destination: m.destination.unwrap_or_default(),
        mode: m.mode.unwrap_or_default(),
        rw: m.rw.unwrap_or(false),
    }
}

fn to_endpoint(e: RawEndpoint) -> NetworkEndpoint {
    NetworkEndpoint {
        ip_address: e.ip_address.unwrap_or_default(),
        gateway: e.gateway.unwrap_or_default(),
        mac_address: e.mac_address.unwrap_or_default(),
    }
}

fn ports_from_map(
    map: Option<std::collections::HashMap<String, Option<Vec<PortBinding>>>>,
) -> Vec<PortMapping> {
    let Some(map) = map else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (port, bindings) in map {
        for b in bindings.unwrap_or_default() {
            out.push(PortMapping {
                container_port: port.clone(),
                host_ip: b.host_ip.unwrap_or_default(),
                host_port: b.host_port.unwrap_or_default(),
            });
        }
    }
    out
}
