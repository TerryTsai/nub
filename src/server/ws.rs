use crate::config::TrustEntry;
use crate::ops::OpHandler;

use super::wire;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use axum::Extension;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;

const WS_BUF: usize = 64;

type Shared = Arc<dyn OpHandler>;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(h): State<Shared>,
    Extension(caller): Extension<TrustEntry>,
) -> Response {
    // Browser clients offer ["nub", "bearer.<token>"]; we echo back "nub" so
    // the handshake completes. The bearer is consumed by the auth middleware.
    ws.protocols(["nub"])
        .on_upgrade(move |socket| handle_socket(socket, h, caller))
}

async fn handle_socket(socket: WebSocket, h: Shared, caller: TrustEntry) {
    let (sink, stream) = socket.split();
    let (in_tx, in_rx) = mpsc::channel::<String>(WS_BUF);
    let (out_tx, out_rx) = mpsc::channel::<String>(WS_BUF);

    let reader = tokio::spawn(read_loop(stream, in_tx));
    let writer = tokio::spawn(write_loop(sink, out_rx));

    wire::serve(h, caller, in_rx, out_tx).await;

    let _ = reader.await;
    let _ = writer.await;
}

async fn read_loop(mut stream: SplitStream<WebSocket>, in_tx: mpsc::Sender<String>) {
    while let Some(msg) = stream.next().await {
        match msg {
            Ok(Message::Text(t)) => {
                if in_tx.send(t).await.is_err() {
                    break;
                }
            }
            Ok(Message::Close(_)) | Err(_) => break,
            _ => continue,
        }
    }
}

async fn write_loop(mut sink: SplitSink<WebSocket, Message>, mut out_rx: mpsc::Receiver<String>) {
    while let Some(s) = out_rx.recv().await {
        if sink.send(Message::Text(s)).await.is_err() {
            break;
        }
    }
    let _ = sink.close().await;
}
