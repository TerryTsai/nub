//! `docker container start` — `POST /containers/{id}/start`. Compat
//! path works on both engines.

use anyhow::Result;

use crate::client::Req;
use crate::ops::EngineHandler;

pub(crate) async fn run(h: &EngineHandler, id: String) -> Result<()> {
    h.engine
        .conn()
        .await?
        .send_unary(Req::post(format!("/containers/{id}/start")).build()?)
        .await?
        .ok()?;
    Ok(())
}
