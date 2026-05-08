//! Volume wire types — list summaries and inspect detail.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct VolumeSummary {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
    pub created_at: String,
    pub scope: String,
    /// True if at least one container (running or stopped) mounts this volume.
    pub in_use: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VolumeDetail {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
    pub created_at: String,
    pub scope: String,
    pub labels: HashMap<String, String>,
    pub options: HashMap<String, String>,
    /// Number of containers using this volume (from /system/df).
    pub ref_count: i64,
    /// Disk usage bytes (-1 if not reported).
    pub size: i64,
}
