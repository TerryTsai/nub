//! Tells volumes/networks list endpoints which entries are mounted or
//! attached. Two parallel probes joined locally — Docker doesn't return
//! per-entry usage on its list endpoints, and N+1 inspects would be far
//! worse.
//!
//! Volumes use `/system/df` (returns `RefCount` per volume on both
//! engines). Networks use `/containers/json` and parse whichever field
//! the engine populates: libpod gives `Networks: []string`, compat gives
//! `NetworkSettings.Networks: {name: ...}`.

use std::collections::{HashMap, HashSet};

use anyhow::Result;
use serde::Deserialize;

use crate::client::{EngineKind, Query, Req};
use crate::ops::EngineHandler;

pub(super) struct Usage {
    pub volumes: HashSet<String>,
    pub networks: HashSet<String>,
}

pub(super) async fn compute(h: &EngineHandler) -> Result<Usage> {
    let (volumes, networks) = tokio::try_join!(probe_volumes(h), probe_networks(h))?;
    Ok(Usage { volumes, networks })
}

async fn probe_volumes(h: &EngineHandler) -> Result<HashSet<String>> {
    let resp: SystemDf = h
        .engine
        .conn()
        .await?
        .send_unary(Req::get("/system/df").build()?)
        .await?
        .json()?;
    let mut out = HashSet::new();
    for v in resp.volumes {
        if v.usage_data.as_ref().map(|u| u.ref_count).unwrap_or(0) > 0 {
            out.insert(v.name);
        }
    }
    Ok(out)
}

async fn probe_networks(h: &EngineHandler) -> Result<HashSet<String>> {
    let mut q = Query::new();
    q.push_bool("all", true);
    let path = format!("{}{}", containers_path(h.engine.kind()), q.finish());
    let raw: Vec<RawContainer> = h
        .engine
        .conn()
        .await?
        .send_unary(Req::get(path).build()?)
        .await?
        .json()?;
    let mut out = HashSet::new();
    for c in raw {
        // libpod populates `networks` directly; compat leaves it empty
        // and fills `network_settings` instead.
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

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SystemDf {
    #[serde(default)]
    volumes: Vec<DfVolume>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DfVolume {
    #[serde(default)]
    name: String,
    #[serde(default)]
    usage_data: Option<DfUsage>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DfUsage {
    #[serde(default)]
    ref_count: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawContainer {
    #[serde(default)]
    networks: Vec<String>,
    #[serde(default)]
    network_settings: RawNetworkSettings,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawNetworkSettings {
    #[serde(default)]
    networks: HashMap<String, serde_json::Value>,
}
