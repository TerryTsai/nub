use crate::proto::*;
use anyhow::Result;
use async_trait::async_trait;
use bollard::container::LogOutput;
use bollard::errors::Error as BollardError;
use bollard::models::{
    ContainerInspectResponse, ContainerSummary as RawSummary, CreateImageInfo,
    EndpointSettings as RawEndpoint, ImageSummary as RawImage, MountPoint as RawMount,
    Network as RawNetwork, PortBinding, Volume as RawVolume,
};
use bollard::Docker;
use futures::stream::{BoxStream, Stream, StreamExt};
use std::future::Future;
use std::pin::Pin;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::sync::mpsc;

type ExecOutput = Pin<Box<dyn Stream<Item = std::result::Result<LogOutput, BollardError>> + Send>>;
type ExecInput = Pin<Box<dyn AsyncWrite + Send>>;

pub enum HandlerOutput {
    Unary(OpResult),
    Stream(BoxStream<'static, StreamChunk>),
}

#[async_trait]
pub trait OpHandler: Send + Sync + 'static {
    async fn handle(&self, op: Op, input: mpsc::Receiver<StreamChunk>) -> HandlerOutput;
}

/// A receiver pre-closed at construction. Use for transports that don't support
/// client-to-server stream chunks (e.g. unary HTTP /op).
pub fn closed_input() -> mpsc::Receiver<StreamChunk> {
    let (_tx, rx) = mpsc::channel::<StreamChunk>(1);
    rx
}

pub struct DockerHandler {
    docker: Docker,
}

impl DockerHandler {
    pub fn connect() -> Result<Self> {
        Ok(Self {
            docker: Docker::connect_with_local_defaults()?,
        })
    }

    async fn host_info(&self) -> Result<HostInfo> {
        let info = self.docker.info().await?;
        let ver = self.docker.version().await?;
        Ok(HostInfo {
            engine: ver
                .platform
                .map(|p| p.name)
                .unwrap_or_else(|| "docker".into()),
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

    async fn list_containers(&self, all: bool) -> Result<Vec<ContainerSummary>> {
        use bollard::container::ListContainersOptions;
        let opts = ListContainersOptions::<String> {
            all,
            ..Default::default()
        };
        let cs = self.docker.list_containers(Some(opts)).await?;
        Ok(cs.into_iter().map(summarize).collect())
    }

    async fn inspect_container(&self, id: String) -> Result<ContainerDetail> {
        let r = self.docker.inspect_container(&id, None).await?;
        Ok(to_detail(r))
    }

    async fn container_action(&self, id: String, action: Action) -> Result<()> {
        use bollard::container::{
            KillContainerOptions, RemoveContainerOptions, RestartContainerOptions,
            StopContainerOptions,
        };
        match action {
            Action::Start => {
                self.docker.start_container::<String>(&id, None).await?;
            }
            Action::Stop { timeout } => {
                let opts = timeout.map(|t| StopContainerOptions { t });
                self.docker.stop_container(&id, opts).await?;
            }
            Action::Restart { timeout } => {
                let opts = timeout.map(|t| RestartContainerOptions { t: t as isize });
                self.docker.restart_container(&id, opts).await?;
            }
            Action::Kill { signal } => {
                let opts = signal.map(|s| KillContainerOptions { signal: s });
                self.docker.kill_container(&id, opts).await?;
            }
            Action::Remove { force, volumes } => {
                let opts = RemoveContainerOptions {
                    force,
                    v: volumes,
                    link: false,
                };
                self.docker.remove_container(&id, Some(opts)).await?;
            }
        }
        Ok(())
    }

    fn stream_logs(
        &self,
        id: String,
        follow: bool,
        tail: Option<u32>,
    ) -> BoxStream<'static, StreamChunk> {
        let docker = self.docker.clone();
        spawn_chunked(move |tx| run_logs(docker, id, follow, tail, tx))
    }

    fn stream_stats(&self, id: String) -> BoxStream<'static, StreamChunk> {
        let docker = self.docker.clone();
        spawn_chunked(move |tx| run_stats(docker, id, tx))
    }

    fn exec(
        &self,
        id: String,
        cmd: Vec<String>,
        tty: bool,
        input: mpsc::Receiver<StreamChunk>,
    ) -> BoxStream<'static, StreamChunk> {
        let docker = self.docker.clone();
        spawn_chunked(move |tx| run_exec(docker, id, cmd, tty, input, tx))
    }

