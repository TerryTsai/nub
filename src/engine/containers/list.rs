//! Decoder for `/containers/json` (compat) and `/libpod/containers/json`
//! (libpod). Both shape responses similarly enough that one decoder works
//! for both — only the `Created` field differs (Unix int vs ISO string).

use serde::Deserialize;

use super::types::ContainerSummary;
use crate::engine::util::short_id;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct ListItem {
    #[serde(rename = "Id")]
    id: String,
    #[serde(default)]
    names: Vec<String>,
    #[serde(default)]
    image: String,
    #[serde(default)]
    state: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    created: serde_json::Value,
}

impl ListItem {
    pub(super) fn into_summary(self) -> ContainerSummary {
        ContainerSummary {
            id: short_id(&self.id),
            name: self
                .names
                .into_iter()
                .next()
                .map(|n| n.trim_start_matches('/').to_string())
                .unwrap_or_default(),
            image: self.image,
            state: self.state,
            status: self.status,
            created: created_to_string(self.created),
        }
    }
}

fn created_to_string(v: serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s,
        serde_json::Value::Number(n) => n.to_string(),
        _ => String::new(),
    }
}
