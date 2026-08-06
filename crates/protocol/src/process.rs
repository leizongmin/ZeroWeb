//! 多进程管理器 — 浏览器进程和渲染进程的协调。
//!
//! ## 与 Chromium 的对应关系（非 fork/CoW 共享内存模型）
//!
//! - **Browser 进程**：UI、Tab 管理、网络/存储策略与代理（本 crate 由浏览器主进程调用）。
//! - **Renderer 进程**：独立地址空间 + 独立 `zero-renderer` 二进制；页面状态不与其他进程共享。
//! - **Network 能力**：当前合并在 Browser 进程（`FetchRequest` IPC 由浏览器代发），与 Chromium
//!   早期「browser 代网络」一致；后续可拆为独立 network 进程。
//!
//! 子进程通过 `Command::spawn` + stdin/stdout 管道 IPC 创建，**不是** fork 后与父进程 CoW 共享 DOM。

use std::io;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::channel::IpcChannel;
use crate::message::{FetchResponseParams, IpcMessage, IpcMessageKind, NavigateParams};
use crate::transport::PipeTransport;
use crate::{ProcessRole, ProtocolError};

/// 渲染进程 ID 类型。
pub type RendererId = u64;

/// 心跳超时时间。
const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(30);

/// 全局渲染进程 ID 计数器。
static NEXT_RENDERER_ID: AtomicU64 = AtomicU64::new(1);

/// 渲染进程状态。
#[derive(Debug, Clone, PartialEq)]
pub enum RendererState {
    /// 正在启动。
    Starting,
    /// 正在运行（已连接）。
    Running,
    /// 已崩溃（等待恢复）。
    Crashed(String),
    /// 已关闭。
    Closed,
}

/// 渲染进程句柄 — 浏览器进程管理一个渲染进程的完整上下文。
pub struct RendererHandle {
    /// 渲染进程唯一 ID。
    pub id: RendererId,
    /// 子进程句柄。
    child: Option<Child>,
    /// 向渲染进程写入 IPC（stdin）。
    send_transport: Option<PipeTransport<io::Empty, ChildStdin>>,
    /// 渲染进程 → 浏览器 IPC 消息队列（后台读线程填充）。
    inbound_rx: Receiver<IpcMessage>,
    /// stdout 读线程（子进程退出后 join）。
    reader_thread: Option<JoinHandle<()>>,
    /// 进程状态。
    state: RendererState,
    /// 上次心跳时间。
    last_heartbeat: Instant,
    /// 渲染进程正在加载的 URL。
    current_url: Option<String>,
}

/// 构造子进程启动参数（对齐 Chromium `--type=` 约定）。
pub fn child_process_args(role: ProcessRole, instance_id: u64) -> Vec<String> {
    let type_name = match role {
        ProcessRole::Browser => "browser",
        ProcessRole::Renderer => "renderer",
        ProcessRole::Network => "network",
        ProcessRole::ImageDecoder => "image-decoder",
    };
    vec![format!("--type={type_name}"), format!("--instance-id={instance_id}")]
}

