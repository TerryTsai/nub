//! `docker image rm` — `DELETE /images/{id}`.

use anyhow::Result;

use crate::client::{Query, Req};
use crate::ops::EngineHandler;

pub(crate) async fn run(h: &EngineHandler, id: String, force: bool) -> Result<()> {
    let mut q = Query::new();
    q.push_bool("force", force);
    let path = format!("/images/{id}{}", q.finish());
    h.engine
        .conn()
        .await?
        .send_unary(Req::delete(path).build()?)
        .await?
        .ok()?;
    Ok(())
}
