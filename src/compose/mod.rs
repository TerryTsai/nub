//! Parse compose-shaped YAML into a stack spec that nub deploys via its
//! existing container/network/volume ops. We don't reimplement compose
//! orchestration (no `depends_on` ordering, no healthcheck-conditioned
//! startup, no diff/recreate); we just translate the YAML into nub
//! primitives and let the engine do the rest.

mod configs;
mod duration;
mod parse;
mod secrets;
mod substitute;
mod transform;
mod types;
mod wire;

pub use parse::parse;
pub use types::{ServiceConfigRef, ServiceSecretRef, ServiceSpec, StackSpec};