impl RendererHandle {
    /// 创建新的渲染进程。
    ///
    /// 启动 `zero-renderer` 子进程，通过 stdin/stdout 管道建立 IPC 通道。
    pub fn spawn(renderer_bin: &str) -> Result<Self, ProtocolError> {
        let id = NEXT_RENDERER_ID.fetch_add(1, Ordering::Relaxed);

        let mut child = Command::new(renderer_bin);
        for arg in child_process_args(ProcessRole::Renderer, id) {
            child.arg(arg);
        }
        // Windows：阻止子进程分配控制台窗口（双保险：即使子系统是 CUI 也不弹窗；
        // 同时不影响 stdin/stdout/stderr 管道继承）。
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            child.creation_flags(CREATE_NO_WINDOW);
        }
        let mut child = child
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| ProtocolError::Process(format!("启动渲染进程失败: {e}")))?;

        // Windows：把子进程挂到进程级 Job Object，确保 browser 进程以任何方式退出
        // （含 Ctrl+C / `process::exit` / 强杀）时，OS 自动 kill 该 renderer。
        #[cfg(windows)]
        crate::job::assign_child(child.id());

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| ProtocolError::Process("无法获取 stdout 管道".into()))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| ProtocolError::Process("无法获取 stdin 管道".into()))?;

        let (inbound_tx, inbound_rx) = std::sync::mpsc::channel();
        let recv_transport = PipeTransport::new(stdout, io::empty());
        let reader_thread = std::thread::Builder::new()
            .name(format!("renderer-{id}-ipc-in"))
            .spawn(move || {
                let mut transport = recv_transport;
                while let Ok(msg) = transport.recv() {
                    if inbound_tx.send(msg).is_err() {
                        break;
                    }
                }
            })
            .map_err(|e| ProtocolError::Process(format!("启动 IPC 读线程失败: {e}")))?;

        let send_transport = PipeTransport::new(io::empty(), stdin);

        Ok(Self {
            id,
            child: Some(child),
            send_transport: Some(send_transport),
            inbound_rx,
            reader_thread: Some(reader_thread),
            state: RendererState::Starting,
            last_heartbeat: Instant::now(),
            current_url: None,
        })
    }

    /// 获取当前状态。
    pub fn state(&self) -> &RendererState {
        &self.state
    }

    /// 获取当前 URL。
    pub fn current_url(&self) -> Option<&str> {
        self.current_url.as_deref()
    }

    /// 发送 IPC 消息到渲染进程。
    pub fn send(&mut self, msg: IpcMessage) -> Result<(), ProtocolError> {
        match &mut self.send_transport {
            Some(ch) => ch.send(msg),
            None => Err(ProtocolError::Channel("通道已关闭".into())),
        }
    }

    /// 记录渲染进程存活（任意 IPC 消息均视为活跃，避免长页面加载期间误判心跳超时）。
    fn touch_activity(&mut self) {
        self.last_heartbeat = Instant::now();
    }

    /// 收到心跳时自动回复。
    fn reply_heartbeat_if_needed(&mut self, msg: &IpcMessage) -> Result<(), ProtocolError> {
        if matches!(msg.kind, IpcMessageKind::Heartbeat) {
            self.send(IpcMessage {
                id: msg.id,
                kind: IpcMessageKind::Heartbeat,
            })?;
        }
        Ok(())
    }

    /// 从渲染进程接收 IPC 消息（阻塞直到有消息）。
    pub fn recv(&mut self) -> Result<IpcMessage, ProtocolError> {
        let msg = self
            .inbound_rx
            .recv()
            .map_err(|e| ProtocolError::Channel(format!("IPC 接收失败: {e}")))?;
        self.touch_activity();
        self.reply_heartbeat_if_needed(&msg)?;
        Ok(msg)
    }

    /// 尝试非阻塞接收。
    pub fn try_recv(&mut self) -> Result<Option<IpcMessage>, ProtocolError> {
        match self.inbound_rx.try_recv() {
            Ok(msg) => {
                self.touch_activity();
                self.reply_heartbeat_if_needed(&msg)?;
                Ok(Some(msg))
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => Ok(None),
            Err(std::sync::mpsc::TryRecvError::Disconnected) => Err(ProtocolError::Channel("IPC 通道已关闭".into())),
        }
    }

    /// 发送导航命令。
    pub fn navigate(&mut self, url: &str, referrer: Option<&str>, navigation_epoch: u64) -> Result<(), ProtocolError> {
        self.send(IpcMessage {
            id: 0,
            kind: IpcMessageKind::Navigate(NavigateParams {
                url: url.to_string(),
                referrer: referrer.map(|s| s.to_string()),
                navigation_epoch,
            }),
        })?;
        self.current_url = Some(url.to_string());
        self.state = RendererState::Running;
        Ok(())
    }

    /// 发送后退命令。
    pub fn go_back(&mut self) -> Result<(), ProtocolError> {
        self.send(IpcMessage {
            id: 0,
            kind: IpcMessageKind::GoBack,
        })
    }

    /// 发送前进命令。
    pub fn go_forward(&mut self) -> Result<(), ProtocolError> {
        self.send(IpcMessage {
            id: 0,
            kind: IpcMessageKind::GoForward,
        })
    }

    /// 发送心跳。
    pub fn send_heartbeat(&mut self) -> Result<(), ProtocolError> {
        self.send(IpcMessage {
            id: 0,
            kind: IpcMessageKind::Heartbeat,
        })
    }

    /// 发送网络响应到渲染进程。
    pub fn send_fetch_response(
        &mut self,
        request_id: u64,
        status_code: u16,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
    ) -> Result<(), ProtocolError> {
        self.send(IpcMessage {
            id: request_id,
            kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id,
                status_code,
                headers,
                body,
            }),
        })
    }

    /// 发送存储操作结果。
    pub fn send_storage_response(&mut self, msg_id: u64, _result: &str) -> Result<(), ProtocolError> {
        self.send(IpcMessage {
            id: msg_id,
            kind: IpcMessageKind::Ok,
        })
    }

    /// 处理从渲染进程接收的消息（阻塞直到有消息）。
    pub fn poll(&mut self) -> Result<IpcMessage, ProtocolError> {
        self.recv()
    }

    /// 检查心跳是否超时。
    pub fn check_heartbeat(&self) -> bool {
        self.last_heartbeat.elapsed() > HEARTBEAT_TIMEOUT
    }

    /// 检查子进程是否仍在运行。
    pub fn is_alive(&mut self) -> bool {
        if let Some(ref mut child) = self.child {
            match child.try_wait() {
                Ok(None) => true,     // 仍在运行
                Ok(Some(_)) => false, // 已退出
                Err(_) => false,
            }
        } else {
            false
        }
    }

    /// 关闭渲染进程。
    pub fn shutdown(&mut self) -> Result<(), ProtocolError> {
        // 先终止子进程，避免仅关闭 stdin 后 renderer 仍向 stdout 写导致 Broken pipe 刷屏。
        if let Some(ref mut child) = self.child {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.child = None;

        if let Some(ref mut ch) = self.send_transport {
            ch.close();
        }
        self.send_transport = None;

        if let Some(handle) = self.reader_thread.take() {
            let _ = handle.join();
        }

        self.state = RendererState::Closed;
        Ok(())
    }

    /// 强制终止渲染进程。
    pub fn kill(&mut self) -> Result<(), ProtocolError> {
        if let Some(ref mut child) = self.child {
            child
                .kill()
                .map_err(|e| ProtocolError::Process(format!("终止进程失败: {e}")))?;
            let _ = child.wait();
        }
        self.send_transport = None;
        self.child = None;
        if let Some(handle) = self.reader_thread.take() {
            let _ = handle.join();
        }
        self.state = RendererState::Closed;
        Ok(())
    }
}

