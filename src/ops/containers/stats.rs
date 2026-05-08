//! `docker container stats` — `GET /containers/{id}/stats?stream=true`.
//! NDJSON; we also compute the cpu_pct delta here so callers don't need to
//! know the math.

use std::collections::HashMap;

use futures::stream::{BoxStream, StreamExt};
use http_body_util::BodyExt as _;
use tokio::sync::mpsc;

use super::wire::stats::{RawCpu, RawNet, RawStats};
use crate::client::{Engine, LineStream, Query, Req};
use crate::ops::{spawn_chunked, EngineHandler};
use crate::proto::StreamChunk;

pub(crate) fn run(h: &EngineHandler, id: String) -> BoxStream<'static, StreamChunk> {
    let engine = h.engine.clone();
    spawn_chunked(move |tx| pump(engine, id, tx))
}

async fn pump(engine: Engine, id: String, tx: mpsc::Sender<StreamChunk>) -> Result<(), String> {
    let mut conn = engine.conn().await.map_err(|e| e.to_string())?;
    let path = format!("/containers/{id}/stats{}", stats_query());
    let res = conn
        .send_streaming(Req::get(path).build().map_err(|e| e.to_string())?)
        .await
        .map_err(|e| e.to_string())?;
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
        let raw: RawStats = serde_json::from_slice(&line).map_err(|e| e.to_string())?;
        if tx.send(to_chunk(&raw)).await.is_err() {
            return Ok(());
        }
    }
    Ok(())
}

fn stats_query() -> String {
    let mut q = Query::new();
    q.push_bool("stream", true);
    q.finish()
}

fn to_chunk(s: &RawStats) -> StreamChunk {
    let cpu_pct = compute_cpu_pct(&s.cpu_stats, &s.precpu_stats);
    let (net_rx, net_tx) = sum_net(s.networks.as_ref());
    StreamChunk::Stats {
        cpu_pct,
        mem_used: s.memory_stats.usage,
        mem_limit: s.memory_stats.limit,
        net_rx,
        net_tx,
    }
}

fn compute_cpu_pct(cur: &RawCpu, prev: &RawCpu) -> f64 {
    let cpu_delta = cur.cpu_usage.total_usage as i128 - prev.cpu_usage.total_usage as i128;
    let sys_delta = cur.system_cpu_usage as i128 - prev.system_cpu_usage as i128;
    let online = if cur.online_cpus > 0 {
        cur.online_cpus
    } else {
        cur.cpu_usage.percpu_usage.as_ref().map(|p| p.len() as u64).unwrap_or(1)
    };
    if sys_delta > 0 && cpu_delta > 0 {
        (cpu_delta as f64 / sys_delta as f64) * online as f64 * 100.0
    } else {
        0.0
    }
}

fn sum_net(nets: Option<&HashMap<String, RawNet>>) -> (u64, u64) {
    nets.map(|nets| {
        nets.values().fold((0u64, 0u64), |(rx, tx), n| {
            (rx.saturating_add(n.rx_bytes), tx.saturating_add(n.tx_bytes))
        })
    })
    .unwrap_or((0, 0))
}
