//! Request builder for typical engine ops: GET / POST / DELETE with optional
//! JSON body and a few extra headers (host, connection, upgrade).

use bytes::Bytes;
use hyper::header::{HeaderValue, CONNECTION, CONTENT_TYPE, HOST, UPGRADE};
use hyper::{Method, Request};
use serde::Serialize;

use super::conn::Body;
use super::Error;

pub(crate) struct Req {
    method: Method,
    path: String,
    body: Body,
    content_type: Option<&'static str>,
    upgrade: Option<&'static str>,
}

impl Req {
    pub(crate) fn get(path: impl Into<String>) -> Self {
        Self::new(Method::GET, path)
    }

    pub(crate) fn post(path: impl Into<String>) -> Self {
        Self::new(Method::POST, path)
    }

    pub(crate) fn delete(path: impl Into<String>) -> Self {
        Self::new(Method::DELETE, path)
    }

    fn new(method: Method, path: impl Into<String>) -> Self {
        Self {
            method,
            path: path.into(),
            body: Body::Empty,
            content_type: None,
            upgrade: None,
        }
    }

    pub(crate) fn json<T: Serialize>(mut self, value: &T) -> Result<Self, Error> {
        let bytes = serde_json::to_vec(value).map_err(|e| Error::Decode(format!("{e}")))?;
        self.body = Body::Bytes(Bytes::from(bytes));
        self.content_type = Some("application/json");
        Ok(self)
    }

    pub(crate) fn bytes(mut self, content_type: &'static str, body: Bytes) -> Self {
        self.body = Body::Bytes(body);
        self.content_type = Some(content_type);
        self
    }

    /// Mark as an HTTP upgrade. Used by exec to flip the connection into a
    /// raw bidirectional byte stream after a 101 response. Switches the
    /// `Connection` header from `close` to `upgrade` (servers reject both).
    pub(crate) fn upgrade(mut self, protocol: &'static str) -> Self {
        self.upgrade = Some(protocol);
        self
    }

    pub(crate) fn build(self) -> Result<Request<Body>, Error> {
        let mut builder = Request::builder().method(self.method).uri(self.path).header(HOST, "localhost");
        if let Some(protocol) = self.upgrade {
            builder = builder.header(CONNECTION, "upgrade").header(UPGRADE, HeaderValue::from_static(protocol));
        } else {
            // Per-call sockets — close the connection after the response.
            builder = builder.header(CONNECTION, "close");
        }
        if let Some(ct) = self.content_type {
            builder = builder.header(CONTENT_TYPE, ct);
        }
        builder.body(self.body).map_err(|e| Error::Decode(format!("{e}")))
    }
}
