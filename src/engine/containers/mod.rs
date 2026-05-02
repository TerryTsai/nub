//! Containers: list, inspect, create, lifecycle actions. Public types and
//! Engine method impls live here; wire decoders/encoders are in submodules.

mod action;
mod create;
mod inspect;
mod list;
mod types;

pub use types::{
    ContainerAction, ContainerCreated, ContainerDetail, ContainerSummary, CreateContainer,
    MountPoint, NetworkEndpoint, PortBinding, PortMapping, RestartPolicy, VolumeMount,
};

use super::{http::Query, Engine, EngineKind, Req, Result};

impl Engine {
    pub async fn list_containers(&self, all: bool) -> Result<Vec<ContainerSummary>> {
        let mut q = Query::new();
        q.push_bool("all", all);
        let path = format!("{}{}", self.list_path(), q.finish());
        let mut conn = self.conn().await?;
        let raw: Vec<list::ListItem> = conn.send_unary(Req::get(path).build()?).await?.json()?;
        Ok(raw.into_iter().map(list::ListItem::into_summary).collect())
    }

    pub async fn inspect_container(&self, id: &str) -> Result<ContainerDetail> {
        // Compat inspect path is stable on both engines and gives us a
        // richer body shape than libpod's.
        let path = format!("/containers/{id}/json");
        let mut conn = self.conn().await?;
        let raw: inspect::InspectResp = conn.send_unary(Req::get(path).build()?).await?.json()?;
        Ok(raw.into_detail())
    }

    pub async fn container_action(&self, id: &str, op: ContainerAction) -> Result<()> {
        action::run(self, id, op).await
    }

    pub async fn create_container(&self, spec: CreateContainer) -> Result<ContainerCreated> {
        create::run(self, spec).await
    }

    fn list_path(&self) -> &'static str {
        match self.kind() {
            // Libpod requires a version prefix on most paths (unlike compat,
            // which 301s a missing version to its default). v4.0.0 is broadly
            // accepted across podman 3+.
            EngineKind::Podman => "/v4.0.0/libpod/containers/json",
            EngineKind::Docker => "/containers/json",
        }
    }
}
