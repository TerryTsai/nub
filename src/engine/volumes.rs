//! Volumes: list, remove.

use serde::Deserialize;

use super::{Engine, Query, Req, Result};

#[derive(Debug, Clone)]
pub struct VolumeSummary {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
    pub created_at: String,
    pub scope: String,
}

impl Engine {
    pub async fn list_volumes(&self) -> Result<Vec<VolumeSummary>> {
        let mut conn = self.conn().await?;
        let resp: VolumeListResp = conn.send_unary(Req::get("/volumes").build()?).await?.json()?;
        Ok(resp.volumes.into_iter().map(RawVolume::into_summary).collect())
    }

    pub async fn remove_volume(&self, name: &str, force: bool) -> Result<()> {
        let mut q = Query::new();
        q.push_bool("force", force);
        let path = format!("/volumes/{name}{}", q.finish());
        let mut conn = self.conn().await?;
        conn.send_unary(Req::delete(path).build()?).await?.ok()
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct VolumeListResp {
    #[serde(default)]
    volumes: Vec<RawVolume>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawVolume {
    name: String,
    #[serde(default)]
    driver: String,
    #[serde(default)]
    mountpoint: String,
    #[serde(default)]
    created_at: String,
    #[serde(default)]
    scope: String,
}

impl RawVolume {
    fn into_summary(self) -> VolumeSummary {
        VolumeSummary {
            name: self.name,
            driver: self.driver,
            mountpoint: self.mountpoint,
            created_at: self.created_at,
            scope: self.scope,
        }
    }
}
