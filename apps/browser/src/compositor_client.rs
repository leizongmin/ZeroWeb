//! Browser compositor 客户端。
//!
//! 阻塞式管道 IPC 和 compositor 子进程由专用 worker 独占。UI 线程只向
//! 有界命令队列提交最新帧，并从有界完成缓存非阻塞读取结果。

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

use zero_protocol::message::{IpcMessage, IpcMessageKind};
use zero_protocol::paint_snapshot::PaintSnapshotParams;
use zero_protocol::transport::PipeTransport;
use zero_protocol::{IpcChannel, ProcessRole, ProtocolError, child_process_args};

const MAX_PENDING_SURFACES: usize = 64;
const MAX_COMPLETED_SURFACES: usize = 64;

/// Compositor 完成帧（含可选 dma-buf GPU 导入）。
pub struct CompositorFrameResult {
    pub surface_id: u64,
    pub navigation_epoch: u64,
    pub frame_id: u64,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub scroll_x: f32,
    pub scroll_y: f32,
    #[cfg(target_os = "linux")]
    pub dmabuf: Option<CompositorDmabufResult>,
}

#[cfg(target_os = "linux")]
pub struct CompositorDmabufResult {
    pub fd: std::os::fd::OwnedFd,
    pub stride: u32,
    pub drm_fourcc: u32,
    pub drm_modifier: u64,
}

/// Compositor client 当前连接状态。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompositorStatus {
    /// Compositor 模式未启用。
    Disabled,
    /// Worker 正在启动 compositor 子进程。
    Starting,
    /// Compositor IPC 可用。
    Healthy,
    /// 子进程启动失败或 IPC 已断开。
    Disconnected,
}

impl CompositorStatus {
    fn encode(self) -> u8 {
        match self {
            Self::Disabled => 0,
            Self::Starting => 1,
            Self::Healthy => 2,
            Self::Disconnected => 3,
        }
    }

    fn decode(value: u8) -> Self {
        match value {
            1 => Self::Starting,
            2 => Self::Healthy,
            3 => Self::Disconnected,
            _ => Self::Disabled,
        }
    }
}

struct SharedStatus(AtomicU8);

impl SharedStatus {
    fn new(status: CompositorStatus) -> Self {
        Self(AtomicU8::new(status.encode()))
    }

    fn load(&self) -> CompositorStatus {
        CompositorStatus::decode(self.0.load(Ordering::Acquire))
    }

    fn store(&self, status: CompositorStatus) {
        self.0.store(status.encode(), Ordering::Release);
    }
}

#[derive(Default)]
struct FrameWatchdog {
    last_progress: Option<std::time::Instant>,
}

impl FrameWatchdog {
    fn begin(&mut self, now: std::time::Instant) {
        self.last_progress = Some(now);
    }

    fn progress(&mut self, now: std::time::Instant) {
        if self.last_progress.is_some() {
            self.last_progress = Some(now);
        }
    }

    fn complete(&mut self) {
        self.last_progress = None;
    }

    fn is_stalled(&self, now: std::time::Instant) -> bool {
        self.last_progress
            .is_some_and(|progress| now.saturating_duration_since(progress) > std::time::Duration::from_secs(10))
    }
}

struct PendingFrame {
    surface_id: u64,
    navigation_epoch: u64,
    frame_id: u64,
    paint: PaintSnapshotParams,
}

enum WorkerCommand {
    Frame(Box<PendingFrame>),
    ReleaseSurface(u64),
    SetScroll {
        surface_id: u64,
        navigation_epoch: u64,
        frame_id: u64,
        scroll_x: f32,
        scroll_y: f32,
    },
    RegisterUiSurface(zero_protocol::CompositorUiSurfaceInfo),
    RegisterWindowSurface(zero_protocol::CompositorWindowSurfaceInfo),
    UiFrame {
        surface_id: u64,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    },
    FetchPresent {
        page_surface_id: u64,
        ui_surface_id: u64,
        width: u32,
        height: u32,
    },
}

impl WorkerCommand {
    fn surface_id(&self) -> u64 {
        match self {
            Self::Frame(frame) => frame.surface_id,
            Self::ReleaseSurface(surface_id) => *surface_id,
            Self::SetScroll { surface_id, .. } => *surface_id,
            Self::RegisterUiSurface(info) => info.surface_id,
            Self::RegisterWindowSurface(info) => info.surface_id,
            Self::UiFrame { surface_id, .. } => *surface_id,
            Self::FetchPresent { page_surface_id, .. } => *page_surface_id,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FrameKey {
    surface_id: u64,
    navigation_epoch: u64,
    frame_id: u64,
}

impl From<&PendingFrame> for FrameKey {
    fn from(frame: &PendingFrame) -> Self {
        Self {
            surface_id: frame.surface_id,
            navigation_epoch: frame.navigation_epoch,
            frame_id: frame.frame_id,
        }
    }
}

struct CommandState {
    pending: VecDeque<WorkerCommand>,
    closed: bool,
}

/// 有界的 surface 命令通道；同一 surface 始终以新帧替换旧帧。
struct CommandChannel {
    capacity: usize,
    state: Mutex<CommandState>,
    ready: Condvar,
}

impl CommandChannel {
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            state: Mutex::new(CommandState {
                pending: VecDeque::new(),
                closed: false,
            }),
            ready: Condvar::new(),
        }
    }

    fn send_frame(&self, frame: PendingFrame) -> bool {
        self.send(WorkerCommand::Frame(Box::new(frame)))
    }

    fn release_surface(&self, surface_id: u64) -> bool {
        self.send(WorkerCommand::ReleaseSurface(surface_id))
    }

    fn set_scroll(&self, surface_id: u64, navigation_epoch: u64, frame_id: u64, scroll_x: f32, scroll_y: f32) -> bool {
        self.send(WorkerCommand::SetScroll {
            surface_id,
            navigation_epoch,
            frame_id,
            scroll_x,
            scroll_y,
        })
    }

    fn register_ui_surface(&self, info: zero_protocol::CompositorUiSurfaceInfo) -> bool {
        self.send(WorkerCommand::RegisterUiSurface(info))
    }

    fn register_window_surface(&self, info: zero_protocol::CompositorWindowSurfaceInfo) -> bool {
        self.send(WorkerCommand::RegisterWindowSurface(info))
    }

    fn forward_ui_frame(&self, surface_id: u64, width: u32, height: u32, rgba: Vec<u8>) -> bool {
        self.send(WorkerCommand::UiFrame {
            surface_id,
            width,
            height,
            rgba,
        })
    }

    fn fetch_present(&self, page_surface_id: u64, ui_surface_id: u64, width: u32, height: u32) -> bool {
        self.send(WorkerCommand::FetchPresent {
            page_surface_id,
            ui_surface_id,
            width,
            height,
        })
    }

    fn send(&self, command: WorkerCommand) -> bool {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if state.closed {
            return false;
        }
        let surface_id = command.surface_id();
        if let Some(index) = state
            .pending
            .iter()
            .position(|pending| pending.surface_id() == surface_id)
        {
            state.pending.remove(index);
        } else if state.pending.len() == self.capacity {
            state.pending.pop_front();
        }
        state.pending.push_back(command);
        self.ready.notify_one();
        true
    }

    fn recv(&self) -> Option<Vec<WorkerCommand>> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        while state.pending.is_empty() && !state.closed {
            state = self.ready.wait(state).unwrap_or_else(|error| error.into_inner());
        }
        if state.closed {
            return None;
        }
        Some(state.pending.drain(..).collect())
    }

