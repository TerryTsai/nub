//! Talks to a Docker or Podman socket. Just transport + framing primitives —
//! no per-op typed methods. Each `ops/*` file builds its own paths and
//! decoders against the helpers exposed here.

mod conn;
mod engine;
mod framing;
mod query;
mod req;

pub(crate) use conn::upgrade;
pub use engine::{Engine, EngineKind, Error};
pub(crate) use framing::{short_id, LineStream, Multiplexer, MultiplexerMode, MuxFrame};
pub(crate) use query::Query;
pub(crate) use req::Req;
