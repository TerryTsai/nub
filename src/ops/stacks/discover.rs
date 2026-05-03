//! Discover stack-managed resources via labels. Slice-2 contract:
//! containers and networks created by a stack carry `nub.stack=<name>`.

use anyhow::Result;

use crate::ops::containers;
use crate::ops::EngineHandler;
use crate::proto::ContainerSummary;

use super::labels::STACK_LABEL;

pub(super) async fn list_stack_containers(h: &EngineHandler, stack: &str) -> Result<Vec<ContainerSummary>> {
    let mut all = containers::list::run(h, true).await?;
    all.retain(|c| c.labels.get(STACK_LABEL).map(String::as_str) == Some(stack));
    Ok(all)
}

/// Status rollup for the stack. `active` = all running, `idle` = none
/// running, `pending` = mixed or zero containers found (i.e. the stack
/// exists on disk but its services aren't materialized).
pub(super) fn rollup_status(containers: &[ContainerSummary]) -> &'static str {
    if containers.is_empty() {
        return "pending";
    }
    let running = containers.iter().filter(|c| c.state == "running").count();
    if running == containers.len() {
        return "active";
    }
    if running == 0 {
        return "idle";
    }
    "pending"
}
