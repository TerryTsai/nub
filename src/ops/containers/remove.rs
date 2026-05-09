//! `docker container rm` — `DELETE /containers/{id}?force=<bool>`.
//! `force=true` SIGKILLs a running container before removal.

use anyhow::Result;

use crate::client::Req;
use crate::ops::EngineHandler;

pub(crate) async fn run(h: &EngineHandler, id: String, force: bool) -> Result<()> {
    let path = format!("/containers/{id}?force={force}");
    h.engine.conn().await?.send_unary(Req::delete(path)).await?.ok()?;
    Ok(())
}
