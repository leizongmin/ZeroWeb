//! QuickJS Web Worker 运行时（对应 V8 的 worker.rs）。
//!
//! QuickJS Runtime 非 Send（Rc），但 Worker 在独立线程内创建自己的 Runtime
//! （不跨线程），通过通道与主线程通信（同 V8 Worker）。
//!
//! 提供 Dedicated Worker：独立线程 + 持久 QuickJS Context + postMessage/onmessage。

use crate::{SandboxConfig, ScriptError};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

/// `terminate()` join 的墙上墙钟上限（同 [`crate::worker`] V8 实现）。超时则 detach，
/// 确保 Drop/terminate **永不无限阻塞**主线程。R3399：QuickJS worker 对称修复。
const TERMINATE_JOIN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// Worker 线程接收的命令。
enum WorkerCommand {
    Execute(String),
    PostMessage(String),
    Terminate,
}

/// Worker 线程发往主线程的消息。
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum WorkerEvent {
    /// Worker 发送消息到主线程（onmessage）。
    Message(String),
    /// Worker 脚本执行出错。
    Error(String),
    /// Worker 已正常退出。
    Closed,
}

/// Worker 生命周期状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum WorkerState {
    /// Worker 已创建，正在初始化。
    Initializing,
    /// Worker 正在运行，可以接收消息。
    Running,
    /// Worker 已终止。
    Terminated,
}

/// 限时 join worker 线程；超时则 detach（不阻塞调用线程）。同 [`crate::worker`] V8 实现。
fn join_bounded_or_detach(handle: JoinHandle<()>) {
    let start = std::time::Instant::now();
    while start.elapsed() < TERMINATE_JOIN_TIMEOUT {
        if handle.is_finished() {
            let _ = handle.join();
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    drop(handle);
}

/// Dedicated Worker 运行时（QuickJS 实现）。
///
/// 在独立线程中运行 QuickJS Runtime + 持久 Context，通过通道与主线程通信。
/// Worker 的 JS 环境在所有脚本执行间保持状态一致。
pub struct WorkerRuntime {
    cmd_sender: Sender<WorkerCommand>,
    event_receiver: Receiver<WorkerEvent>,
    worker_handle: Option<JoinHandle<()>>,
    state: WorkerState,
    /// 终止标志（worker 线程 interrupt handler 据此返 true 中断死循环）。
    /// R3399：QuickJS worker 对称修复——terminate() 置位 + bounded join + detach 兜底，
    /// 防 page-supplied 死循环 worker 致 Drop/terminate 永久挂死主线程。
    terminate_flag: Arc<AtomicBool>,
}

impl WorkerRuntime {
    /// 创建新的 Dedicated Worker（QuickJS）。
    ///
    /// Worker 在独立线程中创建自己的 QuickJS Runtime + Context 并执行初始化脚本。
    pub fn new(script: &str, config: SandboxConfig) -> Result<Self, ScriptError> {
        let (cmd_sender, cmd_receiver) = mpsc::channel::<WorkerCommand>();
        let (event_sender, event_receiver) = mpsc::channel::<WorkerEvent>();
        let script = script.to_string();

        // R3399：终止标志（主线程 terminate() 与 worker 线程 interrupt handler 共享）。
        let terminate_flag = Arc::new(AtomicBool::new(false));
        let worker_terminate_flag = terminate_flag.clone();

        let handle = thread::Builder::new()
            .name("zero-quickjs-worker".to_string())
            .spawn(move || {
                quickjs_worker_thread_fn(script, config, cmd_receiver, event_sender, worker_terminate_flag);
            })
            .map_err(|e| ScriptError::EngineUnavailable(format!("Failed to spawn worker thread: {e}")))?;

        Ok(Self {
            cmd_sender,
            event_receiver,
            worker_handle: Some(handle),
            state: WorkerState::Running,
            terminate_flag,
        })
    }

    /// 向 Worker 发送消息（模拟主线程 postMessage）。
    pub fn post_message(&mut self, message: &str) -> Result<(), ScriptError> {
        if self.state == WorkerState::Terminated {
            return Err(ScriptError::InvalidInput(
                "Cannot post message to terminated worker".into(),
            ));
        }
        self.cmd_sender
            .send(WorkerCommand::PostMessage(message.to_string()))
            .map_err(|_| ScriptError::RuntimeError("Worker thread disconnected".into()))
    }

    /// 向 Worker 发送要执行的额外脚本。
    pub fn execute_script(&mut self, code: &str) -> Result<(), ScriptError> {
        if self.state == WorkerState::Terminated {
            return Err(ScriptError::InvalidInput(
                "Cannot execute script on terminated worker".into(),
            ));
        }
        self.cmd_sender
            .send(WorkerCommand::Execute(code.to_string()))
            .map_err(|_| ScriptError::RuntimeError("Worker thread disconnected".into()))
    }

    /// 尝试接收 Worker 发出的事件（非阻塞）。
    pub fn try_recv(&self) -> Option<WorkerEvent> {
        self.event_receiver.try_recv().ok()
    }

    /// 阻塞等待接收 Worker 发出的事件。
    pub fn recv(&self) -> Result<WorkerEvent, ScriptError> {
        self.event_receiver
            .recv()
            .map_err(|_| ScriptError::RuntimeError("Worker channel closed".into()))
    }

    /// 带超时地接收 Worker 事件。
    pub fn recv_timeout(&self, timeout: std::time::Duration) -> Result<WorkerEvent, ScriptError> {
        self.event_receiver.recv_timeout(timeout).map_err(|e| match e {
            mpsc::RecvTimeoutError::Timeout => ScriptError::Timeout("Worker recv timeout".into()),
            mpsc::RecvTimeoutError::Disconnected => ScriptError::RuntimeError("Worker channel closed".into()),
        })
    }

    /// 终止 Worker。
    ///
    /// R3399：置终止标志（worker interrupt handler 据此返 true 中断死循环）后发 Terminate
    /// 命令，再限时 join——确保即便 worker 卡在 page-supplied 死循环也能及时退出；超时则
    /// detach（绝不无限阻塞调用线程，Drop 也走此路径）。QuickJS 对称 V8 worker 修复。
    pub fn terminate(&mut self) {
        if self.state == WorkerState::Terminated {
            return;
        }
        self.terminate_flag.store(true, Ordering::Release);
        let _ = self.cmd_sender.send(WorkerCommand::Terminate);
        if let Some(handle) = self.worker_handle.take() {
            join_bounded_or_detach(handle);
        }
        self.state = WorkerState::Terminated;
    }

    /// 获取 Worker 当前状态。
    pub fn state(&self) -> WorkerState {
        self.state
    }

    /// Worker 是否仍在运行。
    pub fn is_running(&self) -> bool {
        self.state == WorkerState::Running
    }
}

impl Drop for WorkerRuntime {
    fn drop(&mut self) {
        self.terminate();
    }
}

impl std::fmt::Debug for WorkerRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkerRuntime").field("state", &self.state).finish()
    }
}

