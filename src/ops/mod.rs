//! Op handlers — one file per op family. Each op owns its full pipeline:
//! build the engine path, send, decode JSON or stream, return proto types.
//! No middle layer between proto and the socket.

pub mod configs;
pub mod secrets;
pub mod stacks;

mod containers;
mod dockerfiles;
mod handler;
mod host;
mod images;
mod names;
mod networks;
mod policy;
mod serde_helpers;
mod time;
mod tmpfs;
mod volumes;

pub use handler::{closed_input, EngineHandler, HandlerOutput, Shared};
pub use policy::Policy;

pub(crate) use handler::{log_chunk, spawn_chunked};
