//! Compose-shaped YAML wire types. Many compose fields accept either a
//! string or a list (`command: "npm start"` vs `["npm", "start"]`) or
//! either a list or a map (`environment: ["K=v"]` vs `{ K: v }`); these
//! `untagged` enums absorb both shapes.

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Default)]
#[serde(default)]
pub(super) struct Compose {
    pub services: HashMap<String, ServiceYaml>,
    pub volumes: HashMap<String, VolumeYaml>,
    /// Compose schema version. Informational; ignored.
    pub version: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub(super) struct ServiceYaml {
    pub image: Option<String>,
    pub container_name: Option<String>,
    pub command: Option<StringOrList>,
    pub entrypoint: Option<StringOrList>,
    pub environment: Option<MapOrList>,
    pub ports: Vec<String>,
    pub volumes: Vec<String>,
    pub network_mode: Option<String>,
    pub restart: Option<String>,
    pub working_dir: Option<String>,
    pub user: Option<String>,
    pub labels: Option<MapOrList>,
    pub hostname: Option<String>,
    pub healthcheck: Option<HealthcheckYaml>,
    pub cap_add: Vec<String>,
    pub cap_drop: Vec<String>,
    pub privileged: bool,
    pub extra_hosts: Vec<String>,
    pub init: Option<bool>,
    pub expose: Vec<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub(super) struct HealthcheckYaml {
    pub test: Option<StringOrList>,
    pub interval: Option<String>,
    pub timeout: Option<String>,
    pub retries: Option<i64>,
    pub start_period: Option<String>,
    pub disable: bool,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub(super) struct VolumeYaml {
    pub external: bool,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub(super) enum StringOrList {
    Str(String),
    List(Vec<String>),
}

impl StringOrList {
    /// `command:`/`entrypoint:` semantics — a bare string is shell-split,
    /// a list is taken verbatim. Matches what compose runtimes do.
    pub fn shell_split(self) -> Vec<String> {
        match self {
            Self::Str(s) => s.split_whitespace().map(String::from).collect(),
            Self::List(v) => v,
        }
    }

    pub fn into_list(self) -> Vec<String> {
        match self {
            Self::Str(s) => vec![s],
            Self::List(v) => v,
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub(super) enum MapOrList {
    Map(HashMap<String, Option<String>>),
    List(Vec<String>),
}

impl MapOrList {
    /// Flatten to KEY=VALUE strings (engine wire format for env).
    pub fn into_kv_list(self) -> Vec<String> {
        match self {
            Self::Map(m) => m
                .into_iter()
                .map(|(k, v)| match v {
                    Some(val) => format!("{k}={val}"),
                    None => k,
                })
                .collect(),
            Self::List(v) => v,
        }
    }

    /// Flatten to a map. List entries without `=` are dropped.
    pub fn into_kv_map(self) -> HashMap<String, String> {
        match self {
            Self::Map(m) => m.into_iter().filter_map(|(k, v)| v.map(|val| (k, val))).collect(),
            Self::List(list) => list
                .into_iter()
                .filter_map(|s| s.split_once('=').map(|(k, v)| (k.to_string(), v.to_string())))
                .collect(),
        }
    }
}
