//! Engine wire shape for `GET /containers/json` (compat) and
//! `/v4.0.0/libpod/containers/json` (libpod). Field set is the union
//! of both — both engines use PascalCase so a single struct decodes
//! both.

use std::collections::HashMap;

use serde::Deserialize;

use crate::client::short_id;
use crate::ops::serde_helpers::null_to_default;
use crate::proto::ContainerSummary;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(in crate::ops::containers) struct RawListItem {
    #[serde(rename = "Id")]
    id: String,
    #[serde(default, deserialize_with = "null_to_default")]
    names: Vec<String>,
    #[serde(default)]
    image: String,
    #[serde(default)]
    state: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    created: serde_json::Value,
    #[serde(default)]
    exit_code: i32,
    #[serde(default, deserialize_with = "null_to_default")]
    labels: HashMap<String, String>,
}

impl RawListItem {
    pub(in crate::ops::containers) fn into_summary(self) -> ContainerSummary {
        let health = parse_health(&self.status);
        ContainerSummary {
            id: short_id(&self.id),
            name: first_name(&self.names),
            image: self.image,
            state: self.state,
            status: self.status,
            created: created_to_string(self.created),
            exit_code: self.exit_code,
            health,
            labels: self.labels,
        }
    }
}

/// Pull a healthcheck state out of the engine's free-form Status string.
/// Both engines render `Up 5 minutes (healthy)` / `(unhealthy)` /
/// `(health: starting)`; we parse the parenthesized hint without pretending
/// it's a structured field.
fn parse_health(status: &str) -> String {
    let (Some(open), Some(close)) = (status.find('('), status.rfind(')')) else {
        return String::new();
    };
    if open >= close {
        return String::new();
    }
    let inner = status[open + 1..close].trim();
    let after_colon = inner.strip_prefix("health: ").unwrap_or(inner);
    match after_colon {
        "healthy" | "unhealthy" | "starting" => after_colon.to_string(),
        _ => String::new(),
    }
}

fn first_name(names: &[String]) -> String {
    names
        .first()
        .map(|n| n.trim_start_matches('/').to_string())
        .unwrap_or_default()
}

fn created_to_string(v: serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s,
        serde_json::Value::Number(n) => n.to_string(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_health;

    #[test]
    fn extracts_known_states() {
        assert_eq!(parse_health("Up 5 minutes (healthy)"), "healthy");
        assert_eq!(parse_health("Up 12 hours (unhealthy)"), "unhealthy");
        assert_eq!(parse_health("Up 3 seconds (health: starting)"), "starting");
    }

    #[test]
    fn ignores_unrelated_parens() {
        assert_eq!(parse_health("Exited (137) 2 weeks ago"), "");
        assert_eq!(parse_health("Up 5 minutes"), "");
        assert_eq!(parse_health(""), "");
    }
}
