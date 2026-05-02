//! `docker image build` — `POST /build`. Streams line-delimited JSON
//! progress events. Build context is a single-file ustar tar containing
//! just the Dockerfile (read from the configured dockerfiles directory).
//! `COPY` / `ADD` of other files is unsupported by design — keep the
//! "phone-shaped" scope, fail loudly at engine build time if someone
//! tries.

use std::collections::HashMap;
use std::path::PathBuf;

use bytes::Bytes;
use futures::stream::{BoxStream, StreamExt};
use http_body_util::BodyExt as _;
use serde::Deserialize;
use tokio::sync::mpsc;

use crate::client::{Engine, LineStream, Query, Req};
use crate::ops::dockerfiles::valid_name;
use crate::ops::{spawn_chunked, EngineHandler};
use crate::proto::StreamChunk;

use super::tar;

pub(crate) fn run(
    h: &EngineHandler,
    dockerfile: String,
    tag: String,
    build_args: HashMap<String, String>,
) -> BoxStream<'static, StreamChunk> {
    let engine = h.engine.clone();
    let root = h.policy.dockerfiles_root.clone();
    spawn_chunked(move |tx| pump(engine, root, dockerfile, tag, build_args, tx))
}

async fn pump(
    engine: Engine,
    root: PathBuf,
    dockerfile: String,
    tag: String,
    build_args: HashMap<String, String>,
    tx: mpsc::Sender<StreamChunk>,
) -> Result<(), String> {
    if !valid_name(&dockerfile) {
        return Err(format!("invalid dockerfile name: {dockerfile:?}"));
    }
    let path = root.join(&dockerfile);
    let content = tokio::fs::read(&path)
        .await
        .map_err(|e| format!("reading {}: {e}", path.display()))?;
    let body = tar::one_file(b"Dockerfile", &content);
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
