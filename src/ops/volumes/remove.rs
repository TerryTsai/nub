//! `docker volume rm` — `DELETE /volumes/{name}`. The engine refuses if
//! the volume is in use; nub doesn't force.

use anyhow::Result;

use crate::client::Req;
use crate::ops::EngineHandler;

pub(crate) async fn run(h: &EngineHandler, name: String) -> Result<()> {
    let path = format!("/volumes/{name}");
    h.engine.unit(Req::delete(path)).await?;
    Ok(())
}
