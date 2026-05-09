//! `docker container logs` — `GET /containers/{id}/logs?follow=...`. Both
//! engines use the same compat path. Output is multiplexed (8-byte headers)
//! for non-TTY containers and raw for TTY containers; we sniff the first
//! byte to decide.

use anyhow::{bail, Result};
use bytes::Bytes;
use futures::stream::BoxStream;
use http_body_util::BodyExt as _;
use hyper::body::Incoming;
use tokio::sync::mpsc;

use crate::client::{Engine, Multiplexer, MultiplexerMode, MuxFrame, Query, Req};
use crate::ops::{log_chunk, spawn_chunked, EngineHandler};
use crate::proto::StreamChunk;

pub(crate) fn run(h: &EngineHandler, id: String, follow: bool, tail: Option<u32>) -> BoxStream<'static, StreamChunk> {
    let engine = h.engine.clone();
    spawn_chunked(move |tx| pump(engine, id, follow, tail, tx))
}

async fn pump(
    engine: Engine,
    id: String,
    follow: bool,
    tail: Option<u32>,
    tx: mpsc::Sender<StreamChunk>,
) -> Result<()> {
    let res = engine.conn().await?.send_streaming(Req::get(logs_path(&id, follow, tail))).await?;
    if !res.status().is_success() {
        let status = res.status().as_u16();
        let body = collect_body(res.into_body()).await;
        bail!("engine returned {status}: {}", String::from_utf8_lossy(&body));
    }
    forward_frames(res.into_body(), tx).await
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

async fn forward_frames(mut body: Incoming, tx: mpsc::Sender<StreamChunk>) -> Result<()> {
    let mut mux = Multiplexer::new(MultiplexerMode::Detect);
    while let Some(frame) = body.frame().await {
        let frame = frame?;
        if let Ok(data) = frame.into_data() {
            mux.push(&data);
            if !drain_into_tx(&mut mux, &tx).await {
                return Ok(());
            }
        }
    }
    if let Some(leftover) = mux.finish() {
        let _ = tx.send(to_chunk(&leftover)).await;
    }
    Ok(())
}

async fn drain_into_tx(mux: &mut Multiplexer, tx: &mpsc::Sender<StreamChunk>) -> bool {
    while let Some(frame) = mux.next_frame() {
        if tx.send(to_chunk(&frame)).await.is_err() {
            return false;
        }
    }
    true
}

fn to_chunk(frame: &MuxFrame) -> StreamChunk {
    log_chunk(frame.stderr, &frame.data)
}

async fn collect_body(body: Incoming) -> Bytes {
    body.collect().await.map(|c| c.to_bytes()).unwrap_or_default()
}
