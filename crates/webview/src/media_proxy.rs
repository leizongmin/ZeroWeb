//! 媒体宿主进程代理（`media-process` feature 门控；方案 C C2 客户端面）。
//!
//! webview/renderer 侧的媒体 host 分发面（IF-001 会话与生命周期）：
//! - 懒惰 spawn：首个媒体源登记时启动 `zero-media-host`（image_decoder.rs:211 先例）；
//!   每 webview 至多一个 host（registry 持有，C2 接线）。
//! - 握手：spawn 后等待 host 首发 `MediaReady`（5s 超时），校验协议版本；不符按
//!   spawn 失败处理。
//! - 降级纪律（IF-006，C3 重启面）：初始 spawn/握手失败 → 永久错误面（本生命周期
//!   不重试、**不内联回落**——image_decoder SPAWN_FAILED 先例语义镜像）；运行期
//!   连接失效（send 失败即时检测 + 泵周期 `try_wait` 兜底，DESIGN §11.2 双路）→
//!   退避重启（1s/2s/4s，n≤3，握手成功重置计数）→ 耗尽转永久错误面；host 回复的
//!   `MediaError`（配额/播放错误）为播放级错误面，不置失败（host 进程仍健康）。
//!   显式 shutdown/teardown 回 Idle——reload 后全新 host（DC-3「重载恢复」）。
//! - 事件面（C2）：[`MediaHostClient::try_recv_event`] 非阻塞排空 host 自发推送
//!   （状态/帧/元数据/错误）——渲染泵被动消费「排空待决帧队列」的取数口（DESIGN
//!   §10 push 供数）；未连接时不触发懒 spawn（无 host 即无事件）。
//!
//! 读路径：后台线程持 stdout 只读传输（`PipeTransport` 的 `try_recv` 不支持非阻塞，
//! 线程化收信是泵被动消费的前置），主线程持 stdin 只写传输；等待特定响应期间非匹配
//! 消息进独立 pending 队列（等待退出后按序恢复，防自旋丢信）。

use std::collections::VecDeque;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, TryRecvError, channel};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use zero_protocol::media::{
    MEDIA_PROTOCOL_VERSION, MediaDataAckParams, MediaDataChunkParams, MediaErrorCode, MediaErrorParams,
    MediaFrameParams, MediaPauseParams, MediaPlayParams, MediaReadyParams, MediaRegisterSourceParams,
    MediaReleaseSourceParams, MediaSeekParams, MediaSeekResultParams, MediaSetGainParams, MediaSetLoopParams,
    MediaSetRateParams, MediaSourceReadyParams, MediaStateUpdateParams,
};
use zero_protocol::message::{IpcMessage, IpcMessageKind};
use zero_protocol::transport::PipeTransport;
use zero_protocol::{IpcChannel, ProcessRole, child_process_args};

/// 握手/请求等待超时（设计定值，5s 量级）。
const WAIT_TIMEOUT: Duration = Duration::from_secs(5);

/// 崩溃重启退避序列（DESIGN §11.3——重试 n≤3：1s/2s/4s；握手成功重置计数）。
const RESTART_BACKOFF: [Duration; 3] = [Duration::from_secs(1), Duration::from_secs(2), Duration::from_secs(4)];

/// 保留 host stderr 的最后部分用于崩溃诊断（16KiB 截断——子进程不因管道写满
/// 阻塞；renderer 先例 zero-protocol process.rs `STDERR_TAIL_LIMIT` 同值）。
const STDERR_TAIL_LIMIT: usize = 16 * 1024;

/// 宿主连接生命周期（DESIGN §11.3 状态机）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HostLifecycle {
    /// 未 spawn（初始或 shutdown/teardown 后——下次命令懒惰重 spawn，reload 恢复路径）。
    Idle,
    /// 握手成功，连接健康。
    Healthy,
    /// 运行期连接失效后的退避重启中（`attempt` = 已失败的重试次数；`next_at` =
    /// 下次重连时刻）。
    Restarting { attempt: u32, next_at: Instant },
    /// 重启耗尽或初始 spawn 失败——本 webview 生命周期永久错误面（不内联回落；
    /// teardown 后回 [`HostLifecycle::Idle`]，DC-3「重载恢复」）。
    Failed,
}

/// 媒体 host 操作错误（IF-006 错误面）。
#[derive(Debug, Clone)]
pub struct MediaHostError {
    /// 结构化错误码（传输/spawn 失败归 `HostInternal`）。
    pub code: MediaErrorCode,
    /// 人类可读诊断。
    pub message: String,
}

impl MediaHostError {
    fn internal(message: impl Into<String>) -> Self {
        Self {
            code: MediaErrorCode::HostInternal,
            message: message.into(),
        }
    }
}

/// host 自发推送事件（[`MediaHostClient::try_recv_event`] 消费面；非请求响应）。
#[derive(Debug)]
pub enum MediaHostEvent {
    /// 状态/时钟推送（DC-2：epoch 过期数据丢弃归镜像面）。
    State(MediaStateUpdateParams),
    /// 解码帧（内联 `rgba`；shm 传输面归后续切片）。
    Frame(MediaFrameParams),
    /// 元数据推送（play 建解码器后的 `SourceReady` 真值，id 0）。
    SourceReady(MediaSourceReadyParams),
    /// 错误推送（播放级，不置永久失败——IF-006 边界；含先前等待超时的请求级
    /// 错误补投）。
    Error(MediaErrorParams),
}

