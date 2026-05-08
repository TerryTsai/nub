//! Transport-agnostic WebSocket pump. Drives the `Frame`/`StreamChunk`
//! protocol against an mpsc pair so the same logic serves any transport
//! that can deliver text frames in and out.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use futures::stream::BoxStream;
use futures::StreamExt;
use tokio::sync::{broadcast, mpsc};

use crate::auth::{introspect, Claims};
use crate::ops::{HandlerOutput, Shared};
use crate::proto::{Frame, Op, OpResult, StreamChunk};

const STREAM_BUF: usize = 64;
const INPUT_BUF: usize = 64;
/// Per-connection inbound-routing table: request id → input channel for
/// the spawned handler task. We use `std::sync::Mutex` rather than
/// tokio's because the lock is taken only for short, synchronous
/// HashMap insert/get/remove calls and is NEVER held across `.await`.
type Routes = Arc<Mutex<HashMap<u64, mpsc::Sender<StreamChunk>>>>;

enum Inbound {
    Request { id: u64, op: Op },
    Stream { id: u64, chunk: StreamChunk },
}

/// Drive the wire protocol on a transport-agnostic mpsc pair. The connection
/// has been authenticated as `caller`; per-Request `caller.allows()` gates
/// what can run. Returns when `in_rx` is closed.
pub async fn serve(handler: Shared, caller: Claims, mut in_rx: mpsc::Receiver<String>, out_tx: mpsc::Sender<String>) {
    let routes: Routes = Arc::new(Mutex::new(HashMap::new()));
    while let Some(text) = in_rx.recv().await {
        if out_tx.is_closed() {
            break;
        }
        dispatch(&text, &handler, &caller, &out_tx, &routes).await;
    }
}

async fn dispatch(text: &str, h: &Shared, caller: &Claims, out: &mpsc::Sender<String>, routes: &Routes) {
    match parse_inbound(text) {
        Ok(Inbound::Request { id, op }) => start_request(id, op, h, caller, out, routes),
        Ok(Inbound::Stream { id, chunk }) => forward_to_route(id, chunk, routes).await,
        Err(msg) => {
            send_frame(out, err_reply(0, msg)).await;
        }
    }
}

fn start_request(id: u64, op: Op, h: &Shared, caller: &Claims, out: &mpsc::Sender<String>, routes: &Routes) {
    if let Some(result) = introspect(caller, &op) {
        let out = out.clone();
        tokio::spawn(async move {
            send_frame(&out, response(id, result)).await;
        });
        return;
    }
    if !caller.allows(&op) {
        tracing::warn!("caller {} denied op {}", caller.sub, op.name());
        let out = out.clone();
        let name = op.name().to_string();
        tokio::spawn(async move {
            send_frame(&out, err_reply(id, format!("op `{name}` not in allowed list"))).await;
        });
        return;
    }
    let (in_tx, in_rx) = mpsc::channel::<StreamChunk>(INPUT_BUF);
    routes.lock().unwrap().insert(id, in_tx);
    let h = h.clone();
    let out = out.clone();
    let routes = routes.clone();
    let claims = caller.clone();
    tokio::spawn(async move {
        handle_request(id, op, h, &claims, in_rx, out).await;
        routes.lock().unwrap().remove(&id);
    });
}

async fn forward_to_route(id: u64, chunk: StreamChunk, routes: &Routes) {
    let tx = routes.lock().unwrap().get(&id).cloned();
    if let Some(tx) = tx {
        let _ = tx.send(chunk).await;
    }
}

fn parse_inbound(text: &str) -> Result<Inbound, String> {
    let frame: Frame = serde_json::from_str(text).map_err(|e| format!("bad frame: {e}"))?;
    match frame {
        Frame::Request { id, op } => Ok(Inbound::Request { id, op }),
        Frame::Stream { id, chunk } => Ok(Inbound::Stream { id, chunk }),
        Frame::Response { .. } => Err("unexpected response from peer".into()),
    }
}

fn err_reply(id: u64, message: String) -> Frame {
    Frame::Response {
        id,
        result: OpResult::Err { message },
    }
}

fn end_err(msg: &str) -> StreamChunk {
    StreamChunk::End {
        ok: false,
        err: Some(msg.into()),
    }
}

async fn send_frame(out: &mpsc::Sender<String>, frame: Frame) -> bool {
    let Ok(json) = serde_json::to_string(&frame) else {
        tracing::warn!("frame serialize failed");
        return true;
    };
    out.send(json).await.is_ok()
}

fn response(id: u64, result: OpResult) -> Frame {
    Frame::Response { id, result }
}

fn stream_frame(id: u64, chunk: StreamChunk) -> Frame {
    Frame::Stream { id, chunk }
}

async fn handle_request(
    id: u64,
    op: Op,
    h: Shared,
    claims: &Claims,
    in_rx: mpsc::Receiver<StreamChunk>,
    out: mpsc::Sender<String>,
) {
    match h.handle(op, claims, in_rx).await {
        HandlerOutput::Unary(result) => {
            send_frame(&out, response(id, result)).await;
        }
        HandlerOutput::Stream(source) => {
            send_frame(&out, response(id, OpResult::StreamStarted)).await;
            pump_stream(id, source, out).await;
        }
    }
}

async fn pump_stream(id: u64, source: BoxStream<'static, StreamChunk>, out: mpsc::Sender<String>) {
    let (b_tx, mut b_rx) = broadcast::channel::<StreamChunk>(STREAM_BUF);
    let producer = tokio::spawn(forward_to_broadcast(source, b_tx));
    loop {
        let chunk = match b_rx.recv().await {
            Ok(c) => c,
            Err(broadcast::error::RecvError::Lagged(n)) => StreamChunk::Lagging { dropped: n as u32 },
            Err(broadcast::error::RecvError::Closed) => {
                send_frame(&out, stream_frame(id, end_err("source ended without End chunk"))).await;
                break;
            }
        };
        let is_end = matches!(chunk, StreamChunk::End { .. });
        let alive = send_frame(&out, stream_frame(id, chunk)).await;
        if is_end || !alive {
            break;
        }
    }
    producer.abort();
}

async fn forward_to_broadcast(mut source: BoxStream<'static, StreamChunk>, tx: broadcast::Sender<StreamChunk>) {
    while let Some(chunk) = source.next().await {
        if tx.send(chunk).is_err() {
            break;
        }
    }
}
