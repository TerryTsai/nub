//! Top-level entry point for the compose parser. Wires together the
//! substitute → wire (serde-yaml) → transform pipeline.

use std::collections::HashMap;

use anyhow::{Context, Result};

use super::substitute;
use super::transform;
use super::types::StackSpec;
use super::wire;

/// Parse compose YAML into nub's `StackSpec`. Variable references
/// (`${VAR}`, `${VAR:-default}`) without a default fail at parse time
/// — nub never threads a shell env through to compose YAML; values
/// must inline or use `:-default`.
pub fn parse(yaml: &str) -> Result<StackSpec> {
    let substituted = substitute::substitute(yaml, &HashMap::new())?;
    let raw: wire::Compose = serde_yaml::from_str(&substituted).context("yaml")?;
    transform::transform(raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_service() {
        let yaml = "services:\n  app:\n    image: nginx:1.27\n";
        let spec = parse(yaml).unwrap();
        assert_eq!(spec.services.len(), 1);
        assert_eq!(spec.services[0].name, "app");
        assert_eq!(spec.services[0].container.image, "nginx:1.27");
    }

    #[test]
    fn parse_missing_image_errors() {
        let yaml = "services:\n  app: {}\n";
        let err = parse(yaml).unwrap_err().to_string();
        assert!(err.contains("image"));
    }

    #[test]
    fn parse_environment_map_and_list() {
        let yaml = r#"
services:
  a:
    image: x
    environment:
      KEY: val
      OTHER: thing
  b:
    image: x
    environment:
      - K=v
      - LONE
"#;
        let spec = parse(yaml).unwrap();
        let a = spec.services.iter().find(|s| s.name == "a").unwrap();
        let mut ae = a.container.env.clone();
        ae.sort();
        assert_eq!(ae, vec!["KEY=val", "OTHER=thing"]);
        let b = spec.services.iter().find(|s| s.name == "b").unwrap();
        let mut be = b.container.env.clone();
        be.sort();
        assert_eq!(be, vec!["K=v", "LONE"]);
    }

    #[test]
    fn parse_command_string_is_shell_split() {
        let yaml = "services:\n  app:\n    image: x\n    command: npm run start\n";
        let spec = parse(yaml).unwrap();
        assert_eq!(spec.services[0].container.cmd, vec!["npm", "run", "start"]);
    }

    #[test]
    fn parse_command_list_is_verbatim() {
        let yaml = "services:\n  app:\n    image: x\n    command: [\"sh\", \"-c\", \"echo hi\"]\n";
        let spec = parse(yaml).unwrap();
        assert_eq!(spec.services[0].container.cmd, vec!["sh", "-c", "echo hi"]);
    }

    #[test]
    fn parse_ports_short_form() {
        let yaml = r#"
services:
  app:
    image: x
    ports:
      - "8080:80"
      - "443"
      - "127.0.0.1:9090:9090"
"#;
        let spec = parse(yaml).unwrap();
        let ports = &spec.services[0].container.ports;
        assert_eq!(ports.len(), 3);
        assert_eq!(ports[0].host, "8080");
        assert_eq!(ports[0].container, "80");
        assert_eq!(ports[1].host, "443");
        assert_eq!(ports[1].container, "443");
        assert_eq!(ports[2].host, "127.0.0.1:9090");
        assert_eq!(ports[2].container, "9090");
    }

    #[test]
    fn parse_volumes_ro_and_rw() {
        let yaml = r#"
services:
  app:
    image: x
    volumes:
      - ./data:/data
      - ./readonly:/etc/conf:ro
"#;
        let spec = parse(yaml).unwrap();
        let vols = &spec.services[0].container.volumes;
        assert_eq!(vols[0].source, "./data");
        assert_eq!(vols[0].target, "/data");
        assert!(!vols[0].read_only);
        assert_eq!(vols[1].source, "./readonly");
        assert!(vols[1].read_only);
    }

    #[test]
    fn parse_restart_on_failure_with_count() {
        let yaml = "services:\n  app:\n    image: x\n    restart: on-failure:5\n";
        let spec = parse(yaml).unwrap();
        let r = spec.services[0].container.restart.as_ref().unwrap();
        match r {
            crate::proto::RestartPolicySpec::OnFailure { max_retries } => assert_eq!(*max_retries, Some(5)),
            _ => panic!("wrong policy"),
        }
    }

    #[test]
    fn parse_healthcheck_durations_to_nanos() {
        let yaml = r#"
services:
  app:
    image: x
    healthcheck:
      test: ["CMD", "curl", "http://localhost"]
      interval: 1m30s
      timeout: 500ms
      retries: 3
      start_period: 10s
"#;
        let spec = parse(yaml).unwrap();
        let hc = spec.services[0].container.healthcheck.as_ref().unwrap();
        assert_eq!(hc.test, vec!["CMD", "curl", "http://localhost"]);
        assert_eq!(hc.interval_ns, Some(90_000_000_000));
        assert_eq!(hc.timeout_ns, Some(500_000_000));
        assert_eq!(hc.retries, Some(3));
        assert_eq!(hc.start_period_ns, Some(10_000_000_000));
    }

    #[test]
    fn parse_collects_unsupported_top_level_and_service_keys() {
        // `configs:` is now supported (see configs.rs tests). Use an
        // x-extension to verify the unsupported-key plumbing still works.
        let yaml = r#"
services:
  app:
    image: x
    build: ./Dockerfile
    depends_on:
      - db
x-custom:
  ignored: true
"#;
        let spec = parse(yaml).unwrap();
        assert!(spec.unsupported.contains(&"x-custom".to_string()));
        let svc = &spec.services[0];
        assert!(svc.unsupported.contains(&"build".to_string()));
        assert!(svc.unsupported.contains(&"depends_on".to_string()));
    }

    #[test]
    fn parse_top_level_volumes_and_external_flag() {
        let yaml = r#"
services:
  app:
    image: x
volumes:
  data: {}
  shared:
    external: true
"#;
        let spec = parse(yaml).unwrap();
        assert_eq!(spec.volumes.len(), 2);
        let ext = spec.volumes.iter().find(|v| v.name == "shared").unwrap();
        assert!(ext.external);
    }

    #[test]
    fn parse_substitutes_variables_before_yaml() {
        let yaml = "services:\n  app:\n    image: nginx:${TAG:-latest}\n";
        let spec = parse(yaml).unwrap();
        assert_eq!(spec.services[0].container.image, "nginx:latest");
    }
}
