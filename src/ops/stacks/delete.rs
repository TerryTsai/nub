//! `delete_stack` — stop and remove all containers labeled with this
//! stack, drop the stack network (best-effort), then remove the manifest
//! directory. Named volumes are preserved so data survives a delete; the
//! user can prune volumes through the existing volume ops if they want.

use anyhow::Result;

use crate::ops::containers;
use crate::ops::secrets;
use crate::ops::EngineHandler;
use crate::proto::Action;

use super::discover::list_stack_containers;
use super::engine;
use super::labels::network_name;
use super::store;

pub(crate) async fn run(h: &EngineHandler, name: String) -> Result<()> {
    store::validate_name(&name)?;
    teardown_resources(h, &name).await?;
    store::delete_dir(&h.policy.stacks_root, &name)?;
    Ok(())
}

/// Removes engine resources for a stack without touching the on-disk
/// manifest. Used by both `delete_stack` (which then drops the dir) and
/// `redeploy_stack` (which re-creates after).
pub(super) async fn teardown_resources(h: &EngineHandler, name: &str) -> Result<()> {
    let containers = list_stack_containers(h, name).await?;
    for c in &containers {
        let _ = containers::action::run(h, c.id.clone(), Action::Stop { timeout: Some(10) }).await;
    }
    for c in containers {
        let _ = containers::action::run(
            h,
            c.id.clone(),
            Action::Remove {
                force: true,
                volumes: false,
            },
        )
        .await;
    }
    engine::remove_network(h, &network_name(name)).await?;
    secrets::runtime::cleanup_stack(name).await;
    Ok(())
}
