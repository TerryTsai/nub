//! Engine wire shapes for `/info` and `/version`.

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct InfoResp {
    pub operating_system: String,
    pub architecture: String,
    pub kernel_version: String,
    #[serde(rename = "NCPU")]
    pub ncpu: u64,
    pub mem_total: u64,
    pub containers_running: u64,
    pub containers: u64,
    pub images: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct VersionResp {
    pub version: String,
    #[serde(default)]
    pub platform: Option<Platform>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct Platform {
    pub name: String,
}
