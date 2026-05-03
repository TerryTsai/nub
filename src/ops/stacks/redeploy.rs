//! `redeploy_stack` — tear down running resources, then re-deploy from
//! the stored manifest. Brief downtime per stack; acceptable at the
//! homelab "3 services" scale we target.

use std::collections::HashMap;

use anyhow::{anyhow, Result};

use crate::compose;
use crate::ops::EngineHandler;
use crate::proto::StackCreated;

use super::create;
use super::delete;
use super::store;

pub(crate) async fn run(h: &EngineHandler, name: String) -> Result<StackCreated> {
    store::validate_name(&name)?;
    if !store::exists(&h.policy.stacks_root, &name) {
        return Err(anyhow!("stack `{name}` not found"));
    }
    let yaml = store::read_yaml(&h.policy.stacks_root, &name)?;
    let spec = compose::parse(&yaml, &HashMap::new()).map_err(|e| anyhow!("compose: {e}"))?;
    delete::teardown_resources(h, &name).await?;
    let ids = create::deploy_from_spec(h, &name, spec).await?;
    Ok(StackCreated { name, container_ids: ids })
}
