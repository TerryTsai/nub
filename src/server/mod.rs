//! HTTP and WebSocket transport. The router and unary `/api/op` handler
//! live in `router`; the long-lived WebSocket pump in `wire` + `ws`.
//! TLS termination and embedded-UI fallback are siblings.

pub mod tls;
pub mod ui;

mod router;
mod wire;
mod ws;

pub use router::router;
