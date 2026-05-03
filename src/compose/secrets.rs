//! Compose `secrets:` translation. Top-level declarations + per-service
//! refs become `SecretSpec` and `ServiceSecretRef` entries on the
//! `StackSpec`. Split out from `transform.rs` to keep that file under
//! the per-file line cap.

use std::collections::{HashMap, HashSet};

use super::spec::{ParseError, SecretSpec, ServiceSecretRef};
use super::wire::{SecretYaml, ServiceSecretYaml};

pub(super) fn transform_top_level(raw: HashMap<String, SecretYaml>) -> Result<Vec<SecretSpec>, ParseError> {
    let mut out = Vec::with_capacity(raw.len());
    for (name, decl) in raw {
        if decl.file.is_some() {
            return Err(ParseError(format!(
                "secret `{name}`: `file:` source not supported by nub. \
                 Run `nub secret put {name}` and use `external: true` to reference it."
            )));
        }
        if decl.environment.is_some() {
            return Err(ParseError(format!(
                "secret `{name}`: `environment:` source not supported by nub. \
                 Run `nub secret put {name}` and use `external: true`."
            )));
        }
        if !decl.external {
            return Err(ParseError(format!(
                "secret `{name}`: must declare `external: true` (nub-managed sources only)."
            )));
        }
        let lookup = decl.name.unwrap_or_else(|| name.clone());
        out.push(SecretSpec { name, lookup });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

pub(super) fn transform_service_refs(
    svc: &str,
    refs: Vec<ServiceSecretYaml>,
    declared: &HashSet<String>,
) -> Result<Vec<ServiceSecretRef>, ParseError> {
    refs.into_iter()
        .map(|r| {
            let (source, target) = resolve_ref(r);
            if !declared.contains(&source) {
                return Err(ParseError(format!(
                    "service `{svc}` references secret `{source}` which isn't declared at top level"
                )));
            }
            Ok(ServiceSecretRef { source, target })
        })
        .collect()
}

fn resolve_ref(r: ServiceSecretYaml) -> (String, String) {
    match r {
        ServiceSecretYaml::Short(name) => {
            let target = default_target(&name);
            (name, target)
        }
        ServiceSecretYaml::Long(long) => {
            let target = long
                .target
                .map(resolve_target)
                .unwrap_or_else(|| default_target(&long.source));
            (long.source, target)
        }
    }
}

/// Compose-spec: a bare filename target lives under `/run/secrets/`.
/// An absolute path is taken as-is.
fn resolve_target(t: String) -> String {
    if t.starts_with('/') {
        t
    } else {
        format!("/run/secrets/{t}")
    }
}

fn default_target(name: &str) -> String {
    format!("/run/secrets/{name}")
}
