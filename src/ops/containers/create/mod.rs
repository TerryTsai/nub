//! `docker container create` — `POST /containers/create`. Optionally starts
//! the container after create (the `start: true` field on the request).
//! Compat path works on both engines.
//!
//! Validation enforces "no implicit resource creation": the image must
//! already be local (no engine auto-pull), and every named volume mount
//! must reference a volume that already exists. Caller is responsible for
//! pre-pulling and pre-creating; the API layer never spawns side resources.

mod build;
mod wire;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

use crate::client::{Query, Req};
use crate::ops::EngineHandler;
use crate::proto::{ContainerCreated, CreateContainerReq};

pub(crate) async fn run(h: &EngineHandler, req: CreateContainerReq) -> Result<ContainerCreated> {
    validate_static(&req, &h.policy.allowed_binds)?;
    require_image_local(h, &req.image).await?;
    require_named_volumes_exist(h, &req.volumes).await?;

    let body = build::body(&req);
    let path = format!("/containers/create{}", create_query(req.name.as_deref()));
    let resp: wire::CreateResp = h
        .engine
        .conn()
        .await?
        .send_unary(Req::post(path).json(&body)?.build()?)
        .await?
        .json()?;

    Ok(ContainerCreated {
        id: resp.id,
        warnings: resp.warnings.unwrap_or_default(),
    })
}

fn create_query(name: Option<&str>) -> String {
    let mut q = Query::new();
    if let Some(n) = name {
        q.push("name", n);
    }
    q.finish()
}

fn validate_static(req: &CreateContainerReq, allowed_binds: &[PathBuf]) -> Result<()> {
    if let Some(net) = &req.network {
        if net == "host" || net.starts_with("container:") {
            return Err(anyhow!("network mode '{net}' not allowed"));
        }
    }
    for v in &req.volumes {
        if v.source.is_empty() {
            return Err(anyhow!(
                "anonymous volumes not supported — name the volume and create it first",
            ));
        }
        if !is_host_path(&v.source) {
            continue;
        }
        let src = Path::new(&v.source);
        // nub-managed tmpfs paths are implicitly allowed — they live in
        // `/run` (no persistence) and are written by nub itself during
        // stack deploy, not user-supplied.
        if crate::ops::secrets::runtime::is_managed_path(src) || crate::ops::configs::runtime::is_managed_path(src) {
            continue;
        }
        if !allowed_binds.iter().any(|p| src.starts_with(p)) {
            return Err(anyhow!("bind source '{}' not in allowed_binds", v.source));
        }
    }
    Ok(())
}

/// Engine probe: `GET /images/{ref}/json`. 200 → local; 404 → not local.
/// Caller must `images:pull` first.
async fn require_image_local(h: &EngineHandler, reference: &str) -> Result<()> {
    let path = format!("/images/{reference}/json");
    let res = h.engine.conn().await?.send_unary(Req::get(path).build()?).await?;
    if res.status.as_u16() == 404 {
        return Err(anyhow!("image '{reference}' not local — pull it first (images:pull)",));
    }
    res.ok()?;
    Ok(())
}

/// Every named (non-host-path, non-managed-tmpfs) mount source must
/// resolve to a volume that already exists. The static pass already
/// rejected empty sources and validated host paths.
async fn require_named_volumes_exist(h: &EngineHandler, mounts: &[crate::proto::VolumeMount]) -> Result<()> {
    let needed: Vec<&str> = mounts
        .iter()
        .filter(|v| !is_host_path(&v.source))
        .filter(|v| {
            let src = Path::new(&v.source);
            !crate::ops::secrets::runtime::is_managed_path(src) && !crate::ops::configs::runtime::is_managed_path(src)
        })
        .map(|v| v.source.as_str())
        .collect();
    if needed.is_empty() {
        return Ok(());
    }
    let volumes = crate::ops::volumes::list::run(h).await?;
    let known: HashSet<&str> = volumes.iter().map(|v| v.name.as_str()).collect();
    for name in needed {
        if !known.contains(name) {
            return Err(anyhow!("volume '{name}' not found — create it first (volumes:create)",));
        }
    }
    Ok(())
}

pub(super) fn is_host_path(s: &str) -> bool {
    s.starts_with('/') || s.starts_with("./") || s.starts_with("../")
}
