use crate::handler::{HandlerOutput, OpHandler};
use crate::proto::*;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use futures::stream::BoxStream;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

const WRITE_BUF: usize = 64;
const STREAM_BUF: usize = 64;

type Shared = Arc<dyn OpHandler>;

pub async fn ws_handler(ws: WebSocketUpgrade, State(h): State<Shared>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, h))
}

async fn handle_socket(socket: WebSocket, h: Shared) {
    let (mut sink, mut stream) = socket.split();
    let (out_tx, mut out_rx) = mpsc::channel::<Frame>(WRITE_BUF);

    let writer = tokio::spawn(async move {
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
    });

    while let Some(msg) = stream.next().await {
        let text = match msg {
            Ok(Message::Text(t)) => t,
            Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => continue,
            Ok(Message::Binary(_)) => continue,
            Ok(Message::Close(_)) | Err(_) => break,
        };
        let frame: Frame = match serde_json::from_str(&text) {
            Ok(f) => f,
            Err(e) => {
                let _ = out_tx
                    .send(Frame::Response {
                        id: 0,
                        result: OpResult::Err {
                            message: format!("bad frame: {e}"),
                        },
                    })
                    .await;
                continue;
            }
        };
        let (id, op) = match frame {
            Frame::Request { id, op } => (id, op),
            _ => {
                let _ = out_tx
                    .send(Frame::Response {
                        id: 0,
                        result: OpResult::Err {
                            message: "expected request frame".into(),
                        },
                    })
                    .await;
                continue;
            }
        };
        tokio::spawn(handle_request(id, op, h.clone(), out_tx.clone()));
    }

    drop(out_tx);
    let _ = writer.await;
}

async fn handle_request(id: u64, op: Op, h: Shared, out: mpsc::Sender<Frame>) {
    match h.handle(op).await {
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
    let producer = tokio::spawn(async move {
        let mut source = source;
        while let Some(chunk) = source.next().await {
            if b_tx.send(chunk).is_err() {
                break;
            }
        }
    });

    loop {
        match b_rx.recv().await {
            Ok(chunk) => {
                let is_end = matches!(chunk, StreamChunk::End { .. });
                if out.send(Frame::Stream { id, chunk }).await.is_err() {
                    break;
                }
                if is_end {
                    break;
                }
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                let _ = out
                    .send(Frame::Stream {
                        id,
                        chunk: StreamChunk::Lagging { dropped: n as u32 },
                    })
                    .await;
            }
            Err(broadcast::error::RecvError::Closed) => {
                let _ = out
                    .send(Frame::Stream {
                        id,
                        chunk: StreamChunk::End {
                            ok: false,
                            err: Some("source ended without End chunk".into()),
                        },
                    })
                    .await;
                break;
            }
        }
    }
    producer.abort();
}
