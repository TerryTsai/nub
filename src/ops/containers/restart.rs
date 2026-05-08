//! `docker container restart` — `POST /containers/{id}/restart?t=<seconds>`.
//! `t` is the stop-grace before SIGKILL; the engine defaults if omitted.

use anyhow::Result;

use crate::client::{Query, Req};
use crate::ops::EngineHandler;

pub(crate) async fn run(h: &EngineHandler, id: String, timeout: Option<i64>) -> Result<()> {
    let mut q = Query::new();
    if let Some(t) = timeout {
        q.push("t", &t.to_string());
    }
    let path = format!("/containers/{id}/restart{}", q.finish());
    h.engine
        .conn()
        .await?
        .send_unary(Req::post(path).build()?)
        .await?
        .ok()?;
    Ok(())
}
