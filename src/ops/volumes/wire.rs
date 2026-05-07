//! Engine wire shapes for volume ops. Split out so `mod.rs` stays under
//! the project's per-file line limit.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::proto::VolumeSummary;

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct CreateBody {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub labels: HashMap<String, String>,
    #[serde(rename = "DriverOpts", skip_serializing_if = "HashMap::is_empty")]
    pub driver_opts: HashMap<String, String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct CompatList {
    #[serde(default)]
    pub volumes: Vec<RawCompatVolume>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct RawCompatVolume {
    pub name: String,
    #[serde(default)]
    pub driver: String,
    #[serde(default)]
    pub mountpoint: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub scope: String,
}

impl RawCompatVolume {
    pub(super) fn into_summary(self, in_use: bool) -> VolumeSummary {
        VolumeSummary {
            name: self.name,
            driver: self.driver,
            mountpoint: self.mountpoint,
            created_at: self.created_at,
            scope: self.scope,
            in_use,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct RawLibpodVolume {
    pub name: String,
    #[serde(default)]
    pub driver: String,
    #[serde(default)]
    pub mountpoint: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub scope: String,
    #[serde(default)]
    pub mount_count: i64,
}

impl RawLibpodVolume {
    pub(super) fn into_summary(self) -> VolumeSummary {
        VolumeSummary {
            name: self.name,
            driver: self.driver,
            mountpoint: self.mountpoint,
            created_at: self.created_at,
            scope: self.scope,
            in_use: self.mount_count > 0,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct ContainerWithMounts {
    #[serde(default)]
    pub mounts: Vec<RawMount>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct RawMount {
    /// Volume name. Empty for non-volume mounts (bind, tmpfs).
    #[serde(default)]
    pub name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct RawInspect {
    pub name: String,
    #[serde(default)]
    pub driver: String,
    #[serde(default)]
    pub mountpoint: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub scope: String,
    #[serde(default, deserialize_with = "crate::ops::util::null_to_default")]
    pub labels: HashMap<String, String>,
    #[serde(default, deserialize_with = "crate::ops::util::null_to_default")]
    pub options: HashMap<String, String>,
    #[serde(default)]
    pub usage_data: Option<RawUsage>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct RawUsage {
    #[serde(default)]
    pub size: i64,
    #[serde(default)]
    pub ref_count: i64,
}