    fn close(&self) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.closed = true;
        state.pending.clear();
        self.ready.notify_all();
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .pending
            .len()
    }
}

struct CompletedFrame {
    key: FrameKey,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    scroll_x: f32,
    scroll_y: f32,
    #[cfg(target_os = "linux")]
    dmabuf: Option<CompletedDmabufFrame>,
}

#[cfg(target_os = "linux")]
struct CompletedDmabufFrame {
    fd: std::os::fd::OwnedFd,
    stride: u32,
    drm_fourcc: u32,
    drm_modifier: u64,
}

/// Worker 到 UI 的有界事件缓存；每个 surface 只保留最新完整位图。
struct CompletedFrameChannel {
    capacity: usize,
    frames: Mutex<VecDeque<CompletedFrame>>,
}

impl CompletedFrameChannel {
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            frames: Mutex::new(VecDeque::new()),
        }
    }

    fn send(&self, frame: CompletedFrame) {
        let mut frames = self.frames.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(index) = frames
            .iter()
            .position(|cached| cached.key.surface_id == frame.key.surface_id)
        {
            if (frames[index].key.navigation_epoch, frames[index].key.frame_id)
                > (frame.key.navigation_epoch, frame.key.frame_id)
            {
                return;
            }
            frames.remove(index);
        } else if frames.len() == self.capacity {
            frames.pop_front();
        }
        frames.push_back(frame);
    }

    fn try_recv(&self, surface_id: u64, navigation_epoch: u64, frame_id: u64) -> Option<CompletedFrame> {
        let mut frames = self.frames.lock().unwrap_or_else(|error| error.into_inner());
        let index = frames.iter().position(|frame| {
            frame.key.surface_id == surface_id
                && frame.key.navigation_epoch == navigation_epoch
                && frame.key.frame_id >= frame_id
        })?;
        frames.remove(index)
    }

    fn clear(&self) {
        self.frames.lock().unwrap_or_else(|error| error.into_inner()).clear();
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.frames.lock().unwrap_or_else(|error| error.into_inner()).len()
    }
}

struct PresentFrame {
    page_surface_id: u64,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

/// compositor present 帧缓存（每个 page surface 最新一帧）。
struct PresentFrameChannel {
    frame: Mutex<Option<PresentFrame>>,
}

impl PresentFrameChannel {
    fn new() -> Self {
        Self {
            frame: Mutex::new(None),
        }
    }

    fn store(&self, frame: PresentFrame) {
        *self.frame.lock().unwrap_or_else(|error| error.into_inner()) = Some(frame);
    }

    fn take(&self, page_surface_id: u64) -> Option<(u32, u32, Vec<u8>)> {
        let mut slot = self.frame.lock().unwrap_or_else(|error| error.into_inner());
        if slot
            .as_ref()
            .is_some_and(|frame| frame.page_surface_id == page_surface_id)
        {
            slot.take().map(|frame| (frame.width, frame.height, frame.rgba))
        } else {
            None
        }
    }

    fn clear(&self) {
        *self.frame.lock().unwrap_or_else(|error| error.into_inner()) = None;
    }
}

trait WorkerTransport: Send {
    fn send(&mut self, message: IpcMessage) -> Result<(), ProtocolError>;
    fn recv(&mut self) -> Result<IpcMessage, ProtocolError>;
}

impl WorkerTransport for PipeTransport<std::process::ChildStdout, std::process::ChildStdin> {
    fn send(&mut self, message: IpcMessage) -> Result<(), ProtocolError> {
        IpcChannel::send(self, message)
    }

    fn recv(&mut self) -> Result<IpcMessage, ProtocolError> {
        IpcChannel::recv(self)
    }
}

struct Client {
    commands: Arc<CommandChannel>,
    completed: Arc<CompletedFrameChannel>,
    present: Arc<PresentFrameChannel>,
    status: Arc<SharedStatus>,
    child: Arc<Mutex<Option<Child>>>,
    worker: Option<JoinHandle<()>>,
    /// 帧响应看门狗；仅由 IPC worker 在真实请求开始、推进和完成时更新。
    frame_watchdog: Arc<Mutex<FrameWatchdog>>,
}

impl Client {
    fn start() -> Self {
        let commands = Arc::new(CommandChannel::new(MAX_PENDING_SURFACES));
        let completed = Arc::new(CompletedFrameChannel::new(MAX_COMPLETED_SURFACES));
        let present = Arc::new(PresentFrameChannel::new());
        let status = Arc::new(SharedStatus::new(CompositorStatus::Starting));
        let child = Arc::new(Mutex::new(None));

        let worker_commands = Arc::clone(&commands);
        let worker_completed = Arc::clone(&completed);
        let worker_present = Arc::clone(&present);
        let worker_status = Arc::clone(&status);
        let worker_child = Arc::clone(&child);
        let frame_watchdog = Arc::new(Mutex::new(FrameWatchdog::default()));
        let worker_frame_watchdog = Arc::clone(&frame_watchdog);
        let worker = thread::Builder::new()
            .name("compositor-client".to_string())
            .spawn(move || {
                let connection_child = Arc::clone(&worker_child);
                worker_main(
                    worker_commands,
                    worker_completed,
                    worker_present,
                    worker_status,
                    worker_frame_watchdog,
                    || spawn_transport(connection_child),
                );
                reap_child(&worker_child);
            })
            .ok();
        if worker.is_none() {
            status.store(CompositorStatus::Disconnected);
        }

        Self {
            commands,
            completed,
            present,
            status,
            child,
            worker,
            frame_watchdog,
        }
    }

    /// R3254-F10：帧响应看门狗——有帧已发送但长时间（10s）无任何 compositor 响应 →
    /// 视为断连（compositor 进程活着但不响应：光栅化挂起/重负载卡死）。此前 worker 在
    /// process_batch 的阻塞 recv 上永久卡住，后续帧全部堆积、页面冻结且无回退信号。
    fn check_stall(&self) {
        if self.status.load() == CompositorStatus::Disconnected {
            return;
        }
        if self
            .frame_watchdog
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .is_stalled(std::time::Instant::now())
        {
            tracing::warn!("compositor: 帧响应超时（10s 无响应），视为断连并回退 legacy");
            self.status.store(CompositorStatus::Disconnected);
        }
    }

