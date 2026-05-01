use crate::proto::*;
use anyhow::Result;
use bollard::models::{CreateImageInfo, ImageSummary as RawImage};
use bollard::Docker;
use futures::stream::{BoxStream, StreamExt};
use tokio::sync::mpsc;

use super::util::{short_id, spawn_chunked};
use super::DockerHandler;

impl DockerHandler {
    pub(super) async fn list_images(&self) -> Result<Vec<ImageSummary>> {
        let imgs = self.docker.list_images::<String>(None).await?;
        Ok(imgs.into_iter().map(summarize_image).collect())
    }

    pub(super) async fn remove_image(&self, id: String, force: bool) -> Result<()> {
        use bollard::image::RemoveImageOptions;
        let opts = RemoveImageOptions {
            force,
            noprune: false,
        };
        self.docker.remove_image(&id, Some(opts), None).await?;
        Ok(())
    }

    pub(super) fn pull_image(&self, reference: String) -> BoxStream<'static, StreamChunk> {
        let docker = self.docker.clone();
        spawn_chunked(move |tx| run_pull(docker, reference, tx))
    }
}

async fn run_pull(
    docker: Docker,
    reference: String,
    tx: mpsc::Sender<StreamChunk>,
) -> std::result::Result<(), String> {
    use bollard::image::CreateImageOptions;
    let opts = CreateImageOptions::<String> {
        from_image: reference,
        ..Default::default()
    };
    let stream = docker.create_image(Some(opts), None, None);
    futures::pin_mut!(stream);
    while let Some(item) = stream.next().await {
        let info = item.map_err(|e| e.to_string())?;
        if let Some(err) = info.error {
            return Err(err);
        }
        if tx.send(pull_chunk(info)).await.is_err() {
            return Ok(());
        }
    }
    Ok(())
}

fn pull_chunk(info: CreateImageInfo) -> StreamChunk {
    let (current, total) = info
        .progress_detail
        .map(|d| (d.current.unwrap_or(0) as u64, d.total.unwrap_or(0) as u64))
        .unwrap_or((0, 0));
    StreamChunk::PullProgress {
        id: info.id.unwrap_or_default(),
        status: info.status.unwrap_or_default(),
        current,
        total,
    }
}

fn summarize_image(i: RawImage) -> ImageSummary {
    ImageSummary {
        id: short_id(&i.id),
        repo_tag: i
            .repo_tags
            .into_iter()
            .next()
            .unwrap_or_else(|| "<none>".into()),
        created: i.created,
        size: i.size,
        containers: i.containers,
    }
}
