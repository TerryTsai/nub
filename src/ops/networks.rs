//! `docker network ls/rm`. Podman's compat `/networks` shares the brittleness
//! of `/containers/json` — a single bad container 500s the whole endpoint.
//! Use libpod on Podman; compat on Docker.

use anyhow::Result;
use serde::Deserialize;

use crate::client::{short_id, EngineKind, Req};
use crate::proto::NetworkSummary;

use super::EngineHandler;

pub(super) async fn list(h: &EngineHandler) -> Result<Vec<NetworkSummary>> {
    match h.engine.kind() {
        EngineKind::Podman => list_libpod(h).await,
        EngineKind::Docker => list_compat(h).await,
    }
}

pub(super) async fn remove(h: &EngineHandler, id: String) -> Result<()> {
    let path = format!("/networks/{id}");
    h.engine.conn().await?.send_unary(Req::delete(path).build()?).await?.ok()?;
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
        }
    }
}
