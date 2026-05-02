//! Images: list, remove, pull (streaming).

use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::{Buf, Bytes, BytesMut};
use futures::stream::{Stream, StreamExt};
use http_body::Body as _;
use http_body_util::BodyExt;
use hyper::body::Incoming;
use serde::Deserialize;

use super::http::Conn;
use super::{Engine, Error, Query, Req, Result};

#[derive(Debug, Clone)]
pub struct ImageSummary {
    pub id: String,
    pub repo_tag: String,
    pub created: i64,
    pub size: i64,
    pub containers: i64,
}

#[derive(Debug, Clone)]
pub struct PullProgress {
    pub layer_id: String,
    pub status: String,
    pub current: u64,
    pub total: u64,
}

impl Engine {
    pub async fn list_images(&self) -> Result<Vec<ImageSummary>> {
        // Compat path works on both engines.
        let mut conn = self.conn().await?;
        let raw: Vec<RawImage> = conn.send_unary(Req::get("/images/json").build()?).await?.json()?;
        Ok(raw.into_iter().map(RawImage::into_summary).collect())
    }

    pub async fn remove_image(&self, id: &str, force: bool) -> Result<()> {
        let mut q = Query::new();
        q.push_bool("force", force);
        let path = format!("/images/{id}{}", q.finish());
        let mut conn = self.conn().await?;
        conn.send_unary(Req::delete(path).build()?).await?.ok()
    }

    /// Stream pull progress events. Returns when the pull finishes (or errors).
    /// Each chunk is one progress update from the engine.
    pub async fn pull_image(&self, reference: &str) -> Result<PullStream> {
        let mut q = Query::new();
        q.push("fromImage", reference);
        let path = format!("/images/create{}", q.finish());
        let mut conn = self.conn().await?;
        let res = conn.send_streaming(Req::post(path).build()?).await?;
        if !res.status().is_success() {
            // Drain status-error body for a useful message.
            let status = res.status().as_u16();
            let body = res.into_body().collect().await
                .map_err(|e| Error::Transport(format!("{e}")))?
                .to_bytes();
            return Err(Error::Status {
                code: status,
                message: String::from_utf8_lossy(&body).into_owned(),
            });
        }
        Ok(PullStream {
            inner: LineStream::new(res.into_body()),
            _conn: conn,
        })
    }
}

/// Stream of typed pull progress events. Holds the underlying connection so
/// it stays open until the stream finishes.
pub struct PullStream {
    inner: LineStream,
    /// Owned to keep the http connection alive for the duration of the stream.
    _conn: Conn,
}

impl Stream for PullStream {
    type Item = Result<PullProgress>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            let line = match futures::ready!(self.inner.poll_next_unpin(cx)) {
                None => return Poll::Ready(None),
                Some(Err(e)) => return Poll::Ready(Some(Err(e))),
                Some(Ok(l)) => l,
            };
            if line.iter().all(u8::is_ascii_whitespace) {
                continue;
            }
            return Poll::Ready(Some(parse_pull_line(&line)));
        }
    }
}

fn parse_pull_line(line: &[u8]) -> Result<PullProgress> {
    let info: CreateImageInfo = serde_json::from_slice(line).map_err(|e| Error::Decode(format!("{e}")))?;
    if let Some(err) = info.error {
        return Err(Error::Status { code: 0, message: err });
    }
    let (current, total) = info
        .progress_detail
        .map(|d| (d.current, d.total))
        .unwrap_or((0, 0));
    Ok(PullProgress {
        layer_id: info.id.unwrap_or_default(),
        status: info.status.unwrap_or_default(),
        current,
        total,
    })
}

#[derive(Deserialize)]
struct CreateImageInfo {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default, rename = "error")]
    error: Option<String>,
    #[serde(default, rename = "progressDetail")]
    progress_detail: Option<ProgressDetail>,
}

#[derive(Deserialize)]
struct ProgressDetail {
    #[serde(default)]
    current: u64,
    #[serde(default)]
    total: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawImage {
    #[serde(rename = "Id")]
    id: String,
    #[serde(default)]
    repo_tags: Option<Vec<String>>,
    #[serde(default)]
    created: i64,
    #[serde(default)]
    size: i64,
    #[serde(default)]
    containers: i64,
}

impl RawImage {
    fn into_summary(self) -> ImageSummary {
        ImageSummary {
            id: short_id(&self.id),
            repo_tag: self
                .repo_tags
                .and_then(|t| t.into_iter().next())
                .unwrap_or_else(|| "<none>".into()),
            created: self.created,
            size: self.size,
            containers: self.containers,
        }
    }
}

fn short_id(id: &str) -> String {
    id.strip_prefix("sha256:").unwrap_or(id).chars().take(12).collect()
}

// ---- Line-delimited frame stream -------------------------------------------

/// Adapts a hyper `Incoming` body into a stream of newline-delimited byte
/// blobs. Buffers until a `\n` arrives, then yields each line (without the
/// trailing newline).
pub(crate) struct LineStream {
    body: Incoming,
    buf: BytesMut,
    eof: bool,
}

impl LineStream {
    pub(crate) fn new(body: Incoming) -> Self {
        Self {
            body,
            buf: BytesMut::with_capacity(4096),
            eof: false,
        }
    }
}

impl Stream for LineStream {
    type Item = Result<Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            if let Some(line) = self.take_line() {
                return Poll::Ready(Some(Ok(line)));
            }
            if self.eof {
                return Poll::Ready(self.flush_eof().map(Ok));
            }
            match futures::ready!(self.as_mut().poll_body(cx)) {
                Err(e) => return Poll::Ready(Some(Err(e))),
                Ok(()) => continue,
            }
        }
    }
}

impl LineStream {
    fn take_line(&mut self) -> Option<Bytes> {
        let pos = self.buf.iter().position(|&b| b == b'\n')?;
        let line = self.buf.split_to(pos).freeze();
        self.buf.advance(1);
        Some(line)
    }

    fn flush_eof(&mut self) -> Option<Bytes> {
        (!self.buf.is_empty()).then(|| std::mem::take(&mut self.buf).freeze())
    }

    fn poll_body(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        let this = self.get_mut();
        match Pin::new(&mut this.body).poll_frame(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => {
                this.eof = true;
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Err(Error::Transport(format!("{e}")))),
            Poll::Ready(Some(Ok(frame))) => {
                if let Ok(data) = frame.into_data() {
                    this.buf.extend_from_slice(&data);
                }
                Poll::Ready(Ok(()))
            }
        }
    }
}
