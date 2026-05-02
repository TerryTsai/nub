//! HTTP/1.1 client over Unix socket or TCP. One connection per call.

mod conn;
mod query;
mod req;

pub(crate) use conn::{upgrade, Address, Conn};
pub(crate) use query::Query;
pub(crate) use req::Req;
