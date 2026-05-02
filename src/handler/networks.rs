use crate::engine;
use crate::proto::*;
use anyhow::Result;

use super::EngineHandler;

impl EngineHandler {
    pub(super) async fn list_networks(&self) -> Result<Vec<NetworkSummary>> {
        let ns = self.engine.list_networks().await?;
        Ok(ns.into_iter().map(to_summary).collect())
    }

    pub(super) async fn remove_network(&self, id: String) -> Result<()> {
        self.engine.remove_network(&id).await?;
        Ok(())
    }
}

fn to_summary(n: engine::NetworkSummary) -> NetworkSummary {
    NetworkSummary {
        id: n.id,
        name: n.name,
        driver: n.driver,
        scope: n.scope,
        created: n.created,
        internal: n.internal,
    }
}
