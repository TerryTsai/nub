//! `docker container exec` — two-step protocol: `POST /containers/{id}/exec`
//! to create the instance, then `POST /exec/{id}/start` with an HTTP upgrade
//! to a bidirectional byte stream. Output multiplexes stdout/stderr the same
//! way as logs.

use futures::stream::BoxStream;
use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::sync::mpsc;

use super::wire::exec::{CreateExecBody, CreateExecResp, StartExecBody};
use crate::client::{upgrade, Engine, Multiplexer, MultiplexerMode, MuxFrame, Req};
use crate::ops::{log_chunk, spawn_chunked, EngineHandler};
use crate::proto::StreamChunk;

pub(crate) fn run(
    h: &EngineHandler,
    id: String,
    cmd: Vec<String>,
    tty: bool,
    input: mpsc::Receiver<StreamChunk>,
) -> BoxStream<'static, StreamChunk> {
    let engine = h.engine.clone();
    spawn_chunked(move |tx| pump(engine, id, cmd, tty, input, tx))
}

async fn pump(
    engine: Engine,
    id: String,
    cmd: Vec<String>,
    tty: bool,
    in_rx: mpsc::Receiver<StreamChunk>,
    out_tx: mpsc::Sender<StreamChunk>,
) -> Result<(), String> {
    let exec_id = create_exec(&engine, &id, &cmd, tty).await?;
    let upgraded = start_exec(&engine, &exec_id, tty).await?;
    let (reader, writer) = tokio::io::split(upgraded);

    let stdin_handle = tokio::spawn(forward_stdin(writer, in_rx));
    let result = forward_output(reader, tty, out_tx).await;
    stdin_handle.abort();
    result
}

async fn create_exec(engine: &Engine, container_id: &str, cmd: &[String], tty: bool) -> Result<String, String> {
    let body = CreateExecBody {
        attach_stdin: true,
        attach_stdout: true,
        attach_stderr: true,
        tty,
        cmd: cmd.to_vec(),
    };
    let resp: CreateExecResp = engine
        .conn()
        .await
        .map_err(|e| e.to_string())?
        .send_unary(
            Req::post(format!("/containers/{container_id}/exec"))
                .json(&body)
                .map_err(|e| e.to_string())?
                .build()
                .map_err(|e| e.to_string())?,
        )
        .await
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;
    Ok(resp.id)
}

async fn start_exec(engine: &Engine, exec_id: &str, tty: bool) -> Result<TokioIo<Upgraded>, String> {
    let body = StartExecBody { detach: false, tty };
    let req = Req::post(format!("/exec/{exec_id}/start"))
        .json(&body)
        .map_err(|e| e.to_string())?
        .upgrade("tcp")
        .build()
        .map_err(|e| e.to_string())?;
    let res = engine
        .conn()
        .await
        .map_err(|e| e.to_string())?
        .send_streaming(req)
        .await
        .map_err(|e| e.to_string())?;
    upgrade(res).await.map_err(|e| e.to_string())
}

async fn forward_stdin(mut writer: WriteHalf<TokioIo<Upgraded>>, mut in_rx: mpsc::Receiver<StreamChunk>) {
    while let Some(chunk) = in_rx.recv().await {
        match chunk {
            StreamChunk::Stdin { data } => {
                if writer.write_all(data.as_bytes()).await.is_err() {
                    return;
                }
            }
            StreamChunk::StdinClose => {
                let _ = writer.shutdown().await;
                return;
            }
            _ => continue,
        }
    }
}

async fn forward_output(
    mut reader: ReadHalf<TokioIo<Upgraded>>,
    tty: bool,
    tx: mpsc::Sender<StreamChunk>,
) -> Result<(), String> {
    let mut mux = Multiplexer::new(if tty {
        MultiplexerMode::Tty
    } else {
        MultiplexerMode::Multiplexed
    });
    let mut tmp = [0u8; 4096];
    loop {
        let n = reader.read(&mut tmp).await.map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        mux.push(&tmp[..n]);
        if !drain_into_tx(&mut mux, &tx).await {
            return Ok(());
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
