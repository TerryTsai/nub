use crate::compose::parse;
use std::collections::HashMap;

#[test]
fn parse_configs_inline_content_short_and_long_form() {
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
    let spec = parse(yaml, &HashMap::new()).unwrap();
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
fn parse_configs_file_source_rejected() {
    let yaml = r#"
services:
  app:
    image: x
configs:
  cfg:
    file: ./cfg
"#;
    let err = parse(yaml, &HashMap::new()).unwrap_err().to_string();
    assert!(err.contains("cfg"), "got: {err}");
    assert!(err.contains("content:"), "got: {err}");
}

#[test]
fn parse_configs_external_rejected() {
    let yaml = r#"
services:
  app:
    image: x
configs:
  cfg:
    external: true
"#;
    let err = parse(yaml, &HashMap::new()).unwrap_err().to_string();
    assert!(err.contains("external"), "got: {err}");
    assert!(err.contains("content:"), "got: {err}");
}

#[test]
fn parse_configs_environment_rejected() {
    let yaml = r#"
services:
  app:
    image: x
configs:
  cfg:
    environment: SOME_VAR
"#;
    let err = parse(yaml, &HashMap::new()).unwrap_err().to_string();
    assert!(err.contains("environment"), "got: {err}");
}

#[test]
fn parse_configs_requires_content() {
    let yaml = r#"
services:
  app:
    image: x
configs:
  cfg: {}
"#;
    let err = parse(yaml, &HashMap::new()).unwrap_err().to_string();
    assert!(err.contains("content:"), "got: {err}");
}

#[test]
fn parse_service_config_must_be_declared() {
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
    let err = parse(yaml, &HashMap::new()).unwrap_err().to_string();
    assert!(err.contains("rogue"), "got: {err}");
    assert!(err.contains("isn't declared"), "got: {err}");
}

#[test]
fn parse_config_target_relative_lands_at_root() {
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
    let spec = parse(yaml, &HashMap::new()).unwrap();
    assert_eq!(spec.services[0].configs[0].target, "/app.conf");
}
