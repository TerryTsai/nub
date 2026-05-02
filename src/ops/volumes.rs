//! `docker volume ls/rm` — compat paths work on both engines.

use anyhow::Result;
use serde::Deserialize;

use crate::client::{Query, Req};
use crate::proto::VolumeSummary;

use super::EngineHandler;

pub(super) async fn list(h: &EngineHandler) -> Result<Vec<VolumeSummary>> {
    let resp: ListResp = h
        .engine
        .conn()
        .await?
        .send_unary(Req::get("/volumes").build()?)
        .await?
        .json()?;
    Ok(resp.volumes.into_iter().map(RawVolume::into_summary).collect())
}

pub(super) async fn remove(h: &EngineHandler, name: String, force: bool) -> Result<()> {
    let mut q = Query::new();
    q.push_bool("force", force);
    let path = format!("/volumes/{name}{}", q.finish());
    h.engine.conn().await?.send_unary(Req::delete(path).build()?).await?.ok()?;
    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ListResp {
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