impl std::fmt::Display for MediaHostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "media-host {:?}: {}", self.code, self.message)
    }
}

/// media-host 二进制文件名（平台后缀）。
fn media_host_binary_filename() -> &'static str {
    #[cfg(windows)]
    {
        "zero-media-host.exe"
    }
    #[cfg(not(windows))]
    {
        "zero-media-host"
    }
}

fn media_host_candidates_near_executable(exe: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(parent) = exe.parent() {
        candidates.push(parent.join(media_host_binary_filename()));
        if let Some(grandparent) = parent.parent() {
            candidates.push(grandparent.join(media_host_binary_filename()));
        }
    }
    candidates
}

/// 解析 zero-media-host 可执行文件路径。
///
/// 与 image-decoder 同模式查找链（guidelines #12：完整链 + 找不到显式落错误面）：
/// `ZW_MEDIA_HOST_BIN` → `CARGO_BIN_EXE_zero-media-host` → current_exe 同目录
/// （测试二进制 `target/debug/deps/` 上溯 `target/debug/`）→ PATH 兜底。
fn resolve_media_host_bin() -> PathBuf {
    if let Some(bin) = zero_runtime_config::optional_path("ZW_MEDIA_HOST_BIN") {
        return bin;
    }
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_zero-media-host")
        && Path::new(&path).is_file()
    {
        return PathBuf::from(path);
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(candidate) = media_host_candidates_near_executable(&exe)
            .into_iter()
            .find(|path| path.is_file())
    {
        return candidate;
    }
    PathBuf::from(media_host_binary_filename())
}

/// 校验 host 握手（IF-001：版本不符按 spawn 失败处理）。
fn check_handshake(ready: &MediaReadyParams) -> Result<(), String> {
    if ready.protocol_version != MEDIA_PROTOCOL_VERSION {
        return Err(format!(
            "media-host protocol version {} != expected {MEDIA_PROTOCOL_VERSION}",
            ready.protocol_version
        ));
    }
    Ok(())
}

/// 有界 stderr tail 追加（超限丢头部——保最后 16KiB；`Mutex` 中毒忽略：诊断面
/// 不因锁中毒放大失败，renderer 先例 process.rs `append_stderr_tail` 同构）。
fn append_stderr_tail(tail: &Mutex<VecDeque<u8>>, chunk: &[u8]) {
    let Ok(mut tail) = tail.lock() else {
        return;
    };
    tail.extend(chunk.iter().copied());
    while tail.len() > STDERR_TAIL_LIMIT {
        tail.pop_front();
    }
}

/// 读取 stderr tail 快照（截断 trim）。
fn stderr_tail_text(tail: &Mutex<VecDeque<u8>>) -> String {
    let Ok(tail) = tail.lock() else {
        return String::new();
    };
    let bytes: Vec<u8> = tail.iter().copied().collect();
    String::from_utf8_lossy(&bytes).trim().to_string()
}

/// 与 media-host 子进程的连接（读线程 + 只写传输 + 消息 id 计数 + stderr tail）。
struct HostConnection {
    child: Child,
    /// stdin 只写传输（读半由 [`spawn_reader`] 线程持有；Drop 时关闭 → host EOF 退出）。
    writer: Option<PipeTransport<io::Empty, ChildStdin>>,
    /// 读线程推送的入站消息（`Err` = 传输失效）。
    inbound: Receiver<Result<IpcMessage, String>>,
    reader: Option<JoinHandle<()>>,
    next_id: u64,
    /// 崩溃诊断 tail（后台线程持续收纳，进程退出后读快照——`exit_reason` 面）。
    stderr_tail: Arc<Mutex<VecDeque<u8>>>,
    stderr_reader: Option<JoinHandle<()>>,
}

/// 后台读线程：阻塞 recv → mpsc 推送；断开/错误推送 `Err` 后退出。
fn spawn_reader(
    mut reader: PipeTransport<ChildStdout, io::Sink>,
    tx: Sender<Result<IpcMessage, String>>,
) -> JoinHandle<()> {
    std::thread::Builder::new()
        .name("media-host-reader".into())
        .spawn(move || {
            loop {
                match reader.recv() {
                    Ok(message) => {
                        if tx.send(Ok(message)).is_err() {
                            return; // 主端已弃置连接
                        }
                    }
                    Err(error) => {
                        let _ = tx.send(Err(error.to_string()));
                        return;
                    }
                }
            }
        })
        .expect("media-host reader thread spawn")
}

impl HostConnection {
    fn spawn() -> Result<Self, String> {
        let bin = resolve_media_host_bin();
        let mut cmd = Command::new(&bin);
        for arg in child_process_args(ProcessRole::Media, 0) {
            cmd.arg(arg);
        }
        // Windows：阻止子进程分配控制台窗口（与 zero-protocol spawn 同款双保险）。
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("media-host spawn 失败（{bin:?}）: {e}"))?;
        let stdout: ChildStdout = child.stdout.take().ok_or("无法获取 media-host stdout 管道")?;
        let stdin: ChildStdin = child.stdin.take().ok_or("无法获取 media-host stdin 管道")?;
        let mut stderr: ChildStderr = child.stderr.take().ok_or("无法获取 media-host stderr 管道")?;
        let (tx, inbound) = channel();
        let reader = spawn_reader(PipeTransport::new(stdout, io::sink()), tx);
        // stderr 收纳线程：持续读入有界 tail（崩溃诊断面——DESIGN §11.4）。生产
        // 环境若无消费（无崩溃）仅占 16KiB 上界内存。
        let stderr_tail = Arc::new(Mutex::new(VecDeque::with_capacity(STDERR_TAIL_LIMIT)));
        let stderr_reader = {
            let tail = Arc::clone(&stderr_tail);
            std::thread::Builder::new()
                .name("media-host-stderr".into())
                .spawn(move || {
                    let mut buffer = [0; 4096];
                    loop {
                        match stderr.read(&mut buffer) {
                            Ok(0) | Err(_) => break,
                            Ok(read) => append_stderr_tail(&tail, &buffer[..read]),
                        }
                    }
                })
                .expect("media-host stderr reader thread spawn")
        };
        Ok(Self {
            child,
            writer: Some(PipeTransport::new(io::empty(), stdin)),
            inbound,
            reader: Some(reader),
            next_id: 1,
            stderr_tail,
            stderr_reader: Some(stderr_reader),
        })
    }

    /// try_wait 退出诊断（`None` = 仍在运行；renderer `exit_reason` 先例同构——
    /// zero-protocol process.rs:441，崩溃管理器语义）。
    fn exit_reason(&mut self) -> Option<String> {
        match self.child.try_wait() {
            Ok(None) => None,
            Ok(Some(status)) => Some(format!("media-host exited: {status}")),
            Err(error) => Some(format!("无法检查 media-host 进程状态: {error}")),
        }
    }

    /// 发送控制消息（fire-and-forget 或请求的发送半），返回信封 id。
    fn send(&mut self, kind: IpcMessageKind) -> Result<u64, String> {
        let id = self.next_id;
        self.next_id += 1;
        self.writer
            .as_mut()
            .ok_or_else(|| "media-host 写半已关闭".to_string())?
            .send(IpcMessage { id, kind })
            .map_err(|e| format!("media-host send: {e}"))?;
        Ok(id)
    }

    /// 等待信封 `request_id` 的响应；非匹配消息进 `pending`（按序暂存，稍后恢复）。
    fn wait_for(&mut self, pending: &mut VecDeque<IpcMessage>, request_id: u64) -> Result<IpcMessage, String> {
        while let Some(message) = pending.pop_front() {
            if message.id == request_id {
                return Ok(message);
            }
        }
        let deadline = Instant::now() + WAIT_TIMEOUT;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match self.inbound.recv_timeout(remaining) {
                Ok(Ok(message)) => {
                    if message.id == request_id {
                        return Ok(message);
                    }
                    pending.push_back(message);
                }
                Ok(Err(error)) => return Err(format!("media-host 传输失效: {error}")),
                Err(RecvTimeoutError::Timeout) => return Err("media-host 响应超时".to_string()),
                Err(RecvTimeoutError::Disconnected) => return Err("media-host 读线程已退出".to_string()),
            }
        }
    }
}

