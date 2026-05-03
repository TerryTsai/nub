//! `get_stack` — read the manifest, parse for unsupported-keys metadata,
//! and join with the containers currently owned by this stack.

use std::collections::HashMap;

use anyhow::Result;

use crate::compose;
use crate::ops::EngineHandler;
use crate::proto::StackDetail;

use super::discover::list_stack_containers;
use super::labels::network_name;
use super::store;

pub(crate) async fn run(h: &EngineHandler, name: String) -> Result<Box<StackDetail>> {
    store::validate_name(&name)?;
    let yaml = store::read_yaml(&h.policy.stacks_root, &name)?;
    let modified_at = store::modified_at(&h.policy.stacks_root, &name);
    let containers = list_stack_containers(h, &name).await?;
    let parsed = compose::parse(&yaml, &HashMap::new()).ok();
    let (unsupported, service_unsupported) = match parsed {
        Some(spec) => {
            let svc_map: HashMap<String, Vec<String>> = spec
                .services
                .into_iter()
                .filter(|s| !s.unsupported.is_empty())
                .map(|s| (s.name, s.unsupported))
                .collect();
            (spec.unsupported, svc_map)
        }
        None => (Vec::new(), HashMap::new()),
    };
    Ok(Box::new(StackDetail {
        network_name: if containers.is_empty() {
            String::new()
        } else {
            network_name(&name)
        },
        name,
        yaml,
        modified_at,
        containers,
        unsupported,
        service_unsupported,
    }))
}
