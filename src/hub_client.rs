use crate::handler::OpHandler;
use crate::wire;
use anyhow::{anyhow, Context, Result};
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub url: String,
    pub node_token: String,
}

const RECONNECT_INITIAL: Duration = Duration::from_secs(1);
const RECONNECT_MAX: Duration = Duration::from_secs(30);
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const READ_TIMEOUT: Duration = Duration::from_secs(90);
const WS_BUF: usize = 64;

type Ws = WebSocketStream<MaybeTlsStream<TcpStream>>;
type Sink = SplitSink<Ws, Message>;
type Stream = SplitStream<Ws>;

pub async fn run(handler: Arc<dyn OpHandler>, cfg: Config) {
    let mut backoff = RECONNECT_INITIAL;
    loop {
        let result = connect_and_serve(&handler, &cfg).await;
        log_attempt(&result);
        backoff = next_backoff(&result, backoff);
        tracing::info!(?backoff, "hub: reconnecting");
        tokio::time::sleep(backoff).await;
    }
}

// tracing's macros expand to enough code that even a trivial 2-arm match
// crosses the cognitive_complexity threshold.
#[allow(clippy::cognitive_complexity)]
fn log_attempt(result: &Result<()>) {
    match result {
        Ok(()) => tracing::info!("hub connection closed cleanly"),
        Err(e) => tracing::warn!("hub connection failed: {e}"),
    }
}

fn next_backoff(result: &Result<()>, current: Duration) -> Duration {
    match result {
        Ok(()) => RECONNECT_INITIAL,
        Err(_) => (current * 2).min(RECONNECT_MAX),
    }
}

async fn connect_and_serve(handler: &Arc<dyn OpHandler>, cfg: &Config) -> Result<()> {
    let mut req = cfg.url.as_str().into_client_request().context("invalid hub URL")?;
    let bearer = format!("Bearer {}", cfg.node_token)
        .parse()
        .map_err(|_| anyhow!("token contains invalid header bytes"))?;
    req.headers_mut().insert("Authorization", bearer);

    let (ws, _resp) = connect_async(req).await.context("connect failed")?;
    tracing::info!(url = %cfg.url, "hub connected");
    let (sink, stream) = ws.split();

    let (in_tx, in_rx) = mpsc::channel::<String>(WS_BUF);
    let (out_tx, out_rx) = mpsc::channel::<String>(WS_BUF);

    let reader = tokio::spawn(read_loop(stream, in_tx));
    let writer = tokio::spawn(write_loop(sink, out_rx));

    wire::serve(handler.clone(), in_rx, out_tx).await;

    let _ = reader.await;
    let _ = writer.await;
    Ok(())
}

async fn read_loop(mut stream: Stream, in_tx: mpsc::Sender<String>) {
    loop {
        let next = tokio::time::timeout(READ_TIMEOUT, stream.next()).await;
        let item = match next {
            Ok(Some(item)) => item,
            Ok(None) => return,
            Err(_) => {
                tracing::warn!("hub read timeout (no traffic for {READ_TIMEOUT:?})");
                return;
            }
        };
        match item {
            Ok(Message::Text(t)) => {
                if in_tx.send(t).await.is_err() {
                    return;
                }
            }
            Ok(Message::Close(_)) | Err(_) => return,
            _ => continue,
        }
    }
}

async fn write_loop(mut sink: Sink, mut out_rx: mpsc::Receiver<String>) {
    let mut ping = tokio::time::interval(HEARTBEAT_INTERVAL);
    ping.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    ping.tick().await;
    loop {
        tokio::select! {
            msg = out_rx.recv() => {
                let Some(s) = msg else { break };
                if sink.send(Message::Text(s)).await.is_err() {
                    break;
                }
            }
            _ = ping.tick() => {
                if sink.send(Message::Ping(Vec::new())).await.is_err() {
                    break;
                }
            }
        }
    }
    let _ = sink.close().await;
}
