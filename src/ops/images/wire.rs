//! Engine wire shapes for image ops — list/inspect responses, plus the
//! per-line progress envelopes that pull and build emit.

use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct RawImage {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(default)]
    pub repo_tags: Option<Vec<String>>,
    #[serde(default)]
    pub created: i64,
    #[serde(default)]
    pub size: i64,
    #[serde(default)]
    pub containers: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct RawInspect {
    #[serde(rename = "Id", default)]
    pub id: String,
    #[serde(default)]
    pub repo_tags: Option<Vec<String>>,
    #[serde(default)]
    pub repo_digests: Option<Vec<String>>,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub size: i64,
    #[serde(default)]
    pub architecture: String,
    #[serde(default)]
    pub os: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub comment: String,
    #[serde(default)]
    pub config: RawConfig,
    #[serde(default)]
    pub root_fs: RawRootFs,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct RawConfig {
    // Docker emits explicit `null` for empty container slices and maps in
    // image inspect responses. `Option<...>` + `unwrap_or_default` accepts
    // both `null` and missing without serde griping.
    #[serde(default)]
    pub cmd: Option<Vec<String>>,
    #[serde(default)]
    pub entrypoint: Option<Vec<String>>,
    #[serde(default)]
    pub env: Option<Vec<String>>,
    #[serde(default)]
    pub working_dir: String,
    #[serde(default)]
    pub user: String,
    #[serde(default)]
    pub exposed_ports: Option<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub labels: Option<HashMap<String, String>>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct RawRootFs {
    #[serde(default)]
    pub layers: Vec<String>,
}

#[derive(Deserialize)]
pub(super) struct BuildInfo {
    #[serde(default)]
    pub stream: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub aux: Option<BuildAux>,
}

#[derive(Deserialize)]
pub(super) struct BuildAux {
    #[serde(default, rename = "ID")]
    pub id: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct CreateImageInfo {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default, rename = "error")]
    pub error: Option<String>,
    #[serde(default, rename = "progressDetail")]
    pub progress_detail: Option<ProgressDetail>,
}

#[derive(Deserialize)]
pub(super) struct ProgressDetail {
    #[serde(default)]
    pub current: u64,
    #[serde(default)]
    pub total: u64,
}
