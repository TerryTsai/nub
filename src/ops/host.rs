//! `GET /info` + `GET /version` → `proto::HostInfo`. Combined into one op
//! since callers always want both fields.

use anyhow::Result;
use serde::Deserialize;

use crate::client::{EngineKind, Req};
use crate::proto::HostInfo;
use crate::version::NUB_VERSION;

use super::EngineHandler;

pub(super) async fn run(h: &EngineHandler) -> Result<HostInfo> {
    let info: InfoResp = h
        .engine
        .conn()
        .await?
        .send_unary(Req::get("/info").build()?)
        .await?
        .json()?;
    let version: VersionResp = h
        .engine
        .conn()
        .await?
        .send_unary(Req::get("/version").build()?)
        .await?
        .json()?;
    Ok(HostInfo {
        nub: NUB_VERSION.to_string(),
        engine: engine_name(h.engine.kind(), &version),
        version: version.version,
        os: info.operating_system,
        arch: info.architecture,
        kernel: info.kernel_version,
        cpus: info.ncpu,
        mem_total: info.mem_total,
        containers_running: info.containers_running,
        containers_total: info.containers,
        images: info.images,
    })
}

fn engine_name(kind: EngineKind, version: &VersionResp) -> String {
    match kind {
        EngineKind::Podman => "podman".to_string(),
        EngineKind::Docker => version
            .platform
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "docker".to_string()),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct InfoResp {
    operating_system: String,
    architecture: String,
    kernel_version: String,
    #[serde(rename = "NCPU")]
    ncpu: u64,
    mem_total: u64,
    containers_running: u64,
    containers: u64,
    images: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct VersionResp {
    version: String,
    #[serde(default)]
    platform: Option<Platform>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Platform {
    name: String,
}
