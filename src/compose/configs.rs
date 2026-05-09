//! Compose `configs:` translation. Top-level declarations + per-service
//! refs become `ConfigSpec` and `ServiceConfigRef` entries on the
//! `StackSpec`. Mirror of `secrets.rs`, minus the encryption.
//!
//! Only `content:` (inline string) is supported as the source. `file:`,
//! `external:`, and `environment:` are rejected with errors that point
//! the user at the inline form.

use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, bail, ensure, Result};

use super::types::{ConfigSpec, ServiceConfigRef};
use super::wire::{ConfigYaml, ServiceConfigYaml};

pub(super) fn transform_top_level(raw: HashMap<String, ConfigYaml>) -> Result<Vec<ConfigSpec>> {
    let mut out = Vec::with_capacity(raw.len());
    for (name, decl) in raw {
        if decl.file.is_some() {
            bail!(
                "config `{name}`: `file:` source not supported by nub. \
                 Inline the value with `content:` instead."
            );
        }
        if decl.external {
            bail!(
                "config `{name}`: `external: true` not supported by nub. \
                 Inline the value with `content:` instead."
            );
        }
        if decl.environment.is_some() {
            bail!(
                "config `{name}`: `environment:` source not supported by nub. \
                 Inline the value with `content:` instead."
            );
        }
        let content = decl
            .content
            .ok_or_else(|| anyhow!("config `{name}`: must declare a `content:` source (the only supported form)."))?;
        out.push(ConfigSpec { name, content });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

pub(super) fn transform_service_refs(
    svc: &str,
    refs: Vec<ServiceConfigYaml>,
    declared: &HashSet<String>,
) -> Result<Vec<ServiceConfigRef>> {
    refs.into_iter()
        .map(|r| {
            let (source, target) = resolve_ref(r);
            ensure!(
                declared.contains(&source),
                "service `{svc}` references config `{source}` which isn't declared at top level"
            );
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
            let target = long.target.map(resolve_target).unwrap_or_else(|| default_target(&long.source));
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

#[cfg(test)]
mod tests {
    use crate::compose::parse::parse;

    #[test]
    fn inline_content_short_and_long_form() {
        let yaml = r#"
services:
  app:
    image: nginx
    configs:
      - nginx_conf
      - source: site_html
        target: /usr/share/nginx/html/index.html
configs:
  nginx_conf:
    content: |
      server { listen 80; }
  site_html:
    content: <h1>hello</h1>
"#;
        let spec = parse(yaml).unwrap();
        assert_eq!(spec.configs.len(), 2);
        let names: Vec<_> = spec.configs.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["nginx_conf", "site_html"]);
        assert!(spec.configs[0].content.contains("listen 80"));
        assert_eq!(spec.configs[1].content, "<h1>hello</h1>");

        let svc = &spec.services[0];
        assert_eq!(svc.configs.len(), 2);
        let nginx = svc.configs.iter().find(|r| r.source == "nginx_conf").unwrap();
        // Short form defaults target to /<name> per compose-spec.
        assert_eq!(nginx.target, "/nginx_conf");
        let site = svc.configs.iter().find(|r| r.source == "site_html").unwrap();
        assert_eq!(site.target, "/usr/share/nginx/html/index.html");
    }

    #[test]
    fn file_source_rejected() {
        let yaml = r#"
services:
  app:
    image: x
configs:
  cfg:
    file: ./cfg
"#;
        let err = parse(yaml).unwrap_err().to_string();
        assert!(err.contains("cfg"), "got: {err}");
        assert!(err.contains("content:"), "got: {err}");
    }

    #[test]
    fn external_rejected() {
        let yaml = r#"
services:
  app:
    image: x
configs:
  cfg:
    external: true
"#;
        let err = parse(yaml).unwrap_err().to_string();
        assert!(err.contains("external"), "got: {err}");
        assert!(err.contains("content:"), "got: {err}");
    }

    #[test]
    fn environment_rejected() {
        let yaml = r#"
services:
  app:
    image: x
configs:
  cfg:
    environment: SOME_VAR
"#;
        let err = parse(yaml).unwrap_err().to_string();
        assert!(err.contains("environment"), "got: {err}");
    }

    #[test]
    fn requires_content() {
        let yaml = r#"
services:
  app:
    image: x
configs:
  cfg: {}
"#;
        let err = parse(yaml).unwrap_err().to_string();
        assert!(err.contains("content:"), "got: {err}");
    }

    #[test]
    fn service_config_must_be_declared() {
        let yaml = r#"
services:
  app:
    image: x
    configs:
      - rogue
configs:
  declared:
    content: value
"#;
        let err = parse(yaml).unwrap_err().to_string();
        assert!(err.contains("rogue"), "got: {err}");
        assert!(err.contains("isn't declared"), "got: {err}");
    }

    #[test]
    fn target_relative_lands_at_root() {
        // Compose-spec: bare filename targets land at /<name> (the
        // container fs root), unlike secrets which land under /run/secrets/.
        let yaml = r#"
services:
  app:
    image: x
    configs:
      - source: cfg
        target: app.conf
configs:
  cfg:
    content: x
"#;
        let spec = parse(yaml).unwrap();
        assert_eq!(spec.services[0].configs[0].target, "/app.conf");
    }
}