impl Drop for RendererHandle {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

/// 多进程管理器 — 浏览器进程使用此管理器协调所有渲染进程。
pub struct ProcessManager {
    /// 活跃的渲染进程列表。
    renderers: Vec<RendererHandle>,
    /// 渲染进程二进制路径。
    renderer_bin: String,
    /// 消息 ID 计数器。
    next_msg_id: AtomicU64,
}

impl ProcessManager {
    /// 创建新的进程管理器。
    ///
    /// `renderer_bin` 是 `zero-renderer` 二进制文件的路径。
    pub fn new(renderer_bin: &str) -> Self {
        Self {
            renderers: Vec::new(),
            renderer_bin: renderer_bin.to_string(),
            next_msg_id: AtomicU64::new(1),
        }
    }

    /// 分配下一个消息 ID。
    pub fn next_msg_id(&self) -> u64 {
        self.next_msg_id.fetch_add(1, Ordering::Relaxed)
    }

    /// 启动新的渲染进程。
    ///
    /// 返回新创建的渲染进程 ID。
    pub fn spawn_renderer(&mut self) -> Result<RendererId, ProtocolError> {
        let handle = RendererHandle::spawn(&self.renderer_bin)?;
        let id = handle.id;
        self.renderers.push(handle);
        Ok(id)
    }