impl Drop for HostConnection {
    fn drop(&mut self) {
        // 关闭写半 → host 见 stdin EOF 自行退出；兜底 kill 防宿主异常挂死
        // （RendererHandle::shutdown 先例：避免只关 stdin 后仍向 stdout 写刷屏）。
        drop(self.writer.take());
        let deadline = Instant::now() + WAIT_TIMEOUT;
        while Instant::now() < deadline {
            match self.child.try_wait() {
                Ok(Some(_)) | Err(_) => break,
                Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            }
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(handle) = self.reader.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.stderr_reader.take() {
            let _ = handle.join();
        }
    }
}

/// 媒体宿主客户端（per-webview 所有权；C2 起 registry 持有）。
///
/// 降级纪律（IF-006，C3 重启面）：初始 spawn/握手失败 → 永久错误面（不重试、
/// 不内联回落）；运行期连接失效 → 退避重启（[`Self::poll_health`] 泵周期推进，
/// n≤3）→ 耗尽转永久错误面；显式 [`Self::shutdown`] 回 Idle（reload 恢复）。
pub struct MediaHostClient {
    connection: Option<HostConnection>,
    /// 非匹配入站消息（等待特定响应期间暂存，按序恢复）。
    pending: VecDeque<IpcMessage>,
    lifecycle: HostLifecycle,
    /// 待消费崩溃通知（one-shot：[`Self::poll_health`] 取走后清零——镜像广播
    /// 归 facade，§11.3）。
    crash_pending: Option<MediaHostError>,
}

impl Default for MediaHostClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaHostClient {
    /// 创建未连接的客户端（首个操作时懒惰 spawn + 握手）。
    pub fn new() -> Self {
        Self {
            connection: None,
            pending: VecDeque::new(),
            lifecycle: HostLifecycle::Idle,
            crash_pending: None,
        }
    }

