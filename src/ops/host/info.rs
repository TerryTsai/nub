//! `GET /info` + `GET /version` → `proto::HostInfo`. Combined into one
//! op since callers always want both.

use anyhow::Result;

use super::wire::{InfoResp, VersionResp};
use crate::client::{EngineKind, Req};
use crate::ops::EngineHandler;
use crate::proto::HostInfo;
use crate::version::NUB_VERSION;

pub(crate) async fn run(h: &EngineHandler) -> Result<HostInfo> {
    let info: InfoResp = h
        .engine
        .conn()
        .await?
        .send_unary(Req::get("/info"))
        .await?
        .json()?;
    let version: VersionResp = h
        .engine
        .conn()
        .await?
        .send_unary(Req::get("/version"))
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
