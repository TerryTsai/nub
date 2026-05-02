//! `docker image ls` — `GET /images/json`. Compat path works on both engines.

use anyhow::Result;
use serde::Deserialize;

use crate::client::{short_id, Req};
use crate::ops::EngineHandler;
use crate::proto::ImageSummary;

pub(crate) async fn run(h: &EngineHandler) -> Result<Vec<ImageSummary>> {
    let raw: Vec<RawImage> = h
        .engine
        .conn()
        .await?
        .send_unary(Req::get("/images/json").build()?)
        .await?
        .json()?;
    Ok(raw.into_iter().map(RawImage::into_summary).collect())
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawImage {
    #[serde(rename = "Id")]
    id: String,
    #[serde(default)]
    repo_tags: Option<Vec<String>>,
    #[serde(default)]
    created: i64,
    #[serde(default)]
    size: i64,
    #[serde(default)]
    containers: i64,
}

impl RawImage {
    fn into_summary(self) -> ImageSummary {
        ImageSummary {
            id: short_id(&self.id),
            repo_tag: self
                .repo_tags
                .and_then(|t| t.into_iter().next())
                .unwrap_or_else(|| "<none>".into()),
            created: self.created,
            size: self.size,
            containers: self.containers,
        }
    }
}
