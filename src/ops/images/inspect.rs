//! `docker image inspect` — `GET /images/{id}/json`. Compat shape;
//! both engines return the same body.

use anyhow::Result;

use super::wire::RawInspect;
use crate::client::Req;
use crate::ops::EngineHandler;
use crate::proto::ImageDetail;

pub(crate) async fn run(h: &EngineHandler, id: String) -> Result<Box<ImageDetail>> {
    let path = format!("/images/{id}/json");
    let raw: RawInspect = h.engine.conn().await?.send_unary(Req::get(path)).await?.json()?;
    Ok(Box::new(into_detail(raw)))
}

fn into_detail(raw: RawInspect) -> ImageDetail {
    let mut exposed: Vec<String> = raw.config.exposed_ports.unwrap_or_default().into_keys().collect();
    exposed.sort();
    ImageDetail {
        id: raw.id,
        repo_tags: raw.repo_tags.unwrap_or_default(),
        repo_digests: raw.repo_digests.unwrap_or_default(),
        created: raw.created,
        size: raw.size,
        architecture: raw.architecture,
        os: raw.os,
        author: raw.author,
        comment: raw.comment,
        cmd: raw.config.cmd.unwrap_or_default(),
        entrypoint: raw.config.entrypoint.unwrap_or_default(),
        env: raw.config.env.unwrap_or_default(),
        working_dir: raw.config.working_dir,
        user: raw.config.user,
        exposed_ports: exposed,
        labels: raw.config.labels.unwrap_or_default(),
        layers: raw.root_fs.layers.len() as u32,
    }
}
