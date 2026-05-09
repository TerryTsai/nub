//! `docker volume inspect` — `GET /volumes/{name}`. Compat shape works
//! on both engines.

use anyhow::Result;

use super::wire::RawInspect;
use crate::client::Req;
use crate::ops::EngineHandler;
use crate::proto::VolumeDetail;

pub(crate) async fn run(h: &EngineHandler, name: &str) -> Result<Box<VolumeDetail>> {
    let path = format!("/volumes/{name}");
    let raw: RawInspect = h
        .engine
        .conn()
        .await?
        .send_unary(Req::get(path))
        .await?
        .json()?;
    Ok(Box::new(VolumeDetail {
        name: raw.name,
        driver: raw.driver,
        mountpoint: raw.mountpoint,
        created_at: raw.created_at,
        scope: raw.scope,
        labels: raw.labels,
        options: raw.options,
        ref_count: raw.usage_data.as_ref().map(|u| u.ref_count).unwrap_or(0),
        size: raw.usage_data.map(|u| u.size).unwrap_or(-1),
    }))
}
