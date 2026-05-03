use crate::compose::parse;
use std::collections::HashMap;

#[test]
fn parse_secrets_external_short_and_long_form() {
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
fn parse_secrets_file_source_rejected_with_clear_error() {
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
fn parse_secrets_requires_external_true() {
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
fn parse_service_secret_must_be_declared() {
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
fn parse_secret_target_relative_goes_under_run_secrets() {
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
fn parse_secret_name_override_threads_through_lookup() {
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
