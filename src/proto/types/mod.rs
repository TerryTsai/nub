//! Wire types for op responses, grouped by domain. Each submodule
//! holds the request, summary, and detail shapes for one resource
//! family.

mod container;
mod dockerfile;
mod host;
mod image;
mod network;
mod secret;
mod stack;
mod volume;

pub use container::{
    ContainerCreated, ContainerDetail, ContainerSummary, CreateContainerReq, DeviceMapping, HealthcheckSpec,
    MountPoint, NetworkEndpoint, PortMapping, PortPublish, RestartPolicySpec, UlimitSpec, VolumeMount,
};
pub use dockerfile::{DockerfileContent, DockerfileSummary};
pub use host::{HostInfo, WhoamiInfo};
pub use image::{ImageDetail, ImageSummary};
pub use network::{IpamConfig, NetworkContainer, NetworkDetail, NetworkSummary};
pub use secret::{SecretSummary, SecretValue};
pub use stack::{StackCreated, StackDetail, StackSummary};
pub use volume::{VolumeDetail, VolumeSummary};