    /// 是否已进入永久错误面（重启耗尽或初始 spawn 失败；teardown 后复位）。
    pub fn is_failed(&self) -> bool {
        self.lifecycle == HostLifecycle::Failed
    }

    /// 宿主子进程 pid（诊断/契约测试 SIGKILL 注入面；未连接 `None`）。
    pub fn host_pid(&self) -> Option<u32> {
        self.connection.as_ref().map(|conn| conn.child.id())
    }

    /// 连接失效处理（检测双路汇聚点，DESIGN §11.2/§11.3）：弃置前取退出诊断
    ///（`Drop` 会 kill 收尸）→ stderr tail 入 tracing → 崩溃通知挂起（one-shot）
    /// → Healthy 转重启退避；pending 作废（新连接 id 重新计数，旧响应不得误配）。
    fn connection_lost(&mut self, mut reason: String, conn: &mut HostConnection) -> MediaHostError {
        if let Some(exit) = conn.exit_reason() {
            reason = exit; // 进程已退：exit status 权威（send 面只见 broken pipe）
        }
        let tail = stderr_tail_text(&conn.stderr_tail);
        if tail.is_empty() {
            tracing::warn!("media-host 连接失效 → 重启面: {reason}");
        } else {
            tracing::warn!("media-host 连接失效 → 重启面: {reason}; stderr tail: {tail}");
        }
        self.pending.clear();
        let crash = MediaHostError::internal(reason);
        self.crash_pending = Some(crash.clone());
        self.lifecycle = match self.lifecycle {
            HostLifecycle::Healthy => HostLifecycle::Restarting {
                attempt: 1,
                next_at: Instant::now() + RESTART_BACKOFF[0],
            },
            _ => HostLifecycle::Failed,
        };
        crash
    }

    /// 泵周期健康巡检（检测双路之兜底，DESIGN §11.2-b——`try_wait` 覆盖在途
    /// 消息/无写入期；16ms tick 下 syscall 成本可忽略）+ 重启退避推进。
    ///
    /// 返回 `Err` = one-shot 崩溃通知（本次或先前命令路径检测到连接失效）——调用方
    /// 应对全部活跃源广播 `MediaError{HostInternal}`（§11.3）；后续轮询返回 `Ok`。
    /// 空闲/退避未到期恒 `Ok`。
    pub fn poll_health(&mut self) -> Result<(), MediaHostError> {
        if let Some(crash) = self.crash_pending.take() {
            return Err(crash);
        }
        match self.lifecycle {
            HostLifecycle::Healthy => {
                let mut exited = None;
                if let Some(conn) = self.connection.as_mut() {
                    exited = conn.exit_reason();
                }
                if let Some(reason) = exited {
                    let mut conn = self.connection.take().expect("connection checked above");
                    let crash = self.connection_lost(reason, &mut conn);
                    drop(conn);
                    // 同步返回面已送达本通知（connection_lost 同时挂起的 one-shot
                    // 不再二次投递——命令路径的存而待发语义不受影响）。
                    self.crash_pending = None;
                    return Err(crash);
                }
                Ok(())
            }
            HostLifecycle::Restarting { attempt, next_at } => {
                if Instant::now() < next_at {
                    return Ok(());
                }
                match Self::connect() {
                    Ok(conn) => {
                        self.connection = Some(conn);
                        self.lifecycle = HostLifecycle::Healthy;
                        tracing::info!("media-host 重启成功（第 {attempt} 次尝试，计数重置）");
                        Ok(())
                    }
                    Err(error) => {
                        if attempt >= RESTART_BACKOFF.len() as u32 {
                            tracing::warn!("media-host 重启耗尽（{error}）→ 永久错误面（不内联回落）");
                            self.lifecycle = HostLifecycle::Failed;
                        } else {
                            let next = attempt + 1;
                            let delay = RESTART_BACKOFF[(next - 1) as usize];
                            tracing::warn!("media-host 重启尝试 {attempt}/3 失败（{error}），{delay:?} 后重试");
                            self.lifecycle = HostLifecycle::Restarting {
                                attempt: next,
                                next_at: Instant::now() + delay,
                            };
                        }
                        Ok(())
                    }
                }
            }
            HostLifecycle::Idle | HostLifecycle::Failed => Ok(()),
        }
    }

    /// 在活动连接上执行操作。
    ///
    /// 外层 `Err(String)` = 传输级失败（转重启面并弃置连接）；内层
    /// `Err(MediaHostError)` = host 协议错误面（连接保持健康，不置失败）。
    fn with_conn<R>(
        &mut self,
        op: impl FnOnce(&mut HostConnection, &mut VecDeque<IpcMessage>) -> Result<Result<R, MediaHostError>, String>,
    ) -> Result<R, MediaHostError> {
        match self.lifecycle {
            HostLifecycle::Failed => {
                return Err(MediaHostError::internal(
                    "media-host 永久错误面（重启耗尽；reload teardown 后恢复）",
                ));
            }
            HostLifecycle::Restarting { .. } => {
                return Err(MediaHostError::internal("media-host 重启退避中（泵 tick 推进重连）"));
            }
            HostLifecycle::Idle | HostLifecycle::Healthy => {}
        }
        if self.connection.is_none() {
            match Self::connect() {
                Ok(conn) => {
                    self.connection = Some(conn);
                    self.lifecycle = HostLifecycle::Healthy;
                }
                Err(error) => {
                    // 初始 spawn/握手失败 → 本生命周期永久错误面（IF-006/§11.1，
                    // 不重试不内联；reload teardown 回 Idle 后可恢复）。
                    self.lifecycle = HostLifecycle::Failed;
                    tracing::warn!("{error}");
                    return Err(MediaHostError::internal(error));
                }
            }
        }
        let mut conn = self.connection.take().expect("connection checked above");
        match op(&mut conn, &mut self.pending) {
            Ok(inner) => {
                self.connection = Some(conn);
                inner
            }
            Err(error) => {
                // 传输级失败（§11.2-a send 失败即时检测）：弃置前取退出诊断，
                // 崩溃通知经 [`Self::poll_health`] one-shot 下发。
                let crash = self.connection_lost(error, &mut conn);
                drop(conn);
                Err(crash)
            }
        }
    }

