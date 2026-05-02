//! `docker container ls` — `GET /containers/json` (compat) or
//! `GET /v4.0.0/libpod/containers/json` (libpod). Compat is brittle on
//! Podman: a single container in `Removing` state 500s the whole endpoint.
//! Libpod tolerates it, so we use libpod when we know we're on Podman.

use anyhow::Result;
use serde::Deserialize;

use crate::client::{short_id, EngineKind, Query, Req};
use crate::ops::EngineHandler;
use crate::proto::ContainerSummary;

pub(crate) async fn run(h: &EngineHandler, all: bool) -> Result<Vec<ContainerSummary>> {
    let path = format!("{}{}", list_path(h.engine.kind()), query(all));
    let raw: Vec<RawListItem> = h
        .engine
        .conn()
        .await?
        .send_unary(Req::get(path).build()?)
        .await?
        .json()?;
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

fn query(all: bool) -> String {
    let mut q = Query::new();
    q.push_bool("all", all);
    q.finish()
}

/// Field set is the union of compat and libpod shapes; both use PascalCase
/// so a single struct decodes both. `Created` differs (Unix int vs ISO
/// string) so we accept either via `serde_json::Value`. ExitCode comes from
/// libpod; Docker compat omits it on the list response (defaults to 0,
/// which falls into the "Stopped" bucket — accept the loss of fidelity
/// rather than parsing the free-form Status string).
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawListItem {
    #[serde(rename = "Id")]
    id: String,
    #[serde(default)]
    names: Vec<String>,
    #[serde(default)]
    image: String,
    #[serde(default)]
    state: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    created: serde_json::Value,
    #[serde(default)]
    exit_code: i32,
}

impl RawListItem {
    fn into_summary(self) -> ContainerSummary {
        ContainerSummary {
            id: short_id(&self.id),
            name: first_name(&self.names),
            image: self.image,
            state: self.state,
            status: self.status,
            created: created_to_string(self.created),
            exit_code: self.exit_code,
        }
    }
}

fn first_name(names: &[String]) -> String {
    names
        .first()
        .map(|n| n.trim_start_matches('/').to_string())
        .unwrap_or_default()
}

fn created_to_string(v: serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s,
        serde_json::Value::Number(n) => n.to_string(),
        _ => String::new(),
    }
}
