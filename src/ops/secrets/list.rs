//! List the secret names known to nub. Values are never returned by
//! this op — the privileged `get` op (admin-only) is the only path
//! that exposes plaintext over the wire.

use std::path::Path;

use anyhow::Result;

use super::store;
use crate::proto::SecretSummary;

pub async fn run(root: &Path) -> Result<Vec<SecretSummary>> {
    store::list(root).await
}
