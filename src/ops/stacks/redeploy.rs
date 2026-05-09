//! `redeploy_stack` — tear down running resources, then re-deploy from
//! the stored manifest. Brief downtime per stack; acceptable at the
//! homelab "3 services" scale we target.

use anyhow::{ensure, Context, Result};

use crate::auth::Claims;
use crate::compose;
use crate::ops::EngineHandler;
use crate::proto::StackCreated;

use super::create;
use super::delete;
use super::store;

pub(crate) async fn run(h: &EngineHandler, claims: &Claims, name: String) -> Result<StackCreated> {
    store::validate_name(&name)?;
    ensure!(store::exists(&h.policy.stacks_root, &name), "stack `{name}` not found");
    let yaml = store::read_yaml(&h.policy.stacks_root, &name)?;
    let spec = compose::parse(&yaml).context("compose")?;
    delete::teardown_resources(h, claims, &name).await?;
    let ids = create::deploy_from_spec(h, claims, &name, spec).await?;
    Ok(StackCreated {
        name,
        container_ids: ids,
    })
}
