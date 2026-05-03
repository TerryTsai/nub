//! `docker container create` — `POST /containers/create`. Optionally starts
//! the container after create (the `start: true` field on the request).
//! Compat path works on both engines.

mod build;
mod wire;

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

use crate::client::{Query, Req};
use crate::ops::EngineHandler;
use crate::proto::{ContainerCreated, CreateContainerReq};

pub(crate) async fn run(h: &EngineHandler, req: CreateContainerReq) -> Result<ContainerCreated> {
    validate(&req, &h.policy.allowed_binds)?;
    let body = build::body(&req);
    let path = format!("/containers/create{}", create_query(req.name.as_deref()));
    let resp: wire::CreateResp = h
        .engine
        .conn()
        .await?
        .send_unary(Req::post(path).json(&body)?.build()?)
        .await?
        .json()?;

    let mut started = false;
    if req.start {
        h.engine
            .conn()
            .await?
            .send_unary(Req::post(format!("/containers/{}/start", resp.id)).build()?)
            .await?
            .ok()?;
        started = true;
    }
    Ok(ContainerCreated {
        id: resp.id,
        started,
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

fn validate(req: &CreateContainerReq, allowed_binds: &[PathBuf]) -> Result<()> {
    if let Some(net) = &req.network {
        if net == "host" || net.starts_with("container:") {
            return Err(anyhow!("network mode '{net}' not allowed"));
        }
    }
    for v in &req.volumes {
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

pub(super) fn is_host_path(s: &str) -> bool {
    s.starts_with('/') || s.starts_with("./") || s.starts_with("../")
}
