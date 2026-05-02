//! Container exec. Two-step protocol: create the exec instance, then "start"
//! it with an HTTP upgrade to a bidirectional byte stream. Output is the
//! same multiplexed format as logs (8-byte headers in non-TTY mode, raw
//! bytes in TTY mode); we expose a reader stream + writer pair so the
//! caller can pump stdin and read stdout/stderr concurrently without locks.

use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::{Buf, Bytes, BytesMut};
use futures::stream::Stream;
use hyper::header::HeaderValue;
use hyper_util::rt::TokioIo;
use hyper::upgrade::Upgraded;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWriteExt, ReadBuf, ReadHalf, WriteHalf};

use super::http;
use super::{Engine, Req, Result};

#[derive(Debug, Clone)]
pub struct ExecOptions {
    pub cmd: Vec<String>,
    pub tty: bool,
}

/// One direction of the exec bidirectional stream.
#[derive(Debug, Clone)]
pub enum ExecOutput {
    Stdout(Bytes),
    Stderr(Bytes),
}

/// Reader half: a Stream of stdout/stderr frames.
pub struct ExecReader {
    reader: ReadHalf<TokioIo<Upgraded>>,
    buf: BytesMut,
    mode: Mode,
    eof: bool,
}

/// Writer half: send to the exec process's stdin.
pub struct ExecWriter {
    writer: WriteHalf<TokioIo<Upgraded>>,
}

enum Mode {
    Multiplexed,
    Tty,
}

/// Combined return type so callers receive both halves from one call.
pub struct ExecStream {
    pub reader: ExecReader,
    pub writer: ExecWriter,
}

impl Engine {
    pub async fn exec(&self, container_id: &str, opts: ExecOptions) -> Result<ExecStream> {
        // Step 1: create the exec instance.
        let mut conn = self.conn().await?;
        let create_body = CreateExecBody {
            attach_stdin: true,
            attach_stdout: true,
            attach_stderr: true,
            tty: opts.tty,
            cmd: opts.cmd,
        };
        let resp: CreateExecResp = conn
            .send_unary(Req::post(format!("/containers/{container_id}/exec")).json(&create_body)?.build()?)
            .await?
            .json()?;

        // Step 2: start with HTTP upgrade. Needs a fresh connection because
        // the upgrade hijacks it.
        let mut conn2 = self.conn().await?;
        let start_body = StartExecBody { detach: false, tty: opts.tty };
        let req = Req::post(format!("/exec/{}/start", resp.id))
            .json(&start_body)?
            .upgrade("tcp")
            .header(hyper::header::HOST, HeaderValue::from_static("localhost"))
            .build()?;
        let res = conn2.send_streaming(req).await?;
        let upgraded = http::upgrade(res).await?;
        let (reader, writer) = tokio::io::split(upgraded);

        Ok(ExecStream {
            reader: ExecReader {
                reader,
                buf: BytesMut::with_capacity(8192),
                mode: if opts.tty { Mode::Tty } else { Mode::Multiplexed },
                eof: false,
            },
            writer: ExecWriter { writer },
        })
    }
}

impl ExecWriter {
    pub async fn write_stdin(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(data).await
    }

    pub async fn close_stdin(&mut self) -> std::io::Result<()> {
        self.writer.shutdown().await
    }
}

impl Stream for ExecReader {
    type Item = Result<ExecOutput>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            if let Some(out) = self.take_frame() {
                return Poll::Ready(Some(Ok(out)));
            }
            if self.eof {
                return Poll::Ready(self.flush_eof().map(Ok));
            }
            match futures::ready!(self.as_mut().poll_read_into_buf(cx)) {
                Err(e) => return Poll::Ready(Some(Err(e))),
                Ok(()) => continue,
            }
        }
    }
}

impl ExecReader {
    /// If the buffer contains a complete frame, take it.
    fn take_frame(&mut self) -> Option<ExecOutput> {
        match self.mode {
            Mode::Multiplexed => self.take_multiplexed_frame(),
            Mode::Tty => (!self.buf.is_empty()).then(|| {
                let payload = std::mem::take(&mut self.buf).freeze();
                ExecOutput::Stdout(payload)
            }),
        }
    }

    fn take_multiplexed_frame(&mut self) -> Option<ExecOutput> {
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
        Some(if stream_byte == 2 { ExecOutput::Stderr(payload) } else { ExecOutput::Stdout(payload) })
    }

    /// At EOF, emit any partial-frame bytes as best-effort stdout, otherwise
    /// signal end-of-stream.
    fn flush_eof(&mut self) -> Option<ExecOutput> {
        if self.buf.is_empty() {
            return None;
        }
        Some(ExecOutput::Stdout(std::mem::take(&mut self.buf).freeze()))
    }

    /// Poll a chunk from the underlying connection into the buffer. Sets
    /// `eof` if the read returns 0 bytes.
    fn poll_read_into_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        let this = self.get_mut();
        let mut tmp = [0u8; 4096];
        let mut rb = ReadBuf::new(&mut tmp);
        match Pin::new(&mut this.reader).poll_read(cx, &mut rb) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(super::Error::Transport(format!("{e}")))),
            Poll::Ready(Ok(())) => {
                let n = rb.filled().len();
                if n == 0 {
                    this.eof = true;
                } else {
                    this.buf.extend_from_slice(&tmp[..n]);
                }
                Poll::Ready(Ok(()))
            }
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct CreateExecBody {
    attach_stdin: bool,
    attach_stdout: bool,
    attach_stderr: bool,
    tty: bool,
    cmd: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct CreateExecResp {
    #[serde(rename = "Id")]
    id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct StartExecBody {
    detach: bool,
    tty: bool,
}
