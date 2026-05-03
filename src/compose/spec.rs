//! Public output of the compose parser. `StackSpec` is what the slice-2
//! stack runtime will consume — a list of nub-shaped container specs
//! plus declared networks/volumes plus a list of compose keys we
//! recognized but didn't translate (so the UI can surface them).

use crate::proto::CreateContainerReq;

#[derive(Debug)]
pub struct StackSpec {
    pub services: Vec<ServiceSpec>,
    pub volumes: Vec<VolumeSpec>,
    /// Top-level keys we don't process (e.g. `secrets`, `configs`,
    /// `x-extensions`). Sorted alphabetically.
    pub unsupported: Vec<String>,
}

#[derive(Debug)]
pub struct ServiceSpec {
    /// Service key from compose's `services:` map.
    pub name: String,
    /// Translated container spec. The slice-2 runtime applies stack
    /// labels and resolves the container name before calling create.
    pub container: CreateContainerReq,
    /// Service-level keys we don't translate (e.g. `build`,
    /// `depends_on`, `secrets`). Sorted alphabetically.
    pub unsupported: Vec<String>,
}

#[derive(Debug)]
pub struct VolumeSpec {
    pub name: String,
    pub external: bool,
}

#[derive(Debug)]
pub struct ParseError(pub String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ParseError {}
