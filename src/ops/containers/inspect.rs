//! `docker container inspect` — `GET /containers/{id}/json`. Compat
//! shape; both engines return the same body.

use anyhow::Result;

use super::wire::inspect::RawInspect;
use crate::client::Req;
use crate::ops::EngineHandler;
use crate::proto::ContainerDetail;

pub(crate) async fn run(h: &EngineHandler, id: String) -> Result<Box<ContainerDetail>> {
    let path = format!("/containers/{id}/json");
    let raw: RawInspect = h
        .engine
        .conn()
        .await?
        .send_unary(Req::get(path))
        .await?
        .json()?;
    Ok(Box::new(raw.into_detail()))
}
