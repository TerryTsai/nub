//! Low-level engine calls used by the stack runtime. We don't go through
//! the public op layer because we need to attach labels (which the
//! existing `create_network` / volume create paths don't expose) and
//! treat 404s on remove as success.

use std::collections::HashMap;

use anyhow::{anyhow, Result};
use futures::stream::StreamExt;
use serde::Serialize;

use crate::client::Req;
use crate::ops::{images, EngineHandler};
use crate::proto::StreamChunk;

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct NetworkCreateBody {
    name: String,
    driver: &'static str,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    labels: HashMap<String, String>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct VolumeCreateBody {
    name: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    labels: HashMap<String, String>,
}

pub(super) async fn create_network(h: &EngineHandler, name: &str, labels: HashMap<String, String>) -> Result<()> {
    let body = NetworkCreateBody {
        name: name.into(),
        driver: "bridge",
        labels,
    };
    h.engine
        .conn()
        .await?
        .send_unary(Req::post("/networks/create").json(&body)?.build()?)
        .await?
        .ok()?;
    Ok(())
}

pub(super) async fn create_volume(h: &EngineHandler, name: &str, labels: HashMap<String, String>) -> Result<()> {
    let body = VolumeCreateBody {
        name: name.into(),
        labels,
    };
    h.engine
        .conn()
        .await?
        .send_unary(Req::post("/volumes/create").json(&body)?.build()?)
        .await?
        .ok()?;
    Ok(())
}

/// Drain `images::pull`'s streaming output into a unary result. Used by
/// stack deploy to honor the "no implicit pull" rule on `CreateContainer`
/// — every `FROM`/`image:` ref must be local before the engine sees it.
pub(super) async fn pull_image(h: &EngineHandler, reference: &str) -> Result<()> {
    let mut stream = images::pull::run(h, reference.to_string());
    while let Some(chunk) = stream.next().await {
        if let StreamChunk::End { ok: false, err } = chunk {
            return Err(anyhow!("pull {reference}: {}", err.unwrap_or_default()));
        }
    }
    Ok(())
}

pub(super) async fn remove_network(h: &EngineHandler, id: &str) -> Result<()> {
    let path = format!("/networks/{id}");
    let resp = h.engine.conn().await?.send_unary(Req::delete(path).build()?).await?;
    if resp.status.as_u16() == 404 {
        return Ok(());
    }
    resp.ok()?;
    Ok(())
}
