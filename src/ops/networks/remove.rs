//! `docker network rm` — `DELETE /networks/{id}`. The idempotent
//! variant treats engine 404 as success; used by stack teardown where
//! the network may already have been pruned.

use anyhow::Result;

use crate::client::Req;
use crate::ops::EngineHandler;

pub(crate) async fn run(h: &EngineHandler, id: String) -> Result<()> {
    let path = format!("/networks/{id}");
    h.engine.unit(Req::delete(path)).await?;
    Ok(())
}

/// Same as `run` but treats engine 404 as success — for stack
/// teardown, where the network may already have been pruned.
pub(crate) async fn run_idempotent(h: &EngineHandler, id: &str) -> Result<()> {
    let path = format!("/networks/{id}");
    let resp = h.engine.conn().await?.send_unary(Req::delete(path)).await?;
    if resp.status.as_u16() == 404 {
        return Ok(());
    }
    resp.ok()?;
    Ok(())
}
