use crate::proto::*;
use bollard::container::LogOutput;
use bollard::errors::Error as BollardError;
use bollard::Docker;
use futures::stream::{BoxStream, Stream, StreamExt};
use std::pin::Pin;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::sync::mpsc;

use super::util::{log_chunk, spawn_chunked};
use super::DockerHandler;

type ExecOutput = Pin<Box<dyn Stream<Item = std::result::Result<LogOutput, BollardError>> + Send>>;
type ExecInput = Pin<Box<dyn AsyncWrite + Send>>;

impl DockerHandler {
    pub(super) fn stream_logs(
        &self,
        id: String,
        follow: bool,
        tail: Option<u32>,
    ) -> BoxStream<'static, StreamChunk> {
        let docker = self.docker.clone();
        spawn_chunked(move |tx| run_logs(docker, id, follow, tail, tx))
    }

    pub(super) fn stream_stats(&self, id: String) -> BoxStream<'static, StreamChunk> {
        let docker = self.docker.clone();
        spawn_chunked(move |tx| run_stats(docker, id, tx))
    }

    pub(super) fn exec(
        &self,
        id: String,
        cmd: Vec<String>,
        tty: bool,
        input: mpsc::Receiver<StreamChunk>,
    ) -> BoxStream<'static, StreamChunk> {
        let docker = self.docker.clone();
        spawn_chunked(move |tx| run_exec(docker, id, cmd, tty, input, tx))
    }
}

async fn run_logs(
    docker: Docker,
    id: String,
    follow: bool,
    tail: Option<u32>,
    tx: mpsc::Sender<StreamChunk>,
) -> std::result::Result<(), String> {
    use bollard::container::LogsOptions;
    let opts = LogsOptions::<String> {
        follow,
        stdout: true,
        stderr: true,
        tail: tail.map(|n| n.to_string()).unwrap_or_else(|| "all".into()),
        ..Default::default()
    };
    let logs = docker.logs(&id, Some(opts));
    futures::pin_mut!(logs);
    while let Some(item) = logs.next().await {
        let chunk = match item {
            Ok(LogOutput::StdOut { message }) => log_chunk(false, &message),
            Ok(LogOutput::StdErr { message }) => log_chunk(true, &message),
            Ok(_) => continue,
            Err(e) => return Err(e.to_string()),
        };
        if tx.send(chunk).await.is_err() {
            return Ok(());
        }
    }
    Ok(())
}

async fn run_stats(
    docker: Docker,
    id: String,
    tx: mpsc::Sender<StreamChunk>,
) -> std::result::Result<(), String> {
    use bollard::container::StatsOptions;
    let opts = StatsOptions {
        stream: true,
        one_shot: false,
    };
    let stats = docker.stats(&id, Some(opts));
    futures::pin_mut!(stats);
    while let Some(item) = stats.next().await {
        let chunk = match item {
            Ok(s) => to_stats_chunk(&s),
            Err(e) => return Err(e.to_string()),
        };
        if tx.send(chunk).await.is_err() {
            return Ok(());
        }
    }
    Ok(())
}

async fn run_exec(
    docker: Docker,
    id: String,
    cmd: Vec<String>,
    tty: bool,
    in_rx: mpsc::Receiver<StreamChunk>,
    out_tx: mpsc::Sender<StreamChunk>,
) -> std::result::Result<(), String> {
    use bollard::exec::{CreateExecOptions, StartExecOptions, StartExecResults};
    let exec = docker
        .create_exec(
            &id,
            CreateExecOptions::<String> {
                cmd: Some(cmd),
                attach_stdin: Some(true),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                tty: Some(tty),
                ..Default::default()
            },
        )
        .await
        .map_err(|e| e.to_string())?;
    let started = docker
        .start_exec(
            &exec.id,
            Some(StartExecOptions {
                detach: false,
                tty,
                ..Default::default()
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    let (output, input) = match started {
        StartExecResults::Attached { output, input } => (output, input),
        StartExecResults::Detached => return Err("exec returned detached".into()),
    };
    let stdin_task = tokio::spawn(pump_exec_stdin(input, in_rx));
    let result = pump_exec_output(output, out_tx).await;
    stdin_task.abort();
    result
}

async fn pump_exec_stdin(mut input: ExecInput, mut in_rx: mpsc::Receiver<StreamChunk>) {
    while let Some(chunk) = in_rx.recv().await {
        if !forward_input(&mut input, chunk).await {
            break;
        }
    }
}

async fn forward_input(input: &mut ExecInput, chunk: StreamChunk) -> bool {
    match chunk {
        StreamChunk::Stdin { data } => input.write_all(data.as_bytes()).await.is_ok(),
        StreamChunk::StdinClose => {
            let _ = input.shutdown().await;
            false
        }
        _ => true,
    }
}

async fn pump_exec_output(
    mut output: ExecOutput,
    tx: mpsc::Sender<StreamChunk>,
) -> std::result::Result<(), String> {
    while let Some(item) = output.next().await {
        let chunk = match item {
            Ok(LogOutput::StdOut { message }) => log_chunk(false, &message),
            Ok(LogOutput::StdErr { message }) => log_chunk(true, &message),
            Ok(_) => continue,
            Err(e) => return Err(e.to_string()),
        };
        if tx.send(chunk).await.is_err() {
            return Ok(());
        }
    }
    Ok(())
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
        mem_used: s.memory_stats.usage.unwrap_or(0),
        mem_limit: s.memory_stats.limit.unwrap_or(0),
        net_rx,
        net_tx,
    }
}
