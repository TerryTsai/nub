//! Engine wire shapes for network ops — list/inspect decoders for both
//! Docker compat and Podman libpod, plus the `POST /networks/create`
//! request body. Both engines use PascalCase JSON.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::client::short_id;
use crate::ops::serde_helpers::null_to_default;
use crate::proto::NetworkSummary;

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct CreateBody {
    pub name: String,
    pub driver: String,
    pub internal: bool,
    /// Labels are an internal-only parameter (no wire field on
    /// `Op::CreateNetwork`); the dispatch passes empty, the stack runtime
    /// passes `nub.stack=<name>`. `skip_serializing_if` keeps the
    /// public-op wire body byte-identical to before this refactor.
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub labels: HashMap<String, String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct CompatList {
    #[serde(default, rename = "Id")]
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    driver: String,
    #[serde(default)]
    scope: String,
    #[serde(default)]
    created: String,
    #[serde(default)]
    internal: bool,
}

impl CompatList {
    pub(super) fn into_summary(self) -> NetworkSummary {
        NetworkSummary {
            id: short_id(&self.id),
            name: self.name,
            driver: self.driver,
            scope: self.scope,
            created: self.created,
            internal: self.internal,
            // Filled in by `list::run` after joining with the usage probe.
            in_use: false,
        }
    }
}

#[derive(Deserialize)]
pub(super) struct LibpodList {
    #[serde(default)]
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    driver: String,
    #[serde(default)]
    created: String,
    #[serde(default)]
    internal: bool,
}

impl LibpodList {
    pub(super) fn into_summary(self) -> NetworkSummary {
        NetworkSummary {
            id: short_id(&self.id),
            name: self.name,
            driver: self.driver,
            // libpod doesn't report `scope`; leave empty so the proto shape
            // is stable across engines.
            scope: String::new(),
            created: self.created,
            internal: self.internal,
            in_use: false,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct ContainerNets {
    #[serde(default)]
    pub networks: Vec<String>,
    #[serde(default)]
    pub network_settings: RawNetSettings,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct RawNetSettings {
    #[serde(default, deserialize_with = "null_to_default")]
    pub networks: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct CompatInspect {
    #[serde(default, rename = "Id")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub driver: String,
    #[serde(default)]
    pub scope: String,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub internal: bool,
    #[serde(default)]
    pub ipam: Option<CompatIpam>,
    #[serde(default, deserialize_with = "null_to_default")]
    pub containers: HashMap<String, CompatNetContainer>,
    #[serde(default, deserialize_with = "null_to_default")]
    pub options: HashMap<String, String>,
    #[serde(default, deserialize_with = "null_to_default")]
    pub labels: HashMap<String, String>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct CompatIpam {
    #[serde(default)]
    pub config: Vec<CompatIpamConfig>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct CompatIpamConfig {
    #[serde(default)]
    pub subnet: String,
    #[serde(default)]
    pub gateway: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct CompatNetContainer {
    #[serde(default)]
    pub name: String,
    #[serde(default, rename = "IPv4Address")]
    pub ipv4_address: String,
    #[serde(default, rename = "IPv6Address")]
    pub ipv6_address: String,
}

#[derive(Deserialize)]
pub(super) struct LibpodInspect {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub driver: String,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub internal: bool,
    #[serde(default, deserialize_with = "null_to_default")]
    pub subnets: Vec<LibpodSubnet>,
    #[serde(default, deserialize_with = "null_to_default")]
    pub options: HashMap<String, String>,
    #[serde(default, deserialize_with = "null_to_default")]
    pub labels: HashMap<String, String>,
}

#[derive(Deserialize)]
pub(super) struct LibpodSubnet {
    #[serde(default)]
    pub subnet: String,
    #[serde(default)]
    pub gateway: String,
}
