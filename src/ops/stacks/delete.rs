//! `delete_stack` — stop and remove all containers labeled with this
//! stack, drop the stack network (best-effort), then remove the manifest
//! directory. Named volumes are preserved so data survives a delete; the
//! user can prune volumes through the existing volume ops if they want.

use anyhow::Result;

use crate::auth::scope::Scope;
use crate::auth::Claims;
use crate::ops::configs;
use crate::ops::containers;
use crate::ops::secrets;
use crate::ops::EngineHandler;

use super::auth::require;
use super::discover::list_stack_containers;
use super::engine;
use super::labels::network_name;
use super::store;

pub(crate) async fn run(h: &EngineHandler, claims: &Claims, name: String) -> Result<()> {
    store::validate_name(&name)?;
    teardown_resources(h, claims, &name).await?;
    store::delete_dir(&h.policy.stacks_root, &name)?;
    Ok(())
}

/// Removes engine resources for a stack without touching the on-disk
/// manifest. Used by both `delete_stack` (which then drops the dir) and
/// `redeploy_stack` (which re-creates after).
///
/// Sub-op scopes (`containers:stop`, `containers:remove`, `networks:delete`)
/// are each checked against `claims` so the auth layer remains pure.
pub(super) async fn teardown_resources(h: &EngineHandler, claims: &Claims, name: &str) -> Result<()> {
    let stack_containers = list_stack_containers(h, name).await?;
    if !stack_containers.is_empty() {
        require(claims, Scope::ContainersStop)?;
        require(claims, Scope::ContainersRemove)?;
    }
    for c in &stack_containers {
        let _ = containers::action::stop(h, c.id.clone(), Some(10)).await;
    }
    for c in stack_containers {
        let _ = containers::action::remove(h, c.id.clone(), true).await;
    }
    require(claims, Scope::NetworksDelete)?;
    engine::remove_network(h, &network_name(name)).await?;
    secrets::runtime::cleanup_stack(name).await;
    configs::runtime::cleanup_stack(name).await;
    Ok(())
}
