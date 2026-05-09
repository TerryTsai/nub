//! `docker image ls` — `GET /images/json`. Compat path works on both engines.

use anyhow::Result;

use super::wire::RawImage;
use crate::client::{short_id, Req};
use crate::ops::EngineHandler;
use crate::proto::ImageSummary;

pub(crate) async fn run(h: &EngineHandler) -> Result<Vec<ImageSummary>> {
    let raw: Vec<RawImage> = h.engine.conn().await?.send_unary(Req::get("/images/json")).await?.json()?;
    Ok(raw.into_iter().map(into_summary).collect())
}

fn into_summary(raw: RawImage) -> ImageSummary {
    ImageSummary {
        id: short_id(&raw.id),
        repo_tag: raw.repo_tags.and_then(|t| t.into_iter().next()).unwrap_or_else(|| "<none>".into()),
        created: raw.created,
        size: raw.size,
        containers: raw.containers,
    }
}
