use crate::engine::{Engine, ExecOptions, ExecOutput, ExecReader, ExecWriter, LogsOptions};
use crate::proto::*;
use futures::stream::{BoxStream, StreamExt};
use tokio::sync::mpsc;

use super::util::{log_chunk, spawn_chunked};
use super::EngineHandler;

impl EngineHandler {
    pub(super) fn stream_logs(&self, id: String, follow: bool, tail: Option<u32>) -> BoxStream<'static, StreamChunk> {
        let engine = self.engine.clone();
        spawn_chunked(move |tx| run_logs(engine, id, follow, tail, tx))
    }

    pub(super) fn stream_stats(&self, id: String) -> BoxStream<'static, StreamChunk> {
        let engine = self.engine.clone();
        spawn_chunked(move |tx| run_stats(engine, id, tx))
    }

    pub(super) fn exec(
        &self,
        id: String,
        cmd: Vec<String>,
        tty: bool,
        input: mpsc::Receiver<StreamChunk>,
    ) -> BoxStream<'static, StreamChunk> {
        let engine = self.engine.clone();
        spawn_chunked(move |tx| run_exec(engine, id, cmd, tty, input, tx))
    }
}

async fn run_logs(
    engine: Engine,
    id: String,
    follow: bool,
    tail: Option<u32>,
    tx: mpsc::Sender<StreamChunk>,
) -> Result<(), String> {
    let opts = LogsOptions { follow, tail };
    let mut stream = engine.stream_logs(&id, opts).await.map_err(|e| e.to_string())?;
    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| e.to_string())?;
        if tx.send(log_chunk(chunk.stderr, &chunk.data)).await.is_err() {
            return Ok(());
        }
    }
    Ok(())
}

async fn run_stats(engine: Engine, id: String, tx: mpsc::Sender<StreamChunk>) -> Result<(), String> {
    let mut stream = engine.stream_stats(&id).await.map_err(|e| e.to_string())?;
    while let Some(item) = stream.next().await {
        let s = item.map_err(|e| e.to_string())?;
        let chunk = StreamChunk::Stats {
            cpu_pct: s.cpu_pct,
            mem_used: s.mem_used,
            mem_limit: s.mem_limit,
            net_rx: s.net_rx,
            net_tx: s.net_tx,
        };
        if tx.send(chunk).await.is_err() {
            return Ok(());
        }
    }
    Ok(())
}

async fn run_exec(
    engine: Engine,
    id: String,
    cmd: Vec<String>,
    tty: bool,
    in_rx: mpsc::Receiver<StreamChunk>,
    out_tx: mpsc::Sender<StreamChunk>,
) -> Result<(), String> {
    let opts = ExecOptions { cmd, tty };
    let stream = engine.exec(&id, opts).await.map_err(|e| e.to_string())?;
    // Reader and writer halves run independently — split() means no shared
    // state to coordinate, no locks.
    let stdin_handle = tokio::spawn(pump_stdin(stream.writer, in_rx));
    let result = pump_output(stream.reader, out_tx).await;
    stdin_handle.abort();
    result
}

async fn pump_stdin(mut writer: ExecWriter, mut in_rx: mpsc::Receiver<StreamChunk>) {
    while let Some(chunk) = in_rx.recv().await {
        match chunk {
            StreamChunk::Stdin { data } => {
                if writer.write_stdin(data.as_bytes()).await.is_err() {
                    return;
                }
            }
            StreamChunk::StdinClose => {
                let _ = writer.close_stdin().await;
                return;
            }
            _ => continue,
        }
    }
}

async fn pump_output(mut reader: ExecReader, tx: mpsc::Sender<StreamChunk>) -> Result<(), String> {
    while let Some(item) = reader.next().await {
        let out = item.map_err(|e| e.to_string())?;
        let chunk = match out {
            ExecOutput::Stdout(b) => log_chunk(false, &b),
            ExecOutput::Stderr(b) => log_chunk(true, &b),
        };
        if tx.send(chunk).await.is_err() {
            return Ok(());
        }
    }
    Ok(())
}
