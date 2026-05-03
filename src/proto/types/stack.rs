use super::container::ContainerSummary;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct StackSummary {
    pub name: String,
    /// `active` (all containers up), `idle` (all stopped), `pending`
    /// (mixed or partially deployed). Same vocabulary as ContainerSummary.
    pub status: String,
    pub container_count: u32,
    /// ISO 8601 mtime of the stored compose.yml. Empty when the FS
    /// doesn't expose mtime.
    pub modified_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StackDetail {
    pub name: String,
    pub yaml: String,
    pub modified_at: String,
    pub containers: Vec<ContainerSummary>,
    /// Network nub created for this stack. Empty before first deploy.
    pub network_name: String,
    /// Top-level compose keys we recognized but didn't translate
    /// (e.g. `secrets`, `configs`, `x-extensions`). Sorted.
    pub unsupported: Vec<String>,
    /// Service-level compose keys we recognized but didn't translate
    /// (e.g. `build`, `depends_on`), keyed by service name.
    pub service_unsupported: HashMap<String, Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StackCreated {
    pub name: String,
    /// Container IDs nub created and started.
    pub container_ids: Vec<String>,
}
