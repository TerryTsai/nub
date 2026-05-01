use crate::proto::*;
use anyhow::Result;
use async_trait::async_trait;
use bollard::Docker;
use futures::stream::{BoxStream, StreamExt};

pub enum HandlerOutput {
    Unary(OpResult),
    Stream(BoxStream<'static, StreamChunk>),
}

#[async_trait]
pub trait OpHandler: Send + Sync + 'static {
    async fn handle(&self, op: Op) -> HandlerOutput;
}

pub struct DockerHandler {
    docker: Docker,
}

impl DockerHandler {
    pub fn connect() -> Result<Self> {
        let docker = Docker::connect_with_local_defaults()?;
        Ok(Self { docker })
    }
}

#[async_trait]
impl OpHandler for DockerHandler {
    async fn handle(&self, op: Op) -> HandlerOutput {
        match op {
            Op::HostInfo => match self.host_info().await {
                Ok(info) => HandlerOutput::Unary(OpResult::HostInfo(info)),
                Err(e) => HandlerOutput::Unary(OpResult::Err { message: e.to_string() }),
            },
            Op::ListContainers { all } => match self.list_containers(all).await {
                Ok(cs) => HandlerOutput::Unary(OpResult::Containers(cs)),
                Err(e) => HandlerOutput::Unary(OpResult::Err { message: e.to_string() }),
            },
            Op::StreamLogs { id, follow, tail } => {
                HandlerOutput::Stream(self.stream_logs(id, follow, tail))
            }
            Op::StreamStats { id } => HandlerOutput::Stream(self.stream_stats(id)),
        }
    }
}

impl DockerHandler {
    async fn host_info(&self) -> Result<HostInfo> {
        let info = self.docker.info().await?;
        let ver = self.docker.version().await?;
        Ok(HostInfo {
            engine: ver
                .platform
                .map(|p| p.name)
                .unwrap_or_else(|| "docker".into()),
            version: ver.version.unwrap_or_default(),
            os: info.operating_system.unwrap_or_default(),
            arch: info.architecture.unwrap_or_default(),
            kernel: info.kernel_version.unwrap_or_default(),
            cpus: info.ncpu.unwrap_or(0) as u64,
            mem_total: info.mem_total.unwrap_or(0) as u64,
            containers_running: info.containers_running.unwrap_or(0) as u64,
            containers_total: info.containers.unwrap_or(0) as u64,
            images: info.images.unwrap_or(0) as u64,
        })
    }

    async fn list_containers(&self, all: bool) -> Result<Vec<ContainerSummary>> {
        use bollard::container::ListContainersOptions;
        let opts = ListContainersOptions::<String> {
            all,
            ..Default::default()
        };
        let cs = self.docker.list_containers(Some(opts)).await?;
        Ok(cs
            .into_iter()
            .map(|c| ContainerSummary {
                id: c.id.unwrap_or_default().chars().take(12).collect(),
                name: c
                    .names
                    .and_then(|n| n.into_iter().next())
                    .map(|n| n.trim_start_matches('/').to_string())
                    .unwrap_or_default(),
                image: c.image.unwrap_or_default(),
                state: c.state.unwrap_or_default(),
                status: c.status.unwrap_or_default(),
                created: c.created.unwrap_or(0),
            })
            .collect())
    }

    fn stream_logs(
        &self,
        id: String,
        follow: bool,
        tail: Option<u32>,
    ) -> BoxStream<'static, StreamChunk> {
        use bollard::container::{LogOutput, LogsOptions};
        let docker = self.docker.clone();
        let (tx, rx) = tokio::sync::mpsc::channel::<StreamChunk>(32);
        tokio::spawn(async move {
            let opts = LogsOptions::<String> {
                follow,
                stdout: true,
                stderr: true,
                tail: tail.map(|n| n.to_string()).unwrap_or_else(|| "all".into()),
                ..Default::default()
            };
            let logs = docker.logs(&id, Some(opts));
            futures::pin_mut!(logs);
            let mut err: Option<String> = None;
            while let Some(item) = logs.next().await {
                let chunk = match item {
                    Ok(LogOutput::StdOut { message }) => StreamChunk::Log {
                        stderr: false,
                        data: String::from_utf8_lossy(&message).into_owned(),
                    },
                    Ok(LogOutput::StdErr { message }) => StreamChunk::Log {
                        stderr: true,
                        data: String::from_utf8_lossy(&message).into_owned(),
                    },
                    Ok(_) => continue,
                    Err(e) => {
                        err = Some(e.to_string());
                        break;
                    }
                };
                if tx.send(chunk).await.is_err() {
                    return;
                }
            }
            let _ = tx
                .send(StreamChunk::End { ok: err.is_none(), err })
                .await;
        });
        Box::pin(futures::stream::unfold(rx, |mut rx| async move {
            rx.recv().await.map(|c| (c, rx))
        }))
    }

    fn stream_stats(&self, id: String) -> BoxStream<'static, StreamChunk> {
        use bollard::container::StatsOptions;
        let docker = self.docker.clone();
        let (tx, rx) = tokio::sync::mpsc::channel::<StreamChunk>(32);
        tokio::spawn(async move {
            let opts = StatsOptions {
                stream: true,
                one_shot: false,
            };
            let stats = docker.stats(&id, Some(opts));
            futures::pin_mut!(stats);
            let mut err: Option<String> = None;
            while let Some(item) = stats.next().await {
                let chunk = match item {
                    Ok(s) => to_stats_chunk(&s),
                    Err(e) => {
                        err = Some(e.to_string());
                        break;
                    }
                };
                if tx.send(chunk).await.is_err() {
                    return;
                }
            }
            let _ = tx
                .send(StreamChunk::End { ok: err.is_none(), err })
                .await;
        });
        Box::pin(futures::stream::unfold(rx, |mut rx| async move {
            rx.recv().await.map(|c| (c, rx))
        }))
    }
}

fn to_stats_chunk(s: &bollard::container::Stats) -> StreamChunk {
    let cpu_delta =
        s.cpu_stats.cpu_usage.total_usage as i128 - s.precpu_stats.cpu_usage.total_usage as i128;
    let sys_delta = s.cpu_stats.system_cpu_usage.unwrap_or(0) as i128
        - s.precpu_stats.system_cpu_usage.unwrap_or(0) as i128;
    let online = s.cpu_stats.online_cpus.unwrap_or_else(|| {
        s.cpu_stats
            .cpu_usage
            .percpu_usage
            .as_ref()
            .map(|p| p.len() as u64)
            .unwrap_or(1)
    });
    let cpu_pct = if sys_delta > 0 && cpu_delta > 0 {
        (cpu_delta as f64 / sys_delta as f64) * online as f64 * 100.0
    } else {
        0.0
    };
    let mem_used = s.memory_stats.usage.unwrap_or(0);
    let mem_limit = s.memory_stats.limit.unwrap_or(0);
    let (net_rx, net_tx) = s
        .networks
        .as_ref()
        .map(|nets| {
            nets.values().fold((0u64, 0u64), |(rx, tx), n| {
                (rx.saturating_add(n.rx_bytes), tx.saturating_add(n.tx_bytes))
            })
        })
        .unwrap_or((0, 0));
    StreamChunk::Stats {
        cpu_pct,
        mem_used,
        mem_limit,
        net_rx,
        net_tx,
    }
}