    /// 获取指定渲染进程的句柄。
    pub fn get_renderer(&mut self, id: RendererId) -> Option<&mut RendererHandle> {
        self.renderers.iter_mut().find(|r| r.id == id)
    }

    /// 关闭指定渲染进程。
    pub fn shutdown_renderer(&mut self, id: RendererId) -> Result<(), ProtocolError> {
        if let Some(pos) = self.renderers.iter().position(|r| r.id == id) {
            let mut handle = self.renderers.swap_remove(pos);
            handle.shutdown()?;
        }
        Ok(())
    }

    /// 关闭所有渲染进程。
    pub fn shutdown_all(&mut self) {
        for mut renderer in self.renderers.drain(..) {
            let _ = renderer.shutdown();
        }
    }

    /// 获取活跃渲染进程数量。
    pub fn active_count(&self) -> usize {
        self.renderers.len()
    }

    /// 获取所有活跃渲染进程 ID。
    pub fn active_ids(&self) -> Vec<RendererId> {
        self.renderers.iter().map(|r| r.id).collect()
    }

    /// 检测并处理崩溃的渲染进程。
    ///
    /// 返回崩溃的渲染进程 ID 列表。崩溃的进程会被自动关闭。
    /// 仅以子进程是否退出为准；长页面加载/后台标签不发送 IPC 不算崩溃。
    pub fn check_crashes(&mut self) -> Vec<(RendererId, String)> {
        let mut crashed = Vec::new();
        let mut to_remove = Vec::new();

        for (i, renderer) in self.renderers.iter_mut().enumerate() {
            if !renderer.is_alive() {
                renderer.state = RendererState::Crashed("进程已退出".into());
                to_remove.push(i);
                crashed.push((renderer.id, "进程已退出".to_string()));
            }
        }

        for i in to_remove.into_iter().rev() {
            let mut handle = self.renderers.swap_remove(i);
            let _ = handle.kill();
        }

        crashed
    }
}

impl Drop for ProcessManager {
    fn drop(&mut self) {
        self.shutdown_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{FetchParams, StorageOpParams};
    use crate::transport::shared_channel_pair;

    /// 测试 RendererState 比较和转换。
    #[test]
    fn test_renderer_state_equality() {
        assert_eq!(RendererState::Starting, RendererState::Starting);
        assert_eq!(RendererState::Running, RendererState::Running);
        assert_eq!(RendererState::Closed, RendererState::Closed);
        assert_ne!(RendererState::Starting, RendererState::Running);

        let crashed = RendererState::Crashed("test".into());
        assert!(matches!(crashed, RendererState::Crashed(_)));
    }

    /// 测试 ProcessManager 创建和基本属性。
    #[test]
    fn test_process_manager_creation() {
        let pm = ProcessManager::new("/usr/bin/zero-renderer");
        assert_eq!(pm.active_count(), 0);
        assert!(pm.active_ids().is_empty());
        assert_eq!(pm.renderer_bin, "/usr/bin/zero-renderer");
    }

    /// 测试消息 ID 分配。
    #[test]
    fn test_msg_id_allocation() {
        let pm = ProcessManager::new("/usr/bin/zero-renderer");
        let id1 = pm.next_msg_id();
        let id2 = pm.next_msg_id();
        assert!(id2 > id1);
    }

    /// 测试 get_renderer 返回 None 对不存在的 ID。
    #[test]
    fn test_get_renderer_nonexistent() {
        let mut pm = ProcessManager::new("/usr/bin/zero-renderer");
        assert!(pm.get_renderer(999).is_none());
    }

    /// 测试 shutdown_all 在空管理器上不崩溃。
    #[test]
    fn test_shutdown_all_empty() {
        let mut pm = ProcessManager::new("/usr/bin/zero-renderer");
        pm.shutdown_all();
        assert_eq!(pm.active_count(), 0);
    }

    /// 测试 shutdown_renderer 对不存在的 ID 不报错。
    #[test]
    fn test_shutdown_renderer_nonexistent() {
        let mut pm = ProcessManager::new("/usr/bin/zero-renderer");
        assert!(pm.shutdown_renderer(999).is_ok());
    }

    /// 测试 check_crashes 在空管理器上返回空列表。
    #[test]
    fn test_check_crashes_empty() {
        let mut pm = ProcessManager::new("/usr/bin/zero-renderer");
        let crashed = pm.check_crashes();
        assert!(crashed.is_empty());
    }

    /// 测试 NEXT_RENDERER_ID 递增。
    #[test]
    fn test_renderer_id_increment() {
        let start = NEXT_RENDERER_ID.load(Ordering::Relaxed);
        let _ = NEXT_RENDERER_ID.fetch_add(1, Ordering::Relaxed);
        let after = NEXT_RENDERER_ID.load(Ordering::Relaxed);
        assert_eq!(after, start + 1);
    }

    /// 测试 Chromium 风格子进程启动参数。
    #[test]
    fn test_child_process_args_renderer() {
        let args = child_process_args(ProcessRole::Renderer, 7);
        assert_eq!(args, vec!["--type=renderer".to_string(), "--instance-id=7".to_string()]);
    }

    /// 测试心跳超时常量。
    #[test]
    fn test_heartbeat_constants() {
        assert_eq!(HEARTBEAT_TIMEOUT, Duration::from_secs(30));
    }

    /// 测试共享通道在进程模拟场景中的使用。
    #[test]
    fn test_simulated_renderer_communication() {
        let (mut browser_ch, mut renderer_ch) = shared_channel_pair();

        // 浏览器发送导航命令
        browser_ch
            .send(IpcMessage {
                id: 1,
                kind: IpcMessageKind::Navigate(NavigateParams {
                    url: "https://example.com".into(),
                    referrer: None,
                    navigation_epoch: 0,
                }),
            })
            .unwrap();

        // 渲染进程接收
        let msg = renderer_ch.recv().unwrap();
        assert_eq!(msg.id, 1);
        if let IpcMessageKind::Navigate(params) = msg.kind {
            assert_eq!(params.url, "https://example.com");
            assert!(params.referrer.is_none());
        } else {
            panic!("期望 Navigate 消息");
        }

        // 渲染进程发送加载完成
        renderer_ch
            .send(IpcMessage {
                id: 2,
                kind: IpcMessageKind::LoadComplete,
            })
            .unwrap();

        // 浏览器接收
        let msg = browser_ch.recv().unwrap();
        assert!(matches!(msg.kind, IpcMessageKind::LoadComplete));
    }

    /// 测试模拟的网络请求/响应流程。
    #[test]
    fn test_simulated_fetch_flow() {
        let (mut browser_ch, mut renderer_ch) = shared_channel_pair();

        // 渲染进程发起网络请求
        renderer_ch
            .send(IpcMessage {
                id: 10,
                kind: IpcMessageKind::FetchRequest(FetchParams {
                    request_id: 100,
                    url: "https://example.com/style.css".into(),
                    method: "GET".into(),
                    headers: vec![],
                    body: None,
                }),
            })
            .unwrap();

        // 浏览器进程接收并回复
        let msg = browser_ch.recv().unwrap();
        if let IpcMessageKind::FetchRequest(params) = &msg.kind {
            assert_eq!(params.url, "https://example.com/style.css");

            browser_ch
                .send(IpcMessage {
                    id: msg.id,
                    kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                        request_id: params.request_id,
                        status_code: 200,
                        headers: vec![],
                        body: b"body{}".to_vec(),
                    }),
                })
                .unwrap();
        } else {
            panic!("期望 FetchRequest 消息");
        }

