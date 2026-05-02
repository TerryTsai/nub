//! `/info` and `/version` — combined into one `HostInfo` since callers
//! always want both.

use serde::Deserialize;

use super::{Engine, EngineKind, Req, Result};

#[derive(Debug, Clone)]
pub struct HostInfo {
    pub engine: String,  // e.g. "docker", "podman"
    pub version: String, // engine version string
    pub os: String,
    pub arch: String,
    pub kernel: String,
    pub cpus: u64,
    pub mem_total: u64,
    pub containers_running: u64,
    pub containers_total: u64,
    pub images: u64,
}

impl Engine {
    pub async fn host_info(&self) -> Result<HostInfo> {
        let mut conn = self.conn().await?;
        let info: InfoResp = conn
            .send_unary(Req::get("/info").build()?)
            .await?
            .json()?;

        // /version and /info both work on docker and podman compat. Use them
        // together so we can populate `engine` and `version` cleanly.
        let mut conn2 = self.conn().await?;
        let version: VersionResp = conn2
            .send_unary(Req::get("/version").build()?)
            .await?
            .json()?;

        let engine = match self.kind() {
            EngineKind::Podman => "podman".to_string(),
            EngineKind::Docker => version
                .platform
                .as_ref()
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "docker".to_string()),
        };

        Ok(HostInfo {
            engine,
            version: version.version,
            os: info.operating_system,
            arch: info.architecture,
            kernel: info.kernel_version,
            cpus: info.ncpu,
            mem_total: info.mem_total,
            containers_running: info.containers_running,
            containers_total: info.containers,
            images: info.images,
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct InfoResp {
    operating_system: String,
    architecture: String,
    kernel_version: String,
    #[serde(rename = "NCPU")]
    ncpu: u64,
    mem_total: u64,
    containers_running: u64,
    containers: u64,
    images: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct VersionResp {
    version: String,
    #[serde(default)]
    platform: Option<Platform>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Platform {
    name: String,
}
