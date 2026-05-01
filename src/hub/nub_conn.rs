use crate::proto::{Frame, OpResult};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use super::{NubConn, NubEntry};

const WS_BUF: usize = 64;

pub(super) async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<super::State>,
    headers: HeaderMap,
) -> Response {
    let Some(nub_id) = identify(&headers, &state.nubs) else {
        return (StatusCode::UNAUTHORIZED, "unknown nub").into_response();
    };
    ws.on_upgrade(move |socket| handle_socket(socket, nub_id, state))
}

fn identify(headers: &HeaderMap, nubs: &[NubEntry]) -> Option<String> {
    let token = headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")?;
    nubs.iter()
        .find(|n| ct_eq(n.token.as_bytes(), token.as_bytes()))
        .map(|n| n.id.clone())
}

fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

async fn handle_socket(socket: WebSocket, nub_id: String, state: super::State) {
    let (sink, stream) = socket.split();
    let (out_tx, out_rx) = mpsc::channel::<String>(WS_BUF);
    let conn = Arc::new(NubConn {
        out_tx,
        pending: Mutex::new(HashMap::new()),
        next_id: AtomicU64::new(1),
    });
    register(&state, &nub_id, conn.clone());
    let writer = tokio::spawn(write_loop(sink, out_rx));
    read_loop(stream, conn.clone()).await;
    unregister(&state, &nub_id);
    conn.pending.lock().unwrap().clear();
    let _ = writer.await;
}

fn register(state: &super::State, id: &str, conn: Arc<NubConn>) {
    state.registry.lock().unwrap().insert(id.to_string(), conn);
    tracing::info!("nub {id} connected");
}

fn unregister(state: &super::State, id: &str) {
    state.registry.lock().unwrap().remove(id);
    tracing::info!("nub {id} disconnected");
}

async fn read_loop(mut stream: SplitStream<WebSocket>, conn: Arc<NubConn>) {
    while let Some(msg) = stream.next().await {
        match msg {
            Ok(Message::Text(t)) => route_frame(&t, &conn),
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

fn route_frame(text: &str, conn: &NubConn) {
    let Some(frame) = parse(text) else { return };
    if let Frame::Response { id, result } = frame {
        deliver(conn, id, result);
    }
    // Stream from nub: streaming proxy not implemented in this slice; drop.
    // Request from nub: shouldn't happen; ignored.
}

fn parse(text: &str) -> Option<Frame> {
    match serde_json::from_str(text) {
        Ok(f) => Some(f),
        Err(e) => {
            tracing::warn!("bad frame from nub: {e}");
            None
        }
    }
}

fn deliver(conn: &NubConn, id: u64, result: OpResult) {
    if let Some(tx) = conn.pending.lock().unwrap().remove(&id) {
        let _ = tx.send(result);
    }
}