    async fn list_images(&self) -> Result<Vec<ImageSummary>> {
        let imgs = self.docker.list_images::<String>(None).await?;
        Ok(imgs.into_iter().map(summarize_image).collect())
    }

    async fn remove_image(&self, id: String, force: bool) -> Result<()> {
        use bollard::image::RemoveImageOptions;
        let opts = RemoveImageOptions {
            force,
            noprune: false,
        };
        self.docker.remove_image(&id, Some(opts), None).await?;
        Ok(())
    }

    fn pull_image(&self, reference: String) -> BoxStream<'static, StreamChunk> {
        let docker = self.docker.clone();
        spawn_chunked(move |tx| run_pull(docker, reference, tx))
    }

    async fn list_volumes(&self) -> Result<Vec<VolumeSummary>> {
        let resp = self.docker.list_volumes::<String>(None).await?;
        Ok(resp
            .volumes
            .unwrap_or_default()
            .into_iter()
            .map(summarize_volume)
            .collect())
    }

    async fn remove_volume(&self, name: String, force: bool) -> Result<()> {
        use bollard::volume::RemoveVolumeOptions;
        let opts = RemoveVolumeOptions { force };
        self.docker.remove_volume(&name, Some(opts)).await?;
        Ok(())
    }

    async fn list_networks(&self) -> Result<Vec<NetworkSummary>> {
        let nets = self.docker.list_networks::<String>(None).await?;
        Ok(nets.into_iter().map(summarize_network).collect())
    }

    async fn remove_network(&self, id: String) -> Result<()> {
        self.docker.remove_network(&id).await?;
        Ok(())
    }
}

async fn run_logs(
    docker: Docker,
    id: String,
    follow: bool,
    tail: Option<u32>,
    tx: mpsc::Sender<StreamChunk>,
) -> std::result::Result<(), String> {
    use bollard::container::{LogOutput, LogsOptions};
    let opts = LogsOptions::<String> {
        follow,
        stdout: true,
        stderr: true,
        tail: tail.map(|n| n.to_string()).unwrap_or_else(|| "all".into()),
        ..Default::default()
    };
    let logs = docker.logs(&id, Some(opts));
    futures::pin_mut!(logs);
    while let Some(item) = logs.next().await {
        let chunk = match item {
            Ok(LogOutput::StdOut { message }) => log_chunk(false, &message),
            Ok(LogOutput::StdErr { message }) => log_chunk(true, &message),
            Ok(_) => continue,
            Err(e) => return Err(e.to_string()),
        };
        if tx.send(chunk).await.is_err() {
            return Ok(());
        }
    }
    Ok(())
}

async fn run_stats(
    docker: Docker,
    id: String,
    tx: mpsc::Sender<StreamChunk>,
) -> std::result::Result<(), String> {
    use bollard::container::StatsOptions;
    let opts = StatsOptions {
        stream: true,
        one_shot: false,
    };
    let stats = docker.stats(&id, Some(opts));
    futures::pin_mut!(stats);
    while let Some(item) = stats.next().await {
        let chunk = match item {
            Ok(s) => to_stats_chunk(&s),
            Err(e) => return Err(e.to_string()),
        };
        if tx.send(chunk).await.is_err() {
            return Ok(());
        }
    }
    Ok(())
}

