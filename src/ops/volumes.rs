//! `docker volume ls/inspect/rm`. Listing is engine-aware so we can pull
//! `MountCount` cheaply on Podman libpod (sub-100ms). Docker compat doesn't
//! expose per-volume usage on the list endpoint, so we derive `in_use` from
//! a parallel `/containers/json` probe instead — far cheaper than
//! `/system/df`, which sizes every entry on disk and can take 30s.

use std::collections::{HashMap, HashSet};

use anyhow::Result;
use serde::Deserialize;

use crate::client::{EngineKind, Query, Req};
use crate::proto::{VolumeDetail, VolumeSummary};

use super::EngineHandler;

pub(super) async fn list(h: &EngineHandler) -> Result<Vec<VolumeSummary>> {
    match h.engine.kind() {
        EngineKind::Podman => list_libpod(h).await,
        EngineKind::Docker => list_compat(h).await,
    }
}

async fn list_libpod(h: &EngineHandler) -> Result<Vec<VolumeSummary>> {
    let raw: Vec<RawLibpodVolume> = h
        .engine
        .conn()
        .await?
        .send_unary(Req::get("/v4.0.0/libpod/volumes/json").build()?)
        .await?
        .json()?;
    Ok(raw.into_iter().map(RawLibpodVolume::into_summary).collect())
}

async fn list_compat(h: &EngineHandler) -> Result<Vec<VolumeSummary>> {
    let (resp, used) = tokio::try_join!(fetch_compat(h), probe_attached_volumes(h))?;
    Ok(resp
        .volumes
        .into_iter()
        .map(|v| {
            let in_use = used.contains(&v.name);
            v.into_summary(in_use)
        })
        .collect())
}

async fn fetch_compat(h: &EngineHandler) -> Result<CompatList> {
    Ok(h.engine
        .conn()
        .await?
        .send_unary(Req::get("/volumes").build()?)
        .await?
        .json()?)
}

/// Walk container mounts (compat shape) and collect the named volumes.
/// Used only on Docker — Podman libpod has MountCount in the volume list.
async fn probe_attached_volumes(h: &EngineHandler) -> Result<HashSet<String>> {
    let mut q = Query::new();
    q.push_bool("all", true);
    let path = format!("/containers/json{}", q.finish());
    let raw: Vec<ContainerWithMounts> = h
        .engine
        .conn()
        .await?
        .send_unary(Req::get(path).build()?)
        .await?
        .json()?;
    let mut out = HashSet::new();
    for c in raw {
        for m in c.mounts {
            if !m.name.is_empty() {
                out.insert(m.name);
            }
        }
    }
    Ok(out)
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
struct CompatList {
    #[serde(default)]
    volumes: Vec<RawCompatVolume>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawCompatVolume {
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

impl RawCompatVolume {
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

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawLibpodVolume {
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
    mount_count: i64,
}

impl RawLibpodVolume {
    fn into_summary(self) -> VolumeSummary {
        VolumeSummary {
            name: self.name,
            driver: self.driver,
            mountpoint: self.mountpoint,
            created_at: self.created_at,
            scope: self.scope,
            in_use: self.mount_count > 0,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ContainerWithMounts {
    #[serde(default)]
    mounts: Vec<RawMount>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawMount {
    /// Volume name. Empty for non-volume mounts (bind, tmpfs).
    #[serde(default)]
    name: String,
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
