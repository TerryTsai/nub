//! `update_stack` — replace the on-disk manifest with new YAML, then
//! redeploy. Pre-parses the new YAML so a bad paste fails before we
//! tear anything down.

use anyhow::{ensure, Context, Result};

use crate::auth::Claims;
use crate::compose;
use crate::ops::EngineHandler;
use crate::proto::StackCreated;

use super::create;
use super::delete;
use super::store;

pub(crate) async fn run(h: &EngineHandler, claims: &Claims, name: String, yaml: String) -> Result<StackCreated> {
    store::validate_name(&name)?;
    ensure!(store::exists(&h.policy.stacks_root, &name), "stack `{name}` not found");
    let spec = compose::parse_no_env(&yaml).context("compose")?;
    ensure!(!spec.services.is_empty(), "stack `{name}` has no services");
    store::write_yaml(&h.policy.stacks_root, &name, &yaml)?;
    delete::teardown_resources(h, claims, &name).await?;
    let ids = create::deploy_from_spec(h, claims, &name, spec).await?;
    Ok(StackCreated {
        name,
        container_ids: ids,
    })
}