async fn run_exec(
    docker: Docker,
    id: String,
    cmd: Vec<String>,
    tty: bool,
    in_rx: mpsc::Receiver<StreamChunk>,
    out_tx: mpsc::Sender<StreamChunk>,
) -> std::result::Result<(), String> {
    use bollard::exec::{CreateExecOptions, StartExecOptions, StartExecResults};
    let exec = docker
        .create_exec(
            &id,
            CreateExecOptions::<String> {
                cmd: Some(cmd),
                attach_stdin: Some(true),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                tty: Some(tty),
                ..Default::default()
            },
        )
        .await
        .map_err(|e| e.to_string())?;
    let started = docker
        .start_exec(
            &exec.id,
            Some(StartExecOptions {
                detach: false,
                tty,
                ..Default::default()
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    let (output, input) = match started {
        StartExecResults::Attached { output, input } => (output, input),
        StartExecResults::Detached => return Err("exec returned detached".into()),
    };
    let stdin_task = tokio::spawn(pump_exec_stdin(input, in_rx));
    let result = pump_exec_output(output, out_tx).await;
    stdin_task.abort();
    result
}

async fn pump_exec_stdin(mut input: ExecInput, mut in_rx: mpsc::Receiver<StreamChunk>) {
    while let Some(chunk) = in_rx.recv().await {
        if !forward_input(&mut input, chunk).await {
            break;
        }
    }
}

async fn forward_input(input: &mut ExecInput, chunk: StreamChunk) -> bool {
    match chunk {
        StreamChunk::Stdin { data } => input.write_all(data.as_bytes()).await.is_ok(),
        StreamChunk::StdinClose => {
            let _ = input.shutdown().await;
            false
        }
        _ => true,
    }
}

async fn pump_exec_output(
    mut output: ExecOutput,
    tx: mpsc::Sender<StreamChunk>,
) -> std::result::Result<(), String> {
    while let Some(item) = output.next().await {
        let chunk = match item {
            Ok(LogOutput::StdOut { message }) => log_chunk(false, &message),
            Ok(LogOutput::StdErr { message }) => log_chunk(true, &message),
            Ok(_) => continue,
            Err(e) => return Err(e.to_string()),
        };
        if tx.send(chunk).await.is_err() {
            return Ok(());
        }
    }
    Ok(())
}

#[async_trait]
impl OpHandler for DockerHandler {
    async fn handle(&self, op: Op, input: mpsc::Receiver<StreamChunk>) -> HandlerOutput {
        match op {
            Op::HostInfo => unary(self.host_info().await, OpResult::HostInfo),
            Op::ListContainers { all } => {
                unary(self.list_containers(all).await, OpResult::Containers)
            }
            Op::StreamLogs { id, follow, tail } => {
                HandlerOutput::Stream(self.stream_logs(id, follow, tail))
            }
            Op::StreamStats { id } => HandlerOutput::Stream(self.stream_stats(id)),
            Op::Exec { id, cmd, tty } => HandlerOutput::Stream(self.exec(id, cmd, tty, input)),
            Op::InspectContainer { id } => unary(self.inspect_container(id).await, |d| {
                OpResult::ContainerDetail(Box::new(d))
            }),
            Op::ContainerAction { id, action } => {
                unary(self.container_action(id, action).await, |()| OpResult::Ok)
            }
            Op::ListImages => unary(self.list_images().await, OpResult::Images),
            Op::RemoveImage { id, force } => {
                unary(self.remove_image(id, force).await, |()| OpResult::Ok)
            }
            Op::PullImage { reference } => HandlerOutput::Stream(self.pull_image(reference)),
            Op::ListVolumes => unary(self.list_volumes().await, OpResult::Volumes),
            Op::RemoveVolume { name, force } => {
                unary(self.remove_volume(name, force).await, |()| OpResult::Ok)
            }
            Op::ListNetworks => unary(self.list_networks().await, OpResult::Networks),
            Op::RemoveNetwork { id } => unary(self.remove_network(id).await, |()| OpResult::Ok),
        }
    }
}

fn unary<T>(r: Result<T>, into: impl FnOnce(T) -> OpResult) -> HandlerOutput {
    HandlerOutput::Unary(match r {
        Ok(v) => into(v),
        Err(e) => OpResult::Err {
            message: e.to_string(),
        },
    })
}

/// Spawns `produce` on a task with a Sender for emitting chunks. When `produce`
/// returns, an `End { ok, err }` chunk is appended automatically. Returns the
/// receiving end as a BoxStream<'static>.
fn spawn_chunked<F, Fut>(produce: F) -> BoxStream<'static, StreamChunk>
where
    F: FnOnce(mpsc::Sender<StreamChunk>) -> Fut + Send + 'static,
    Fut: Future<Output = std::result::Result<(), String>> + Send + 'static,
{
    let (tx, rx) = mpsc::channel::<StreamChunk>(32);
    let inner = tx.clone();
    tokio::spawn(async move {
        let (ok, err) = match produce(inner).await {
            Ok(()) => (true, None),
            Err(e) => (false, Some(e)),
        };
        let _ = tx.send(StreamChunk::End { ok, err }).await;
    });
    Box::pin(futures::stream::unfold(rx, |mut rx| async move {
        rx.recv().await.map(|c| (c, rx))
    }))
}

fn summarize(c: RawSummary) -> ContainerSummary {
    ContainerSummary {
        id: short_id(&c.id.unwrap_or_default()),
        name: c
            .names
            .and_then(|n| n.into_iter().next())
            .map(|n| n.trim_start_matches('/').to_string())
            .unwrap_or_default(),
        image: c.image.unwrap_or_default(),
        state: c.state.unwrap_or_default(),
        status: c.status.unwrap_or_default(),
        created: c.created.unwrap_or(0),
    }
}

fn summarize_image(i: RawImage) -> ImageSummary {
    ImageSummary {
        id: short_id(&i.id),
        repo_tag: i
            .repo_tags
            .into_iter()
            .next()
            .unwrap_or_else(|| "<none>".into()),
        created: i.created,
        size: i.size,
        containers: i.containers,
    }
}

fn summarize_volume(v: RawVolume) -> VolumeSummary {
    VolumeSummary {
        name: v.name,
        driver: v.driver,
        mountpoint: v.mountpoint,
        created_at: v.created_at.map(|d| d.to_string()).unwrap_or_default(),
        scope: v.scope.map(|s| s.to_string()).unwrap_or_default(),
    }
}

fn summarize_network(n: RawNetwork) -> NetworkSummary {
    NetworkSummary {
        id: short_id(&n.id.unwrap_or_default()),
        name: n.name.unwrap_or_default(),
        driver: n.driver.unwrap_or_default(),
        scope: n.scope.unwrap_or_default(),
        created: n.created.map(|d| d.to_string()).unwrap_or_default(),
        internal: n.internal.unwrap_or(false),
    }
}

fn short_id(id: &str) -> String {
    id.strip_prefix("sha256:")
        .unwrap_or(id)
        .chars()
        .take(12)
        .collect()
}

async fn run_pull(
    docker: Docker,
    reference: String,
    tx: mpsc::Sender<StreamChunk>,
) -> std::result::Result<(), String> {
    use bollard::image::CreateImageOptions;
    let opts = CreateImageOptions::<String> {
        from_image: reference,
        ..Default::default()
    };
    let stream = docker.create_image(Some(opts), None, None);
    futures::pin_mut!(stream);
    while let Some(item) = stream.next().await {
        let info = item.map_err(|e| e.to_string())?;
        if let Some(err) = info.error {
            return Err(err);
        }
        if tx.send(pull_chunk(info)).await.is_err() {
            return Ok(());
        }
    }
    Ok(())
}

fn pull_chunk(info: CreateImageInfo) -> StreamChunk {
    let (current, total) = info
        .progress_detail
        .map(|d| (d.current.unwrap_or(0) as u64, d.total.unwrap_or(0) as u64))
        .unwrap_or((0, 0));
    StreamChunk::PullProgress {
        id: info.id.unwrap_or_default(),
        status: info.status.unwrap_or_default(),
        current,
        total,
    }
}

fn log_chunk(stderr: bool, msg: &[u8]) -> StreamChunk {
    StreamChunk::Log {
        stderr,
        data: String::from_utf8_lossy(msg).into_owned(),
    }
}

fn to_detail(r: ContainerInspectResponse) -> ContainerDetail {
    let state = r.state.unwrap_or_default();
    let config = r.config.unwrap_or_default();
    let host = r.host_config.unwrap_or_default();
    let net = r.network_settings.unwrap_or_default();
    ContainerDetail {
        id: r.id.unwrap_or_default(),
        name: r
            .name
            .map(|n| n.trim_start_matches('/').to_string())
            .unwrap_or_default(),
        image: config.image.clone().unwrap_or_default(),
        image_id: r.image.unwrap_or_default(),
        created: r.created.unwrap_or_default(),
        state: state.status.map(|s| s.to_string()).unwrap_or_default(),
        running: state.running.unwrap_or(false),
        started_at: state.started_at.unwrap_or_default(),
        finished_at: state.finished_at.unwrap_or_default(),
        exit_code: state.exit_code.unwrap_or(0),
        error: state.error.unwrap_or_default(),
        restart_count: r.restart_count.unwrap_or(0),
        cmd: config.cmd.unwrap_or_default(),
        entrypoint: config.entrypoint.unwrap_or_default(),
        env: config.env.unwrap_or_default(),
        working_dir: config.working_dir.unwrap_or_default(),
        user: config.user.unwrap_or_default(),
        labels: config.labels.unwrap_or_default(),
        network_mode: host.network_mode.unwrap_or_default(),
        restart_policy: host
            .restart_policy
            .and_then(|p| p.name.map(|n| n.to_string()))
            .unwrap_or_default(),
        privileged: host.privileged.unwrap_or(false),
        memory_limit: host.memory.unwrap_or(0),
        mounts: r
            .mounts
            .unwrap_or_default()
            .into_iter()
            .map(to_mount)
            .collect(),
        networks: net
            .networks
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k, to_endpoint(v)))
            .collect(),
        ports: ports_from_map(net.ports),
    }
}

fn to_mount(m: RawMount) -> MountPoint {
    MountPoint {
        kind: m.typ.map(|t| t.to_string()).unwrap_or_default(),
        source: m.source.unwrap_or_default(),
        destination: m.destination.unwrap_or_default(),
        mode: m.mode.unwrap_or_default(),
        rw: m.rw.unwrap_or(false),
    }
}

fn to_endpoint(e: RawEndpoint) -> NetworkEndpoint {
    NetworkEndpoint {
        ip_address: e.ip_address.unwrap_or_default(),
        gateway: e.gateway.unwrap_or_default(),
        mac_address: e.mac_address.unwrap_or_default(),
    }
}

fn ports_from_map(
    map: Option<std::collections::HashMap<String, Option<Vec<PortBinding>>>>,
) -> Vec<PortMapping> {
    let Some(map) = map else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (port, bindings) in map {
        for b in bindings.unwrap_or_default() {
            out.push(PortMapping {
                container_port: port.clone(),
                host_ip: b.host_ip.unwrap_or_default(),
                host_port: b.host_port.unwrap_or_default(),
            });
        }
    }
    out
}

fn to_stats_chunk(s: &bollard::container::Stats) -> StreamChunk {
    let cpu_delta =
        s.cpu_stats.cpu_usage.total_usage as i128 - s.precpu_stats.cpu_usage.total_usage as i128;
    let sys_delta = s.cpu_stats.system_cpu_usage.unwrap_or(0) as i128
        - s.precpu_stats.system_cpu_usage.unwrap_or(0) as i128;
    let online = s.cpu_stats.online_cpus.unwrap_or_else(|| {
        s.cpu_stats
            .cpu_usage
            .percpu_usage
            .as_ref()
            .map(|p| p.len() as u64)
            .unwrap_or(1)
    });
    let cpu_pct = if sys_delta > 0 && cpu_delta > 0 {
        (cpu_delta as f64 / sys_delta as f64) * online as f64 * 100.0
    } else {
        0.0
    };
    let (net_rx, net_tx) = s
        .networks
        .as_ref()
        .map(|nets| {
            nets.values().fold((0u64, 0u64), |(rx, tx), n| {
                (rx.saturating_add(n.rx_bytes), tx.saturating_add(n.tx_bytes))
            })
        })
        .unwrap_or((0, 0));
    StreamChunk::Stats {
        cpu_pct,
        mem_used: s.memory_stats.usage.unwrap_or(0),
        mem_limit: s.memory_stats.limit.unwrap_or(0),
        net_rx,
        net_tx,
    }
}
