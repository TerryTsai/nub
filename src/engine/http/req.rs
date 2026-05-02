//! Request builder for typical engine ops: GET / POST / DELETE with optional
//! JSON body and a few extra headers (host, connection, upgrade).

use bytes::Bytes;
use hyper::header::{HeaderName, HeaderValue, CONNECTION, CONTENT_TYPE, HOST, UPGRADE};
use hyper::{Method, Request};
use serde::Serialize;

use super::conn::Body;
use crate::engine::Error;

pub(crate) struct Req {
    method: Method,
    path: String,
    body: Body,
    content_type: Option<&'static str>,
    extra_headers: Vec<(HeaderName, HeaderValue)>,
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
            extra_headers: Vec::new(),
        }
    }

    pub(crate) fn json<T: Serialize>(mut self, value: &T) -> Result<Self, Error> {
        let bytes = serde_json::to_vec(value).map_err(|e| Error::Decode(format!("{e}")))?;
        self.body = Body::Bytes(Bytes::from(bytes));
        self.content_type = Some("application/json");
        Ok(self)
    }

    pub(crate) fn header(mut self, name: HeaderName, value: HeaderValue) -> Self {
        self.extra_headers.push((name, value));
        self
    }

    /// Mark as an HTTP upgrade. Used by exec to flip the connection into a
    /// raw bidirectional byte stream after a 101 response.
    pub(crate) fn upgrade(self, protocol: &'static str) -> Self {
        self.header(UPGRADE, HeaderValue::from_static(protocol))
            .header(CONNECTION, HeaderValue::from_static("upgrade"))
    }

    pub(crate) fn build(self) -> Result<Request<Body>, Error> {
        let mut builder = Request::builder().method(self.method).uri(self.path);
        builder = builder.header(HOST, "localhost");
        // Per-stream connections, no keepalive — we open a fresh socket for
        // every op anyway.
        builder = builder.header(CONNECTION, "close");
        if let Some(ct) = self.content_type {
            builder = builder.header(CONTENT_TYPE, ct);
        }
        for (name, value) in self.extra_headers {
            builder = builder.header(name, value);
        }
        builder.body(self.body).map_err(|e| Error::Decode(format!("{e}")))
    }
}
