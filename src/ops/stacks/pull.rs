//! `pull_stack` — pull each service's image, then redeploy. Unary —
//! we don't stream layer-by-layer progress at the stack level (the
//! per-image pull op stays available for that). Stack pull blocks on
//! the engine until all images are present.

use std::collections::HashSet;

use anyhow::{anyhow, bail, Result};

use crate::auth::scope::Scope;
use crate::auth::Claims;
use crate::compose;
use crate::ops::images;
use crate::ops::EngineHandler;
use crate::proto::StackCreated;

use super::redeploy;
use super::store;

pub(crate) async fn run(h: &EngineHandler, claims: &Claims, name: String) -> Result<StackCreated> {
    store::validate_name(&name)?;
    if !store::exists(&h.policy.stacks_root, &name) {
        return Err(anyhow!("stack `{name}` not found"));
    }
    let yaml = store::read_yaml(&h.policy.stacks_root, &name)?;
    let spec = compose::parse_no_env(&yaml).map_err(|e| anyhow!("compose: {e}"))?;
    let unique_images: HashSet<&str> = spec
        .services
        .iter()
        .map(|s| s.container.image.as_str())
        .filter(|i| !i.is_empty())
        .collect();
    if !unique_images.is_empty() && !claims.allows_scope(Scope::ImagesPull) {
        bail!("missing scope: {}", Scope::ImagesPull);
    }
    for img in unique_images {
        images::pull::run_unary(h, img).await?;
    }
    redeploy::run(h, claims, name).await
}
