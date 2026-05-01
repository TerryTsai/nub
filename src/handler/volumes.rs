use crate::proto::*;
use anyhow::Result;
use bollard::models::Volume as RawVolume;

use super::DockerHandler;

impl DockerHandler {
    pub(super) async fn list_volumes(&self) -> Result<Vec<VolumeSummary>> {
        let resp = self.docker.list_volumes::<String>(None).await?;
        Ok(resp
            .volumes
            .unwrap_or_default()
            .into_iter()
            .map(summarize_volume)
            .collect())
    }

    pub(super) async fn remove_volume(&self, name: String, force: bool) -> Result<()> {
        use bollard::volume::RemoveVolumeOptions;
        let opts = RemoveVolumeOptions { force };
        self.docker.remove_volume(&name, Some(opts)).await?;
        Ok(())
    }
}

fn summarize_volume(v: RawVolume) -> VolumeSummary {
    VolumeSummary {
        name: v.name,
        driver: v.driver,
        mountpoint: v.mountpoint,
        created_at: v.created_at.map(|d| d.to_string()).unwrap_or_default(),
        scope: v.scope.map(|s| s.to_string()).unwrap_or_default(),
    }
}
