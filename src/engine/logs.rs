//! Container logs streaming. Both Docker and Podman use the same compat
//! `/containers/{id}/logs` endpoint. Output is either:
//!   - Multiplexed (non-TTY containers): 8-byte header per frame, byte 0 =
//!     stream type, bytes 4-7 = BE length
//!   - Raw (TTY containers): bytes are the terminal output verbatim
//!
//! We sniff the first byte to detect which mode this container is in.

use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::{Buf, Bytes, BytesMut};
use futures::stream::Stream;
use http_body::Body as _;
use http_body_util::BodyExt;
use hyper::body::Incoming;

use super::http::Conn;
use super::{Engine, Error, Query, Req, Result};

#[derive(Debug, Clone, Default)]
pub struct LogsOptions {
    pub follow: bool,
    /// Number of lines from the tail to send up front. None = all.
    pub tail: Option<u32>,
}

/// One chunk of log output. `stderr=false` means stdout. For TTY containers,
/// every chunk is reported as stdout (the engine doesn't separate them).
#[derive(Debug, Clone)]
pub struct LogChunk {
    pub stderr: bool,
    pub data: Bytes,
}

impl Engine {
    pub async fn stream_logs(&self, id: &str, opts: LogsOptions) -> Result<LogStream> {
        let mut q = Query::new();
        q.push_bool("follow", opts.follow);
        q.push_bool("stdout", true);
        q.push_bool("stderr", true);
        let tail = opts.tail.map(|n| n.to_string()).unwrap_or_else(|| "all".into());
        q.push("tail", &tail);
        let path = format!("/containers/{id}/logs{}", q.finish());

        let mut conn = self.conn().await?;
        let res = conn.send_streaming(Req::get(path).build()?).await?;
        if !res.status().is_success() {
            let status = res.status().as_u16();
            let body = res.into_body().collect().await
                .map_err(|e| Error::Transport(format!("{e}")))?
                .to_bytes();
            return Err(Error::Status {
                code: status,
                message: String::from_utf8_lossy(&body).into_owned(),
            });
        }
        Ok(LogStream {
            body: res.into_body(),
            buf: BytesMut::new(),
            mode: Mode::Unknown,
            eof: false,
            _conn: conn,
        })
    }
}

pub struct LogStream {
    body: Incoming,
    buf: BytesMut,
    mode: Mode,
    eof: bool,
    _conn: Conn,
}

enum Mode {
    Unknown,
    Multiplexed,
    Tty,
}

impl Stream for LogStream {
    type Item = Result<LogChunk>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            self.detect_mode();
            if let Some(chunk) = self.take_frame() {
                return Poll::Ready(Some(Ok(chunk)));
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

impl LogStream {
    fn detect_mode(&mut self) {
        if matches!(self.mode, Mode::Unknown) && !self.buf.is_empty() {
            self.mode = match self.buf[0] {
                0..=2 => Mode::Multiplexed,
                _ => Mode::Tty,
            };
        }
    }

    fn take_frame(&mut self) -> Option<LogChunk> {
        match self.mode {
            Mode::Multiplexed => self.take_multiplexed_frame(),
            Mode::Tty => (!self.buf.is_empty()).then(|| LogChunk {
                stderr: false,
                data: std::mem::take(&mut self.buf).freeze(),
            }),
            Mode::Unknown => None,
        }
    }

    fn take_multiplexed_frame(&mut self) -> Option<LogChunk> {
        if self.buf.len() < 8 {
            return None;
        }
        let stream_byte = self.buf[0];
        let len = u32::from_be_bytes([self.buf[4], self.buf[5], self.buf[6], self.buf[7]]) as usize;
        if self.buf.len() < 8 + len {
            return None;
        }
        self.buf.advance(8);
        let payload = self.buf.split_to(len).freeze();
        Some(LogChunk { stderr: stream_byte == 2, data: payload })
    }

    fn flush_eof(&mut self) -> Option<LogChunk> {
        if self.buf.is_empty() {
            return None;
        }
        Some(LogChunk { stderr: false, data: std::mem::take(&mut self.buf).freeze() })
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
