use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SecretSummary {
    pub name: String,
    /// Encrypted blob size on disk.
    pub size: u64,
    /// ISO 8601 mtime, or empty when the FS doesn't expose one.
    pub modified_at: String,
}

/// Plaintext secret value. Returned only by the `get_secret` op, which
/// is gated behind `secrets:reveal` (admin-only). Never shipped to the
/// phone UI.
#[derive(Debug, Serialize, Deserialize)]
pub struct SecretValue {
    pub name: String,
    pub value: String,
}
