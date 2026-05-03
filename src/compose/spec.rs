//! Public output of the compose parser. `StackSpec` is what the slice-2
//! stack runtime will consume — a list of nub-shaped container specs
//! plus declared networks/volumes plus a list of compose keys we
//! recognized but didn't translate (so the UI can surface them).

use crate::proto::CreateContainerReq;

#[derive(Debug)]
pub struct StackSpec {
    pub services: Vec<ServiceSpec>,
    pub volumes: Vec<VolumeSpec>,
    pub secrets: Vec<SecretSpec>,
    /// Top-level keys we don't process (e.g. `configs`, `x-extensions`).
    /// Sorted alphabetically.
    pub unsupported: Vec<String>,
}

#[derive(Debug)]
pub struct ServiceSpec {
    /// Service key from compose's `services:` map.
    pub name: String,
    /// Translated container spec. The slice-2 runtime applies stack
    /// labels and resolves the container name before calling create.
    pub container: CreateContainerReq,
    /// Secret references resolved from this service's `secrets:` list.
    /// Each entry's `source` matches a `SecretSpec.name` in the parent
    /// `StackSpec`. Empty when the service uses no secrets.
    pub secrets: Vec<ServiceSecretRef>,
    /// Service-level keys we don't translate (e.g. `build`,
    /// `depends_on`). Sorted alphabetically.
    pub unsupported: Vec<String>,
}

#[derive(Debug)]
pub struct VolumeSpec {
    pub name: String,
    pub external: bool,
}

#[derive(Debug)]
pub struct SecretSpec {
    /// Compose key (the entry under top-level `secrets:`). Used both as
    /// the default container-side filename and as the lookup key when
    /// `name` isn't set.
    pub name: String,
    /// Resolved lookup key against `nub secret`. Same as `name` unless
    /// the YAML overrode it via `name:`.
    pub lookup: String,
}

/// One service's reference to a top-level secret.
#[derive(Debug)]
pub struct ServiceSecretRef {
    /// Matches `SecretSpec.name`.
    pub source: String,
    /// Container-side mount target. Defaults to `/run/secrets/<source>`
    /// per the compose spec.
    pub target: String,
}

#[derive(Debug)]
pub struct ParseError(pub String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ParseError {}
