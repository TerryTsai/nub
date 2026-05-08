//! Dockerfile-store wire types — listings and inline content.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DockerfileSummary {
    pub name: String,
    /// File size in bytes.
    pub size: u64,
    /// ISO 8601 mtime, or empty when the FS doesn't expose one.
    pub modified_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DockerfileContent {
    pub name: String,
    pub content: String,
    pub size: u64,
    pub modified_at: String,
}