/// Worker 线程主函数（QuickJS 实现）。
///
/// 创建 QuickJS Runtime + 持久 Context，注入 Worker bootstrap，执行初始化脚本，
/// 然后循环处理命令（Execute/PostMessage/Terminate）。
///
/// R3399：`terminate_flag` 经 `set_interrupt_handler` 注册——terminate() 置位后，
/// QuickJS 解释器周期性调 interrupt handler 返 true → 抛 uncatchable 异常中断
/// page-supplied 死循环，worker 线程得以退出 recv 循环（对称 V8 worker 修复）。
fn quickjs_worker_thread_fn(
    init_script: String,
    config: SandboxConfig,
    cmd_receiver: Receiver<WorkerCommand>,
    event_sender: Sender<WorkerEvent>,
    terminate_flag: Arc<AtomicBool>,
) {
    let runtime = match rquickjs::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            let _ = event_sender.send(WorkerEvent::Error(format!("QuickJS runtime init failed: {e}")));
            return;
        }
    };
    if config.heap_limit > 0 {
        runtime.set_memory_limit(config.heap_limit);
    }
    // R3399：注册 interrupt handler——终止标志置位时返 true 中断死循环。
    // handler clone 一份 terminate_flag（Arc，廉价）；QuickJS 周期性调用它。
    {
        let flag = terminate_flag.clone();
        runtime.set_interrupt_handler(Some(Box::new(move || flag.load(Ordering::Acquire))));
    }
    let ctx = match rquickjs::Context::full(&runtime) {
        Ok(c) => c,
        Err(e) => {
            let _ = event_sender.send(WorkerEvent::Error(format!("QuickJS context init failed: {e}")));
            return;
        }
    };

    // Worker 全局环境（同 V8 worker.rs 的 bootstrap）
    let bootstrap = r#"
        var _workerMessageQueue = [];
        var onmessage = null;
        var postMessage = function(data) {
            _workerMessageQueue.push(typeof data === 'string' ? data : JSON.stringify(data));
        };
        var _dispatchMessage = function(data) {
            if (typeof onmessage === 'function') {
                onmessage({ data: data });
            }
        };
    "#;

    ctx.with(|mut ctx| {
        let _ = ctx.eval::<rquickjs::Value, _>(bootstrap.to_string());
        if !init_script.trim().is_empty() {
            let _ = ctx.eval::<rquickjs::Value, _>(init_script.clone());
        }
        quickjs_drain(&mut ctx, &event_sender);
    });

    while let Ok(cmd) = cmd_receiver.recv() {
        match cmd {
            WorkerCommand::Execute(code) => {
                ctx.with(|mut ctx| {
                    let _ = ctx.eval::<rquickjs::Value, _>(code.clone());
                    quickjs_drain(&mut ctx, &event_sender);
                });
            }
            WorkerCommand::PostMessage(msg) => {
                ctx.with(|mut ctx| {
                    let dispatch = format!("_dispatchMessage({})", json_stringify(&msg));
                    let _ = ctx.eval::<rquickjs::Value, _>(dispatch);
                    quickjs_drain(&mut ctx, &event_sender);
                });
            }
            WorkerCommand::Terminate => break,
        }
    }

    let _ = event_sender.send(WorkerEvent::Closed);
}

