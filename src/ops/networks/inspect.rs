//! `docker network inspect`. On Podman the compat path inherits the
//! same brittleness as `/containers/json` (one bad container 500s the
//! whole response), so we use libpod there. Output shapes differ; we
//! decode each separately and project into the same proto type.

use std::collections::HashMap;

use anyhow::Result;
use serde::Deserialize;

use crate::client::{short_id, EngineKind, Req};
use crate::ops::EngineHandler;
use crate::proto::{IpamConfig, NetworkContainer, NetworkDetail};

pub(super) async fn run(h: &EngineHandler, id: &str) -> Result<Box<NetworkDetail>> {
    match h.engine.kind() {
        EngineKind::Podman => libpod(h, id).await,
        EngineKind::Docker => compat(h, id).await,
    }
}

async fn compat(h: &EngineHandler, id: &str) -> Result<Box<NetworkDetail>> {
    let path = format!("/networks/{id}");
    let raw: RawCompat = h
        .engine
        .conn()
        .await?
        .send_unary(Req::get(path).build()?)
        .await?
        .json()?;
    let ipam = raw
        .ipam
        .unwrap_or_default()
        .config
        .into_iter()
        .map(|c| IpamConfig {
            subnet: c.subnet,
            gateway: c.gateway,
        })
        .collect();
    let containers = raw
        .containers
        .into_iter()
        .map(|(cid, c)| NetworkContainer {
            id: short_id(&cid),
            name: c.name,
            ipv4: c.ipv4_address,
            ipv6: c.ipv6_address,
        })
        .collect();
    Ok(Box::new(NetworkDetail {
        id: short_id(&raw.id),
        name: raw.name,
        driver: raw.driver,
        scope: raw.scope,
        created: raw.created,
        internal: raw.internal,
        ipam,
        containers,
        options: raw.options,
        labels: raw.labels,
    }))
}

async fn libpod(h: &EngineHandler, id: &str) -> Result<Box<NetworkDetail>> {
    let path = format!("/v4.0.0/libpod/networks/{id}/json");
    let raw: RawLibpod = h
        .engine
        .conn()
        .await?
        .send_unary(Req::get(path).build()?)
        .await?
        .json()?;
    let ipam = raw
        .subnets
        .into_iter()
        .map(|s| IpamConfig {
            subnet: s.subnet,
            gateway: s.gateway,
        })
        .collect();
    Ok(Box::new(NetworkDetail {
        id: short_id(&raw.id),
        name: raw.name,
        driver: raw.driver,
        // libpod doesn't report `scope`; keep empty so the proto stays stable.
        scope: String::new(),
        created: raw.created,
        internal: raw.internal,
        ipam,
        // libpod's `containers` field has a different shape (per-iface map);
        // detail page just doesn't show attached containers on Podman.
        containers: Vec::new(),
        options: raw.options,
        labels: raw.labels,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawCompat {
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
    #[serde(default)]
    ipam: Option<RawIpam>,
    #[serde(default, deserialize_with = "crate::ops::util::null_to_default")]
    containers: HashMap<String, RawNetContainer>,
    #[serde(default, deserialize_with = "crate::ops::util::null_to_default")]
    options: HashMap<String, String>,
    #[serde(default, deserialize_with = "crate::ops::util::null_to_default")]
    labels: HashMap<String, String>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawIpam {
    #[serde(default)]
    config: Vec<RawIpamConfig>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawIpamConfig {
    #[serde(default)]
    subnet: String,
    #[serde(default)]
    gateway: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawNetContainer {
    #[serde(default)]
    name: String,
    #[serde(default, rename = "IPv4Address")]
    ipv4_address: String,
    #[serde(default, rename = "IPv6Address")]
    ipv6_address: String,
}

#[derive(Deserialize)]
struct RawLibpod {
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
    #[serde(default, deserialize_with = "crate::ops::util::null_to_default")]
    subnets: Vec<RawLibpodSubnet>,
    #[serde(default, deserialize_with = "crate::ops::util::null_to_default")]
    options: HashMap<String, String>,
    #[serde(default, deserialize_with = "crate::ops::util::null_to_default")]
    labels: HashMap<String, String>,
}

#[derive(Deserialize)]
struct RawLibpodSubnet {
    #[serde(default)]
    subnet: String,
    #[serde(default)]
    gateway: String,
}
