use crate::proto::HostInfo;
use anyhow::Result;

use super::DockerHandler;

impl DockerHandler {
    pub(super) async fn host_info(&self) -> Result<HostInfo> {
        let info = self.docker.info().await?;
        let ver = self.docker.version().await?;
        Ok(HostInfo {
            engine: ver.platform.map(|p| p.name).unwrap_or_else(|| "docker".into()),
            version: ver.version.unwrap_or_default(),
            os: info.operating_system.unwrap_or_default(),
            arch: info.architecture.unwrap_or_default(),
            kernel: info.kernel_version.unwrap_or_default(),
            cpus: info.ncpu.unwrap_or(0) as u64,
            mem_total: info.mem_total.unwrap_or(0) as u64,
            containers_running: info.containers_running.unwrap_or(0) as u64,
            containers_total: info.containers.unwrap_or(0) as u64,
            images: info.images.unwrap_or(0) as u64,
        })
    }
}
