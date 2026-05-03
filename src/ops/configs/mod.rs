//! Compose `configs:` runtime — materialize inline content to tmpfs at
//! deploy time and bind-mount read-only into containers. No CLI noun
//! today; configs live inline in the compose YAML.

pub mod runtime;
