//! Dockerfile-store ops. CRUD on a flat directory of text files.
//! Filenames are whitelisted, symlinks are rejected, writes are atomic.
//! One file per Op variant; the shared filesystem layer lives in
//! `store.rs`.

pub(super) mod delete;
pub(super) mod get;
pub(super) mod list;
pub(super) mod put;

mod store;
