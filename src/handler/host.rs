use crate::proto::HostInfo;
use anyhow::Result;

use super::EngineHandler;

impl EngineHandler {
    pub(super) async fn host_info(&self) -> Result<HostInfo> {
        let info = self.engine.host_info().await?;
        Ok(HostInfo {
            engine: info.engine,
            version: info.version,
            os: info.os,
            arch: info.arch,
            kernel: info.kernel,
            cpus: info.cpus,
            mem_total: info.mem_total,
            containers_running: info.containers_running,
            containers_total: info.containers_total,
            images: info.images,
        })
    }
}
