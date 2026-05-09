//! Label conventions and naming helpers for stack-managed resources.
//! Slice-2 contract: every container/network/volume created by a stack
//! carries `nub.stack=<name>`, so cleanup never depends on filesystem
//! state — we list-by-label and remove what we find.

use std::collections::HashMap;

pub const STACK_LABEL: &str = "nub.stack";
pub const SERVICE_LABEL: &str = "nub.service";

pub fn network_name(stack: &str) -> String {
    stack.to_string()
}

pub fn volume_name(stack: &str, declared: &str) -> String {
    format!("{stack}_{declared}")
}

pub fn container_name(stack: &str, service: &str, override_name: Option<&str>) -> String {
    override_name.map(|s| s.to_string()).unwrap_or_else(|| format!("{stack}_{service}"))
}

pub fn stack_labels(stack: &str, service: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert(STACK_LABEL.into(), stack.into());
    m.insert(SERVICE_LABEL.into(), service.into());
    m
}