        // 渲染进程接收响应
        let msg = renderer_ch.recv().unwrap();
        if let IpcMessageKind::FetchResponse(params) = &msg.kind {
            assert_eq!(params.status_code, 200);
            assert_eq!(params.body, b"body{}");
        } else {
            panic!("期望 FetchResponse 消息");
        }
    }

    /// 测试模拟的存储操作流程。
    #[test]
    fn test_simulated_storage_flow() {
        let (mut browser_ch, mut renderer_ch) = shared_channel_pair();

        // 渲染进程请求存储操作
        renderer_ch
            .send(IpcMessage {
                id: 20,
                kind: IpcMessageKind::StorageOp(StorageOpParams {
                    storage_type: crate::message::StorageType::Local,
                    operation: crate::message::StorageOperation::Set,
                    key: "theme".into(),
                    value: Some("dark".into()),
                    origin: "https://example.com".into(),
                }),
            })
            .unwrap();

        // 浏览器处理存储操作并回复
        let msg = browser_ch.recv().unwrap();
        assert!(matches!(msg.kind, IpcMessageKind::StorageOp(_)));

        browser_ch
            .send(IpcMessage {
                id: 20,
                kind: IpcMessageKind::Ok,
            })
            .unwrap();

        // 渲染进程确认
        let msg = renderer_ch.recv().unwrap();
        assert!(matches!(msg.kind, IpcMessageKind::Ok));
    }