    fn send(&self, surface_id: u64, navigation_epoch: u64, frame_id: u64, paint: PaintSnapshotParams) {
        if self.status.load() == CompositorStatus::Disconnected {
            return;
        }
        let _ = self.commands.send_frame(PendingFrame {
            surface_id,
            navigation_epoch,
            frame_id,
            paint,
        });
    }

    fn release_surface(&self, surface_id: u64) {
        if self.status.load() != CompositorStatus::Disconnected {
            let _ = self.commands.release_surface(surface_id);
        }
    }

    fn set_scroll(&self, surface_id: u64, navigation_epoch: u64, frame_id: u64, scroll_x: f32, scroll_y: f32) {
        if self.status.load() != CompositorStatus::Disconnected {
            let _ = self
                .commands
                .set_scroll(surface_id, navigation_epoch, frame_id, scroll_x, scroll_y);
        }
    }

    fn register_ui_surface(&self, info: zero_protocol::CompositorUiSurfaceInfo) {
        if self.status.load() != CompositorStatus::Disconnected {
            let _ = self.commands.register_ui_surface(info);
        }
    }

    fn register_window_surface(&self, info: zero_protocol::CompositorWindowSurfaceInfo) {
        if self.status.load() != CompositorStatus::Disconnected {
            let _ = self.commands.register_window_surface(info);
        }
    }

    fn forward_ui_frame(&self, surface_id: u64, width: u32, height: u32, rgba: Vec<u8>) {
        if self.status.load() != CompositorStatus::Disconnected {
            let _ = self.commands.forward_ui_frame(surface_id, width, height, rgba);
        }
    }

    fn fetch_present(&self, page_surface_id: u64, ui_surface_id: u64, width: u32, height: u32) {
        if self.status.load() != CompositorStatus::Disconnected {
            let _ = self
                .commands
                .fetch_present(page_surface_id, ui_surface_id, width, height);
        }
    }

    fn try_recv(&self, surface_id: u64, navigation_epoch: u64, frame_id: u64) -> Option<CompositorFrameResult> {
        let frame = self.completed.try_recv(surface_id, navigation_epoch, frame_id)?;
        Some(CompositorFrameResult {
            surface_id: frame.key.surface_id,
            navigation_epoch: frame.key.navigation_epoch,
            frame_id: frame.key.frame_id,
            width: frame.width,
            height: frame.height,
            rgba: frame.rgba,
            scroll_x: frame.scroll_x,
            scroll_y: frame.scroll_y,
            #[cfg(target_os = "linux")]
            dmabuf: frame.dmabuf.map(|d| CompositorDmabufResult {
                fd: d.fd,
                stride: d.stride,
                drm_fourcc: d.drm_fourcc,
                drm_modifier: d.drm_modifier,
            }),
        })
    }

    fn shutdown(&mut self) {
        self.commands.close();
        kill_child(&self.child);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        reap_child(&self.child);
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// R3254：compositor 二进制文件名（平台后缀）。
fn compositor_binary_filename() -> &'static str {
    #[cfg(windows)]
    {
        "zero-compositor.exe"
    }
    #[cfg(not(windows))]
    {
        "zero-compositor"
    }
}

fn compositor_candidates_near_executable(exe: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    #[cfg(target_os = "macos")]
    for contents_dir in exe
        .ancestors()
        .filter(|path| path.file_name().and_then(|name| name.to_str()) == Some("Contents"))
    {
        candidates.push(
            contents_dir
                .join("Frameworks")
                .join("ZeroBrowser Helper (Compositor).app")
                .join("Contents")
                .join("MacOS")
                .join("ZeroBrowser Helper (Compositor)"),
        );
    }

    if let Some(parent) = exe.parent() {
        candidates.push(parent.join(compositor_binary_filename()));
        if let Some(grandparent) = parent.parent() {
            candidates.push(grandparent.join(compositor_binary_filename()));
        }
    }
    candidates
}

#[allow(clippy::zombie_processes)]
fn spawn_transport(child_slot: Arc<Mutex<Option<Child>>>) -> Result<Box<dyn WorkerTransport>, String> {
    // R3254：与 renderer 二进制解析同模式——cargo test / 开发环境 PATH 不含 target/debug，
    // 此前 spawn 失败 → Disconnected → CompositorFrame 全被丢弃（测试与 CLI 直跑差异）。
    // 查找顺序：ZW_COMPOSITOR_BIN → CARGO_BIN_EXE_zero-compositor → macOS Helper app
    // → current_exe 同目录（测试二进制 target/debug/deps/ 上溯 target/debug/）→ PATH 兜底。
    let bin = match zero_runtime_config::optional_path("ZW_COMPOSITOR_BIN") {
        Some(bin) => bin.to_string_lossy().into_owned(),
        None => {
            let mut candidate = std::env::var("CARGO_BIN_EXE_zero-compositor").ok().map(PathBuf::from);
            if candidate.as_ref().is_none_or(|p| !p.is_file())
                && let Ok(exe) = std::env::current_exe()
            {
                candidate = compositor_candidates_near_executable(&exe)
                    .into_iter()
                    .find(|path| path.is_file());
            }
            candidate
                .filter(|p| p.is_file())
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|| "zero-compositor".to_string())
        }
    };
    let mut command = Command::new(&bin);
    for argument in child_process_args(ProcessRole::Compositor, 0) {
        command.arg(argument);
    }
    // Windows：阻止子进程分配控制台窗口（双保险：即使子系统是 CUI 也不弹窗；
    // 同时不影响 stdin/stdout/stderr 管道继承）。与 zero-protocol spawn 同款。
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| format!("failed to spawn {bin}: {error}"))?;
    let Some(stdout) = child.stdout.take() else {
        let _ = child.kill();
        let _ = child.wait();
        return Err("compositor stdout pipe is unavailable".to_string());
    };
    let Some(stdin) = child.stdin.take() else {
        let _ = child.kill();
        let _ = child.wait();
        return Err("compositor stdin pipe is unavailable".to_string());
    };
    *child_slot.lock().unwrap_or_else(|error| error.into_inner()) = Some(child);
    Ok(Box::new(PipeTransport::new(stdout, stdin)))
}

fn kill_child(child_slot: &Mutex<Option<Child>>) {
    if let Some(child) = child_slot.lock().unwrap_or_else(|error| error.into_inner()).as_mut() {
        if child.try_wait().ok().flatten().is_none() {
            let _ = child.kill();
        }
    }
}

fn reap_child(child_slot: &Mutex<Option<Child>>) {
    if let Some(mut child) = child_slot.lock().unwrap_or_else(|error| error.into_inner()).take() {
        if child.try_wait().ok().flatten().is_none() {
            let _ = child.kill();
        }
        let _ = child.wait();
    }
}

enum ExpectedResponse {
    Result(FrameKey),
    Data(FrameKey),
    Scroll(FrameKey),
    ReleaseSurface,
    PresentData { page_surface_id: u64 },
}

