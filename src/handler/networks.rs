use crate::proto::*;
use anyhow::Result;
use bollard::models::Network as RawNetwork;

use super::util::short_id;
use super::DockerHandler;

impl DockerHandler {
    pub(super) async fn list_networks(&self) -> Result<Vec<NetworkSummary>> {
        let nets = self.docker.list_networks::<String>(None).await?;
        Ok(nets.into_iter().map(summarize_network).collect())
    }

    pub(super) async fn remove_network(&self, id: String) -> Result<()> {
        self.docker.remove_network(&id).await?;
        Ok(())
    }
}

fn summarize_network(n: RawNetwork) -> NetworkSummary {
    NetworkSummary {
        id: short_id(&n.id.unwrap_or_default()),
        name: n.name.unwrap_or_default(),
        driver: n.driver.unwrap_or_default(),
        scope: n.scope.unwrap_or_default(),
        created: n.created.map(|d| d.to_string()).unwrap_or_default(),
        internal: n.internal.unwrap_or(false),
    }
}
