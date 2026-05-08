//! Compose `secrets:` translation. Top-level declarations + per-service
//! refs become `SecretSpec` and `ServiceSecretRef` entries on the
//! `StackSpec`. Split out from `transform.rs` to keep that file under
//! the per-file line cap.

use std::collections::{HashMap, HashSet};

use super::types::{ParseError, SecretSpec, ServiceSecretRef};
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

#[cfg(test)]
mod tests {
    use crate::compose::parse::parse;
    use std::collections::HashMap;

    #[test]
    fn external_short_and_long_form() {
        let yaml = r#"
services:
  app:
    image: x
    secrets:
      - api_key
      - source: db_password
        target: /etc/db.pw
secrets:
  api_key:
    external: true
  db_password:
    external: true
"#;
        let spec = parse(yaml, &HashMap::new()).unwrap();
        assert_eq!(spec.secrets.len(), 2);
        let names: Vec<_> = spec.secrets.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, ["api_key", "db_password"]);

        let svc = &spec.services[0];
        assert_eq!(svc.secrets.len(), 2);
        let api = svc.secrets.iter().find(|r| r.source == "api_key").unwrap();
        assert_eq!(api.target, "/run/secrets/api_key");
        let db = svc.secrets.iter().find(|r| r.source == "db_password").unwrap();
        assert_eq!(db.target, "/etc/db.pw");
    }

    #[test]
    fn file_source_rejected_with_clear_error() {
        let yaml = r#"
services:
  app:
    image: x
secrets:
  api_key:
    file: ./key
"#;
        let err = parse(yaml, &HashMap::new()).unwrap_err().to_string();
        assert!(err.contains("api_key"), "got: {err}");
        assert!(err.contains("nub secret put"), "got: {err}");
    }

    #[test]
    fn requires_external_true() {
        let yaml = r#"
services:
  app:
    image: x
secrets:
  api_key: {}
"#;
        let err = parse(yaml, &HashMap::new()).unwrap_err().to_string();
        assert!(err.contains("external: true"), "got: {err}");
    }

    #[test]
    fn service_secret_must_be_declared() {
        let yaml = r#"
services:
  app:
    image: x
    secrets:
      - rogue
secrets:
  declared:
    external: true
"#;
        let err = parse(yaml, &HashMap::new()).unwrap_err().to_string();
        assert!(err.contains("rogue"), "got: {err}");
        assert!(err.contains("isn't declared"), "got: {err}");
    }

    #[test]
    fn secret_target_relative_goes_under_run_secrets() {
        let yaml = r#"
services:
  app:
    image: x
    secrets:
      - source: db_password
        target: db_pw
secrets:
  db_password:
    external: true
"#;
        let spec = parse(yaml, &HashMap::new()).unwrap();
        let r = &spec.services[0].secrets[0];
        assert_eq!(r.target, "/run/secrets/db_pw");
    }

    #[test]
    fn secret_name_override_threads_through_lookup() {
        let yaml = r#"
services:
  app:
    image: x
    secrets:
      - api_key
secrets:
  api_key:
    external: true
    name: prod_api_key
"#;
        let spec = parse(yaml, &HashMap::new()).unwrap();
        assert_eq!(spec.secrets[0].name, "api_key");
        assert_eq!(spec.secrets[0].lookup, "prod_api_key");
    }
}