    /// spawn + 握手（版本校验；不符按 spawn 失败处理）。
    fn connect() -> Result<HostConnection, String> {
        let conn = HostConnection::spawn()?;
        let ready = match conn.inbound.recv_timeout(WAIT_TIMEOUT) {
            Ok(Ok(message)) => message,
            Ok(Err(error)) => return Err(format!("media-host 握手传输失效: {error}")),
            Err(RecvTimeoutError::Timeout) => return Err("media-host 握手超时".to_string()),
            Err(RecvTimeoutError::Disconnected) => return Err("media-host 握手读线程已退出".to_string()),
        };
        match ready.kind {
            IpcMessageKind::MediaReady(params) => check_handshake(&params).map(|_| conn),
            other => Err(format!("media-host 首消息非 MediaReady: {other:?}")),
        }
    }

    /// 登记媒体源（响应 `MediaSourceReady` / `MediaError{QuotaExceeded}`）。
    pub fn register_source(
        &mut self,
        source_key: u64,
        total_len: u64,
        mime: &str,
    ) -> Result<MediaSourceReadyParams, MediaHostError> {
        self.with_conn(|conn, pending| {
            let id = conn.send(IpcMessageKind::MediaRegisterSource(MediaRegisterSourceParams {
                source_key,
                total_len,
                mime: mime.to_string(),
            }))?;
            Ok(match conn.wait_for(pending, id)? {
                IpcMessage {
                    kind: IpcMessageKind::MediaSourceReady(ready),
                    ..
                } => Ok(ready),
                IpcMessage {
                    kind: IpcMessageKind::MediaError(err),
                    ..
                } => Err(err.into()),
                other => Err(MediaHostError::internal(format!(
                    "register_source 响应类型异常: {:?}",
                    other.kind
                ))),
            })
        })
    }

    /// 喂入源字节分片（响应 `MediaDataAck`）。
    pub fn send_data_chunk(
        &mut self,
        source_key: u64,
        seq: u64,
        data: Vec<u8>,
    ) -> Result<MediaDataAckParams, MediaHostError> {
        self.with_conn(|conn, pending| {
            let id = conn.send(IpcMessageKind::MediaDataChunk(MediaDataChunkParams {
                source_key,
                seq,
                data,
            }))?;
            Ok(match conn.wait_for(pending, id)? {
                IpcMessage {
                    kind: IpcMessageKind::MediaDataAck(ack),
                    ..
                } => Ok(ack),
                IpcMessage {
                    kind: IpcMessageKind::MediaError(err),
                    ..
                } => Err(err.into()),
                other => Err(MediaHostError::internal(format!(
                    "send_data_chunk 响应类型异常: {:?}",
                    other.kind
                ))),
            })
        })
    }

    /// 起播/重播。成功即回 `MediaStateUpdate{Playing}` 载荷（内核即时推送兼作
    /// 回执，IF-002 id 配对；media_time_ms 为镜像即时真值）；未知源/字节未齐/
    /// 解码失败为 `MediaError` 错误面。
    pub fn play(&mut self, source_key: u64, position_ms: u64) -> Result<MediaStateUpdateParams, MediaHostError> {
        self.with_conn(|conn, pending| {
            let id = conn.send(IpcMessageKind::MediaPlay(MediaPlayParams {
                source_key,
                position_ms,
            }))?;
            Ok(match conn.wait_for(pending, id)? {
                IpcMessage {
                    kind: IpcMessageKind::MediaStateUpdate(state),
                    ..
                } => Ok(state),
                IpcMessage {
                    kind: IpcMessageKind::MediaError(err),
                    ..
                } => Err(err.into()),
                other => Err(MediaHostError::internal(format!("play 响应类型异常: {:?}", other.kind))),
            })
        })
    }

    /// 暂停（成功即回 `MediaStateUpdate{Paused}` 载荷，IF-002 id 配对）。
    pub fn pause(&mut self, source_key: u64) -> Result<MediaStateUpdateParams, MediaHostError> {
        self.with_conn(|conn, pending| {
            let id = conn.send(IpcMessageKind::MediaPause(MediaPauseParams { source_key }))?;
            Ok(match conn.wait_for(pending, id)? {
                IpcMessage {
                    kind: IpcMessageKind::MediaStateUpdate(state),
                    ..
                } => Ok(state),
                IpcMessage {
                    kind: IpcMessageKind::MediaError(err),
                    ..
                } => Err(err.into()),
                other => Err(MediaHostError::internal(format!(
                    "pause 响应类型异常: {:?}",
                    other.kind
                ))),
            })
        })
    }

