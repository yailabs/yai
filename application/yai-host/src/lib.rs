//! Resident same-user YAI application host for one explicit `YAI_HOME`.
//!
//! The host owns process lifecycle, local IPC, ephemeral client attachments,
//! update observation and telemetry. It carries `yai-application` requests; it
//! owns no Case, authority, Workflow, provider or RuntimeInstance semantics.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
#[cfg(unix)]
use std::os::fd::AsRawFd;
#[cfg(unix)]
use std::os::unix::fs::{FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt};
#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use yai_application::{
    CaseUpdate, LocalApplication, OperationRequest, OperationResult, ResultState,
};
use yai_core_engine::context::stable_digest;
use yai_core_engine::resource_control::LocalProcessIdentity;

pub const HOST_PROTOCOL: &str = "yai.local_host.v1";
pub const HOST_DISCOVERY_SCHEMA: &str = "yai.local_host.discovery.v1";
pub const HOST_TELEMETRY_SCHEMA: &str = "yai.local_host.telemetry.v1";
const MAX_FRAME_BYTES: usize = 8 * 1024 * 1024;
const EVENT_POLL: Duration = Duration::from_millis(500);
const START_TIMEOUT: Duration = Duration::from_secs(8);
const CLIENT_COUNTER_START: u64 = 1;

static CLIENT_COUNTER: AtomicU64 = AtomicU64::new(CLIENT_COUNTER_START);

/// Operational supervision only: these facts neither admit work nor describe
/// its outcome. The scheduler remains responsible for leases and recovery.
#[derive(Clone, Copy, Debug)]
pub enum RuntimeSupervisionPosture {
    WaitingForIdentity,
    Starting,
    Attached,
    Running,
    Stopped,
    Failed,
}

impl RuntimeSupervisionPosture {
    fn label(self) -> &'static str {
        match self {
            Self::WaitingForIdentity => "waiting_for_identity",
            Self::Starting => "starting",
            Self::Attached => "attached_existing_runtime",
            Self::Running => "supervised_running",
            Self::Stopped => "stopped",
            Self::Failed => "failed",
        }
    }
}

/// In-process Host lifetime, not a serializable authorization token. A client
/// disconnect never clears it. The runtime must drain its own workers on stop.
#[derive(Clone)]
pub struct RuntimeSupervisionControl {
    running: Arc<AtomicBool>,
    state: Arc<Mutex<SharedState>>,
}

