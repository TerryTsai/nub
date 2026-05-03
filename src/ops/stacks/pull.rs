//! `pull_stack` — pull each service's image, then redeploy. Unary —
//! we don't stream layer-by-layer progress at the stack level (the
//! per-image pull op stays available for that). Stack pull blocks on
//! the engine until all images are present.

use std::collections::HashMap;

use anyhow::{anyhow, Result};
use http_body_util::BodyExt as _;
use serde::Deserialize;

use crate::client::{LineStream, Query, Req};
use crate::compose;
use crate::ops::EngineHandler;
use crate::proto::StackCreated;
use futures::stream::StreamExt as _;

use super::redeploy;
use super::store;

pub(crate) async fn run(h: &EngineHandler, name: String) -> Result<StackCreated> {
    store::validate_name(&name)?;
    if !store::exists(&h.policy.stacks_root, &name) {
        return Err(anyhow!("stack `{name}` not found"));
    }
    let yaml = store::read_yaml(&h.policy.stacks_root, &name)?;
    let spec = compose::parse(&yaml, &HashMap::new()).map_err(|e| anyhow!("compose: {e}"))?;
    for svc in &spec.services {
        pull_image(h, &svc.container.image).await?;
    }
    redeploy::run(h, name).await
}

async fn pull_image(h: &EngineHandler, reference: &str) -> Result<()> {
    let mut q = Query::new();
    q.push("fromImage", reference);
    let path = format!("/images/create{}", q.finish());
    let mut conn = h.engine.conn().await?;
    let res = conn.send_streaming(Req::post(path).build()?).await?;
    if !res.status().is_success() {
        let status = res.status().as_u16();
        let body = res.into_body().collect().await?.to_bytes();
        return Err(anyhow!(
            "engine returned {status} pulling `{reference}`: {}",
            String::from_utf8_lossy(&body)
        ));
    }
    let mut lines = LineStream::new(res.into_body());
    while let Some(line) = lines.next().await {
        let line = line?;
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        let info: PullLine = serde_json::from_slice(&line).map_err(|e| anyhow!("pull stream: {e}"))?;
        if let Some(err) = info.error {
            return Err(anyhow!("pulling `{reference}`: {err}"));
        }
    }
    Ok(())
}

#[derive(Deserialize)]
struct PullLine {
    #[serde(default)]
    error: Option<String>,
}
