//! `docker network create` — `POST /networks/create`. Driver is hard-set
//! to `bridge`; `internal: true` blocks external traffic. The `labels`
//! map is internal-only — the public op passes empty, the stack runtime
//! passes `nub.stack=<name>`.

use std::collections::HashMap;

use anyhow::Result;

use super::wire::CreateBody;
use crate::client::Req;
use crate::ops::EngineHandler;

pub(crate) async fn run(
    h: &EngineHandler,
    name: String,
    internal: bool,
    labels: HashMap<String, String>,
) -> Result<()> {
    let body = CreateBody {
        name,
        driver: "bridge".into(),
        internal,
        labels,
    };
    h.engine
        .conn()
        .await?
        .send_unary(Req::post("/networks/create").json(&body)?.build()?)
        .await?
        .ok()?;
    Ok(())
}
