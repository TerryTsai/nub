//! `docker network ls` — engine-aware listing. Podman's compat
//! `/networks` inherits the brittleness of `/containers/json` (one bad
//! container 500s the whole endpoint), so we use libpod there. The
//! `in_use` flag joins per-container attachments via a parallel probe.

use std::collections::HashSet;

use anyhow::Result;

use super::wire::{CompatList, ContainerNets, LibpodList};
use crate::client::{EngineKind, Query, Req};
use crate::ops::EngineHandler;
use crate::proto::NetworkSummary;

pub(crate) async fn run(h: &EngineHandler) -> Result<Vec<NetworkSummary>> {
    let (mut nets, used) = tokio::try_join!(fetch_list(h), probe_attached(h))?;
    for n in &mut nets {
        n.in_use = used.contains(&n.name);
    }
    Ok(nets)
}

async fn fetch_list(h: &EngineHandler) -> Result<Vec<NetworkSummary>> {
    match h.engine.kind() {
        EngineKind::Podman => list_libpod(h).await,
        EngineKind::Docker => list_compat(h).await,
    }
}

async fn list_compat(h: &EngineHandler) -> Result<Vec<NetworkSummary>> {
    let raw: Vec<CompatList> = h.engine.conn().await?.send_unary(Req::get("/networks")).await?.json()?;
    Ok(raw.into_iter().map(CompatList::into_summary).collect())
}

async fn list_libpod(h: &EngineHandler) -> Result<Vec<NetworkSummary>> {
    let raw: Vec<LibpodList> = h
        .engine
        .conn()
        .await?
        .send_unary(Req::get("/v4.0.0/libpod/networks/json"))
        .await?
        .json()?;
    Ok(raw.into_iter().map(LibpodList::into_summary).collect())
}

/// Walk container network attachments (libpod string array OR compat
/// `NetworkSettings.Networks` map) and collect names. Sub-second on both
/// engines — cheap enough to pair with every list call.
async fn probe_attached(h: &EngineHandler) -> Result<HashSet<String>> {
    let mut q = Query::new();
    q.push_bool("all", true);
    let path = format!("{}{}", containers_path(h.engine.kind()), q.finish());
    let raw: Vec<ContainerNets> = h.engine.conn().await?.send_unary(Req::get(path)).await?.json()?;
    let mut out = HashSet::new();
    for c in raw {
        for n in c.networks {
            out.insert(n);
        }
        for n in c.network_settings.networks.into_keys() {
            out.insert(n);
        }
    }
    Ok(out)
}

fn containers_path(kind: EngineKind) -> &'static str {
    match kind {
        EngineKind::Podman => "/v4.0.0/libpod/containers/json",
        EngineKind::Docker => "/containers/json",
    }
}