    /// 测试心跳往返。
    #[test]
    fn test_heartbeat_roundtrip() {
        let (mut browser_ch, mut renderer_ch) = shared_channel_pair();

        // 渲染进程发送心跳
        renderer_ch
            .send(IpcMessage {
                id: 0,
                kind: IpcMessageKind::Heartbeat,
            })
            .unwrap();

        // 浏览器接收并回复
        let msg = browser_ch.recv().unwrap();
        assert!(matches!(msg.kind, IpcMessageKind::Heartbeat));
        browser_ch
            .send(IpcMessage {
                id: 0,
                kind: IpcMessageKind::Heartbeat,
            })
            .unwrap();

        // 渲染进程收到回复
        let msg = renderer_ch.recv().unwrap();
        assert!(matches!(msg.kind, IpcMessageKind::Heartbeat));
    }

    /// 测试输入事件转发。
    #[test]
    fn test_input_event_forwarding() {
        let (mut browser_ch, mut renderer_ch) = shared_channel_pair();

        // 浏览器转发鼠标事件
        browser_ch
            .send(IpcMessage {
                id: 0,
                kind: IpcMessageKind::MouseEvent(crate::message::MouseEventParams {
                    x: 100.0,
                    y: 200.0,
                    button: 0,
                    event_type: crate::message::MouseEventType::Click,
                }),
            })
            .unwrap();

        let msg = renderer_ch.recv().unwrap();
        if let IpcMessageKind::MouseEvent(params) = &msg.kind {
            assert_eq!(params.x, 100.0);
            assert_eq!(params.y, 200.0);
        } else {
            panic!("期望 MouseEvent");
        }

        // 浏览器转发键盘事件
        browser_ch
            .send(IpcMessage {
                id: 0,
                kind: IpcMessageKind::KeyboardEvent(crate::message::KeyboardEventParams {
                    key: "Enter".into(),
                    code: "Enter".into(),
                    ctrl: false,
                    shift: false,
                    alt: false,
                    meta: false,
                    event_type: crate::message::KeyboardEventType::Press,
                }),
            })
            .unwrap();

        let msg = renderer_ch.recv().unwrap();
        if let IpcMessageKind::KeyboardEvent(params) = &msg.kind {
            assert_eq!(params.key, "Enter");
        } else {
            panic!("期望 KeyboardEvent");
        }

        // 浏览器转发滚动事件
        browser_ch
            .send(IpcMessage {
                id: 0,
                kind: IpcMessageKind::ScrollEvent(crate::message::ScrollEventParams {
                    delta_x: 0.0,
                    delta_y: 100.0,
                }),
            })
            .unwrap();

        let msg = renderer_ch.recv().unwrap();
        if let IpcMessageKind::ScrollEvent(params) = &msg.kind {
            assert_eq!(params.delta_y, 100.0);
        } else {
            panic!("期望 ScrollEvent");
        }
    }

