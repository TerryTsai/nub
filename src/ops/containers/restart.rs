//! `docker container restart` — `POST /containers/{id}/restart?t=<seconds>`.
//! `t` is the stop-grace before SIGKILL; the engine defaults if omitted.

use anyhow::Result;

use crate::client::Req;
use crate::ops::EngineHandler;

pub(crate) async fn run(h: &EngineHandler, id: String, timeout: Option<i64>) -> Result<()> {
    let path = match timeout {
        Some(t) => format!("/containers/{id}/restart?t={t}"),
        None => format!("/containers/{id}/restart"),
    };
    h.engine.unit(Req::post(path)).await?;
    Ok(())
}
