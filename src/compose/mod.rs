//! Parse compose-shaped YAML into a stack spec that nub deploys via its
//! existing container/network/volume ops. We don't reimplement compose
//! orchestration (no `depends_on` ordering, no healthcheck-conditioned
//! startup, no diff/recreate); we just translate the YAML into nub
//! primitives and let the engine do the rest.

mod configs;
mod duration;
mod secrets;
mod spec;
mod substitute;
mod transform;
mod wire;

#[cfg(test)]
mod tests;

pub use spec::{ParseError, ServiceConfigRef, ServiceSecretRef, ServiceSpec, StackSpec};

use std::collections::HashMap;

pub fn parse(yaml: &str, env: &HashMap<String, String>) -> Result<StackSpec, ParseError> {
    let substituted = substitute::substitute(yaml, env).map_err(|e| ParseError(e.to_string()))?;
    let raw: wire::Compose = serde_yaml::from_str(&substituted).map_err(|e| ParseError(format!("yaml: {e}")))?;
    transform::transform(raw)
}

/// Parse without env substitution. Stack ops never run YAML through a
/// shell-style env, so this is the form every internal caller wants;
/// `parse` stays available for any future call site that does.
pub fn parse_no_env(yaml: &str) -> Result<StackSpec, ParseError> {
    parse(yaml, &HashMap::new())
}
