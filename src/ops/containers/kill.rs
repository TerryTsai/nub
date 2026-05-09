//! `docker container kill` — `POST /containers/{id}/kill?signal=<NAME>`.
//! Default signal is engine-determined (SIGKILL on both).

use anyhow::Result;

use crate::client::Req;
use crate::ops::EngineHandler;

pub(crate) async fn run(h: &EngineHandler, id: String, signal: Option<String>) -> Result<()> {
    let path = match signal {
        Some(s) => format!("/containers/{id}/kill?signal={s}"),
        None => format!("/containers/{id}/kill"),
    };
    h.engine.conn().await?.send_unary(Req::post(path)).await?.ok()?;
    Ok(())
}
