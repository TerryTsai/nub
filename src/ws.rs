use crate::handler::{HandlerOutput, OpHandler};
use crate::proto::*;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use futures::stream::{BoxStream, SplitSink};
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, mpsc};

const WRITE_BUF: usize = 64;
const STREAM_BUF: usize = 64;
const INPUT_BUF: usize = 64;

type Shared = Arc<dyn OpHandler>;
type Routes = Arc<Mutex<HashMap<u64, mpsc::Sender<StreamChunk>>>>;

enum Inbound {
    Request { id: u64, op: Op },
    Stream { id: u64, chunk: StreamChunk },
}

pub async fn ws_handler(ws: WebSocketUpgrade, State(h): State<Shared>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, h))
}

async fn handle_socket(socket: WebSocket, h: Shared) {
    let (sink, mut stream) = socket.split();
    let (out_tx, out_rx) = mpsc::channel::<Frame>(WRITE_BUF);
    let writer = tokio::spawn(write_loop(sink, out_rx));
    let routes: Routes = Arc::new(Mutex::new(HashMap::new()));

    while let Some(msg) = stream.next().await {
        match msg {
            Ok(Message::Text(t)) => dispatch(&t, &h, &out_tx, &routes).await,
            Ok(Message::Close(_)) | Err(_) => break,
            _ => continue,
        }
    }

    drop(out_tx);
    let _ = writer.await;
}

async fn write_loop(mut sink: SplitSink<WebSocket, Message>, mut out_rx: mpsc::Receiver<Frame>) {
    while let Some(frame) = out_rx.recv().await {
        let json = match serde_json::to_string(&frame) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(error = %e, "frame serialize failed");
                continue;
            }
        };
        if sink.send(Message::Text(json)).await.is_err() {
            break;
        }
    }
    let _ = sink.close().await;
}

async fn dispatch(text: &str, h: &Shared, out: &mpsc::Sender<Frame>, routes: &Routes) {
    match parse_inbound(text) {
        Ok(Inbound::Request { id, op }) => start_request(id, op, h, out, routes),
        Ok(Inbound::Stream { id, chunk }) => forward_to_route(id, chunk, routes).await,
        Err(msg) => {
            let _ = out.send(err_reply(0, msg)).await;
        }
    }
}

fn start_request(id: u64, op: Op, h: &Shared, out: &mpsc::Sender<Frame>, routes: &Routes) {
    let (in_tx, in_rx) = mpsc::channel::<StreamChunk>(INPUT_BUF);
    routes.lock().unwrap().insert(id, in_tx);
    let h = h.clone();
    let out = out.clone();
    let routes = routes.clone();
    tokio::spawn(async move {
        handle_request(id, op, h, in_rx, out).await;
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
        Frame::Response { .. } => Err("unexpected response from client".into()),
    }
}

fn err_reply(id: u64, message: String) -> Frame {
    Frame::Response {
        id,
        result: OpResult::Err { message },
    }
}

fn stream_frame(id: u64, chunk: StreamChunk) -> Frame {
    Frame::Stream { id, chunk }
}

fn end_err(msg: &str) -> StreamChunk {
    StreamChunk::End {
        ok: false,
        err: Some(msg.into()),
    }
}

async fn handle_request(
    id: u64,
    op: Op,
    h: Shared,
    in_rx: mpsc::Receiver<StreamChunk>,
    out: mpsc::Sender<Frame>,
) {
    match h.handle(op, in_rx).await {
        HandlerOutput::Unary(result) => {
            let _ = out.send(Frame::Response { id, result }).await;
        }
        HandlerOutput::Stream(source) => {
            let _ = out
                .send(Frame::Response {
                    id,
                    result: OpResult::StreamStarted,
                })
                .await;
            pump_stream(id, source, out).await;
        }
    }
}

async fn pump_stream(id: u64, source: BoxStream<'static, StreamChunk>, out: mpsc::Sender<Frame>) {
    let (b_tx, mut b_rx) = broadcast::channel::<StreamChunk>(STREAM_BUF);
    let producer = tokio::spawn(forward_to_broadcast(source, b_tx));
    loop {
        let chunk = match b_rx.recv().await {
            Ok(c) => c,
            Err(broadcast::error::RecvError::Lagged(n)) => {
                StreamChunk::Lagging { dropped: n as u32 }
            }
            Err(broadcast::error::RecvError::Closed) => {
                let _ = out
                    .send(stream_frame(id, end_err("source ended without End chunk")))
                    .await;
                break;
            }
        };
        let is_end = matches!(chunk, StreamChunk::End { .. });
        if out.send(stream_frame(id, chunk)).await.is_err() {
            break;
        }
        if is_end {
            break;
        }
    }
    producer.abort();
}

async fn forward_to_broadcast(
    mut source: BoxStream<'static, StreamChunk>,
    tx: broadcast::Sender<StreamChunk>,
) {
    while let Some(chunk) = source.next().await {
        if tx.send(chunk).is_err() {
            break;
        }
    }
}
