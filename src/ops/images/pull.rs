//! `docker image pull` — `POST /images/create?fromImage=...`. Streams
//! line-delimited JSON progress events. Each line is one layer-status update.

use anyhow::{anyhow, Result};
use futures::stream::{BoxStream, StreamExt};
use http_body_util::BodyExt as _;
use tokio::sync::mpsc;

use super::wire::CreateImageInfo;
use crate::client::{Engine, LineStream, Query, Req};
use crate::ops::{spawn_chunked, EngineHandler};
use crate::proto::StreamChunk;

pub(crate) fn run(h: &EngineHandler, reference: String) -> BoxStream<'static, StreamChunk> {
    let engine = h.engine.clone();
    spawn_chunked(move |tx| pump(engine, reference, tx))
}

/// Drain `run`'s stream into a unary `Result`. Used by stack
/// deploy/pull to honor the "no implicit pull" rule on
/// `CreateContainer` — every `FROM`/`image:` ref must be local before
/// the engine sees it. Streaming `Op::PullImage` (the BoxStream
/// version) stays available for callers that want layer progress.
pub(crate) async fn run_unary(h: &EngineHandler, reference: &str) -> Result<()> {
    let mut stream = run(h, reference.to_string());
    while let Some(chunk) = stream.next().await {
        if let StreamChunk::End { ok: false, err } = chunk {
            return Err(anyhow!("pull {reference}: {}", err.unwrap_or_default()));
        }
    }
    Ok(())
}

async fn pump(engine: Engine, reference: String, tx: mpsc::Sender<StreamChunk>) -> Result<()> {
    let path = format!("/images/create{}", create_query(&reference));
    let res = engine.streaming(Req::post(path)).await?;
    if !res.status().is_success() {
        let status = res.status().as_u16();
        let body = res.into_body().collect().await?.to_bytes();
        anyhow::bail!("engine returned {status}: {}", String::from_utf8_lossy(&body));
    }

    let mut lines = LineStream::new(res.into_body());
    while let Some(line) = lines.next().await {
        let line = line?;
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        let chunk = parse_line(&line)?;
        if tx.send(chunk).await.is_err() {
            return Ok(());
        }
    }
    Ok(())
}

fn create_query(reference: &str) -> String {
    let mut q = Query::new();
    q.push("fromImage", reference);
    q.finish()
}

fn parse_line(line: &[u8]) -> Result<StreamChunk> {
    let info: CreateImageInfo = serde_json::from_slice(line)?;
    if let Some(err) = info.error {
        anyhow::bail!("{err}");
    }
    let (current, total) = info.progress_detail.map(|d| (d.current, d.total)).unwrap_or((0, 0));
    Ok(StreamChunk::PullProgress {
        id: info.id.unwrap_or_default(),
        status: info.status.unwrap_or_default(),
        current,
        total,
    })
}
