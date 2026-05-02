//! Networks: list, remove. Podman's compat `/networks` shares the same
//! brittleness as `/containers/json` — a single bad container can 500 the
//! entire endpoint. We use libpod's `/v4.0.0/libpod/networks/json` on Podman
//! to avoid that, and accept the lowercase field shape it returns.

use serde::Deserialize;

use super::util::short_id;
use super::{Engine, EngineKind, Req, Result};

#[derive(Debug, Clone)]
pub struct NetworkSummary {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub scope: String,
    pub created: String,
    pub internal: bool,
}

impl Engine {
    pub async fn list_networks(&self) -> Result<Vec<NetworkSummary>> {
        match self.kind() {
            EngineKind::Podman => list_libpod(self).await,
            EngineKind::Docker => list_compat(self).await,
        }
    }

    pub async fn remove_network(&self, id: &str) -> Result<()> {
        let path = format!("/networks/{id}");
        let mut conn = self.conn().await?;
        conn.send_unary(Req::delete(path).build()?).await?.ok()
    }
}

async fn list_compat(engine: &Engine) -> Result<Vec<NetworkSummary>> {
    let mut conn = engine.conn().await?;
    let raw: Vec<CompatNet> = conn.send_unary(Req::get("/networks").build()?).await?.json()?;
    Ok(raw.into_iter().map(CompatNet::into_summary).collect())
}

async fn list_libpod(engine: &Engine) -> Result<Vec<NetworkSummary>> {
    let mut conn = engine.conn().await?;
    let raw: Vec<LibpodNet> = conn
        .send_unary(Req::get("/v4.0.0/libpod/networks/json").build()?)
        .await?
        .json()?;
    Ok(raw.into_iter().map(LibpodNet::into_summary).collect())
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct CompatNet {
    #[serde(default, rename = "Id")]
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    driver: String,
    #[serde(default)]
    scope: String,
    #[serde(default)]
    created: String,
    #[serde(default)]
    internal: bool,
}

impl CompatNet {
    fn into_summary(self) -> NetworkSummary {
        NetworkSummary {
            id: short_id(&self.id),
            name: self.name,
            driver: self.driver,
            scope: self.scope,
            created: self.created,
            internal: self.internal,
        }
    }
}

#[derive(Deserialize)]
struct LibpodNet {
    #[serde(default)]
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    driver: String,
    #[serde(default)]
    created: String,
    #[serde(default)]
    internal: bool,
}

impl LibpodNet {
    fn into_summary(self) -> NetworkSummary {
        NetworkSummary {
            id: short_id(&self.id),
            name: self.name,
            driver: self.driver,
            // libpod doesn't report a `scope` field; leave empty so callers
            // see the same shape regardless of engine.
            scope: String::new(),
            created: self.created,
            internal: self.internal,
        }
    }
}
