//! `list_stacks` — enumerate stacks from disk and reconcile each with
//! the running containers it owns. Single engine call (`list_containers`)
//! shared across all stacks for efficiency.

use std::collections::HashMap;

use anyhow::Result;

use crate::ops::containers;
use crate::ops::EngineHandler;
use crate::proto::{ContainerSummary, StackSummary};

use super::discover::rollup_status;
use super::labels::STACK_LABEL;
use super::store;

pub(crate) async fn run(h: &EngineHandler) -> Result<Vec<StackSummary>> {
    let names = store::list_names(&h.policy.stacks_root)?;
    let by_stack = group_by_stack(containers::list::run(h, true).await?);
    let mut out = Vec::with_capacity(names.len());
    for name in names {
        let cs: &[ContainerSummary] = by_stack.get(&name).map(Vec::as_slice).unwrap_or(&[]);
        out.push(StackSummary {
            modified_at: store::modified_at(&h.policy.stacks_root, &name),
            container_count: cs.len() as u32,
            status: rollup_status(cs).into(),
            name,
        });
    }
    Ok(out)
}

fn group_by_stack(all: Vec<ContainerSummary>) -> HashMap<String, Vec<ContainerSummary>> {
    let mut out: HashMap<String, Vec<ContainerSummary>> = HashMap::new();
    for c in all {
        if let Some(stack) = c.labels.get(STACK_LABEL).cloned() {
            out.entry(stack).or_default().push(c);
        }
    }
    out
}