fn worker_main<F>(
    commands: Arc<CommandChannel>,
    completed: Arc<CompletedFrameChannel>,
    present: Arc<PresentFrameChannel>,
    status: Arc<SharedStatus>,
    frame_watchdog: Arc<Mutex<FrameWatchdog>>,
    connect: F,
) where
    F: FnOnce() -> Result<Box<dyn WorkerTransport>, String>,
{
    let mut transport = match connect() {
        Ok(transport) => transport,
        Err(error) => {
            tracing::warn!("Compositor connection failed: {error}");
            status.store(CompositorStatus::Disconnected);
            completed.clear();
            present.clear();
            commands.close();
            return;
        }
    };
    status.store(CompositorStatus::Healthy);
    if std::env::var("ZERO_BROWSER_PRODUCT_SMOKE").as_deref() == Ok("1") {
        tracing::info!("SMOKE_EVENT component=compositor_client status=Healthy");
    }
    let mut next_message_id = 1u64;

    while let Some(frames) = commands.recv() {
        if let Err(error) = process_batch(
            transport.as_mut(),
            frames,
            &completed,
            &present,
            &mut next_message_id,
            &frame_watchdog,
        ) {
            tracing::warn!("Compositor IPC disconnected: {error}");
            status.store(CompositorStatus::Disconnected);
            completed.clear();
            present.clear();
            commands.close();
            return;
        }
    }
}

fn process_batch(
    transport: &mut dyn WorkerTransport,
    commands: Vec<WorkerCommand>,
    completed: &CompletedFrameChannel,
    present: &PresentFrameChannel,
    next_message_id: &mut u64,
    frame_watchdog: &Arc<Mutex<FrameWatchdog>>,
) -> Result<(), ProtocolError> {
    for command in commands {
        let mut outstanding = HashMap::new();
        let message_id = take_message_id(next_message_id);
        match command {
            WorkerCommand::Frame(frame) => {
                let key = FrameKey::from(frame.as_ref());
                if std::env::var("ZERO_BROWSER_PRODUCT_SMOKE").as_deref() == Ok("1") {
                    tracing::info!(
                        "SMOKE_EVENT component=compositor_client event=frame_submitted surface={} epoch={} frame={}",
                        key.surface_id,
                        key.navigation_epoch,
                        key.frame_id
                    );
                }
                frame_watchdog
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .begin(std::time::Instant::now());
                transport.send(IpcMessage {
                    id: message_id,
                    kind: IpcMessageKind::CompositorFrame {
                        surface_id: key.surface_id,
                        navigation_epoch: key.navigation_epoch,
                        frame_id: key.frame_id,
                        paint: Box::new(frame.paint),
                    },
                })?;
                outstanding.insert(message_id, ExpectedResponse::Result(key));
            }
            WorkerCommand::ReleaseSurface(surface_id) => {
                transport.send(IpcMessage {
                    id: message_id,
                    kind: IpcMessageKind::ReleaseCompositorSurface { surface_id },
                })?;
                outstanding.insert(message_id, ExpectedResponse::ReleaseSurface);
            }
            WorkerCommand::SetScroll {
                surface_id,
                navigation_epoch,
                frame_id,
                scroll_x,
                scroll_y,
            } => {
                transport.send(IpcMessage {
                    id: message_id,
                    kind: IpcMessageKind::CompositorSetScroll {
                        surface_id,
                        scroll_x,
                        scroll_y,
                    },
                })?;
                outstanding.insert(
                    message_id,
                    ExpectedResponse::Scroll(FrameKey {
                        surface_id,
                        navigation_epoch,
                        frame_id,
                    }),
                );
            }
            WorkerCommand::RegisterUiSurface(info) => {
                transport.send(IpcMessage {
                    id: message_id,
                    kind: IpcMessageKind::CompositorRegisterUiSurface(info),
                })?;
                outstanding.insert(message_id, ExpectedResponse::ReleaseSurface);
            }
            WorkerCommand::RegisterWindowSurface(info) => {
                transport.send(IpcMessage {
                    id: message_id,
                    kind: IpcMessageKind::CompositorRegisterWindowSurface(info),
                })?;
                outstanding.insert(message_id, ExpectedResponse::ReleaseSurface);
            }
            WorkerCommand::UiFrame {
                surface_id,
                width,
                height,
                rgba,
            } => {
                transport.send(IpcMessage {
                    id: message_id,
                    kind: IpcMessageKind::CompositorUiFrame {
                        surface_id,
                        width,
                        height,
                        rgba,
                        shm_name: None,
                    },
                })?;
                outstanding.insert(message_id, ExpectedResponse::ReleaseSurface);
            }
            WorkerCommand::FetchPresent {
                page_surface_id,
                ui_surface_id,
                width,
                height,
            } => {
                transport.send(IpcMessage {
                    id: message_id,
                    kind: IpcMessageKind::GetCompositorPresentFrame {
                        width,
                        height,
                        page_surface_id,
                        ui_surface_id,
                    },
                })?;
                outstanding.insert(message_id, ExpectedResponse::PresentData { page_surface_id });
            }
        }

        // UI 上传与 present 回读都可能超过 20 MiB。每条命令完成请求—响应后再写
        // 下一条，避免两个进程同时向对方写大帧、却都没有读取而形成管道背压死锁。
        while !outstanding.is_empty() {
            let response = transport.recv()?;
            let Some(expected) = outstanding.remove(&response.id) else {
                return Err(ProtocolError::Channel(format!(
                    "unexpected compositor response id {}",
                    response.id
                )));
            };
            if matches!(&expected, ExpectedResponse::Result(_) | ExpectedResponse::Data(_)) {
                frame_watchdog
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .progress(std::time::Instant::now());
            }
            match (expected, response.kind) {
                (
                    ExpectedResponse::Result(expected),
                    IpcMessageKind::CompositorFrameResult {
                        surface_id,
                        navigation_epoch,
                        frame_id,
                    },
                ) if expected
                    == (FrameKey {
                        surface_id,
                        navigation_epoch,
                        frame_id,
                    }) =>
                {
                    let message_id = take_message_id(next_message_id);
                    transport.send(IpcMessage {
                        id: message_id,
                        kind: IpcMessageKind::GetCompositorFrame {
                            surface_id,
                            navigation_epoch,
                            frame_id,
                        },
                    })?;
                    outstanding.insert(message_id, ExpectedResponse::Data(expected));
                }
                (ExpectedResponse::Scroll(expected), IpcMessageKind::Ok) => {
                    let message_id = take_message_id(next_message_id);
                    transport.send(IpcMessage {
                        id: message_id,
                        kind: IpcMessageKind::GetCompositorFrame {
                            surface_id: expected.surface_id,
                            navigation_epoch: expected.navigation_epoch,
                            frame_id: expected.frame_id,
                        },
                    })?;
                    outstanding.insert(message_id, ExpectedResponse::Data(expected));
                }
                (
                    ExpectedResponse::Data(expected),
                    IpcMessageKind::CompositorFrameData {
                        surface_id,
                        navigation_epoch,
                        frame_id,
                        width,
                        height,
                        rgba,
                        shm_name,
                        scroll_x,
                        scroll_y,
                        gpu_image,
                        ..
                    },
                ) if expected
                    == (FrameKey {
                        surface_id,
                        navigation_epoch,
                        frame_id,
                    }) =>
                {
                    frame_watchdog
                        .lock()
                        .unwrap_or_else(|error| error.into_inner())
                        .complete();
                    let resolved_frame = zero_protocol::resolve_compositor_frame_delivery_fenced(
                        width,
                        height,
                        rgba,
                        shm_name,
                        gpu_image,
                        Some(expected.frame_id),
                    )?;
                    #[cfg(target_os = "linux")]
                    let (rgba, dmabuf) = match resolved_frame {
                        zero_protocol::CompositorResolvedFrame::Rgba(bytes) => {
                            validate_frame_data(width, height, &bytes)?;
                            (bytes, None)
                        }
                        #[cfg(target_os = "linux")]
                        zero_protocol::CompositorResolvedFrame::Dmabuf {
                            fd,
                            stride,
                            drm_fourcc,
                            drm_modifier,
                            ..
                        } => (
                            Vec::new(),
                            Some(CompletedDmabufFrame {
                                fd,
                                stride,
                                drm_fourcc,
                                drm_modifier,
                            }),
                        ),
                    };
                    #[cfg(not(target_os = "linux"))]
                    let rgba = match resolved_frame {
                        zero_protocol::CompositorResolvedFrame::Rgba(bytes) => {
                            validate_frame_data(width, height, &bytes)?;
                            bytes
                        }
                    };
                    if std::env::var("ZERO_BROWSER_PRODUCT_SMOKE").as_deref() == Ok("1") {
                        tracing::info!(
                            "SMOKE_EVENT component=compositor_client event=frame_completed surface={} epoch={} frame={} width={} height={}",
                            expected.surface_id,
                            expected.navigation_epoch,
                            expected.frame_id,
                            width,
                            height
                        );
                    }
                    completed.send(CompletedFrame {
                        key: expected,
                        width,
                        height,
                        rgba,
                        scroll_x,
                        scroll_y,
                        #[cfg(target_os = "linux")]
                        dmabuf,
                    });
                }
                (
                    ExpectedResponse::PresentData { page_surface_id },
                    IpcMessageKind::CompositorFrameData {
                        width,
                        height,
                        rgba,
                        shm_name,
                        gpu_image,
                        ..
                    },
                ) => {
                    let rgba = zero_protocol::resolve_compositor_frame_rgba(width, height, rgba, shm_name, gpu_image)?;
                    validate_frame_data(width, height, &rgba)?;
                    present.store(PresentFrame {
                        page_surface_id,
                        width,
                        height,
                        rgba,
                    });
                }
                (ExpectedResponse::ReleaseSurface, IpcMessageKind::Ok) => {}
                _ => {
                    return Err(ProtocolError::Channel(format!(
                        "mismatched compositor response id {}",
                        response.id
                    )));
                }
            }
        }
    }
    Ok(())
}

