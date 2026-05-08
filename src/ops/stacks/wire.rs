//! Engine wire shapes for stack ops. Today only the `containers/json`
//! decoder used by `logs::run` to enumerate per-stack containers.

use std::collections::HashMap;

use serde::Deserialize;

use crate::ops::serde_helpers::null_to_default;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct RawListItem {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub names: Vec<String>,
    #[serde(default, deserialize_with = "null_to_default")]
    pub labels: HashMap<String, String>,
}
