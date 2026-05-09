//! `docker volume ls` — engine-aware listing. Podman libpod returns
//! `MountCount` cheaply (sub-100ms); Docker compat doesn't, so we
//! derive `in_use` from a parallel `/containers/json` probe — far
//! cheaper than `/system/df`, which sizes every entry on disk.

use std::collections::HashSet;

use anyhow::Result;

use super::wire::{CompatList, ContainerWithMounts, RawLibpodVolume};
use crate::client::{EngineKind, Query, Req};
use crate::ops::EngineHandler;
use crate::proto::VolumeSummary;

pub(crate) async fn run(h: &EngineHandler) -> Result<Vec<VolumeSummary>> {
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
        .send_unary(Req::get("/v4.0.0/libpod/volumes/json"))
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
        .send_unary(Req::get("/volumes"))
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
        .send_unary(Req::get(path))
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
