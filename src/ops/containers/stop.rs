//! `docker container stop` — `POST /containers/{id}/stop?t=<seconds>`.
//! `t` is the grace period before SIGKILL; the engine defaults if
//! omitted.

use anyhow::Result;

use crate::client::Req;
use crate::ops::EngineHandler;

pub(crate) async fn run(h: &EngineHandler, id: String, timeout: Option<i64>) -> Result<()> {
    let path = match timeout {
        Some(t) => format!("/containers/{id}/stop?t={t}"),
        None => format!("/containers/{id}/stop"),
    };
    h.engine.unit(Req::post(path)).await?;
    Ok(())
}
