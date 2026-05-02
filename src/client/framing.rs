//! Streaming primitives shared by ops/* — line splitting (NDJSON / docker
//! pull progress) and the docker 8-byte multiplexed log/exec frame parser.

use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::{Buf, Bytes, BytesMut};
use futures::stream::Stream;
use http_body::Body as _;
use hyper::body::Incoming;

use super::Error;

// ---- LineStream: split an Incoming body on '\n' --------------------------

/// Yields one byte slice per line (without the trailing `\n`). At EOF, any
/// trailing partial line is yielded last.
pub struct LineStream {
    body: Incoming,
    buf: BytesMut,
    eof: bool,
}

impl LineStream {
    pub fn new(body: Incoming) -> Self {
        Self { body, buf: BytesMut::with_capacity(4096), eof: false }
    }
}

impl Stream for LineStream {
    type Item = Result<Bytes, Error>;

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

    fn poll_body(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
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

// ---- Multiplexer: docker's 8-byte log/exec frame format ------------------
//
// Frame: byte 0 = stream type (1=stdout, 2=stderr), bytes 1-3 = padding,
// bytes 4-7 = BE u32 payload length, then payload bytes.
//
// TTY containers don't use multiplexing — the bytes are raw terminal output.
// Logs auto-detect via the first byte (TTY content rarely starts with 0/1/2);
// exec knows up front from its `tty` flag.

#[derive(Debug, Clone)]
pub struct MuxFrame {
    pub stderr: bool,
    pub data: Bytes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultiplexerMode {
    /// Sniff the first byte: 0..=2 means multiplexed, anything else is raw.
    Detect,
    /// Always treat input as raw TTY bytes.
    Tty,
    /// Always parse 8-byte headers.
    Multiplexed,
}

/// Stateful frame parser. Push bytes in, pull frames out. EOF produces any
/// trailing partial-frame bytes as a single best-effort stdout chunk.
pub struct Multiplexer {
    buf: BytesMut,
    mode: MultiplexerMode,
}

impl Multiplexer {
    pub fn new(mode: MultiplexerMode) -> Self {
        Self { buf: BytesMut::with_capacity(8192), mode }
    }

    pub fn push(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    pub fn next_frame(&mut self) -> Option<MuxFrame> {
        if matches!(self.mode, MultiplexerMode::Detect) && !self.buf.is_empty() {
            self.mode = match self.buf[0] {
                0..=2 => MultiplexerMode::Multiplexed,
                _ => MultiplexerMode::Tty,
            };
        }
        match self.mode {
            MultiplexerMode::Multiplexed => self.take_multiplexed_frame(),
            MultiplexerMode::Tty => self.take_raw_frame(),
            MultiplexerMode::Detect => None, // need more bytes to decide
        }
    }

    /// Emit any leftover bytes after the source has ended.
    pub fn finish(mut self) -> Option<MuxFrame> {
        (!self.buf.is_empty()).then(|| MuxFrame {
            stderr: false,
            data: std::mem::take(&mut self.buf).freeze(),
        })
    }

    fn take_multiplexed_frame(&mut self) -> Option<MuxFrame> {
        if self.buf.len() < 8 {
            return None;
        }
        let stream_byte = self.buf[0];
        let len = u32::from_be_bytes([self.buf[4], self.buf[5], self.buf[6], self.buf[7]]) as usize;
        if self.buf.len() < 8 + len {
            return None;
        }
        self.buf.advance(8);
        Some(MuxFrame {
            stderr: stream_byte == 2,
            data: self.buf.split_to(len).freeze(),
        })
    }

    fn take_raw_frame(&mut self) -> Option<MuxFrame> {
        (!self.buf.is_empty()).then(|| MuxFrame {
            stderr: false,
            data: std::mem::take(&mut self.buf).freeze(),
        })
    }
}

// ---- short_id helper -----------------------------------------------------

/// Trim docker/podman ids to the conventional 12-char short form, stripping
/// any `sha256:` prefix.
pub fn short_id(id: &str) -> String {
    id.strip_prefix("sha256:").unwrap_or(id).chars().take(12).collect()
}
