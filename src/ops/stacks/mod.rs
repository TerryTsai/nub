//! Stack ops — compose-shaped manifests deployed via nub's existing
//! container/network/volume primitives. We do not shell out to
//! `docker compose` or `podman compose`; nub owns the deploy path
//! end-to-end so policies and observability stay uniform.
//!
//! Slice-2 scope: parallel container deploys, no `depends_on` ordering,
//! always-recreate redeploy. Service-name DNS works because every
//! container in a stack is attached to a single user-defined network
//! created with `nub.stack=<name>`.

pub mod create;
pub mod delete;
pub mod get;
pub mod list;
pub mod logs;
pub mod pull;
pub mod redeploy;
pub mod update;

mod discover;
mod engine;
mod labels;
mod store;
