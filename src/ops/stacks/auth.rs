//! Sub-op scope checks for stack orchestrators. Stack ops compose other
//! resource actions (pull, create network, create volume, create + start
//! container, stop + remove container, delete network) — each step is
//! gated against the caller's token here so the auth layer remains pure:
//! `stacks:create` alone authorizes invocation, but every sub-action
//! also requires its own scope.

use anyhow::{anyhow, Result};

use crate::auth::scope::Scope;
use crate::auth::Claims;

pub(super) fn require(claims: &Claims, scope: Scope) -> Result<()> {
    if claims.allows_scope(scope) {
        Ok(())
    } else {
        Err(anyhow!("missing scope: {}", scope))
    }
}
