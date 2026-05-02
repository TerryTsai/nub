// Hand-mirrored from src/proto/{mod,types,create}.rs.
// Adjacent serde tagging on OpResult: { type: "...", data: ... }.

export type Op =
  | { op: "host_info" }
  | { op: "whoami" }
  | { op: "list_containers"; all: boolean }
  | { op: "inspect_container"; id: string }
  | { op: "container_action"; id: string; action: Action }
  | { op: "create_container"; image: string; name?: string; cmd?: string[]; entrypoint?: string[]; env?: string[]; working_dir?: string; user?: string; labels?: Record<string, string>; ports?: PortPublish[]; volumes?: VolumeMount[]; network?: string; restart?: RestartPolicySpec; memory_limit?: number; cpu_shares?: number; start?: boolean }
  | { op: "stream_logs"; id: string; follow?: boolean; tail?: number }
  | { op: "stream_stats"; id: string }
  | { op: "exec"; id: string; cmd: string[]; tty?: boolean }
  | { op: "list_images" }
  | { op: "remove_image"; id: string; force?: boolean }
  | { op: "pull_image"; reference: string }
  | { op: "list_volumes" }
  | { op: "remove_volume"; name: string; force?: boolean }
  | { op: "list_networks" }
  | { op: "remove_network"; id: string }
  | { op: "list_dockerfiles" }
  | { op: "read_dockerfile"; name: string }
  | { op: "write_dockerfile"; name: string; content: string }
  | { op: "delete_dockerfile"; name: string };

export type Action =
  | { kind: "start" }
  | { kind: "stop"; timeout?: number }
  | { kind: "restart"; timeout?: number }
  | { kind: "kill"; signal?: string }
  | { kind: "remove"; force?: boolean; volumes?: boolean };

export interface PortPublish { container: string; host: string }
export interface VolumeMount { source: string; target: string; read_only?: boolean }
export type RestartPolicySpec =
  | { kind: "no" }
  | { kind: "on_failure"; max_retries?: number }
  | { kind: "always" }
  | { kind: "unless_stopped" };

// Adjacent-tagged OpResult.
export type OpResult =
  | { type: "host_info"; data: HostInfo }
  | { type: "whoami"; data: WhoamiInfo }
  | { type: "containers"; data: ContainerSummary[] }
  | { type: "container_detail"; data: ContainerDetail }
  | { type: "images"; data: ImageSummary[] }
  | { type: "volumes"; data: VolumeSummary[] }
  | { type: "networks"; data: NetworkSummary[] }
  | { type: "container_created"; data: ContainerCreated }
  | { type: "dockerfiles"; data: DockerfileSummary[] }
  | { type: "dockerfile"; data: DockerfileContent }
  | { type: "ok" }
  | { type: "stream_started" }
  | { type: "err"; data: { message: string } };

export interface WhoamiInfo { id: string; allowed: string[] }

export interface HostInfo {
  engine: string; version: string; os: string; arch: string; kernel: string;
  cpus: number; mem_total: number;
  containers_running: number; containers_total: number; images: number;
}

export interface ContainerSummary {
  id: string; name: string; image: string; state: string; status: string; created: string;
  exit_code: number;
}

export interface ContainerDetail {
  id: string; name: string; image: string; image_id: string; created: string;
  state: string; running: boolean; started_at: string; finished_at: string;
  exit_code: number; error: string; restart_count: number;
  cmd: string[]; entrypoint: string[]; env: string[]; working_dir: string; user: string;
  labels: Record<string, string>;
  network_mode: string; restart_policy: string; privileged: boolean; memory_limit: number;
  mounts: MountPoint[]; networks: Record<string, NetworkEndpoint>; ports: PortMapping[];
}

export interface MountPoint { kind: string; source: string; destination: string; mode: string; rw: boolean }
export interface NetworkEndpoint { ip_address: string; gateway: string; mac_address: string }
export interface PortMapping { container_port: string; host_ip: string; host_port: string }

export interface ImageSummary { id: string; repo_tag: string; created: number; size: number; containers: number }
export interface VolumeSummary { name: string; driver: string; mountpoint: string; created_at: string; scope: string; in_use: boolean }
export interface NetworkSummary { id: string; name: string; driver: string; scope: string; created: string; internal: boolean; in_use: boolean }
export interface ContainerCreated { id: string; started: boolean; warnings: string[] }
export interface DockerfileSummary { name: string; size: number; modified_at: string }
export interface DockerfileContent { name: string; content: string; size: number; modified_at: string }

// Stream chunks (internally tagged with `type`).
export type StreamChunk =
  | { type: "log"; stderr: boolean; data: string }
  | { type: "stats"; cpu_pct: number; mem_used: number; mem_limit: number; net_rx: number; net_tx: number }
  | { type: "lagging"; dropped: number }
  | { type: "stdin"; data: string }
  | { type: "stdin_close" }
  | { type: "pull_progress"; id: string; status: string; current: number; total: number }
  | { type: "end"; ok: boolean; err: string | null };

// Frame (internally tagged with `kind`).
export type Frame =
  | { kind: "request"; id: number; op: Op }
  | { kind: "response"; id: number; result: OpResult }
  | { kind: "stream"; id: number; chunk: StreamChunk };