fn take_message_id(next_message_id: &mut u64) -> u64 {
    let id = *next_message_id;
    *next_message_id = next_message_id.wrapping_add(1);
    id
}

fn validate_frame_data(width: u32, height: u32, rgba: &[u8]) -> Result<(), ProtocolError> {
    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| ProtocolError::Channel("compositor frame dimensions overflow".to_string()))?;
    if rgba.len() != expected {
        return Err(ProtocolError::Channel(format!(
            "invalid compositor frame length: expected {expected}, got {}",
            rgba.len()
        )));
    }
    Ok(())
}

static CLIENT: Mutex<Option<Client>> = Mutex::new(None);

/// 浏览器始终使用独立 compositor 进程。
pub fn enabled() -> bool {
    true
}

/// 返回当前 compositor client 状态，不执行阻塞式 IPC。
pub fn status() -> CompositorStatus {
    let guard = CLIENT.lock().unwrap_or_else(|error| error.into_inner());
    // R3254-F10：帧响应看门狗——有帧发送但 10s 无响应 → 断连（回退 legacy 的触发源）。
    if let Some(client) = guard.as_ref() {
        client.check_stall();
    }
    guard
        .as_ref()
        .map_or(CompositorStatus::Starting, |client| client.status.load())
}

/// 非阻塞提交 renderer 发布的指定 surface 帧；队列繁忙时以该 surface 的新帧替换旧帧。
pub fn forward_frame(surface_id: u64, navigation_epoch: u64, frame_id: u64, paint: PaintSnapshotParams) {
    let mut client = CLIENT.lock().unwrap_or_else(|error| error.into_inner());
    client
        .get_or_insert_with(Client::start)
        .send(surface_id, navigation_epoch, frame_id, paint);
}

/// 非阻塞读取指定 surface 的最新完成位图。
///
/// 仅返回相同 `navigation_epoch` 且帧序号不小于 `frame_id` 的缓存，结果格式为
/// `(surface_id, navigation_epoch, frame_id, width, height, rgba, scroll_x, scroll_y)`。
pub fn get_frame(surface_id: u64, navigation_epoch: u64, frame_id: u64) -> Option<CompositorFrameResult> {
    CLIENT
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .as_ref()?
        .try_recv(surface_id, navigation_epoch, frame_id)
}

/// RFC 4.2：向 compositor 推送 surface 滚动偏移（异步滚动默认开，Browser 消费回读值）。
pub fn set_scroll(surface_id: u64, navigation_epoch: u64, frame_id: u64, scroll_x: f32, scroll_y: f32) {
    let mut client = CLIENT.lock().unwrap_or_else(|error| error.into_inner());
    client
        .get_or_insert_with(Client::start)
        .set_scroll(surface_id, navigation_epoch, frame_id, scroll_x, scroll_y);
}

/// 是否启用 compositor 异步滚动（默认开；`ZW_COMPOSITOR_ASYNC_SCROLL=0` 禁用。
/// Browser 使用 compositor 回读 scroll 做位图变换）。
pub fn async_scroll_enabled() -> bool {
    zero_runtime_config::enabled_by_default("ZW_COMPOSITOR_ASYNC_SCROLL")
}

/// 是否启用 compositor 侧 scroll 烘焙（回读 scroll 为 0，Browser 不再偏移位图）。
pub fn scroll_transform_enabled() -> bool {
    zero_protocol::compositor_scroll_transform_enabled()
}

/// RFC 4.4：向 compositor 注册 Chrome UI surface（元数据登记；present 为后续切片）。
pub fn register_ui_surface(info: zero_protocol::CompositorUiSurfaceInfo) {
    let mut client = CLIENT.lock().unwrap_or_else(|error| error.into_inner());
    let client = client.get_or_insert_with(Client::start);
    let surface_id = info.surface_id;
    client.register_ui_surface(info);
    if ui_frames_enabled() || present_enabled() {
        client.forward_ui_frame(surface_id, 1, 1, vec![0, 0, 0, 0]);
    }
}

