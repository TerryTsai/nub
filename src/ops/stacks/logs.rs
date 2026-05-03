//! `stream_stack_logs` — interleave log lines from every container in
//! the stack, prefixed with the container's short name. Engine-level
//! pumping mirrors `containers::logs`; we just fan out across N
//! containers and rewrite chunks before forwarding.

use std::collections::HashMap;

use futures::stream::BoxStream;
use http_body_util::BodyExt as _;
use hyper::body::Incoming;
use serde::Deserialize;
use tokio::sync::mpsc;

use crate::client::{Engine, EngineKind, Multiplexer, MultiplexerMode, Query, Req};
use crate::ops::{spawn_chunked, EngineHandler};
use crate::proto::StreamChunk;

use super::labels::STACK_LABEL;

pub(crate) fn run(h: &EngineHandler, name: String, follow: bool, tail: Option<u32>) -> BoxStream<'static, StreamChunk> {
    let engine = h.engine.clone();
    spawn_chunked(move |tx| pump(engine, name, follow, tail, tx))
}

async fn pump(
    engine: Engine,
    stack: String,
    follow: bool,
    tail: Option<u32>,
    tx: mpsc::Sender<StreamChunk>,
) -> Result<(), String> {
    let members = list_stack_containers(&engine, &stack).await?;
    if members.is_empty() {
        return Err(format!("stack `{stack}` has no containers"));
    }
    let mut handles = Vec::with_capacity(members.len());
    for (id, name) in members {
        let engine = engine.clone();
        let tx = tx.clone();
        let prefix = format!("[{name}] ");
        handles.push(tokio::spawn(forward_one(engine, id, prefix, follow, tail, tx)));
    }
    for h in handles {
        let _ = h.await;
    }
    Ok(())
}

async fn forward_one(
    engine: Engine,
    id: String,
    prefix: String,
    follow: bool,
    tail: Option<u32>,
    tx: mpsc::Sender<StreamChunk>,
) {
    let mut conn = match engine.conn().await {
        Ok(c) => c,
        Err(_) => return,
    };
    let path = logs_path(&id, follow, tail);
    let req = match Req::get(path).build() {
        Ok(r) => r,
        Err(_) => return,
    };
    let res = match conn.send_streaming(req).await {
        Ok(r) => r,
        Err(_) => return,
    };
    if !res.status().is_success() {
        return;
    }
    forward_frames(res.into_body(), prefix, &tx).await;
}

fn logs_path(id: &str, follow: bool, tail: Option<u32>) -> String {
    let mut q = Query::new();
    q.push_bool("follow", follow);
    q.push_bool("stdout", true);
    q.push_bool("stderr", true);
    let tail = tail.map(|n| n.to_string()).unwrap_or_else(|| "all".into());
    q.push("tail", &tail);
    format!("/containers/{id}/logs{}", q.finish())
}

async fn forward_frames(mut body: Incoming, prefix: String, tx: &mpsc::Sender<StreamChunk>) {
    let mut mux = Multiplexer::new(MultiplexerMode::Detect);
    while let Some(frame) = body.frame().await {
        let Ok(frame) = frame else { return };
        let Ok(data) = frame.into_data() else { continue };
        mux.push(&data);
        while let Some(f) = mux.next_frame() {
            let chunk = StreamChunk::Log {
                stderr: f.stderr,
                data: format!("{prefix}{}", String::from_utf8_lossy(&f.data)),
            };
            if tx.send(chunk).await.is_err() {
                return;
            }
        }
    }
    if let Some(f) = mux.finish() {
        let chunk = StreamChunk::Log {
            stderr: f.stderr,
            data: format!("{prefix}{}", String::from_utf8_lossy(&f.data)),
        };
        let _ = tx.send(chunk).await;
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawListItem {
    #[serde(rename = "Id")]
    id: String,
    #[serde(default)]
    names: Vec<String>,
    #[serde(default)]
    labels: HashMap<String, String>,
}

async fn list_stack_containers(engine: &Engine, stack: &str) -> Result<Vec<(String, String)>, String> {
    let path = match engine.kind() {
        EngineKind::Podman => "/v4.0.0/libpod/containers/json?all=true",
        EngineKind::Docker => "/containers/json?all=true",
    };
    let bytes = engine
        .conn()
        .await
        .map_err(|e| e.to_string())?
        .send_unary(Req::get(path.to_string()).build().map_err(|e| e.to_string())?)
        .await
        .map_err(|e| e.to_string())?;
    let raw: Vec<RawListItem> = bytes.json().map_err(|e| e.to_string())?;
    Ok(raw
        .into_iter()
        .filter(|i| i.labels.get(STACK_LABEL).map(String::as_str) == Some(stack))
        .map(|i| (i.id, short_name(&i.names)))
        .collect())
}

fn short_name(names: &[String]) -> String {
    names
        .first()
        .map(|n| n.trim_start_matches('/').to_string())
        .unwrap_or_default()
}