/// 排空 Worker 消息队列，发送到主线程。
fn quickjs_drain(ctx: &mut rquickjs::Ctx, sender: &Sender<WorkerEvent>) {
    let count: i32 = ctx
        .eval::<i32, _>("_workerMessageQueue.length".to_string())
        .unwrap_or(0);
    for i in 0..count {
        let code = format!("_workerMessageQueue[{i}]");
        let msg: String = ctx.eval::<String, _>(code).unwrap_or_default();
        let _ = sender.send(WorkerEvent::Message(msg));
    }
    let _ = ctx.eval::<rquickjs::Value, _>("_workerMessageQueue = []".to_string());
}

/// 简易 JSON 字符串转义（同 worker.rs）。
fn json_stringify(s: &str) -> String {
    let escaped = s
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    format!("\"{escaped}\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_quickjs_worker_basic() {
        let mut worker = WorkerRuntime::new("postMessage('hi');", SandboxConfig::default()).unwrap();
        let event = worker.recv_timeout(Duration::from_secs(5)).unwrap();
        match event {
            WorkerEvent::Message(msg) => assert_eq!(msg, "hi"),
            other => panic!("Expected Message, got: {other:?}"),
        }
        worker.terminate();
    }

    // ── R3399：page-supplied 死循环 QuickJS worker 不应让 terminate()/Drop 永久挂死 ──
    // 对称 V8 worker 修复（interrupt handler + bounded join + detach）。

    #[test]
    fn test_quickjs_terminate_returns_for_infinite_loop_r3399() {
        // QuickJS 死循环——interrupt handler 置位后周期性返 true 中断。
        let mut worker = WorkerRuntime::new("while (true) {}", SandboxConfig::default()).unwrap();
        std::thread::sleep(Duration::from_millis(300));

        let start = std::time::Instant::now();
        worker.terminate();
        let elapsed = start.elapsed();
        assert_eq!(worker.state(), WorkerState::Terminated);
        assert!(
            elapsed < Duration::from_secs(4),
            "QuickJS worker terminate() 挂死（耗时 {:?}），R3399 回归",
            elapsed
        );
    }

    #[test]
    fn test_quickjs_drop_returns_for_infinite_loop_r3399() {
        let worker = WorkerRuntime::new("while (true) {}", SandboxConfig::default()).unwrap();
        std::thread::sleep(Duration::from_millis(300));

        let start = std::time::Instant::now();
        drop(worker);
        let elapsed = start.elapsed();
        assert!(
            elapsed < Duration::from_secs(4),
            "QuickJS worker Drop 挂死（耗时 {:?}），R3399 回归",
            elapsed
        );
    }
}
