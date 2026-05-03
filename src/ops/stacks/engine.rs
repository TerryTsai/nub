//! Low-level engine calls used by the stack runtime. We don't go through
//! the public op layer because we need to attach labels (which the
//! existing `create_network` / volume create paths don't expose) and
//! treat 404s on remove as success.

use std::collections::HashMap;

use anyhow::Result;
use serde::Serialize;

use crate::client::Req;
use crate::ops::EngineHandler;

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
    let body = NetworkCreateBody { name: name.into(), driver: "bridge", labels };
    h.engine
        .conn()
        .await?
        .send_unary(Req::post("/networks/create").json(&body)?.build()?)
        .await?
        .ok()?;
    Ok(())
}

pub(super) async fn create_volume(h: &EngineHandler, name: &str, labels: HashMap<String, String>) -> Result<()> {
    let body = VolumeCreateBody { name: name.into(), labels };
    h.engine
        .conn()
        .await?
        .send_unary(Req::post("/volumes/create").json(&body)?.build()?)
        .await?
        .ok()?;
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
