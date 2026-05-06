//! `docker image build` — `POST /build`. Streams line-delimited JSON
//! progress events. Build context is a single-file ustar tar containing
//! just the Dockerfile content the caller passed in.
//!
//! The handler is content-only: it does NOT touch the dockerfiles
//! directory. Callers fetch dockerfile bytes via `Op::GetDockerfile`
//! (gated by `dockerfiles:get`) and pass them here. This keeps `images:build`
//! from being a transitive `dockerfiles:get`.
//!
//! `pull=never` is forced on the engine, so a missing `FROM` base image
//! fails the build instead of triggering an implicit `images:pull`. The
//! caller pre-pulls explicitly.

use std::collections::HashMap;

use bytes::Bytes;
use futures::stream::{BoxStream, StreamExt};
use http_body_util::BodyExt as _;
use serde::Deserialize;
use tokio::sync::mpsc;

use crate::client::{Engine, LineStream, Query, Req};
use crate::ops::{spawn_chunked, EngineHandler};
use crate::proto::StreamChunk;

use super::tar;

pub(crate) fn run(
    h: &EngineHandler,
    dockerfile_content: String,
    tag: String,
    build_args: HashMap<String, String>,
) -> BoxStream<'static, StreamChunk> {
    let engine = h.engine.clone();
    spawn_chunked(move |tx| pump(engine, dockerfile_content, tag, build_args, tx))
}

async fn pump(
    engine: Engine,
    dockerfile_content: String,
    tag: String,
    build_args: HashMap<String, String>,
    tx: mpsc::Sender<StreamChunk>,
) -> Result<(), String> {
    let body = tar::one_file(b"Dockerfile", dockerfile_content.as_bytes());
    let path = format!("/build{}", build_query(&tag, &build_args)?);
    let req = Req::post(path)
        .bytes("application/x-tar", Bytes::from(body))
        .build()
        .map_err(|e| e.to_string())?;
    let mut conn = engine.conn().await.map_err(|e| e.to_string())?;
    let res = conn.send_streaming(req).await.map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        let status = res.status().as_u16();
        let body = res.into_body().collect().await.map_err(|e| e.to_string())?.to_bytes();
        return Err(format!("engine returned {status}: {}", String::from_utf8_lossy(&body)));
    }
    let mut lines = LineStream::new(res.into_body());
    while let Some(line) = lines.next().await {
        let line = line.map_err(|e| e.to_string())?;
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        let info: BuildInfo = serde_json::from_slice(&line).map_err(|e| e.to_string())?;
        if let Some(err) = info.error {
            return Err(err);
        }
        let chunk = StreamChunk::BuildProgress {
            stream: info.stream.unwrap_or_default(),
            image_id: info.aux.and_then(|a| a.id),
        };
        if tx.send(chunk).await.is_err() {
            return Ok(());
        }
    }
    Ok(())
}

fn build_query(tag: &str, build_args: &HashMap<String, String>) -> Result<String, String> {
    let mut q = Query::new();
    q.push("dockerfile", "Dockerfile");
    // Force engine to use only locally-present base images. A missing FROM
    // image fails the build cleanly rather than implicitly invoking pull.
    q.push("pull", "never");
    if !tag.is_empty() {
        q.push("t", tag);
    }
    if !build_args.is_empty() {
        let json = serde_json::to_string(build_args).map_err(|e| e.to_string())?;
        q.push("buildargs", &json);
    }
    Ok(q.finish())
}

#[derive(Deserialize)]
struct BuildInfo {
    #[serde(default)]
    stream: Option<String>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    aux: Option<BuildAux>,
}

#[derive(Deserialize)]
struct BuildAux {
    #[serde(default, rename = "ID")]
    id: Option<String>,
}
