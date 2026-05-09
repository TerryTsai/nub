//! `docker container ls` — `GET /containers/json` (compat) or
//! `GET /v4.0.0/libpod/containers/json` (libpod). Compat is brittle on
//! Podman: a single container in `Removing` state 500s the whole endpoint.
//! Libpod tolerates it, so we use libpod when we know we're on Podman.

use anyhow::Result;

use super::wire::list::RawListItem;
use crate::client::{EngineKind, Req};
use crate::ops::EngineHandler;
use crate::proto::ContainerSummary;

pub(crate) async fn run(h: &EngineHandler, all: bool) -> Result<Vec<ContainerSummary>> {
    let path = format!("{}?all={all}", list_path(h.engine.kind()));
    let raw: Vec<RawListItem> = h.engine.unary(Req::get(path)).await?;
    Ok(raw.into_iter().map(RawListItem::into_summary).collect())
}

fn list_path(kind: EngineKind) -> &'static str {
    match kind {
        // Libpod requires a version prefix on most paths (compat 301s a
        // missing version). v4.0.0 is broadly accepted across podman 3+.
        EngineKind::Podman => "/v4.0.0/libpod/containers/json",
        EngineKind::Docker => "/containers/json",
    }
}
