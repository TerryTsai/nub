//! `docker image inspect` — `GET /images/{id}/json`. Compat shape;
//! both engines return the same body.

use std::collections::HashMap;

use anyhow::Result;
use serde::Deserialize;

use crate::client::Req;
use crate::ops::EngineHandler;
use crate::proto::ImageDetail;

pub(crate) async fn run(h: &EngineHandler, id: String) -> Result<Box<ImageDetail>> {
    let path = format!("/images/{id}/json");
    let raw: RawInspect = h
        .engine
        .conn()
        .await?
        .send_unary(Req::get(path).build()?)
        .await?
        .json()?;
    Ok(Box::new(raw.into_detail()))
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawInspect {
    #[serde(rename = "Id", default)]
    id: String,
    #[serde(default)]
    repo_tags: Option<Vec<String>>,
    #[serde(default)]
    repo_digests: Option<Vec<String>>,
    #[serde(default)]
    created: String,
    #[serde(default)]
    size: i64,
    #[serde(default)]
    architecture: String,
    #[serde(default)]
    os: String,
    #[serde(default)]
    author: String,
    #[serde(default)]
    comment: String,
    #[serde(default)]
    config: RawConfig,
    #[serde(default)]
    root_fs: RawRootFs,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawConfig {
    // Docker emits explicit `null` for empty container slices and maps in
    // image inspect responses. `Option<...>` + `unwrap_or_default` accepts
    // both `null` and missing without serde griping.
    #[serde(default)]
    cmd: Option<Vec<String>>,
    #[serde(default)]
    entrypoint: Option<Vec<String>>,
    #[serde(default)]
    env: Option<Vec<String>>,
    #[serde(default)]
    working_dir: String,
    #[serde(default)]
    user: String,
    #[serde(default)]
    exposed_ports: Option<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    labels: Option<HashMap<String, String>>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawRootFs {
    #[serde(default)]
    layers: Vec<String>,
}

impl RawInspect {
    fn into_detail(self) -> ImageDetail {
        let mut exposed: Vec<String> = self.config.exposed_ports.unwrap_or_default().into_keys().collect();
        exposed.sort();
        ImageDetail {
            id: self.id,
            repo_tags: self.repo_tags.unwrap_or_default(),
            repo_digests: self.repo_digests.unwrap_or_default(),
            created: self.created,
            size: self.size,
            architecture: self.architecture,
            os: self.os,
            author: self.author,
            comment: self.comment,
            cmd: self.config.cmd.unwrap_or_default(),
            entrypoint: self.config.entrypoint.unwrap_or_default(),
            env: self.config.env.unwrap_or_default(),
            working_dir: self.config.working_dir,
            user: self.config.user,
            exposed_ports: exposed,
            labels: self.config.labels.unwrap_or_default(),
            layers: self.root_fs.layers.len() as u32,
        }
    }
}
