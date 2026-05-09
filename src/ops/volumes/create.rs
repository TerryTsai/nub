//! `docker volume create` — `POST /volumes/create`. Driver, labels, and
//! options all flow through; the public op leaves driver and options
//! empty, the stack runtime fills in labels.

use std::collections::HashMap;

use anyhow::Result;

use super::wire::CreateBody;
use crate::client::Req;
use crate::ops::EngineHandler;

pub(crate) async fn run(
    h: &EngineHandler,
    name: String,
    driver: Option<String>,
    labels: HashMap<String, String>,
    options: HashMap<String, String>,
) -> Result<()> {
    let body = CreateBody {
        name,
        driver,
        labels,
        driver_opts: options,
    };
    h.engine.conn().await?.send_unary(Req::post("/volumes/create").json(&body)?).await?.ok()?;
    Ok(())
}
