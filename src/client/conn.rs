//! Connection lifecycle: socket open, HTTP/1 handshake, request/response,
//! upgrade hand-off. No pooling — every call opens a fresh connection.

use std::io;
use std::path::PathBuf;

use bytes::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper::body::Incoming;
use hyper::client::conn::http1;
use hyper::upgrade::Upgraded;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use serde::de::DeserializeOwned;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpStream, UnixStream};

use super::Error;

#[derive(Debug, Clone)]
pub(crate) enum Address {
    Unix(PathBuf),
    Tcp(String),
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Address::Unix(p) => write!(f, "unix://{}", p.display()),
            Address::Tcp(s) => write!(f, "tcp://{s}"),
        }
    }
}

pub(crate) struct Conn {
    sender: http1::SendRequest<Body>,
}

pub(crate) enum Body {
    Empty,
    Bytes(Bytes),
}

impl http_body::Body for Body {
    type Data = Bytes;
    type Error = std::convert::Infallible;

    fn poll_frame(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        let this = self.get_mut();
        match this {
            Body::Empty => {
                let mut b = Empty::<Bytes>::new();
                std::pin::Pin::new(&mut b).poll_frame(cx)
            }
            Body::Bytes(b) if b.is_empty() => std::task::Poll::Ready(None),
            Body::Bytes(b) => {
                let frame = http_body::Frame::data(std::mem::take(b));
                std::task::Poll::Ready(Some(Ok(frame)))
            }
        }
    }
}

impl Conn {
    pub(crate) async fn connect(addr: &Address) -> Result<Self, Error> {
        match addr {
            Address::Unix(path) => {
                let stream = UnixStream::connect(path).await.map_err(io_err)?;
                handshake(stream).await
            }
            Address::Tcp(host_port) => {
                let stream = TcpStream::connect(host_port).await.map_err(io_err)?;
                handshake(stream).await
            }
        }
    }

    pub(crate) async fn send_unary(&mut self, req: Request<Body>) -> Result<UnaryResponse, Error> {
        let res = self.sender.send_request(req).await.map_err(hyper_err)?;
        let status = res.status();
        let body = res.into_body().collect().await.map_err(hyper_err)?.to_bytes();
        Ok(UnaryResponse { status, body })
    }

    pub(crate) async fn send_streaming(&mut self, req: Request<Body>) -> Result<Response<Incoming>, Error> {
        self.sender.send_request(req).await.map_err(hyper_err)
    }
}

pub(crate) struct UnaryResponse {
    pub(crate) status: StatusCode,
    pub(crate) body: Bytes,
}

impl UnaryResponse {
    pub(crate) fn json<T: DeserializeOwned>(self) -> Result<T, Error> {
        if !self.status.is_success() {
            return Err(self.into_status_error());
        }
        serde_json::from_slice(&self.body).map_err(|e| Error::Decode(format!("{e}")))
    }

    pub(crate) fn ok(self) -> Result<(), Error> {
        if !self.status.is_success() {
            return Err(self.into_status_error());
        }
        Ok(())
    }

    fn into_status_error(self) -> Error {
        let message =
            engine_error_message(&self.body).unwrap_or_else(|| String::from_utf8_lossy(&self.body).into_owned());
        Error::Status {
            code: self.status.as_u16(),
            message,
        }
    }
}

fn engine_error_message(body: &[u8]) -> Option<String> {
    #[derive(serde::Deserialize)]
    struct ErrEnvelope<'a> {
        message: Option<&'a str>,
        cause: Option<&'a str>,
    }
    let env: ErrEnvelope = serde_json::from_slice(body).ok()?;
    env.message.or(env.cause).map(|s| s.to_string())
}

async fn handshake<S>(stream: S) -> Result<Conn, Error>
where
    S: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    let io = TokioIo::new(stream);
    let (sender, conn) = http1::handshake(io).await.map_err(hyper_err)?;
    // with_upgrades() is needed for the exec endpoint, which switches
    // protocols; benign for non-upgrade requests.
    tokio::spawn(async move {
        let _ = conn.with_upgrades().await;
    });
    Ok(Conn { sender })
}

fn io_err(e: io::Error) -> Error {
    Error::Transport(format!("{e}"))
}

fn hyper_err(e: hyper::Error) -> Error {
    Error::Transport(format!("{e}"))
}

/// After a 101 Switching Protocols, hand back the raw bidirectional byte
/// stream. Used by the exec endpoint.
pub(crate) async fn upgrade(res: Response<Incoming>) -> Result<TokioIo<Upgraded>, Error> {
    if res.status() != StatusCode::SWITCHING_PROTOCOLS {
        return Err(Error::Status {
            code: res.status().as_u16(),
            message: format!("expected 101 Switching Protocols, got {}", res.status()),
        });
    }
    let upgraded = hyper::upgrade::on(res).await.map_err(hyper_err)?;
    Ok(TokioIo::new(upgraded))
}