    /// 精确 seek（响应 `MediaSeekResult{request_id}` 配对；结果经事件面/回执
    /// 双达——回执为同步等待面）。
    pub fn seek(&mut self, source_key: u64, target_ms: u64) -> Result<MediaSeekResultParams, MediaHostError> {
        self.with_conn(|conn, pending| {
            let request_id = conn.next_id;
            let id = conn.send(IpcMessageKind::MediaSeek(MediaSeekParams {
                source_key,
                target_ms,
                request_id,
            }))?;
            Ok(match conn.wait_for(pending, id)? {
                IpcMessage {
                    kind: IpcMessageKind::MediaSeekResult(result),
                    ..
                } => Ok(result),
                IpcMessage {
                    kind: IpcMessageKind::MediaError(err),
                    ..
                } => Err(err.into()),
                other => Err(MediaHostError::internal(format!("seek 响应类型异常: {:?}", other.kind))),
            })
        })
    }

    /// 设置循环（fire-and-forget；内核成功面无回执，未知源错误经事件面补投）。
    pub fn set_loop(&mut self, source_key: u64, on: bool) -> Result<(), MediaHostError> {
        self.with_conn(|conn, _pending| {
            conn.send(IpcMessageKind::MediaSetLoop(MediaSetLoopParams { source_key, on }))
                .map(|_| Ok(()))
        })
    }

    /// 设置播放速率（fire-and-forget；语义面同 [`Self::set_loop`]——内核侧只作用
    /// 视频墙钟，registry 已知限制同面）。
    pub fn set_rate(&mut self, source_key: u64, rate: f64) -> Result<(), MediaHostError> {
        self.with_conn(|conn, _pending| {
            conn.send(IpcMessageKind::MediaSetRate(MediaSetRateParams { source_key, rate }))
                .map(|_| Ok(()))
        })
    }

    /// 设置增益/静音（fire-and-forget；语义面同 [`Self::set_loop`]）。
    pub fn set_gain(&mut self, source_key: u64, volume: f64, muted: bool) -> Result<(), MediaHostError> {
        self.with_conn(|conn, _pending| {
            conn.send(IpcMessageKind::MediaSetGain(MediaSetGainParams {
                source_key,
                volume,
                muted,
            }))
            .map(|_| Ok(()))
        })
    }

    /// 释放源（fire-and-forget；导航/`release` 面）。
    pub fn release_source(&mut self, source_key: u64) -> Result<(), MediaHostError> {
        self.with_conn(|conn, _pending| {
            conn.send(IpcMessageKind::MediaReleaseSource(MediaReleaseSourceParams {
                source_key,
            }))
            .map(|_| Ok(()))
        })
    }

    /// 非阻塞排空 host 推送面（C2 泵取数口，DESIGN §10 push 供数）。
    ///
    /// 暂存队列与入站通道按到达序合并，首个推送面消息作为 [`MediaHostEvent`]
    /// 返回；响应面消息留队供 [`Self::seek`] 等请求的 `wait_for` 配对。未连接
    /// （含已 shutdown/退避中/已失败）返回 `Ok(None)`——不触发懒 spawn（spawn
    /// 归请求面，无 host 即无事件）。读半失效 → 连接失效处理（重启面）——崩溃
    /// 通知经 [`Self::poll_health`] one-shot 下发，本 `Err` 仅终结本轮排空。
    pub fn try_recv_event(&mut self) -> Result<Option<MediaHostEvent>, MediaHostError> {
        let Some(mut conn) = self.connection.take() else {
            return Ok(None);
        };
        match drain_host_event(&conn.inbound, &mut self.pending) {
            Ok(event) => {
                self.connection = Some(conn);
                Ok(event)
            }
            Err(error) => {
                let crash = self.connection_lost(error, &mut conn);
                drop(conn);
                Err(crash)
            }
        }
    }

