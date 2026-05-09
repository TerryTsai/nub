//! `docker container kill` — `POST /containers/{id}/kill?signal=<NAME>`.
//! Default signal is engine-determined (SIGKILL on both).

use anyhow::Result;

use crate::client::{Query, Req};
use crate::ops::EngineHandler;

pub(crate) async fn run(h: &EngineHandler, id: String, signal: Option<String>) -> Result<()> {
    let mut q = Query::new();
    if let Some(s) = &signal {
        q.push("signal", s);
    }
    let path = format!("/containers/{id}/kill{}", q.finish());
    h.engine
        .conn()
        .await?
        .send_unary(Req::post(path))
        .await?
        .ok()?;
    Ok(())
}
