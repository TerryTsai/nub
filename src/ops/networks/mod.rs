//! `docker network ls/rm`. Podman's compat `/networks` inherits the
//! brittleness of `/containers/json` — one bad container 500s the whole
//! endpoint. Use libpod on Podman; compat on Docker. Inspect is split
//! into a sibling module since it has its own engine-specific decoders.

use std::collections::HashMap;
use std::collections::HashSet;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::client::{short_id, EngineKind, Query, Req};
use crate::proto::{NetworkDetail, NetworkSummary};

use super::EngineHandler;

mod inspect;

pub(super) async fn list(h: &EngineHandler) -> Result<Vec<NetworkSummary>> {
    let (mut nets, used) = tokio::try_join!(fetch_list(h), probe_attached(h))?;
    for n in &mut nets {
        n.in_use = used.contains(&n.name);
    }
    Ok(nets)
}

/// Walk container network attachments (libpod string array OR compat
/// `NetworkSettings.Networks` map) and collect names. Sub-second on both
/// engines — cheap enough to pair with every list call.
async fn probe_attached(h: &EngineHandler) -> Result<HashSet<String>> {
    let mut q = Query::new();
    q.push_bool("all", true);
    let path = format!("{}{}", containers_path(h.engine.kind()), q.finish());
    let raw: Vec<ContainerNets> = h
        .engine
        .conn()
        .await?
        .send_unary(Req::get(path).build()?)
        .await?
        .json()?;
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

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ContainerNets {
    #[serde(default)]
    networks: Vec<String>,
    #[serde(default)]
    network_settings: RawNetSettings,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawNetSettings {
    #[serde(default, deserialize_with = "crate::ops::serde_util::null_to_default")]
    networks: HashMap<String, serde_json::Value>,
}

async fn fetch_list(h: &EngineHandler) -> Result<Vec<NetworkSummary>> {
    match h.engine.kind() {
        EngineKind::Podman => list_libpod(h).await,
        EngineKind::Docker => list_compat(h).await,
    }
}

pub(super) async fn inspect(h: &EngineHandler, id: &str) -> Result<Box<NetworkDetail>> {
    inspect::run(h, id).await
}

pub(super) async fn create(h: &EngineHandler, name: String, internal: bool) -> Result<()> {
    let body = CreateBody {
        name,
        driver: "bridge".into(),
        internal,
    };
    h.engine
        .conn()
        .await?
        .send_unary(Req::post("/networks/create").json(&body)?.build()?)
        .await?
        .ok()?;
    Ok(())
}

pub(super) async fn remove(h: &EngineHandler, id: String) -> Result<()> {
    let path = format!("/networks/{id}");
    h.engine
        .conn()
        .await?
        .send_unary(Req::delete(path).build()?)
        .await?
        .ok()?;
    Ok(())
}

async fn list_compat(h: &EngineHandler) -> Result<Vec<NetworkSummary>> {
    let raw: Vec<CompatNet> = h
        .engine
        .conn()
        .await?
        .send_unary(Req::get("/networks").build()?)
        .await?
        .json()?;
    Ok(raw.into_iter().map(CompatNet::into_summary).collect())
}

async fn list_libpod(h: &EngineHandler) -> Result<Vec<NetworkSummary>> {
    let raw: Vec<LibpodNet> = h
        .engine
        .conn()
        .await?
        .send_unary(Req::get("/v4.0.0/libpod/networks/json").build()?)
        .await?
        .json()?;
    Ok(raw.into_iter().map(LibpodNet::into_summary).collect())
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct CompatNet {
    #[serde(default, rename = "Id")]
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    driver: String,
    #[serde(default)]
    scope: String,
    #[serde(default)]
    created: String,
    #[serde(default)]
    internal: bool,
}

impl CompatNet {
    fn into_summary(self) -> NetworkSummary {
        NetworkSummary {
            id: short_id(&self.id),
            name: self.name,
            driver: self.driver,
            scope: self.scope,
            created: self.created,
            internal: self.internal,
            // Filled in by `list()` after joining with the usage probe.
            in_use: false,
        }
    }
}

#[derive(Deserialize)]
struct LibpodNet {
    #[serde(default)]
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    driver: String,
    #[serde(default)]
    created: String,
    #[serde(default)]
    internal: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct CreateBody {
    name: String,
    driver: String,
    internal: bool,
}

impl LibpodNet {
    fn into_summary(self) -> NetworkSummary {
        NetworkSummary {
            id: short_id(&self.id),
            name: self.name,
            driver: self.driver,
            // libpod doesn't report `scope`; leave empty so the proto shape
            // is stable across engines.
            scope: String::new(),
            created: self.created,
            internal: self.internal,
            in_use: false,
        }
    }
}
