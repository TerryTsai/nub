// Hand-mirrored from src/proto/{mod,types,create}.rs.
// Adjacent serde tagging on OpResult: { type: "...", data: ... }.

export type Op =
  | { op: "host_info" }
  | { op: "whoami" }
  | { op: "list_containers"; all: boolean }
  | { op: "get_container"; id: string }
  | { op: "container_action"; id: string; action: Action }
  | { op: "create_container"; image: string; name?: string; cmd?: string[]; entrypoint?: string[]; env?: string[]; working_dir?: string; user?: string; labels?: Record<string, string>; ports?: PortPublish[]; volumes?: VolumeMount[]; network?: string; restart?: RestartPolicySpec; memory_limit?: number; cpu_shares?: number; start?: boolean }
  | { op: "stream_logs"; id: string; follow?: boolean; tail?: number }
  | { op: "stream_stats"; id: string }
  | { op: "exec"; id: string; cmd: string[]; tty?: boolean }
  | { op: "list_images" }
  | { op: "get_image"; id: string }
  | { op: "delete_image"; id: string; force?: boolean }
  | { op: "pull_image"; reference: string }
  | { op: "build_image"; dockerfile: string; tag: string; build_args: Record<string, string> }
  | { op: "list_volumes" }
  | { op: "get_volume"; name: string }
  | { op: "delete_volume"; name: string; force?: boolean }
  | { op: "list_networks" }
  | { op: "get_network"; id: string }
  | { op: "create_network"; name: string; internal?: boolean }
  | { op: "delete_network"; id: string }
  | { op: "list_dockerfiles" }
  | { op: "get_dockerfile"; name: string }
  | { op: "put_dockerfile"; name: string; content: string }
  | { op: "delete_dockerfile"; name: string }
  | { op: "create_stack"; name: string; yaml: string }
  | { op: "list_stacks" }
  | { op: "get_stack"; name: string }
  | { op: "delete_stack"; name: string }
  | { op: "redeploy_stack"; name: string }
  | { op: "update_stack"; name: string; yaml: string }
  | { op: "pull_stack"; name: string }
  | { op: "stream_stack_logs"; name: string; follow?: boolean; tail?: number }
  | { op: "list_secrets" }
  | { op: "put_secret"; name: string; value: string }
  | { op: "delete_secret"; name: string }
  | { op: "get_secret"; name: string };

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
  | { type: "image_detail"; data: ImageDetail }
  | { type: "volume_detail"; data: VolumeDetail }
  | { type: "network_detail"; data: NetworkDetail }
  | { type: "dockerfiles"; data: DockerfileSummary[] }
  | { type: "dockerfile"; data: DockerfileContent }
  | { type: "stacks"; data: StackSummary[] }
  | { type: "stack_detail"; data: StackDetail }
  | { type: "stack_created"; data: StackCreated }
  | { type: "secrets"; data: SecretSummary[] }
  | { type: "secret"; data: SecretValue }
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
  /** Healthcheck state: "healthy" | "unhealthy" | "starting" | "" (no healthcheck). */
  health: string;
  labels: Record<string, string>;
}

export interface ContainerDetail {
  id: string; name: string; image: string; image_id: string; created: string;
  state: string; running: boolean; started_at: string; finished_at: string;
  exit_code: number; error: string; restart_count: number;
  health: string;
  cmd: string[]; entrypoint: string[]; env: string[]; working_dir: string; user: string;
  labels: Record<string, string>;
  network_mode: string; restart_policy: string; privileged: boolean; memory_limit: number;
  mounts: MountPoint[]; networks: Record<string, NetworkEndpoint>; ports: PortMapping[];
}

export interface MountPoint { kind: string; source: string; destination: string; mode: string; rw: boolean }
export interface NetworkEndpoint { ip_address: string; gateway: string; mac_address: string }
export interface PortMapping { container_port: string; host_ip: string; host_port: string }

export interface ImageSummary { id: string; repo_tag: string; created: number; size: number; containers: number }
export interface ImageDetail {
  id: string; repo_tags: string[]; repo_digests: string[]; created: string; size: number;
  architecture: string; os: string; author: string; comment: string;
  cmd: string[]; entrypoint: string[]; env: string[]; working_dir: string; user: string;
  exposed_ports: string[]; labels: Record<string, string>; layers: number;
}
export interface VolumeSummary { name: string; driver: string; mountpoint: string; created_at: string; scope: string; in_use: boolean }
export interface VolumeDetail {
  name: string; driver: string; mountpoint: string; created_at: string; scope: string;
  labels: Record<string, string>; options: Record<string, string>;
  ref_count: number; size: number;
}
export interface NetworkSummary { id: string; name: string; driver: string; scope: string; created: string; internal: boolean; in_use: boolean }
export interface NetworkDetail {
  id: string; name: string; driver: string; scope: string; created: string; internal: boolean;
  ipam: { subnet: string; gateway: string }[];
  containers: { id: string; name: string; ipv4: string; ipv6: string }[];
  options: Record<string, string>;
  labels: Record<string, string>;
}
export interface ContainerCreated { id: string; started: boolean; warnings: string[] }
export interface DockerfileSummary { name: string; size: number; modified_at: string }
export interface DockerfileContent { name: string; content: string; size: number; modified_at: string }

export interface StackSummary {
  name: string;
  /** "active" | "idle" | "pending". */
  status: string;
  container_count: number;
  modified_at: string;
}
export interface StackDetail {
  name: string;
  yaml: string;
  modified_at: string;
  containers: ContainerSummary[];
  /** Network nub created for this stack; empty before first deploy. */
  network_name: string;
  /** Top-level compose keys we recognized but didn't translate. */
  unsupported: string[];
  /** Per-service compose keys we recognized but didn't translate. */
  service_unsupported: Record<string, string[]>;
}
export interface StackCreated { name: string; container_ids: string[] }

export interface SecretSummary { name: string; size: number; modified_at: string }
export interface SecretValue { name: string; value: string }

// Stream chunks (internally tagged with `type`).
export type StreamChunk =
  | { type: "log"; stderr: boolean; data: string }
  | { type: "stats"; cpu_pct: number; mem_used: number; mem_limit: number; net_rx: number; net_tx: number }
  | { type: "lagging"; dropped: number }
  | { type: "stdin"; data: string }
  | { type: "stdin_close" }
  | { type: "pull_progress"; id: string; status: string; current: number; total: number }
  | { type: "build_progress"; stream: string; image_id: string | null }
  | { type: "end"; ok: boolean; err: string | null };

// Frame (internally tagged with `kind`).
export type Frame =
  | { kind: "request"; id: number; op: Op }
  | { kind: "response"; id: number; result: OpResult }
  | { kind: "stream"; id: number; chunk: StreamChunk };
