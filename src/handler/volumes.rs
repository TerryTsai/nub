use crate::engine;
use crate::proto::*;
use anyhow::Result;

use super::EngineHandler;

impl EngineHandler {
    pub(super) async fn list_volumes(&self) -> Result<Vec<VolumeSummary>> {
        let vs = self.engine.list_volumes().await?;
        Ok(vs.into_iter().map(to_summary).collect())
    }

    pub(super) async fn remove_volume(&self, name: String, force: bool) -> Result<()> {
        self.engine.remove_volume(&name, force).await?;
        Ok(())
    }
}

fn to_summary(v: engine::VolumeSummary) -> VolumeSummary {
    VolumeSummary {
        name: v.name,
        driver: v.driver,
        mountpoint: v.mountpoint,
        created_at: v.created_at,
        scope: v.scope,
    }
}