    /// 测试完整页面加载生命周期。
    #[test]
    fn test_page_lifecycle() {
        let (mut browser_ch, mut renderer_ch) = shared_channel_pair();

        // 1. 浏览器发送导航命令
        browser_ch
            .send(IpcMessage {
                id: 1,
                kind: IpcMessageKind::Navigate(NavigateParams {
                    url: "https://example.com".into(),
                    referrer: None,
                    navigation_epoch: 0,
                }),
            })
            .unwrap();

        // 2. 渲染进程接收
        let msg = renderer_ch.recv().unwrap();
        assert!(matches!(msg.kind, IpcMessageKind::Navigate(_)));

        // 3. 渲染进程报告 URL 变更
        renderer_ch
            .send(IpcMessage {
                id: 2,
                kind: IpcMessageKind::UrlChanged("https://example.com".into()),
            })
            .unwrap();
        let msg = browser_ch.recv().unwrap();
        assert!(matches!(msg.kind, IpcMessageKind::UrlChanged(_)));

        // 4. 渲染进程报告标题变更
        renderer_ch
            .send(IpcMessage {
                id: 3,
                kind: IpcMessageKind::TitleChanged("Example".into()),
            })
            .unwrap();
        let msg = browser_ch.recv().unwrap();
        assert!(matches!(msg.kind, IpcMessageKind::TitleChanged(_)));

        // 5. 渲染进程报告加载完成
        renderer_ch
            .send(IpcMessage {
                id: 4,
                kind: IpcMessageKind::LoadComplete,
            })
            .unwrap();
        let msg = browser_ch.recv().unwrap();
        assert!(matches!(msg.kind, IpcMessageKind::LoadComplete));
    }

    /// 测试页面加载失败。
    #[test]
    fn test_page_load_failure() {
        let (mut browser_ch, mut renderer_ch) = shared_channel_pair();

        browser_ch
            .send(IpcMessage {
                id: 1,
                kind: IpcMessageKind::Navigate(NavigateParams {
                    url: "https://unreachable.example".into(),
                    referrer: None,
                    navigation_epoch: 0,
                }),
            })
            .unwrap();

        let _ = renderer_ch.recv().unwrap();

        // 渲染进程报告加载失败
        renderer_ch
            .send(IpcMessage {
                id: 2,
                kind: IpcMessageKind::LoadFailed("DNS 解析失败".into()),
            })
            .unwrap();

        let msg = browser_ch.recv().unwrap();
        if let IpcMessageKind::LoadFailed(reason) = &msg.kind {
            assert!(reason.contains("DNS"));
        } else {
            panic!("期望 LoadFailed");
        }
    }

    /// 测试崩溃通知。
    #[test]
    fn test_crash_notification() {
        let (mut browser_ch, mut renderer_ch) = shared_channel_pair();

        renderer_ch
            .send(IpcMessage {
                id: 0,
                kind: IpcMessageKind::CrashNotification("OOM".into()),
            })
            .unwrap();

        let msg = browser_ch.recv().unwrap();
        if let IpcMessageKind::CrashNotification(reason) = &msg.kind {
            assert_eq!(reason, "OOM");
        } else {
            panic!("期望 CrashNotification");
        }
    }

    /// 测试关闭操作。
    #[test]
    fn test_close() {
        let (mut a, mut b) = shared_channel_pair();
        a.close();
        // inbox 已清空，即使 b 发送了消息
        b.send(IpcMessage {
            id: 1,
            kind: IpcMessageKind::Ok,
        })
        .unwrap();
        // a 已关闭后 recv 可能失败或返回空
        // close 只清空 inbox，不影响 peer_inbox
        assert!(a.recv().is_ok());
    }
}
