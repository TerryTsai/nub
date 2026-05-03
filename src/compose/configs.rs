//! Compose `configs:` translation. Top-level declarations + per-service
//! refs become `ConfigSpec` and `ServiceConfigRef` entries on the
//! `StackSpec`. Mirror of `secrets.rs`, minus the encryption.
//!
//! Only `content:` (inline string) is supported as the source. `file:`,
//! `external:`, and `environment:` are rejected with errors that point
//! the user at the inline form.

use std::collections::{HashMap, HashSet};

use super::spec::{ConfigSpec, ParseError, ServiceConfigRef};
use super::wire::{ConfigYaml, ServiceConfigYaml};

pub(super) fn transform_top_level(raw: HashMap<String, ConfigYaml>) -> Result<Vec<ConfigSpec>, ParseError> {
    let mut out = Vec::with_capacity(raw.len());
    for (name, decl) in raw {
        if decl.file.is_some() {
            return Err(ParseError(format!(
                "config `{name}`: `file:` source not supported by nub. \
                 Inline the value with `content:` instead."
            )));
        }
        if decl.external {
            return Err(ParseError(format!(
                "config `{name}`: `external: true` not supported by nub. \
                 Inline the value with `content:` instead."
            )));
        }
        if decl.environment.is_some() {
            return Err(ParseError(format!(
                "config `{name}`: `environment:` source not supported by nub. \
                 Inline the value with `content:` instead."
            )));
        }
        let content = decl.content.ok_or_else(|| {
            ParseError(format!(
                "config `{name}`: must declare a `content:` source (the only supported form)."
            ))
        })?;
        out.push(ConfigSpec { name, content });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

pub(super) fn transform_service_refs(
    svc: &str,
    refs: Vec<ServiceConfigYaml>,
    declared: &HashSet<String>,
) -> Result<Vec<ServiceConfigRef>, ParseError> {
    refs.into_iter()
        .map(|r| {
            let (source, target) = resolve_ref(r);
            if !declared.contains(&source) {
                return Err(ParseError(format!(
                    "service `{svc}` references config `{source}` which isn't declared at top level"
                )));
            }
            Ok(ServiceConfigRef { source, target })
        })
        .collect()
}

fn resolve_ref(r: ServiceConfigYaml) -> (String, String) {
    match r {
        ServiceConfigYaml::Short(name) => {
            let target = default_target(&name);
            (name, target)
        }
        ServiceConfigYaml::Long(long) => {
            let target = long
                .target
                .map(resolve_target)
                .unwrap_or_else(|| default_target(&long.source));
            (long.source, target)
        }
    }
}

/// Compose-spec: a bare filename target lives at `/<name>` (the
/// container fs root). An absolute path is taken as-is. This differs
/// from secrets, which default under `/run/secrets/`.
fn resolve_target(t: String) -> String {
    if t.starts_with('/') {
        t
    } else {
        format!("/{t}")
    }
}

fn default_target(name: &str) -> String {
    format!("/{name}")
}
