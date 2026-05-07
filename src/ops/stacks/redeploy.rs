//! `redeploy_stack` — tear down running resources, then re-deploy from
//! the stored manifest. Brief downtime per stack; acceptable at the
//! homelab "3 services" scale we target.

use anyhow::{anyhow, Result};

use crate::auth::Claims;
use crate::compose;
use crate::ops::EngineHandler;
use crate::proto::StackCreated;

use super::create;
use super::delete;
use super::store;

pub(crate) async fn run(h: &EngineHandler, claims: &Claims, name: String) -> Result<StackCreated> {
    store::validate_name(&name)?;
    if !store::exists(&h.policy.stacks_root, &name) {
        return Err(anyhow!("stack `{name}` not found"));
    }
    let yaml = store::read_yaml(&h.policy.stacks_root, &name)?;
    let spec = compose::parse_no_env(&yaml).map_err(|e| anyhow!("compose: {e}"))?;
    delete::teardown_resources(h, claims, &name).await?;
    let ids = create::deploy_from_spec(h, claims, &name, spec).await?;
    Ok(StackCreated {
        name,
        container_ids: ids,
    })
}