impl RuntimeSupervisionControl {
    pub fn host_is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    pub fn report(&self, posture: RuntimeSupervisionPosture) {
        if let Ok(mut state) = self.state.lock() {
            state.runtime_supervision = posture.label().into();
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientKind {
    Studio,
    Cli,
    Qualification,
}

impl ClientKind {
    fn label(&self) -> &'static str {
        match self {
            Self::Studio => "studio",
            Self::Cli => "cli",
            Self::Qualification => "qualification",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HostDiscovery {
    pub schema: String,
    pub protocol: String,
    pub endpoint: String,
    pub pid: u32,
    pub process_identity: LocalProcessIdentity,
    pub instance_id: String,
    pub started_at_unix_ms: u64,
    pub version: String,
    pub build: String,
    pub yai_home: String,
    pub yai_home_identity: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientAttachment {
    pub client_id: String,
    pub client_kind: ClientKind,
    pub pid: u32,
    pub connected_at_unix_ms: u64,
    pub last_seen_unix_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HostTelemetry {
    pub schema: String,
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at_unix_ms: Option<u64>,
    pub uptime_ms: u64,
    pub protocol: String,
    pub version: String,
    pub build: String,
    pub yai_home: String,
    pub yai_home_identity: String,
    pub transport: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    pub endpoint_posture: String,
    pub application_readiness: String,
    pub connected_clients: usize,
    pub client_kinds: BTreeMap<String, usize>,
    pub clients: Vec<ClientAttachment>,
    pub event_sequence: u64,
    pub last_activity_unix_ms: u64,
    pub runtime_supervision: String,
}

impl HostTelemetry {
    fn stopped(home: &Path) -> Result<Self, String> {
        let canonical = canonical_home(home)?;
        Ok(Self {
            schema: HOST_TELEMETRY_SCHEMA.into(),
            state: "stopped".into(),
            pid: None,
            process_identity: None,
            instance_id: None,
            started_at_unix_ms: None,
            uptime_ms: 0,
            protocol: HOST_PROTOCOL.into(),
            version: env!("CARGO_PKG_VERSION").into(),
            build: build_identity(),
            yai_home: canonical.display().to_string(),
            yai_home_identity: home_identity(&canonical),
            transport: "unix_domain_socket".into(),
            endpoint: None,
            endpoint_posture: "absent".into(),
            application_readiness: "host_absent".into(),
            connected_clients: 0,
            client_kinds: BTreeMap::new(),
            clients: Vec::new(),
            event_sequence: 0,
            last_activity_unix_ms: now_ms(),
            runtime_supervision: "not_integrated".into(),
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ClientFrame {
    Handshake {
        protocol: String,
        client_id: String,
        client_kind: ClientKind,
        pid: u32,
        yai_home_identity: String,
    },
    ApplicationRequest {
        request: OperationRequest,
    },
    Subscribe,
    HostControl {
        action: HostControl,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum HostControl {
    Status,
    Shutdown,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ServerFrame {
    Handshake {
        protocol: String,
        host_instance_id: String,
        yai_home_identity: String,
    },
    ApplicationResponse {
        result: OperationResult,
    },
    Subscribed {
        host_instance_id: String,
    },
    Event {
        event: CaseUpdate,
    },
    Heartbeat {
        telemetry: HostTelemetry,
    },
    HostStatus {
        telemetry: HostTelemetry,
    },
    Shutdown {
        reason: String,
    },
    Error {
        code: String,
        message: String,
    },
}

#[derive(Clone, Debug)]
pub enum HostEvent {
    Case(CaseUpdate),
    Heartbeat(HostTelemetry),
    Shutdown(String),
}

struct SharedState {
    discovery: HostDiscovery,
    clients: BTreeMap<String, ClientAttachment>,
    subscribers: BTreeMap<String, Sender<ServerFrame>>,
    event_sequence: u64,
    last_activity_unix_ms: u64,
    application_readiness: String,
    runtime_supervision: String,
}

impl SharedState {
    fn attach(&mut self, attachment: ClientAttachment) {
        self.last_activity_unix_ms = now_ms();
        self.clients
            .insert(attachment.client_id.clone(), attachment);
    }

    fn touch(&mut self, client_id: &str) {
        let now = now_ms();
        if let Some(client) = self.clients.get_mut(client_id) {
            client.last_seen_unix_ms = now;
        }
        self.last_activity_unix_ms = now;
    }

    fn detach(&mut self, client_id: &str) {
        self.clients.remove(client_id);
        self.subscribers.remove(client_id);
        self.last_activity_unix_ms = now_ms();
    }

    fn telemetry(&self, exclude: Option<&str>) -> HostTelemetry {
        let clients = self
            .clients
            .values()
            .filter(|client| {
                exclude != Some(client.client_id.as_str())
                    && self.subscribers.contains_key(&client.client_id)
            })
            .cloned()
            .collect::<Vec<_>>();
        let mut kinds = BTreeMap::new();
        for client in &clients {
            *kinds
                .entry(client.client_kind.label().to_string())
                .or_insert(0) += 1;
        }
        HostTelemetry {
            schema: HOST_TELEMETRY_SCHEMA.into(),
            state: "running".into(),
            pid: Some(self.discovery.pid),
            process_identity: Some(self.discovery.process_identity.canonical_identity()),
            instance_id: Some(self.discovery.instance_id.clone()),
            started_at_unix_ms: Some(self.discovery.started_at_unix_ms),
            uptime_ms: now_ms().saturating_sub(self.discovery.started_at_unix_ms),
            protocol: self.discovery.protocol.clone(),
            version: self.discovery.version.clone(),
            build: self.discovery.build.clone(),
            yai_home: self.discovery.yai_home.clone(),
            yai_home_identity: self.discovery.yai_home_identity.clone(),
            transport: "unix_domain_socket".into(),
            endpoint: Some(self.discovery.endpoint.clone()),
            endpoint_posture: "private_same_user".into(),
            application_readiness: self.application_readiness.clone(),
            connected_clients: clients.len(),
            client_kinds: kinds,
            clients,
            event_sequence: self.event_sequence,
            last_activity_unix_ms: self.last_activity_unix_ms,
            runtime_supervision: self.runtime_supervision.clone(),
        }
    }

    fn broadcast(&mut self, frame: ServerFrame) {
        self.last_activity_unix_ms = now_ms();
        let mut disconnected = Vec::new();
        for (client_id, sender) in &self.subscribers {
            if sender.send(frame.clone()).is_err() {
                disconnected.push(client_id.clone());
            }
        }
        for client_id in disconnected {
            self.detach(&client_id);
        }
    }
}

pub struct HostServer {
    #[cfg(unix)]
    listener: UnixListener,
    home: PathBuf,
    socket_path: PathBuf,
    discovery_path: PathBuf,
    _lock: File,
    state: Arc<Mutex<SharedState>>,
    running: Arc<AtomicBool>,
    application: Arc<LocalApplication>,
}

impl HostServer {
    #[cfg(unix)]
    pub fn bind(home: impl AsRef<Path>) -> Result<Self, String> {
        #[cfg(not(target_os = "linux"))]
        return Err("host_platform_unqualified".into());

        let home = canonical_home(home.as_ref())?;
        let paths = HostPaths::prepare(&home)?;
        let lock = acquire_singleton_lock(&paths.lock)?;
        reclaim_stale_files(&paths)?;
        let process_identity = LocalProcessIdentity::capture(std::process::id())?;
        let started_at_unix_ms = now_ms();
        let instance_id = format!(
            "host-instance:{}",
            stable_digest(&format!(
                "{}:{}:{}",
                home.display(),
                process_identity.canonical_identity(),
                started_at_unix_ms
            ))
        );
        let listener = UnixListener::bind(&paths.socket)
            .map_err(|error| format!("host_endpoint_bind_failed:{error}"));
        let listener = listener?;
        fs::set_permissions(&paths.socket, fs::Permissions::from_mode(0o600))
            .map_err(|error| format!("host_endpoint_permissions_failed:{error}"))?;
        listener
            .set_nonblocking(true)
            .map_err(|error| format!("host_endpoint_nonblocking_failed:{error}"))?;
        let discovery = HostDiscovery {
            schema: HOST_DISCOVERY_SCHEMA.into(),
            protocol: HOST_PROTOCOL.into(),
            endpoint: paths.socket.display().to_string(),
            pid: std::process::id(),
            process_identity,
            instance_id,
            started_at_unix_ms,
            version: env!("CARGO_PKG_VERSION").into(),
            build: build_identity(),
            yai_home: home.display().to_string(),
            yai_home_identity: home_identity(&home),
        };
        write_discovery(&paths.discovery, &discovery)?;
        let application = Arc::new(LocalApplication::from_yai_home(&home));
        let readiness = application.call(OperationRequest {
            protocol: yai_application::APPLICATION_PROTOCOL.into(),
            operation_ref: "runtime.readiness".into(),
            correlation_ref: "host:startup:readiness".into(),
            input: serde_json::json!({}),
        });
        let application_readiness = result_state_label(readiness.result_state).to_string();
        append_log(
            &paths.log,
            &format!(
                "host_started instance={} pid={}",
                discovery.instance_id, discovery.pid
            ),
        )?;
        Ok(Self {
            listener,
            home,
            socket_path: paths.socket,
            discovery_path: paths.discovery,
            _lock: lock,
            state: Arc::new(Mutex::new(SharedState {
                discovery,
                clients: BTreeMap::new(),
                subscribers: BTreeMap::new(),
                event_sequence: 0,
                last_activity_unix_ms: started_at_unix_ms,
                application_readiness,
                runtime_supervision: "not_integrated".into(),
            })),
            running: Arc::new(AtomicBool::new(true)),
            application,
        })
    }

    #[cfg(not(unix))]
    pub fn bind(_home: impl AsRef<Path>) -> Result<Self, String> {
        Err("host_platform_unqualified".into())
    }

    #[cfg(unix)]
    pub fn run(self) -> Result<(), String> {
        let result = self.run_service();
        append_log(&self.home.join("log/yai-host.log"), "host_stopped")?;
        result
    }

    /// The executable composes the existing scheduler here. No CLI parsing,
    /// command spawning, scheduler policy or work ledger lives in this crate.
    #[cfg(unix)]
    pub fn run_with_runtime<F>(self, runtime: F) -> Result<(), String>
    where
        F: FnOnce(RuntimeSupervisionControl) -> Result<(), String> + Send + 'static,
    {
        let control = RuntimeSupervisionControl {
            running: self.running.clone(),
            state: self.state.clone(),
        };
        control.report(RuntimeSupervisionPosture::Starting);
        let worker = thread::spawn(move || {
            let result = runtime(control.clone());
            control.report(if result.is_ok() {
                RuntimeSupervisionPosture::Stopped
            } else {
                RuntimeSupervisionPosture::Failed
            });
            result
        });
        let service_result = self.run_service();
        self.running.store(false, Ordering::Release);
        // Keep the singleton lock until the existing owner has drained. Do not
        // turn a transport shutdown into cancellation or kill prepared effects.
        let runtime_result = worker.join()
            .map_err(|_| "host_runtime_supervisor_panicked".to_string())?;
        append_log(&self.home.join("log/yai-host.log"), "host_stopped_runtime_joined")?;
        service_result.and(runtime_result)
    }

    #[cfg(unix)]
    fn run_service(&self) -> Result<(), String> {
        let poller = spawn_event_observer(
            self.application.clone(),
            self.state.clone(),
            self.running.clone(),
        );
        while self.running.load(Ordering::Acquire) {
            match self.listener.accept() {
                Ok((stream, _)) => {
                    let application = self.application.clone();
                    let state = self.state.clone();
                    let running = self.running.clone();
                    thread::spawn(move || {
                        let _ = handle_client(stream, application, state, running);
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(25));
                }
                Err(error) => {
                    self.running.store(false, Ordering::Release);
                    return Err(format!("host_accept_failed:{error}"));
                }
            }
        }
        if let Ok(mut state) = self.state.lock() {
            state.broadcast(ServerFrame::Shutdown {
                reason: "operator_stop".into(),
            });
        }
        let _ = poller.join();
        Ok(())
    }

    fn cleanup_owned_paths(&self) {
        let instance = self
            .state
            .lock()
            .ok()
            .map(|state| state.discovery.instance_id.clone());
        let owns_discovery = fs::read(&self.discovery_path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<HostDiscovery>(&bytes).ok())
            .is_some_and(|discovery| Some(discovery.instance_id) == instance);
        if owns_discovery {
            let _ = fs::remove_file(&self.discovery_path);
            let _ = fs::remove_file(&self.socket_path);
        }
    }
}

impl Drop for HostServer {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Release);
        self.cleanup_owned_paths();
    }
}

pub fn serve(home: impl AsRef<Path>) -> Result<(), String> {
    HostServer::bind(home)?.run()
}

pub struct HostClient {
    #[cfg(unix)]
    stream: UnixStream,
    #[cfg(unix)]
    reader: BufReader<UnixStream>,
    pub discovery: HostDiscovery,
}

impl HostClient {
    #[cfg(unix)]
    pub fn connect(home: impl AsRef<Path>, kind: ClientKind) -> Result<Self, String> {
        let home = canonical_home(home.as_ref())?;
        let discovery = read_discovery(&home)?;
        validate_discovery(&home, &discovery)?;
        let stream = UnixStream::connect(&discovery.endpoint)
            .map_err(|error| format!("host_transport_unavailable:{error}"))?;
        verify_peer_same_user(&stream)?;
        let reader_stream = stream
            .try_clone()
            .map_err(|error| format!("host_transport_clone_failed:{error}"))?;
        let mut client = Self {
            stream,
            reader: BufReader::new(reader_stream),
            discovery,
        };
        let client_id = next_client_id(kind.label());
        let frame = ClientFrame::Handshake {
            protocol: HOST_PROTOCOL.into(),
            client_id,
            client_kind: kind,
            pid: std::process::id(),
            yai_home_identity: home_identity(&home),
        };
        write_frame(&mut client.stream, &frame)?;
        match read_frame::<ServerFrame>(&mut client.reader)? {
            ServerFrame::Handshake {
                protocol,
                host_instance_id,
                yai_home_identity,
            } if protocol == HOST_PROTOCOL
                && host_instance_id == client.discovery.instance_id
                && yai_home_identity == client.discovery.yai_home_identity =>
            {
                Ok(client)
            }
            ServerFrame::Error { code, message } => Err(format!("{code}:{message}")),
            _ => Err("host_handshake_invalid".into()),
        }
    }

    #[cfg(not(unix))]
    pub fn connect(_home: impl AsRef<Path>, _kind: ClientKind) -> Result<Self, String> {
        Err("host_platform_unqualified".into())
    }

    #[cfg(unix)]
    pub fn call(mut self, request: OperationRequest) -> Result<OperationResult, String> {
        write_frame(
            &mut self.stream,
            &ClientFrame::ApplicationRequest { request },
        )?;
        match read_frame::<ServerFrame>(&mut self.reader)? {
            ServerFrame::ApplicationResponse { result } => Ok(result),
            ServerFrame::Error { code, message } => Err(format!("{code}:{message}")),
            _ => Err("host_application_response_invalid".into()),
        }
    }

    #[cfg(unix)]
    pub fn status(mut self) -> Result<HostTelemetry, String> {
        write_frame(
            &mut self.stream,
            &ClientFrame::HostControl {
                action: HostControl::Status,
            },
        )?;
        match read_frame::<ServerFrame>(&mut self.reader)? {
            ServerFrame::HostStatus { telemetry } => Ok(telemetry),
            ServerFrame::Error { code, message } => Err(format!("{code}:{message}")),
            _ => Err("host_status_response_invalid".into()),
        }
    }

    #[cfg(unix)]
    pub fn shutdown(mut self) -> Result<HostTelemetry, String> {
        write_frame(
            &mut self.stream,
            &ClientFrame::HostControl {
                action: HostControl::Shutdown,
            },
        )?;
        match read_frame::<ServerFrame>(&mut self.reader)? {
            ServerFrame::HostStatus { telemetry } => Ok(telemetry),
            ServerFrame::Error { code, message } => Err(format!("{code}:{message}")),
            _ => Err("host_shutdown_response_invalid".into()),
        }
    }

    #[cfg(unix)]
    pub fn subscribe<F>(mut self, mut event: F) -> Result<(), String>
    where
        F: FnMut(HostEvent) -> Result<(), String>,
    {
        write_frame(&mut self.stream, &ClientFrame::Subscribe)?;
        match read_frame::<ServerFrame>(&mut self.reader)? {
            ServerFrame::Subscribed { .. } => {}
            ServerFrame::Error { code, message } => return Err(format!("{code}:{message}")),
            _ => return Err("host_subscription_response_invalid".into()),
        }
        loop {
            match read_frame::<ServerFrame>(&mut self.reader)? {
                ServerFrame::Event { event: update } => event(HostEvent::Case(update))?,
                ServerFrame::Heartbeat { telemetry } => event(HostEvent::Heartbeat(telemetry))?,
                ServerFrame::Shutdown { reason } => {
                    event(HostEvent::Shutdown(reason))?;
                    return Ok(());
                }
                ServerFrame::Error { code, message } => return Err(format!("{code}:{message}")),
                _ => return Err("host_subscription_frame_invalid".into()),
            }
        }
    }
}

pub fn observe(home: impl AsRef<Path>) -> Result<HostTelemetry, String> {
    match HostClient::connect(home.as_ref(), ClientKind::Cli) {
        Ok(client) => client.status(),
        Err(error) if is_absent_error(&error) => HostTelemetry::stopped(home.as_ref()),
        Err(error) => Err(error),
    }
}

pub fn start(
    home: impl AsRef<Path>,
    executable: impl AsRef<Path>,
    serve_arguments: &[&str],
) -> Result<HostTelemetry, String> {
    let home = canonical_home(home.as_ref())?;
    if let Ok(status) = HostClient::connect(&home, ClientKind::Cli).and_then(HostClient::status) {
        return Ok(status);
    }
    let log_path = HostPaths::prepare(&home)?.log;
    let stdout = OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .open(&log_path)
        .map_err(|error| format!("host_log_open_failed:{error}"))?;
    let stderr = stdout
        .try_clone()
        .map_err(|error| format!("host_log_clone_failed:{error}"))?;
    let mut command = Command::new(executable.as_ref());
    command
        .args(serve_arguments)
        .env("YAI_HOME", &home)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    #[cfg(unix)]
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let mut child = command
        .spawn()
        .map_err(|error| format!("host_spawn_failed:{error}"))?;
    let deadline = std::time::Instant::now() + START_TIMEOUT;
    let mut child_exit = None;
    loop {
        if let Ok(status) = HostClient::connect(&home, ClientKind::Cli).and_then(HostClient::status)
        {
            return Ok(status);
        }
        if child_exit.is_none() {
            child_exit = child
                .try_wait()
                .map_err(|error| format!("host_spawn_wait_failed:{error}"))?;
        }
        if std::time::Instant::now() >= deadline {
            return Err(child_exit
                .map(|status| format!("host_start_failed:exit={status}"))
                .unwrap_or_else(|| "host_start_timeout".into()));
        }
        thread::sleep(Duration::from_millis(50));
    }
}

pub fn stop(home: impl AsRef<Path>) -> Result<HostTelemetry, String> {
    let home = canonical_home(home.as_ref())?;
    let prior = match HostClient::connect(&home, ClientKind::Cli) {
        Ok(client) => client.shutdown()?,
        Err(error) if is_absent_error(&error) => return HostTelemetry::stopped(&home),
        Err(error) => return Err(error),
    };
    let deadline = std::time::Instant::now() + START_TIMEOUT;
    let paths = HostPaths::prepare(&home)?;
    while std::time::Instant::now() < deadline {
        if HostClient::connect(&home, ClientKind::Cli).is_err()
            && !paths.discovery.exists()
            && !paths.socket.exists()
        {
            return HostTelemetry::stopped(&home);
        }
        thread::sleep(Duration::from_millis(50));
    }
    Err(format!(
        "host_stop_timeout:instance={}",
        prior.instance_id.unwrap_or_default()
    ))
}

pub fn restart(
    home: impl AsRef<Path>,
    executable: impl AsRef<Path>,
    serve_arguments: &[&str],
) -> Result<HostTelemetry, String> {
    let home = canonical_home(home.as_ref())?;
    let _ = stop(&home)?;
    start(&home, executable, serve_arguments)
}

pub fn recent_logs(home: impl AsRef<Path>, limit: usize) -> Result<Vec<String>, String> {
    let path = canonical_home(home.as_ref())?.join("log/yai-host.log");
    let text =
        fs::read_to_string(&path).map_err(|error| format!("host_log_unavailable:{error}"))?;
    let mut lines = text
        .lines()
        .rev()
        .take(limit.clamp(1, 1000))
        .map(str::to_string)
        .collect::<Vec<_>>();
    lines.reverse();
    Ok(lines)
}

#[cfg(unix)]
fn handle_client(
    mut stream: UnixStream,
    application: Arc<LocalApplication>,
    state: Arc<Mutex<SharedState>>,
    running: Arc<AtomicBool>,
) -> Result<(), String> {
    let peer = peer_credentials(&stream)?;
    if peer.uid != unsafe { libc::geteuid() } {
        let _ = write_frame(
            &mut stream,
            &ServerFrame::Error {
                code: "peer_uid_mismatch".into(),
                message: "Local Host accepts only the owning operating-system user.".into(),
            },
        );
        return Err("peer_uid_mismatch".into());
    }
    let reader_stream = stream
        .try_clone()
        .map_err(|error| format!("host_transport_clone_failed:{error}"))?;
    let mut reader = BufReader::new(reader_stream);
    let (client_id, client_kind, client_pid, requested_home) =
        match read_frame::<ClientFrame>(&mut reader)? {
            ClientFrame::Handshake {
                protocol,
                client_id,
                client_kind,
                pid,
                yai_home_identity,
            } => {
                let discovery = state
                    .lock()
                    .map_err(|_| "host_state_poisoned")?
                    .discovery
                    .clone();
                if protocol != HOST_PROTOCOL {
                    write_frame(
                        &mut stream,
                        &ServerFrame::Error {
                            code: "protocol_mismatch".into(),
                            message: format!("host={HOST_PROTOCOL} client={protocol}"),
                        },
                    )?;
                    return Err("protocol_mismatch".into());
                }
                if yai_home_identity != discovery.yai_home_identity {
                    write_frame(
                        &mut stream,
                        &ServerFrame::Error {
                            code: "yai_home_mismatch".into(),
                            message: "Client selected a different YAI_HOME.".into(),
                        },
                    )?;
                    return Err("yai_home_mismatch".into());
                }
                if pid == 0 || peer.pid != pid {
                    write_frame(
                        &mut stream,
                        &ServerFrame::Error {
                            code: "peer_pid_mismatch".into(),
                            message: "Client PID does not match local peer credentials.".into(),
                        },
                    )?;
                    return Err("peer_pid_mismatch".into());
                }
                if client_id.is_empty() || client_id.len() > 256 {
                    return Err("client_id_invalid".into());
                }
                (client_id, client_kind, pid, yai_home_identity)
            }
            _ => return Err("host_handshake_required".into()),
        };
    let discovery = state
        .lock()
        .map_err(|_| "host_state_poisoned")?
        .discovery
        .clone();
    debug_assert_eq!(requested_home, discovery.yai_home_identity);
    {
        let mut shared = state.lock().map_err(|_| "host_state_poisoned")?;
        shared.attach(ClientAttachment {
            client_id: client_id.clone(),
            client_kind,
            pid: client_pid,
            connected_at_unix_ms: now_ms(),
            last_seen_unix_ms: now_ms(),
        });
    }
    write_frame(
        &mut stream,
        &ServerFrame::Handshake {
            protocol: HOST_PROTOCOL.into(),
            host_instance_id: discovery.instance_id.clone(),
            yai_home_identity: discovery.yai_home_identity,
        },
    )?;
    let result = match read_frame::<ClientFrame>(&mut reader) {
        Ok(ClientFrame::ApplicationRequest { request }) => {
            state
                .lock()
                .map_err(|_| "host_state_poisoned")?
                .touch(&client_id);
            let result = application.call(request);
            write_frame(&mut stream, &ServerFrame::ApplicationResponse { result })
        }
        Ok(ClientFrame::HostControl {
            action: HostControl::Status,
        }) => {
            let telemetry = state
                .lock()
                .map_err(|_| "host_state_poisoned")?
                .telemetry(Some(&client_id));
            write_frame(&mut stream, &ServerFrame::HostStatus { telemetry })
        }
        Ok(ClientFrame::HostControl {
            action: HostControl::Shutdown,
        }) => {
            let mut telemetry = state
                .lock()
                .map_err(|_| "host_state_poisoned")?
                .telemetry(Some(&client_id));
            telemetry.state = "stopping".into();
            write_frame(&mut stream, &ServerFrame::HostStatus { telemetry })?;
            running.store(false, Ordering::Release);
            Ok(())
        }
        Ok(ClientFrame::Subscribe) => {
            let (sender, receiver) = mpsc::channel();
            {
                let mut shared = state.lock().map_err(|_| "host_state_poisoned")?;
                shared.subscribers.insert(client_id.clone(), sender);
            }
            write_frame(
                &mut stream,
                &ServerFrame::Subscribed {
                    host_instance_id: discovery.instance_id,
                },
            )?;
            loop {
                match receiver.recv_timeout(Duration::from_secs(2)) {
                    Ok(frame) => {
                        if write_frame(&mut stream, &frame).is_err() {
                            break;
                        }
                        if matches!(frame, ServerFrame::Shutdown { .. }) {
                            break;
                        }
                        state
                            .lock()
                            .map_err(|_| "host_state_poisoned")?
                            .touch(&client_id);
                    }
                    Err(RecvTimeoutError::Timeout) if running.load(Ordering::Acquire) => continue,
                    Err(_) => break,
                }
            }
            Ok(())
        }
        Ok(ClientFrame::Handshake { .. }) => Err("duplicate_handshake".into()),
        Err(error) => Err(error),
    };
    if let Ok(mut shared) = state.lock() {
        shared.detach(&client_id);
    }
    result
}

fn spawn_event_observer(
    application: Arc<LocalApplication>,
    state: Arc<Mutex<SharedState>>,
    running: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut generations = BTreeMap::new();
        let mut initialized = false;
        let mut last_heartbeat = 0;
        while running.load(Ordering::Acquire) {
            if let Ok(current) = application.visible_generations() {
                for (case_ref, generation) in &current {
                    let changed = generations
                        .get(case_ref)
                        .is_some_and(|known| known != generation);
                    let newly_visible = initialized && !generations.contains_key(case_ref);
                    if changed || newly_visible {
                        if let Ok(mut shared) = state.lock() {
                            shared.event_sequence = shared.event_sequence.saturating_add(1);
                            let update = application.update_for(
                                case_ref,
                                *generation,
                                shared.event_sequence,
                            );
                            shared.broadcast(ServerFrame::Event { event: update });
                        }
                    }
                }
                generations = current;
                initialized = true;
                if let Ok(mut shared) = state.lock() {
                    shared.application_readiness = "success".into();
                }
            } else if let Ok(mut shared) = state.lock() {
                shared.application_readiness = "error".into();
            }
            let now = now_ms();
            if now.saturating_sub(last_heartbeat) >= 1_000 {
                if let Ok(mut shared) = state.lock() {
                    let telemetry = shared.telemetry(None);
                    shared.broadcast(ServerFrame::Heartbeat { telemetry });
                }
                last_heartbeat = now;
            }
            thread::sleep(EVENT_POLL);
        }
    })
}

struct HostPaths {
    socket: PathBuf,
    discovery: PathBuf,
    lock: PathBuf,
    log: PathBuf,
}

impl HostPaths {
    fn prepare(home: &Path) -> Result<Self, String> {
        let run = home.join("run").join("host");
        let log_root = home.join("log");
        fs::create_dir_all(&run).map_err(|error| format!("host_run_root_failed:{error}"))?;
        fs::create_dir_all(&log_root).map_err(|error| format!("host_log_root_failed:{error}"))?;
        #[cfg(unix)]
        {
            fs::set_permissions(&run, fs::Permissions::from_mode(0o700))
                .map_err(|error| format!("host_run_permissions_failed:{error}"))?;
            fs::set_permissions(&log_root, fs::Permissions::from_mode(0o700))
                .map_err(|error| format!("host_log_permissions_failed:{error}"))?;
        }
        Ok(Self {
            socket: run.join("application.sock"),
            discovery: run.join("discovery.json"),
            lock: run.join("host.lock"),
            log: log_root.join("yai-host.log"),
        })
    }
}

#[cfg(unix)]
fn acquire_singleton_lock(path: &Path) -> Result<File, String> {
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(|error| format!("host_lock_open_failed:{error}"))?;
    let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if result != 0 {
        return Err("already_running".into());
    }
    Ok(file)
}

#[cfg(unix)]
fn reclaim_stale_files(paths: &HostPaths) -> Result<(), String> {
    if paths.discovery.exists() {
        let discovery = fs::read(&paths.discovery)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<HostDiscovery>(&bytes).ok());
        if discovery
            .as_ref()
            .is_some_and(|value| value.process_identity.is_live())
        {
            return Err("already_running".into());
        }
        owned_file(&paths.discovery, false)?;
        fs::remove_file(&paths.discovery)
            .map_err(|error| format!("stale_discovery_cleanup_failed:{error}"))?;
    }
    if paths.socket.exists() {
        owned_file(&paths.socket, true)?;
        fs::remove_file(&paths.socket)
            .map_err(|error| format!("stale_endpoint_cleanup_failed:{error}"))?;
    }
    Ok(())
}

#[cfg(unix)]
fn owned_file(path: &Path, require_socket: bool) -> Result<(), String> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| format!("host_path_metadata_failed:{error}"))?;
    if metadata.uid() != unsafe { libc::geteuid() } {
        return Err("host_path_not_owned_by_current_user".into());
    }
    if require_socket && !metadata.file_type().is_socket() {
        return Err("host_endpoint_not_socket".into());
    }
    if !require_socket && !metadata.file_type().is_file() {
        return Err("host_discovery_not_regular_file".into());
    }
    Ok(())
}

fn write_discovery(path: &Path, discovery: &HostDiscovery) -> Result<(), String> {
    let temporary = path.with_extension(format!("tmp.{}", std::process::id()));
    let bytes = serde_json::to_vec_pretty(discovery)
        .map_err(|error| format!("host_discovery_serialize_failed:{error}"))?;
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(&temporary)
        .map_err(|error| format!("host_discovery_create_failed:{error}"))?;
    file.write_all(&bytes)
        .map_err(|error| format!("host_discovery_write_failed:{error}"))?;
    file.sync_all()
        .map_err(|error| format!("host_discovery_sync_failed:{error}"))?;
    fs::rename(&temporary, path).map_err(|error| format!("host_discovery_publish_failed:{error}"))
}

fn read_discovery(home: &Path) -> Result<HostDiscovery, String> {
    let path = home.join("run/host/discovery.json");
    let bytes = fs::read(&path).map_err(|error| format!("host_discovery_unavailable:{error}"))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("host_discovery_invalid:{error}"))
}

fn validate_discovery(home: &Path, discovery: &HostDiscovery) -> Result<(), String> {
    #[cfg(unix)]
    {
        let metadata = fs::symlink_metadata(home.join("run/host/discovery.json"))
            .map_err(|error| format!("host_discovery_unavailable:{error}"))?;
        if !metadata.file_type().is_file()
            || metadata.uid() != unsafe { libc::geteuid() }
            || metadata.permissions().mode() & 0o077 != 0
        {
            return Err("host_discovery_security_invalid".into());
        }
    }
    if discovery.schema != HOST_DISCOVERY_SCHEMA || discovery.protocol != HOST_PROTOCOL {
        return Err("host_discovery_protocol_mismatch".into());
    }
    if discovery.yai_home != home.display().to_string()
        || discovery.yai_home_identity != home_identity(home)
    {
        return Err("host_discovery_yai_home_mismatch".into());
    }
    if !discovery.process_identity.is_live() {
        return Err("host_discovery_stale".into());
    }
    let endpoint = Path::new(&discovery.endpoint);
    if endpoint != home.join("run/host/application.sock") {
        return Err("host_discovery_endpoint_mismatch".into());
    }
    #[cfg(unix)]
    {
        let metadata = fs::symlink_metadata(endpoint)
            .map_err(|error| format!("host_endpoint_unavailable:{error}"))?;
        if !metadata.file_type().is_socket()
            || metadata.uid() != unsafe { libc::geteuid() }
            || metadata.permissions().mode() & 0o077 != 0
        {
            return Err("host_endpoint_security_invalid".into());
        }
    }
    Ok(())
}

#[cfg(unix)]
struct PeerCredentials {
    pid: u32,
    uid: u32,
}

#[cfg(unix)]
fn peer_credentials(stream: &UnixStream) -> Result<PeerCredentials, String> {
    let mut credentials = libc::ucred {
        pid: 0,
        uid: 0,
        gid: 0,
    };
    let mut length = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    let result = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            &mut credentials as *mut _ as *mut libc::c_void,
            &mut length,
        )
    };
    if result != 0 {
        return Err(format!(
            "host_peer_credentials_failed:{}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(PeerCredentials {
        pid: credentials.pid as u32,
        uid: credentials.uid,
    })
}

#[cfg(unix)]
fn verify_peer_same_user(stream: &UnixStream) -> Result<(), String> {
    let peer = peer_credentials(stream)?;
    if peer.uid != unsafe { libc::geteuid() } {
        return Err("host_peer_uid_mismatch".into());
    }
    Ok(())
}

fn canonical_home(home: &Path) -> Result<PathBuf, String> {
    if !home.exists() {
        fs::create_dir_all(home).map_err(|error| format!("yai_home_create_failed:{error}"))?;
    }
    let canonical = fs::canonicalize(home).map_err(|error| format!("yai_home_invalid:{error}"))?;
    #[cfg(unix)]
    {
        let metadata = fs::metadata(&canonical)
            .map_err(|error| format!("yai_home_metadata_failed:{error}"))?;
        if metadata.uid() != unsafe { libc::geteuid() } {
            return Err("yai_home_not_owned_by_current_user".into());
        }
    }
    Ok(canonical)
}

fn home_identity(home: &Path) -> String {
    format!("yai-home:{}", stable_digest(&home.display().to_string()))
}

fn next_client_id(kind: &str) -> String {
    let sequence = CLIENT_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{kind}:{}:{sequence}", std::process::id())
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

fn build_identity() -> String {
    option_env!("YAI_BUILD_SHA")
        .unwrap_or("development")
        .to_string()
}

fn result_state_label(state: ResultState) -> &'static str {
    match state {
        ResultState::Success => "success",
        ResultState::Partial => "partial",
        ResultState::Unauthorized => "unauthorized",
        ResultState::Stale => "stale",
        ResultState::CorePending => "core_pending",
        ResultState::NotImplemented => "not_implemented",
        ResultState::TransportUnavailable => "transport_unavailable",
        ResultState::Error => "error",
    }
}

fn is_absent_error(error: &str) -> bool {
    error.starts_with("host_discovery_unavailable")
        || error.starts_with("host_endpoint_unavailable")
        || error.starts_with("host_transport_unavailable")
        || error == "host_discovery_stale"
}

fn append_log(path: &Path, message: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .open(path)
        .map_err(|error| format!("host_log_open_failed:{error}"))?;
    writeln!(file, "{} {}", now_ms(), message)
        .map_err(|error| format!("host_log_write_failed:{error}"))
}

fn write_frame<T: Serialize>(writer: &mut impl Write, frame: &T) -> Result<(), String> {
    let bytes = serde_json::to_vec(frame)
        .map_err(|error| format!("host_frame_serialize_failed:{error}"))?;
    if bytes.len() > MAX_FRAME_BYTES {
        return Err("host_frame_too_large".into());
    }
    writer
        .write_all(&bytes)
        .map_err(|error| format!("host_frame_write_failed:{error}"))?;
    writer
        .write_all(b"\n")
        .map_err(|error| format!("host_frame_write_failed:{error}"))?;
    writer
        .flush()
        .map_err(|error| format!("host_frame_flush_failed:{error}"))
}

fn read_frame<T: for<'de> Deserialize<'de>>(reader: &mut impl BufRead) -> Result<T, String> {
    let mut bytes = Vec::new();
    let read = reader
        .take((MAX_FRAME_BYTES + 1) as u64)
        .read_until(b'\n', &mut bytes)
        .map_err(|error| format!("host_frame_read_failed:{error}"))?;
    if read == 0 {
        return Err("host_transport_closed".into());
    }
    if bytes.len() > MAX_FRAME_BYTES || !bytes.ends_with(b"\n") {
        return Err("host_frame_too_large".into());
    }
    serde_json::from_slice(&bytes).map_err(|error| format!("host_frame_invalid:{error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn home(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "yai-host-{name}-{}-{}",
            std::process::id(),
            now_ms()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn running(
        name: &str,
    ) -> (
        PathBuf,
        Arc<Mutex<SharedState>>,
        thread::JoinHandle<Result<(), String>>,
    ) {
        let home = home(name);
        let server = HostServer::bind(&home).unwrap();
        let state = server.state.clone();
        let handle = thread::spawn(move || server.run());
        (home, state, handle)
    }

    fn stop_server(home: &Path, handle: thread::JoinHandle<Result<(), String>>) {
        let _ = stop(home).unwrap();
        handle.join().unwrap().unwrap();
    }

    fn refused_handshake(home: &Path, protocol: &str, selected_home: &str) -> String {
        let discovery = read_discovery(home).unwrap();
        let mut stream = UnixStream::connect(&discovery.endpoint).unwrap();
        let reader_stream = stream.try_clone().unwrap();
        write_frame(
            &mut stream,
            &ClientFrame::Handshake {
                protocol: protocol.into(),
                client_id: "qualification:raw".into(),
                client_kind: ClientKind::Qualification,
                pid: std::process::id(),
                yai_home_identity: selected_home.into(),
            },
        )
        .unwrap();
        match read_frame::<ServerFrame>(&mut BufReader::new(reader_stream)).unwrap() {
            ServerFrame::Error { code, .. } => code,
            frame => panic!("expected refusal, got {frame:?}"),
        }
    }

    #[test]
    fn discovery_is_private_and_identifies_one_home() {
        let (home, _, handle) = running("discovery");
        let discovery = read_discovery(&home).unwrap();
        assert_eq!(
            discovery.yai_home_identity,
            home_identity(&fs::canonicalize(&home).unwrap())
        );
        assert_eq!(
            fs::metadata(&discovery.endpoint)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert_eq!(
            fs::metadata(home.join("run/host/discovery.json"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert_eq!(
            fs::metadata(home.join("run/host"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        stop_server(&home, handle);
        let _ = fs::remove_dir_all(home);
    }

    #[test]
    fn singleton_lock_refuses_second_host() {
        let (home, _, handle) = running("singleton");
        assert_eq!(HostServer::bind(&home).err().unwrap(), "already_running");
        stop_server(&home, handle);
        let _ = fs::remove_dir_all(home);
    }

    #[test]
    fn handshake_refuses_protocol_and_profile_mismatch_before_operations() {
        let (home, _, handle) = running("handshake");
        let selected = home_identity(&fs::canonicalize(&home).unwrap());
        assert_eq!(
            refused_handshake(&home, "yai.local_host.future", &selected),
            "protocol_mismatch"
        );
        assert_eq!(
            refused_handshake(&home, HOST_PROTOCOL, "yai-home:other"),
            "yai_home_mismatch"
        );
        stop_server(&home, handle);
        let _ = fs::remove_dir_all(home);
    }

    #[test]
    fn stale_discovery_and_owned_socket_are_reclaimed() {
        let home = home("stale");
        let paths = HostPaths::prepare(&fs::canonicalize(&home).unwrap()).unwrap();
        let stale = HostDiscovery {
            schema: HOST_DISCOVERY_SCHEMA.into(),
            protocol: HOST_PROTOCOL.into(),
            endpoint: paths.socket.display().to_string(),
            pid: u32::MAX,
            process_identity: LocalProcessIdentity {
                schema: yai_core_engine::resource_control::PROCESS_IDENTITY_SCHEMA.into(),
                pid: u32::MAX,
                boot_id: "stale".into(),
                start_ticks: 1,
            },
            instance_id: "stale".into(),
            started_at_unix_ms: 1,
            version: "test".into(),
            build: "test".into(),
            yai_home: fs::canonicalize(&home).unwrap().display().to_string(),
            yai_home_identity: home_identity(&fs::canonicalize(&home).unwrap()),
        };
        write_discovery(&paths.discovery, &stale).unwrap();
        let listener = UnixListener::bind(&paths.socket).unwrap();
        drop(listener);
        let server = HostServer::bind(&home).unwrap();
        let handle = thread::spawn(move || server.run());
        stop_server(&home, handle);
        let _ = fs::remove_dir_all(home);
    }

    #[test]
    fn application_requests_are_forwarded_without_semantic_translation() {
        let (home, _, handle) = running("forward");
        let request = OperationRequest {
            protocol: yai_application::APPLICATION_PROTOCOL.into(),
            operation_ref: "application.capabilities".into(),
            correlation_ref: "host:test:forward".into(),
            input: serde_json::json!({}),
        };
        let direct = LocalApplication::from_yai_home(&home).call(request.clone());
        let result = HostClient::connect(&home, ClientKind::Qualification)
            .unwrap()
            .call(request)
            .unwrap();
        assert_eq!(result.result_state, ResultState::Success);
        assert_eq!(result.correlation_ref, "host:test:forward");
        assert_eq!(
            serde_json::to_value(result).unwrap(),
            serde_json::to_value(direct).unwrap()
        );
        stop_server(&home, handle);
        let _ = fs::remove_dir_all(home);
    }

    #[test]
    fn attachments_are_ephemeral_and_counted_by_kind() {
        let (home, _, handle) = running("clients");
        let transient = HostClient::connect(&home, ClientKind::Studio).unwrap();
        let transient_status = HostClient::connect(&home, ClientKind::Qualification)
            .unwrap()
            .status()
            .unwrap();
        assert_eq!(transient_status.connected_clients, 0);
        drop(transient);
        let first = HostClient::connect(&home, ClientKind::Studio).unwrap();
        let second = HostClient::connect(&home, ClientKind::Studio).unwrap();
        let t1 = thread::spawn(move || first.subscribe(|_| Ok(())));
        let t2 = thread::spawn(move || second.subscribe(|_| Ok(())));
        thread::sleep(Duration::from_millis(150));
        let status = HostClient::connect(&home, ClientKind::Qualification)
            .unwrap()
            .status()
            .unwrap();
        assert_eq!(status.connected_clients, 2);
        assert_eq!(status.client_kinds.get("studio"), Some(&2));
        stop(&home).unwrap();
        t1.join().unwrap().unwrap();
        t2.join().unwrap().unwrap();
        handle.join().unwrap().unwrap();
        let _ = fs::remove_dir_all(home);
    }

    #[test]
    fn source_submission_survives_lost_response_and_host_restart_without_redispatch() {
        use std::time::Instant;
        use serde_json::json;
        use yai_core_engine::effect::access::*;
        use yai_core_engine::effect::LocalFilesystemBinding;
        use yai_core_engine::store::lmdb::LmdbRecordStore;
        use yai_core_engine::transition::TransitionPayload;
        let (home, _, handle) = running("source-response-loss");
        let request = |op: &str, input| OperationRequest {
            protocol: yai_application::APPLICATION_PROTOCOL.into(), operation_ref: op.into(),
            correlation_ref: "transport-not-submission-identity".into(), input,
        };
        let call = |op: &str, input| {
            HostClient::connect(&home, ClientKind::Qualification).unwrap()
                .call(request(op, input)).unwrap()
        };
        let success = |op: &str, input| {
            let result = call(op, input);
            assert_eq!(result.result_state, ResultState::Success, "{op}: {result:?}");
            result.data.unwrap()
        };
        success("identity.bootstrap", json!({"tenant_id":"tenant:source-host", "organization_ref":"organization:cli-product"}));
        success("case.create", json!({"tenant_id":"tenant:source-host","case_ref":"case:source-host"}));
        success("participant.role.add", json!({"case_ref":"case:source-host","participant_ref":"participant:operator","role":"operator"}));
        success("participant.principal.link", json!({"case_ref":"case:source-host","participant_ref":"participant:operator","principal_ref":"self"}));
        let root = home.join("source-root");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("policy.json"), include_bytes!("../../../tests/fixtures/cli-product-policy.json")).unwrap();
        let binding = LocalAccessBinding {
            schema: LOCAL_ACCESS_BINDING_SCHEMA.into(), case_id:"case:source-host".into(), attachment_id:"policy-file".into(),
            address: ResourceAddress::Discovery { root:LocalFilesystemBinding::new("case:source-host", "policy-file", &root).unwrap() },
        };
        let access = ResourceAccessContract {
            schema: RESOURCE_ACCESS_SCHEMA.into(), configuration_digest:binding.digest(),
            participant_ids:vec!["participant:operator".into()], operations:vec![AccessKind::Discover],
            read_prefixes:vec!["policy.json".into()], names:vec![], max_output_bytes:65536, max_items:8,
        };
        success("resource.attach", json!({"binding":binding,"access":access,
            "policy_owner_participant_ref":"participant:operator","review_requirement":"automatic"}));
        let declared = success("source.declare", json!({"case_ref":"case:source-host","perimeter":"test",
            "logical_name":"bootstrap","participant_ref":"participant:operator","resource_ref":"policy-file",
            "roles":["policy"],"action":ResourceAction::Discover { path:"policy.json".into() },
            "bootstrap_policy":true,"media_type":"application/json"}));
        let source = declared["state"]["sources"][0]["declaration"]["source_id"].as_str().unwrap();
        let submit = json!({"case_ref":"case:source-host","participant_ref":"participant:operator",
            "source_ref":source,"attempt":1,"expected_generation":declared["state"]["generation"]});
        let observe = json!({"case_ref":"case:source-host","participant_ref":"participant:operator",
            "execution":{"domain":"source_acquisition","source_ref":source,"attempt":1}});
        // A completed handshake without a submitted frame admits nothing.
        drop(HostClient::connect(&home, ClientKind::Studio).unwrap());
        assert_eq!(call("execution.get", observe.clone()).result_state, ResultState::Unauthorized);
        // Flush the complete request then lose the connection without reading
        // its response. The Host must not tie admitted work to that connection.
        let mut lost = HostClient::connect(&home, ClientKind::Studio).unwrap();
        write_frame(&mut lost.stream, &ClientFrame::ApplicationRequest {
            request: request("source.acquire", submit.clone()),
        }).unwrap();
        drop(lost);
        let deadline = Instant::now() + Duration::from_secs(5);
        let observed = loop {
            let result = call("execution.get", observe.clone());
            if let Some(data) = result.data {
                if data["phase"] == "acquired" { break data; }
            }
            assert!(Instant::now() < deadline, "host did not finish disconnected acquisition");
            thread::sleep(Duration::from_millis(10));
        };
        let repeated = success("source.acquire", submit.clone());
        assert_eq!(repeated["created"], false);
        assert_eq!(repeated["execution"], observed);
        stop_server(&home, handle);
        let server = HostServer::bind(&home).unwrap();
        let handle = thread::spawn(move || server.run());
        assert_eq!(success("execution.get", observe), observed);
        let repeated = success("source.acquire", submit);
        assert_eq!(repeated["created"], false);
        assert_eq!(repeated["execution"], observed);
        let store = LmdbRecordStore::open(home.join("store/lmdb")).unwrap();
        let history = store.list_case_transitions("case:source-host").unwrap();
        let progresses = history.iter().filter(|t| matches!(&t.payload,
            TransitionPayload::CaseSourceProgressed { progress } if progress.source_id == source)).count();
        assert_eq!(progresses, 2, "one admission + one terminal result, no retry transitions");
        assert!(store.verify_case_state("case:source-host").unwrap());
        drop(store);
        stop_server(&home, handle);
        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn event_fanout_reaches_two_attached_studio_clients() {
        let (home, state, handle) = running("fanout");
        let first = HostClient::connect(&home, ClientKind::Studio).unwrap();
        let second = HostClient::connect(&home, ClientKind::Studio).unwrap();
        let (seen_tx, seen_rx) = mpsc::channel();
        let first_tx = seen_tx.clone();
        let t1 = thread::spawn(move || {
            first.subscribe(|event| {
                if let HostEvent::Case(update) = event {
                    first_tx.send(update.generation).unwrap();
                }
                Ok(())
            })
        });
        let second_tx = seen_tx.clone();
        let t2 = thread::spawn(move || {
            second.subscribe(|event| {
                if let HostEvent::Case(update) = event {
                    second_tx.send(update.generation).unwrap();
                }
                Ok(())
            })
        });
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while state.lock().unwrap().subscribers.len() != 2 && std::time::Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }
        state.lock().unwrap().broadcast(ServerFrame::Event {
            event: LocalApplication::from_yai_home(&home).update_for("case:test", 7, 1),
        });
        assert_eq!(seen_rx.recv_timeout(Duration::from_secs(2)).unwrap(), 7);
        assert_eq!(seen_rx.recv_timeout(Duration::from_secs(2)).unwrap(), 7);
        stop(&home).unwrap();
        t1.join().unwrap().unwrap();
        t2.join().unwrap().unwrap();
        handle.join().unwrap().unwrap();
        let _ = fs::remove_dir_all(home);
    }

    #[test]
    fn graceful_shutdown_removes_endpoint_and_discovery() {
        let (home, _, handle) = running("shutdown");
        stop_server(&home, handle);
        assert!(!home.join("run/host/application.sock").exists());
        assert!(!home.join("run/host/discovery.json").exists());
        let _ = fs::remove_dir_all(home);
    }

    #[test]
    fn stopped_status_does_not_invent_runtime_supervision() {
        let home = home("stopped");
        let status = observe(&home).unwrap();
        assert_eq!(status.state, "stopped");
        assert_eq!(status.runtime_supervision, "not_integrated");
        let _ = fs::remove_dir_all(home);
    }
}
