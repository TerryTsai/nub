//! Wire protocol — the JSON shapes that cross nub's HTTP and WebSocket
//! API. The transport layer (`server`, `client`) does not see these
//! types directly except through the `Frame` envelope.

mod frame;
mod op;
mod op_name;
mod op_scope;
mod stream;
mod types;

pub use frame::{Frame, OpResult};
pub use op::Op;
pub use stream::StreamChunk;
pub use types::{
    ContainerCreated, ContainerDetail, ContainerSummary, CreateContainerReq, DeviceMapping, DockerfileContent,
    DockerfileSummary, HealthcheckSpec, HostInfo, ImageDetail, ImageSummary, IpamConfig, MountPoint, NetworkContainer,
    NetworkDetail, NetworkEndpoint, NetworkSummary, PortMapping, PortPublish, RestartPolicySpec, SecretSummary,
    SecretValue, StackCreated, StackDetail, StackSummary, UlimitSpec, VolumeDetail, VolumeMount, VolumeSummary,
    WhoamiInfo,
};