/// RFC 4.4-S4：向 compositor 登记最终窗口 surface。
pub fn register_window_surface(info: zero_protocol::CompositorWindowSurfaceInfo) {
    let mut client = CLIENT.lock().unwrap_or_else(|error| error.into_inner());
    client.get_or_insert_with(Client::start).register_window_surface(info);
}

/// 是否启用 compositor 拥有最终窗口 present（`ZW_COMPOSITOR_OWNED_PRESENT=1`）。
pub fn owned_present_enabled() -> bool {
    zero_protocol::compositor_owned_present_enabled()
}

/// RFC 4.4-S2：向 compositor 提交 Chrome UI 位图。
pub fn forward_ui_frame(surface_id: u64, width: u32, height: u32, rgba: Vec<u8>) {
    if !enabled() {
        return;
    }
    let mut client = CLIENT.lock().unwrap_or_else(|error| error.into_inner());
    client
        .get_or_insert_with(Client::start)
        .forward_ui_frame(surface_id, width, height, rgba);
}

/// 是否向 compositor 提交 UI 位图（默认开；`ZW_COMPOSITOR_UI_FRAMES=0` 禁用）。
pub fn ui_frames_enabled() -> bool {
    zero_runtime_config::enabled_by_default("ZW_COMPOSITOR_UI_FRAMES")
}

/// 是否启用 GPU shared image mailbox（`ZW_COMPOSITOR_GPU_IMAGE=1`，Linux）。
pub fn gpu_image_enabled() -> bool {
    zero_protocol::compositor_gpu_image_enabled()
}

/// 是否启用 compositor Viz present（page+UI 合成；`ZW_COMPOSITOR_PRESENT=1`）。
pub fn present_enabled() -> bool {
    zero_protocol::compositor_present_enabled()
}

/// 请求 compositor 合成 present 帧（异步；结果经 [`take_present_frame`] 取回）。
pub fn request_present_frame(page_surface_id: u64, ui_surface_id: u64, width: u32, height: u32) {
    if !enabled() {
        return;
    }
    let mut client = CLIENT.lock().unwrap_or_else(|error| error.into_inner());
    client
        .get_or_insert_with(Client::start)
        .fetch_present(page_surface_id, ui_surface_id, width, height);
}

/// 非阻塞取回指定 page surface 的最新 present 帧。
pub fn take_present_frame(page_surface_id: u64) -> Option<(u32, u32, Vec<u8>)> {
    if !enabled() {
        return None;
    }
    CLIENT
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .as_ref()?
        .present
        .take(page_surface_id)
}

/// Browser Chrome UI 层 surface 标识（与页面 surface 命名空间独立）。
pub const CHROME_UI_SURFACE_ID: u64 = u64::MAX;
/// Browser 窗口 surface（RFC 4.4-S4 compositor 拥有 present 输出）。
pub const CHROME_WINDOW_SURFACE_ID: u64 = u64::MAX - 1;

/// 非阻塞请求 compositor worker 释放指定页面 surface。
pub fn release_surface(surface_id: u64) {
    if !enabled() {
        return;
    }
    if let Some(client) = CLIENT.lock().unwrap_or_else(|error| error.into_inner()).as_ref() {
        client.release_surface(surface_id);
    }
}

/// 终止 compositor 子进程并等待 worker 退出。
///
/// Client 会先移出全局槽位，再执行 kill、join 和 wait，避免持有全局锁等待 worker。
pub fn shutdown() {
    let client = CLIENT.lock().unwrap_or_else(|error| error.into_inner()).take();
    if let Some(mut client) = client {
        client.shutdown();
    }
}

/// 测试：kill compositor 子进程以模拟 crash（不 shutdown worker）。
#[cfg(test)]
pub fn kill_compositor_child_for_test() -> bool {
    let client = CLIENT.lock().unwrap_or_else(|error| error.into_inner());
    if let Some(client) = client.as_ref() {
        kill_child(&client.child);
        return true;
    }
    false
}