    /// 优雅关闭：发 `MediaShutdown` 后收尾子进程与读线程（IF-001 关闭面）。
    /// 显式关停意图清退避重启——状态机回 Idle（§11.4 reload 恢复路径：下次
    /// 命令懒惰 spawn 全新 host，即使先前已 Failed）。
    pub fn shutdown(&mut self) -> Result<(), MediaHostError> {
        let result = if let Some(mut conn) = self.connection.take() {
            let result = conn
                .send(IpcMessageKind::MediaShutdown)
                .map(|_| ())
                .map_err(MediaHostError::internal);
            // HostConnection::drop：stdin EOF → host 退出；兜底 kill + wait + join。
            drop(conn);
            result
        } else {
            Ok(())
        };
        self.lifecycle = HostLifecycle::Idle;
        self.pending.clear();
        self.crash_pending = None;
        result
    }
}

/// 排空推送面：暂存与入站通道按到达序合并，返回首个推送面事件，其余按序回队。
///
/// 传输失效（读线程报错/已退出）为 `Err`——调用方置永久错误面；已收消息随
/// 连接弃置（不再有消费面）。
fn drain_host_event(
    inbound: &Receiver<Result<IpcMessage, String>>,
    pending: &mut VecDeque<IpcMessage>,
) -> Result<Option<MediaHostEvent>, String> {
    let mut queued: Vec<IpcMessage> = pending.drain(..).collect();
    loop {
        match inbound.try_recv() {
            Ok(Ok(message)) => queued.push(message),
            Ok(Err(error)) => return Err(format!("media-host 传输失效: {error}")),
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => return Err("media-host 读线程已退出".to_string()),
        }
    }
    let mut rest = VecDeque::with_capacity(queued.len());
    let mut event = None;
    for message in queued {
        match event_from_message(&message) {
            Some(candidate) if event.is_none() => event = Some(candidate),
            _ => {
                tracing::debug!("media-proxy: 非推送消息留队（等待请求配对）: {:?}", message.kind);
                rest.push_back(message);
            }
        }
    }
    *pending = rest;
    Ok(event)
}

/// 推送面分类：状态/帧/元数据推送与播放级错误为事件；请求响应面（`DataAck`/
/// `SeekResult`/带 id 的 `SourceReady`/`Ready`）返回 `None` 留队（wait_for 配对）。
fn event_from_message(message: &IpcMessage) -> Option<MediaHostEvent> {
    match &message.kind {
        IpcMessageKind::MediaStateUpdate(params) => Some(MediaHostEvent::State(*params)),
        IpcMessageKind::MediaFrame(params) => Some(MediaHostEvent::Frame(params.clone())),
        // MediaError 请求响应（先前等待超时后的补投）与自发推送统一走事件面——
        // 播放级错误不置永久失败（IF-006 边界，host 进程仍健康）。
        IpcMessageKind::MediaError(params) => Some(MediaHostEvent::Error(params.clone())),
        // id 0 = 非请求响应的元数据推送；带 id 的为 register 响应（留队配对）。
        IpcMessageKind::MediaSourceReady(params) if message.id == 0 => {
            Some(MediaHostEvent::SourceReady(params.clone()))
        }
        _ => None,
    }
}

impl From<MediaErrorParams> for MediaHostError {
    fn from(err: MediaErrorParams) -> Self {
        Self {
            code: err.code,
            message: err.message,
        }
    }
}

impl Drop for MediaHostClient {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_protocol::is_disconnected_channel_message;
    use zero_protocol::media::MediaPlaybackState;

    /// 握手校验：版本不符按 spawn 失败处理（IF-001）。
    #[test]
    fn handshake_version_mismatch_is_rejected() {
        let mut ready = MediaReadyParams {
            protocol_version: MEDIA_PROTOCOL_VERSION,
            capabilities: Vec::new(),
        };
        assert!(check_handshake(&ready).is_ok());
        ready.protocol_version = MEDIA_PROTOCOL_VERSION + 1;
        assert!(check_handshake(&ready).is_err());
    }

    /// 二进制解析链：exe 同目录候选先于 PATH 兜底。
    #[test]
    fn binary_resolution_prefers_sibling_of_executable() {
        let exe = Path::new("/workspace/target/release/zero-renderer");
        let candidates = media_host_candidates_near_executable(exe);
        assert_eq!(
            candidates.first(),
            Some(&PathBuf::from("/workspace/target/release").join(media_host_binary_filename()))
        );
    }

    /// 断连诊断消费面（IF-006 传输失效识别）。
    #[test]
    fn disconnect_detection_covers_pipe_eof() {
        assert!(is_disconnected_channel_message(
            "读取帧头失败: failed to fill whole buffer"
        ));
        assert!(!is_disconnected_channel_message("帧过大: 999 字节"));
    }

    fn state_message(id: u64, media_time_ms: u64) -> IpcMessage {
        IpcMessage {
            id,
            kind: IpcMessageKind::MediaStateUpdate(MediaStateUpdateParams {
                source_key: 1,
                epoch: 0,
                state: MediaPlaybackState::Playing { media_time_ms },
            }),
        }
    }

    /// 未连接客户端：事件面返回 `Ok(None)` 且不触发懒 spawn（spawn 归请求面）。
    #[test]
    fn try_recv_event_without_connection_is_none_without_spawn() {
        let mut client = MediaHostClient::new();
        assert!(matches!(client.try_recv_event(), Ok(None)));
        assert!(!client.is_failed());
    }

