use crate::proto::{OpResult, StreamChunk};
use anyhow::Result;
use futures::stream::BoxStream;
use std::future::Future;
use tokio::sync::mpsc;

use super::HandlerOutput;

pub(super) fn unary<T>(r: Result<T>, into: impl FnOnce(T) -> OpResult) -> HandlerOutput {
    HandlerOutput::Unary(match r {
        Ok(v) => into(v),
        Err(e) => OpResult::Err { message: e.to_string() },
    })
}

pub(super) fn stream(s: BoxStream<'static, StreamChunk>) -> HandlerOutput {
    HandlerOutput::Stream(s)
}

pub(super) fn ok(_: ()) -> OpResult {
    OpResult::Ok
}

/// Spawns `produce` on a task with a Sender for emitting chunks. When `produce`
/// returns, an `End { ok, err }` chunk is appended automatically. Returns the
/// receiving end as a BoxStream<'static>.
pub(super) fn spawn_chunked<F, Fut>(produce: F) -> BoxStream<'static, StreamChunk>
where
    F: FnOnce(mpsc::Sender<StreamChunk>) -> Fut + Send + 'static,
    Fut: Future<Output = std::result::Result<(), String>> + Send + 'static,
{
    let (tx, rx) = mpsc::channel::<StreamChunk>(32);
    let inner = tx.clone();
    tokio::spawn(async move {
        let (ok, err) = match produce(inner).await {
            Ok(()) => (true, None),
            Err(e) => (false, Some(e)),
        };
        let _ = tx.send(StreamChunk::End { ok, err }).await;
    });
    Box::pin(futures::stream::unfold(rx, |mut rx| async move {
        rx.recv().await.map(|c| (c, rx))
    }))
}

pub(super) fn short_id(id: &str) -> String {
    id.strip_prefix("sha256:").unwrap_or(id).chars().take(12).collect()
}

pub(super) fn log_chunk(stderr: bool, msg: &[u8]) -> StreamChunk {
    StreamChunk::Log {
        stderr,
        data: String::from_utf8_lossy(msg).into_owned(),
    }
}