/// 测试：重置全局 compositor client。
#[cfg(test)]
pub fn reset_client_for_test() {
    shutdown();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn local_build_uses_sibling_compositor() {
        let executable = Path::new("/workspace/target/release/zero-browser");
        let candidates = compositor_candidates_near_executable(executable);

        assert_eq!(
            candidates.first(),
            Some(&PathBuf::from("/workspace/target/release").join(compositor_binary_filename()))
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_app_prefers_compositor_helper_bundle() {
        let executable = Path::new("/Applications/ZeroBrowser.app/Contents/MacOS/ZeroBrowser");
        let candidates = compositor_candidates_near_executable(executable);

        assert_eq!(
            candidates.first(),
            Some(&PathBuf::from(
                "/Applications/ZeroBrowser.app/Contents/Frameworks/ZeroBrowser Helper (Compositor).app/Contents/MacOS/ZeroBrowser Helper (Compositor)"
            ))
        );
    }

    fn pending(surface_id: u64, navigation_epoch: u64, frame_id: u64) -> PendingFrame {
        PendingFrame {
            surface_id,
            navigation_epoch,
            frame_id,
            paint: PaintSnapshotParams {
                navigation_epoch,
                ..Default::default()
            },
        }
    }

    fn result(id: u64, key: FrameKey) -> IpcMessage {
        IpcMessage {
            id,
            kind: IpcMessageKind::CompositorFrameResult {
                surface_id: key.surface_id,
                navigation_epoch: key.navigation_epoch,
                frame_id: key.frame_id,
            },
        }
    }

    fn data(id: u64, key: FrameKey, pixel: u8) -> IpcMessage {
        IpcMessage {
            id,
            kind: IpcMessageKind::CompositorFrameData {
                surface_id: key.surface_id,
                navigation_epoch: key.navigation_epoch,
                frame_id: key.frame_id,
                width: 1,
                height: 1,
                rgba: vec![pixel, 0, 0, 255],
                shm_name: None,
                scroll_x: 0.0,
                scroll_y: 0.0,
                gpu_image: None,
                present_authoritative: false,
            },
        }
    }

    fn frame_watchdog() -> Arc<Mutex<FrameWatchdog>> {
        Arc::new(Mutex::new(FrameWatchdog::default()))
    }

    #[test]
    fn frame_watchdog_ignores_idle_time_before_a_request_starts() {
        let idle_start = std::time::Instant::now();
        let request_start = idle_start + Duration::from_secs(20);
        let mut watchdog = FrameWatchdog::default();

        assert!(!watchdog.is_stalled(request_start));
        watchdog.begin(request_start);
        assert!(!watchdog.is_stalled(request_start));
        assert!(watchdog.is_stalled(request_start + Duration::from_secs(11)));
    }

    #[test]
    fn frame_watchdog_tracks_progress_and_clears_after_completion() {
        let start = std::time::Instant::now();
        let mut watchdog = FrameWatchdog::default();

        watchdog.begin(start);
        watchdog.progress(start + Duration::from_secs(5));
        assert!(!watchdog.is_stalled(start + Duration::from_secs(11)));
        watchdog.complete();
        assert!(!watchdog.is_stalled(start + Duration::from_secs(30)));
    }

    #[test]
    fn queueing_a_frame_does_not_start_the_worker_watchdog() {
        let frame_watchdog = frame_watchdog();
        let client = Client {
            commands: Arc::new(CommandChannel::new(1)),
            completed: Arc::new(CompletedFrameChannel::new(1)),
            present: Arc::new(PresentFrameChannel::new()),
            status: Arc::new(SharedStatus::new(CompositorStatus::Healthy)),
            child: Arc::new(Mutex::new(None)),
            worker: None,
            frame_watchdog: Arc::clone(&frame_watchdog),
        };

        client.send(1, 1, 1, PaintSnapshotParams::default());
        assert!(frame_watchdog.lock().unwrap().last_progress.is_none());
    }

    struct ScriptedTransport {
        sent: Arc<Mutex<Vec<IpcMessage>>>,
        responses: VecDeque<Result<IpcMessage, ProtocolError>>,
    }

    struct LockstepTransport {
        sent: Arc<Mutex<Vec<IpcMessage>>>,
        responses: VecDeque<Result<IpcMessage, ProtocolError>>,
        awaiting_response: bool,
    }

    impl WorkerTransport for LockstepTransport {
        fn send(&mut self, message: IpcMessage) -> Result<(), ProtocolError> {
            if self.awaiting_response {
                return Err(ProtocolError::Channel(
                    "sent another command before reading the previous response".to_string(),
                ));
            }
            self.sent.lock().unwrap().push(message);
            self.awaiting_response = true;
            Ok(())
        }

        fn recv(&mut self) -> Result<IpcMessage, ProtocolError> {
            if !self.awaiting_response {
                return Err(ProtocolError::Channel(
                    "read without an outstanding command".to_string(),
                ));
            }
            self.awaiting_response = false;
            self.responses.pop_front().expect("lockstep response")
        }
    }

    impl WorkerTransport for ScriptedTransport {
        fn send(&mut self, message: IpcMessage) -> Result<(), ProtocolError> {
            self.sent.lock().unwrap().push(message);
            Ok(())
        }

        fn recv(&mut self) -> Result<IpcMessage, ProtocolError> {
            self.responses.pop_front().expect("scripted response")
        }
    }

    #[test]
    fn pending_commands_are_latest_wins_and_bounded() {
        let commands = CommandChannel::new(2);
        assert!(commands.send_frame(pending(1, 1, 1)));
        assert!(commands.send_frame(pending(1, 1, 2)));
        assert_eq!(commands.len(), 1);
        let batch = commands.recv().unwrap();
        assert_eq!(batch.len(), 1);
        assert!(matches!(
            &batch[0],
            WorkerCommand::Frame(frame) if frame.frame_id == 2
        ));

        assert!(commands.send_frame(pending(1, 1, 3)));
        assert!(commands.send_frame(pending(2, 1, 1)));
        assert!(commands.send_frame(pending(3, 1, 1)));
        assert_eq!(commands.len(), 2);
        let surfaces: Vec<_> = commands
            .recv()
            .unwrap()
            .into_iter()
            .map(|command| command.surface_id())
            .collect();
        assert_eq!(surfaces, vec![2, 3]);
    }

    #[test]
    fn release_surface_replaces_pending_frame_and_sends_control_message() {
        let commands = CommandChannel::new(2);
        assert!(commands.send_frame(pending(44, 1, 3)));
        assert!(commands.release_surface(44));
        let batch = commands.recv().unwrap();
        assert!(matches!(batch.as_slice(), [WorkerCommand::ReleaseSurface(44)]));

        let sent = Arc::new(Mutex::new(Vec::new()));
        let mut transport = ScriptedTransport {
            sent: Arc::clone(&sent),
            responses: VecDeque::from([Ok(IpcMessage {
                id: 1,
                kind: IpcMessageKind::Ok,
            })]),
        };
        let completed = CompletedFrameChannel::new(1);
        let present = PresentFrameChannel::new();
        let mut next_message_id = 1;
        let frame_watchdog = frame_watchdog();
        process_batch(
            &mut transport,
            batch,
            &completed,
            &present,
            &mut next_message_id,
            &frame_watchdog,
        )
        .unwrap();

        let sent = sent.lock().unwrap();
        assert!(matches!(
            sent.as_slice(),
            [IpcMessage {
                id: 1,
                kind: IpcMessageKind::ReleaseCompositorSurface { surface_id: 44 }
            }]
        ));
    }

    #[test]
    fn scroll_ack_requests_and_delivers_the_refreshed_compositor_frame() {
        let key = FrameKey {
            surface_id: 44,
            navigation_epoch: 3,
            frame_id: 8,
        };
        let sent = Arc::new(Mutex::new(Vec::new()));
        let mut transport = ScriptedTransport {
            sent: Arc::clone(&sent),
            responses: VecDeque::from([
                Ok(IpcMessage {
                    id: 1,
                    kind: IpcMessageKind::Ok,
                }),
                Ok(data(2, key, 7)),
            ]),
        };
        let completed = CompletedFrameChannel::new(1);
        let present = PresentFrameChannel::new();
        let mut next_message_id = 1;

        process_batch(
            &mut transport,
            vec![WorkerCommand::SetScroll {
                surface_id: key.surface_id,
                navigation_epoch: key.navigation_epoch,
                frame_id: key.frame_id,
                scroll_x: 0.0,
                scroll_y: 320.0,
            }],
            &completed,
            &present,
            &mut next_message_id,
            &frame_watchdog(),
        )
        .unwrap();

        let sent = sent.lock().unwrap();
        assert!(matches!(sent[0].kind, IpcMessageKind::CompositorSetScroll { .. }));
        assert!(matches!(sent[1].kind, IpcMessageKind::GetCompositorFrame { .. }));
        assert_eq!(
            completed
                .try_recv(key.surface_id, key.navigation_epoch, key.frame_id)
                .unwrap()
                .rgba[0],
            7
        );
    }

    #[test]
    fn completed_frames_are_latest_wins_and_bounded() {
        let completed = CompletedFrameChannel::new(2);
        for (surface_id, frame_id) in [(1, 1), (1, 2), (2, 1), (3, 1)] {
            completed.send(CompletedFrame {
                key: FrameKey {
                    surface_id,
                    navigation_epoch: 1,
                    frame_id,
                },
                width: 1,
                height: 1,
                rgba: vec![0; 4],
                scroll_x: 0.0,
                scroll_y: 0.0,
                #[cfg(target_os = "linux")]
                dmabuf: None,
            });
        }
        assert_eq!(completed.len(), 2);
        assert!(completed.try_recv(1, 1, 0).is_none());
        assert_eq!(completed.try_recv(3, 1, 1).unwrap().key.frame_id, 1);
    }

    #[test]
    fn worker_completes_each_frame_before_sending_the_next() {
        let first = FrameKey {
            surface_id: 10,
            navigation_epoch: 4,
            frame_id: 1,
        };
        let second = FrameKey {
            surface_id: 20,
            navigation_epoch: 7,
            frame_id: 2,
        };
        let sent = Arc::new(Mutex::new(Vec::new()));
        let mut transport = LockstepTransport {
            sent: Arc::clone(&sent),
            responses: VecDeque::from([
                Ok(result(1, first)),
                Ok(data(2, first, 11)),
                Ok(result(3, second)),
                Ok(data(4, second, 22)),
            ]),
            awaiting_response: false,
        };
        let completed = CompletedFrameChannel::new(4);
        let present = PresentFrameChannel::new();
        let mut next_message_id = 1;

        let frame_watchdog = frame_watchdog();
        process_batch(
            &mut transport,
            vec![
                WorkerCommand::Frame(Box::new(pending(10, 4, 1))),
                WorkerCommand::Frame(Box::new(pending(20, 7, 2))),
            ],
            &completed,
            &present,
            &mut next_message_id,
            &frame_watchdog,
        )
        .unwrap();

        assert_eq!(sent.lock().unwrap().len(), 4);
        assert!(frame_watchdog.lock().unwrap().last_progress.is_none());
        assert_eq!(completed.try_recv(10, 4, 1).unwrap().rgba[0], 11);
        assert_eq!(completed.try_recv(20, 7, 2).unwrap().rgba[0], 22);
    }

    struct BlockingTransport {
        entered: Option<mpsc::SyncSender<()>>,
        release: mpsc::Receiver<()>,
    }

    impl WorkerTransport for BlockingTransport {
        fn send(&mut self, _message: IpcMessage) -> Result<(), ProtocolError> {
            Ok(())
        }

        fn recv(&mut self) -> Result<IpcMessage, ProtocolError> {
            if let Some(entered) = self.entered.take() {
                entered.send(()).unwrap();
            }
            let _ = self.release.recv();
            Err(ProtocolError::Channel("fake disconnect".to_string()))
        }
    }

    #[test]
    fn ui_submission_does_not_wait_for_blocking_transport() {
        let commands = Arc::new(CommandChannel::new(4));
        let completed = Arc::new(CompletedFrameChannel::new(4));
        let status = Arc::new(SharedStatus::new(CompositorStatus::Starting));
        let (entered_tx, entered_rx) = mpsc::sync_channel(1);
        let (release_tx, release_rx) = mpsc::sync_channel(1);
        let worker_commands = Arc::clone(&commands);
        let worker_completed = Arc::clone(&completed);
        let worker_status = Arc::clone(&status);
        let worker_present = Arc::new(PresentFrameChannel::new());
        let frame_watchdog = frame_watchdog();
        let worker_frame_watchdog = Arc::clone(&frame_watchdog);
        let worker = thread::spawn(move || {
            worker_main(
                worker_commands,
                worker_completed,
                worker_present,
                worker_status,
                worker_frame_watchdog,
                || {
                    Ok(Box::new(BlockingTransport {
                        entered: Some(entered_tx),
                        release: release_rx,
                    }))
                },
            );
        });

        assert!(commands.send_frame(pending(1, 1, 1)));
        entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert!(
            frame_watchdog
                .lock()
                .unwrap()
                .is_stalled(std::time::Instant::now() + Duration::from_secs(11))
        );
        let (done_tx, done_rx) = mpsc::sync_channel(1);
        let submit_commands = Arc::clone(&commands);
        thread::spawn(move || {
            let sent = submit_commands.send_frame(pending(1, 1, 2));
            done_tx.send(sent).unwrap();
        });
        assert!(done_rx.recv_timeout(Duration::from_millis(100)).unwrap());

        release_tx.send(()).unwrap();
        worker.join().unwrap();
        assert_eq!(status.load(), CompositorStatus::Disconnected);
    }

    #[test]
    fn worker_disconnect_is_observable_and_clears_cache() {
        let commands = Arc::new(CommandChannel::new(2));
        let completed = Arc::new(CompletedFrameChannel::new(2));
        completed.send(CompletedFrame {
            key: FrameKey {
                surface_id: 1,
                navigation_epoch: 1,
                frame_id: 1,
            },
            width: 1,
            height: 1,
            rgba: vec![0; 4],
            scroll_x: 0.0,
            scroll_y: 0.0,
            #[cfg(target_os = "linux")]
            dmabuf: None,
        });
        let status = Arc::new(SharedStatus::new(CompositorStatus::Starting));
        let worker_commands = Arc::clone(&commands);
        let worker_completed = Arc::clone(&completed);
        let worker_status = Arc::clone(&status);
        let worker_present = Arc::new(PresentFrameChannel::new());
        let worker = thread::spawn(move || {
            worker_main(
                worker_commands,
                worker_completed,
                worker_present,
                worker_status,
                frame_watchdog(),
                || {
                    Ok(Box::new(ScriptedTransport {
                        sent: Arc::new(Mutex::new(Vec::new())),
                        responses: VecDeque::from([Err(ProtocolError::Channel("closed".to_string()))]),
                    }))
                },
            );
        });
        assert!(commands.send_frame(pending(1, 1, 1)));
        worker.join().unwrap();

        assert_eq!(status.load(), CompositorStatus::Disconnected);
        assert_eq!(completed.len(), 0);
        assert!(!commands.send_frame(pending(1, 1, 2)));
    }

    #[test]
    fn client_shutdown_closes_idle_worker_without_holding_global_lock() {
        let commands = Arc::new(CommandChannel::new(1));
        let completed = Arc::new(CompletedFrameChannel::new(1));
        let status = Arc::new(SharedStatus::new(CompositorStatus::Starting));
        let worker_commands = Arc::clone(&commands);
        let worker_completed = Arc::clone(&completed);
        let worker_status = Arc::clone(&status);
        let worker_present = Arc::new(PresentFrameChannel::new());
        let worker = thread::spawn(move || {
            worker_main(
                worker_commands,
                worker_completed,
                worker_present,
                worker_status,
                frame_watchdog(),
                || {
                    Ok(Box::new(ScriptedTransport {
                        sent: Arc::new(Mutex::new(Vec::new())),
                        responses: VecDeque::new(),
                    }))
                },
            );
        });

        while status.load() == CompositorStatus::Starting {
            thread::yield_now();
        }
        let mut client = Client {
            frame_watchdog: frame_watchdog(),
            commands,
            completed,
            present: Arc::new(PresentFrameChannel::new()),
            status: Arc::clone(&status),
            child: Arc::new(Mutex::new(None)),
            worker: Some(worker),
        };
        client.shutdown();

        assert!(client.worker.is_none());
        assert_eq!(status.load(), CompositorStatus::Healthy);
    }

    #[test]
    fn invalid_frame_data_is_rejected() {
        assert!(validate_frame_data(1, 1, &[0; 3]).is_err());
        assert!(validate_frame_data(u32::MAX, u32::MAX, &[]).is_err());
    }

    #[test]
    fn compositor_is_always_enabled() {
        assert!(enabled());
    }
}