    /// 事件面分类：状态/帧推送出队为事件；响应面（DataAck）留队保序供配对。
    #[test]
    fn drain_classifies_pushes_and_keeps_responses_in_order() {
        let (tx, rx) = channel();
        let _ = tx.send(Ok(IpcMessage {
            id: 5,
            kind: IpcMessageKind::MediaDataAck(MediaDataAckParams {
                source_key: 1,
                last_seq: 2,
            }),
        }));
        let _ = tx.send(Ok(state_message(0, 12)));
        let _ = tx.send(Ok(IpcMessage {
            id: 0,
            kind: IpcMessageKind::MediaFrame(MediaFrameParams {
                source_key: 1,
                epoch: 0,
                pts_ms: 0,
                key: true,
                width: 2,
                height: 2,
                rgba: vec![0; 16],
                shm_name: None,
            }),
        }));

        let mut pending = VecDeque::new();
        match drain_host_event(&rx, &mut pending).expect("drain ok") {
            Some(MediaHostEvent::State(s)) => {
                assert!(matches!(s.state, MediaPlaybackState::Playing { media_time_ms: 12 }));
            }
            other => panic!("expected State event, got {other:?}"),
        }
        // 排空至首个事件为止：DataAck（响应面）与其后的帧推送按到达序回队。
        assert_eq!(pending.len(), 2, "DataAck 与 Frame 按序回队");
        assert_eq!(pending.front().unwrap().id, 5);
        match drain_host_event(&rx, &mut pending).expect("drain ok") {
            Some(MediaHostEvent::Frame(frame)) => {
                assert_eq!((frame.source_key, frame.epoch, frame.pts_ms), (1, 0, 0));
            }
            other => panic!("expected Frame event, got {other:?}"),
        }
        assert!(drain_host_event(&rx, &mut pending).expect("drain ok").is_none());
        assert_eq!(pending.len(), 1, "DataAck 仍在队");
    }

    /// SourceReady 按信封 id 分流：id 0 = 元数据推送（事件），带 id = register
    /// 响应（留队）；播放级错误推送为事件（不置永久失败）。
    #[test]
    fn source_ready_splits_by_envelope_id_and_error_is_event() {
        let (tx, rx) = channel();
        let _ = tx.send(Ok(IpcMessage {
            id: 7,
            kind: IpcMessageKind::MediaSourceReady(MediaSourceReadyParams {
                source_key: 1,
                duration_ms: None,
                width: None,
                height: None,
            }),
        }));
        let _ = tx.send(Ok(IpcMessage {
            id: 0,
            kind: IpcMessageKind::MediaSourceReady(MediaSourceReadyParams {
                source_key: 1,
                duration_ms: Some(250),
                width: Some(2),
                height: Some(2),
            }),
        }));
        let _ = tx.send(Ok(IpcMessage {
            id: 0,
            kind: IpcMessageKind::MediaError(MediaErrorParams {
                source_key: 1,
                epoch: 0,
                code: MediaErrorCode::DecodeFailure,
                message: "bad frame".into(),
            }),
        }));

        let mut pending = VecDeque::new();
        assert!(matches!(
            drain_host_event(&rx, &mut pending).expect("drain ok"),
            Some(MediaHostEvent::SourceReady(_))
        ));
        match drain_host_event(&rx, &mut pending).expect("drain ok") {
            Some(MediaHostEvent::Error(err)) => {
                assert_eq!(err.code, MediaErrorCode::DecodeFailure);
            }
            other => panic!("expected Error event, got {other:?}"),
        }
        assert!(drain_host_event(&rx, &mut pending).expect("drain ok").is_none());
        assert_eq!(pending.len(), 1, "带 id 的 SourceReady 留队（register 响应）");
        assert_eq!(pending.front().unwrap().id, 7);
    }

    /// 传输失效（读线程报错/退出）为 `Err`——调用方据此转连接失效处理（C3 重启面）。
    #[test]
    fn transport_failure_surfaces_as_err() {
        let (tx, rx) = channel();
        let _ = tx.send(Err("pipe broken".into()));
        let mut pending = VecDeque::new();
        assert!(drain_host_event(&rx, &mut pending).is_err());

        let (tx, rx) = channel();
        drop(tx);
        assert!(drain_host_event(&rx, &mut pending).is_err(), "Disconnected 亦为 Err");
    }

    /// stderr tail 有界：超限丢头部，保最后 [`STDERR_TAIL_LIMIT`] 字节（子进程
    /// 不因管道写满阻塞——renderer 先例同值）。
    #[test]
    fn stderr_tail_keeps_last_bound() {
        let tail = Mutex::new(VecDeque::new());
        let chunk = vec![b'a'; STDERR_TAIL_LIMIT + 100];
        append_stderr_tail(&tail, &chunk);
        assert_eq!(stderr_tail_text(&tail).len(), STDERR_TAIL_LIMIT);
    }

    /// 初始 spawn 失败 → 永久错误面（IF-006/§11.1 短路——本生命周期不重试、
    /// 不内联回落；与运行期失效的重启面区分）。
    #[test]
    #[serial_test::serial]
    fn initial_spawn_failure_is_permanent_short_circuit() {
        // 显式假路径压过解析链（exe 同级可能命中已构建的真实二进制）。
        unsafe {
            std::env::set_var("ZW_MEDIA_HOST_BIN", "/nonexistent/zw-missing-media-host");
        }
        let mut client = MediaHostClient::new();
        let error = client
            .register_source(1, 8, "video/webm")
            .expect_err("spawn 失败必为错误面");
        assert_eq!(error.code, MediaErrorCode::HostInternal);
        assert!(client.is_failed(), "初始 spawn 失败即永久面");
        assert_eq!(client.host_pid(), None);

        let again = client.play(1, 0).expect_err("永久面短路");
        assert!(again.message.contains("永久错误面"), "短路诊断: {}", again.message);
        assert!(!client.poll_health().is_err(), "无连接不产生崩溃通知");
        unsafe {
            std::env::remove_var("ZW_MEDIA_HOST_BIN");
        }
    }
}
