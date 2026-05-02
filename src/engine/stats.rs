//! Container stats. NDJSON stream of metric snapshots; we also compute the
//! CPU percentage delta here so callers don't need to know the math.

use std::pin::Pin;
use std::task::{Context, Poll};

use futures::stream::{Stream, StreamExt};
use http_body_util::BodyExt;
use serde::Deserialize;

use super::http::Conn;
use super::images::LineStream;
use super::{Engine, Error, Query, Req, Result};

#[derive(Debug, Clone)]
pub struct Stats {
    pub cpu_pct: f64,
    pub mem_used: u64,
    pub mem_limit: u64,
    pub net_rx: u64,
    pub net_tx: u64,
}

impl Engine {
    pub async fn stream_stats(&self, id: &str) -> Result<StatsStream> {
        let mut q = Query::new();
        q.push_bool("stream", true);
        let path = format!("/containers/{id}/stats{}", q.finish());
        let mut conn = self.conn().await?;
        let res = conn.send_streaming(Req::get(path).build()?).await?;
        if !res.status().is_success() {
            let status = res.status().as_u16();
            let body = res.into_body().collect().await
                .map_err(|e| Error::Transport(format!("{e}")))?
                .to_bytes();
            return Err(Error::Status {
                code: status,
                message: String::from_utf8_lossy(&body).into_owned(),
            });
        }
        Ok(StatsStream {
            inner: LineStream::new(res.into_body()),
            _conn: conn,
        })
    }
}

pub struct StatsStream {
    inner: LineStream,
    _conn: Conn,
}

impl Stream for StatsStream {
    type Item = Result<Stats>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            let line = match futures::ready!(self.inner.poll_next_unpin(cx)) {
                None => return Poll::Ready(None),
                Some(Err(e)) => return Poll::Ready(Some(Err(e))),
                Some(Ok(l)) => l,
            };
            if line.iter().all(u8::is_ascii_whitespace) {
                continue;
            }
            let parsed = serde_json::from_slice::<RawStats>(&line)
                .map(RawStats::into_stats)
                .map_err(|e| Error::Decode(format!("{e}")));
            return Poll::Ready(Some(parsed));
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawStats {
    #[serde(default)]
    cpu_stats: CpuStats,
    #[serde(default)]
    precpu_stats: CpuStats,
    #[serde(default)]
    memory_stats: MemStats,
    #[serde(default)]
    networks: Option<std::collections::HashMap<String, NetStats>>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct CpuStats {
    #[serde(default)]
    cpu_usage: CpuUsage,
    #[serde(default)]
    system_cpu_usage: u64,
    #[serde(default)]
    online_cpus: u64,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct CpuUsage {
    #[serde(default)]
    total_usage: u64,
    #[serde(default)]
    percpu_usage: Option<Vec<u64>>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct MemStats {
    #[serde(default)]
    usage: u64,
    #[serde(default)]
    limit: u64,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct NetStats {
    #[serde(default)]
    rx_bytes: u64,
    #[serde(default)]
    tx_bytes: u64,
}

impl RawStats {
    fn into_stats(self) -> Stats {
        let cpu_delta = self.cpu_stats.cpu_usage.total_usage as i128
            - self.precpu_stats.cpu_usage.total_usage as i128;
        let sys_delta = self.cpu_stats.system_cpu_usage as i128
            - self.precpu_stats.system_cpu_usage as i128;
        let online = if self.cpu_stats.online_cpus > 0 {
            self.cpu_stats.online_cpus
        } else {
            self.cpu_stats
                .cpu_usage
                .percpu_usage
                .as_ref()
                .map(|p| p.len() as u64)
                .unwrap_or(1)
        };
        let cpu_pct = if sys_delta > 0 && cpu_delta > 0 {
            (cpu_delta as f64 / sys_delta as f64) * online as f64 * 100.0
        } else {
            0.0
        };
        let (net_rx, net_tx) = self
            .networks
            .map(|nets| {
                nets.values().fold((0u64, 0u64), |(rx, tx), n| {
                    (rx.saturating_add(n.rx_bytes), tx.saturating_add(n.tx_bytes))
                })
            })
            .unwrap_or((0, 0));
        Stats {
            cpu_pct,
            mem_used: self.memory_stats.usage,
            mem_limit: self.memory_stats.limit,
            net_rx,
            net_tx,
        }
    }
}
