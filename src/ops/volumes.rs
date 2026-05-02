//! `docker volume ls/inspect/rm` — compat paths work on both engines.

use std::collections::HashMap;

use anyhow::Result;
use serde::Deserialize;

use crate::client::{Query, Req};
use crate::proto::{VolumeDetail, VolumeSummary};

use super::usage;
use super::EngineHandler;

pub(super) async fn list(h: &EngineHandler) -> Result<Vec<VolumeSummary>> {
    let (resp, used) = tokio::try_join!(fetch(h), usage::compute(h))?;
    Ok(resp
        .volumes
        .into_iter()
        .map(|v| {
            let in_use = used.volumes.contains(&v.name);
            v.into_summary(in_use)
        })
        .collect())
}

async fn fetch(h: &EngineHandler) -> Result<ListResp> {
    Ok(h.engine
        .conn()
        .await?
        .send_unary(Req::get("/volumes").build()?)
        .await?
        .json()?)
}

pub(super) async fn inspect(h: &EngineHandler, name: &str) -> Result<Box<VolumeDetail>> {
    let path = format!("/volumes/{name}");
    let raw: RawInspect = h
        .engine
        .conn()
        .await?
        .send_unary(Req::get(path).build()?)
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

pub(super) async fn remove(h: &EngineHandler, name: String, force: bool) -> Result<()> {
    let mut q = Query::new();
    q.push_bool("force", force);
    let path = format!("/volumes/{name}{}", q.finish());
    h.engine
        .conn()
        .await?
        .send_unary(Req::delete(path).build()?)
        .await?
        .ok()?;
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

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawInspect {
    name: String,
    #[serde(default)]
    driver: String,
    #[serde(default)]
    mountpoint: String,
    #[serde(default)]
    created_at: String,
    #[serde(default)]
    scope: String,
    #[serde(default)]
    labels: HashMap<String, String>,
    #[serde(default)]
    options: HashMap<String, String>,
    #[serde(default)]
    usage_data: Option<RawUsage>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawUsage {
    #[serde(default)]
    size: i64,
    #[serde(default)]
    ref_count: i64,
}

impl RawVolume {
    fn into_summary(self, in_use: bool) -> VolumeSummary {
        VolumeSummary {
            name: self.name,
            driver: self.driver,
            mountpoint: self.mountpoint,
            created_at: self.created_at,
            scope: self.scope,
            in_use,
        }
    }
}
