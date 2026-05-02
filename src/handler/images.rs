use crate::engine::{self, Engine};
use crate::proto::*;
use anyhow::Result;
use futures::stream::{BoxStream, StreamExt};
use tokio::sync::mpsc;

use super::util::spawn_chunked;
use super::EngineHandler;

impl EngineHandler {
    pub(super) async fn list_images(&self) -> Result<Vec<ImageSummary>> {
        let imgs = self.engine.list_images().await?;
        Ok(imgs.into_iter().map(to_summary).collect())
    }

    pub(super) async fn remove_image(&self, id: String, force: bool) -> Result<()> {
        self.engine.remove_image(&id, force).await?;
        Ok(())
    }

    pub(super) fn pull_image(&self, reference: String) -> BoxStream<'static, StreamChunk> {
        let engine = self.engine.clone();
        spawn_chunked(move |tx| run_pull(engine, reference, tx))
    }
}

async fn run_pull(engine: Engine, reference: String, tx: mpsc::Sender<StreamChunk>) -> Result<(), String> {
    let mut stream = engine.pull_image(&reference).await.map_err(|e| e.to_string())?;
    while let Some(item) = stream.next().await {
        let prog = item.map_err(|e| e.to_string())?;
        let chunk = StreamChunk::PullProgress {
            id: prog.layer_id,
            status: prog.status,
            current: prog.current,
            total: prog.total,
        };
        if tx.send(chunk).await.is_err() {
            return Ok(());
        }
    }
    Ok(())
}

fn to_summary(i: engine::ImageSummary) -> ImageSummary {
    ImageSummary {
        id: i.id,
        repo_tag: i.repo_tag,
        created: i.created,
        size: i.size,
        containers: i.containers,
    }
}
