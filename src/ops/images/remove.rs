//! `docker image rm` — `DELETE /images/{id}`. No force flag — engine
//! refuses if the image is in use, and the caller is responsible for
//! removing dependents first.

use anyhow::Result;

use crate::client::Req;
use crate::ops::EngineHandler;

pub(crate) async fn run(h: &EngineHandler, id: String) -> Result<()> {
    let path = format!("/images/{id}");
    h.engine.unit(Req::delete(path)).await?;
    Ok(())
}
