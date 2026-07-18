//! Web Worker 运行时 — 在独立线程中执行 JS 脚本。
//!
//! 提供基本的 Dedicated Worker 实现：
//! - 每个 Worker 在独立的 OS 线程中运行自己的 V8 持久上下文
//! - 通过通道（channel）进行 postMessage/onMessage 通信
//! - 支持 terminate 强制终止
//!
//! 与 V8Sandbox 不同，Worker 使用持久化的 V8 Context，
//! 使得脚本状态在多次执行间保持一致。

use crate::{SandboxConfig, ScriptError};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

/// Worker 线程接收的命令。
enum WorkerCommand {
    /// 执行脚本代码。
    Execute(String),
    /// 接收来自主线程的消息（模拟 postMessage）。
    PostMessage(String),
    /// 终止 Worker。
    Terminate,
}

/// Worker 线程发往主线程的消息。
#[derive(Debug, Clone)]
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
pub enum WorkerState {
    /// Worker 已创建，正在初始化。
    Initializing,
    /// Worker 正在运行，可以接收消息。
    Running,
    /// Worker 已终止。
    Terminated,
}

/// Dedicated Worker 运行时。
///
/// 在独立线程中运行 V8 持久上下文，通过通道与主线程通信。
/// Worker 的 JS 环境在所有脚本执行间保持状态一致。
///
/// # 生命周期
///
/// 1. `WorkerRuntime::new()` — 创建 Worker（启动线程和 V8 上下文）
/// 2. `post_message()` — 向 Worker 发送消息
/// 3. `try_recv()` / `recv()` — 接收 Worker 发出的消息
/// 4. `terminate()` — 终止 Worker
pub struct WorkerRuntime {
    /// 向 Worker 线程发送命令的通道。
    cmd_sender: Sender<WorkerCommand>,
    /// 从 Worker 线程接收事件的通道。
    event_receiver: Receiver<WorkerEvent>,
    /// Worker 线程句柄。
    worker_handle: Option<JoinHandle<()>>,
    /// Worker 当前状态。
    state: WorkerState,
}

impl WorkerRuntime {
    /// 创建新的 Dedicated Worker。
    ///
    /// Worker 在独立线程中创建自己的 V8 持久上下文并执行初始化脚本。
    ///
    /// # 参数
    ///
    /// - `script` — Worker 初始化时执行的脚本代码
    /// - `config` — V8 沙箱配置（堆限制等）
    pub fn new(script: &str, config: SandboxConfig) -> Result<Self, ScriptError> {
        let (cmd_sender, cmd_receiver) = mpsc::channel::<WorkerCommand>();
        let (event_sender, event_receiver) = mpsc::channel::<WorkerEvent>();

        let script = script.to_string();

        let handle = thread::Builder::new()
            .name("zero-worker".to_string())
            .spawn(move || {
                worker_thread_fn(script, config, cmd_receiver, event_sender);
            })
            .map_err(|e| ScriptError::EngineUnavailable(format!("Failed to spawn worker thread: {e}")))?;

        Ok(Self {
            cmd_sender,
            event_receiver,
            worker_handle: Some(handle),
            state: WorkerState::Running,
        })
    }

    /// 向 Worker 发送消息（模拟主线程 postMessage）。
    ///
    /// 消息以 JSON 字符串形式传递，Worker 端通过 `onmessage` 回调接收。
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
    /// 发送终止命令并等待 Worker 线程退出。
    /// 已终止的 Worker 不能再发送消息。
    pub fn terminate(&mut self) {
        if self.state == WorkerState::Terminated {
            return;
        }
        let _ = self.cmd_sender.send(WorkerCommand::Terminate);
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
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

// ── Worker 线程实现 ──

/// 在 V8 scope 中执行脚本。
macro_rules! v8_exec {
    ($scope:expr, $code:expr) => {{
        let scope = &mut $scope;
        let Some(v8_code) = v8::String::new(scope, $code) else {
            return;
        };
        let Some(script) = v8::Script::compile(scope, v8_code, None) else {
            return;
        };
        let _ = script.run(scope);
    }};
}

/// 从 Worker 上下文中提取排队的消息并发送到主线程。
macro_rules! v8_drain {
    ($scope:expr, $sender:expr) => {{
        let scope = &mut $scope;
        let Some(len_code) = v8::String::new(scope, "_workerMessageQueue.length") else {
            return;
        };
        let Some(len_script) = v8::Script::compile(scope, len_code, None) else {
            return;
        };
        let Some(len_result) = len_script.run(scope) else {
            return;
        };
        let Some(len_str) = len_result.to_string(scope) else {
            return;
        };
        let len_s = len_str.to_rust_string_lossy(scope);
        let Ok(count) = len_s.parse::<usize>() else {
            return;
        };
        for i in 0..count {
            let get_code = format!("_workerMessageQueue[{i}]");
            let Some(v8_code) = v8::String::new(scope, &get_code) else {
                continue;
            };
            let Some(script) = v8::Script::compile(scope, v8_code, None) else {
                continue;
            };
            let Some(result) = script.run(scope) else {
                continue;
            };
            let Some(str_val) = result.to_string(scope) else {
                continue;
            };
            let msg = str_val.to_rust_string_lossy(scope);
            let _ = $sender.send(WorkerEvent::Message(msg));
        }
        let Some(clear_code) = v8::String::new(scope, "_workerMessageQueue = []") else {
            return;
        };
        if let Some(clear_script) = v8::Script::compile(scope, clear_code, None) {
            let _ = clear_script.run(scope);
        }
    }};
}

/// Worker 线程的主函数。
///
/// 使用持久化的 V8 Context，使得脚本状态在多次执行间保持一致。
fn worker_thread_fn(
    init_script: String,
    config: SandboxConfig,
    cmd_receiver: Receiver<WorkerCommand>,
    event_sender: Sender<WorkerEvent>,
) {
    // 确保 V8 已初始化
    crate::v8_runtime::V8Sandbox::new().ok();

    // 创建 Isolate + 持久化 Context
    let mut create_params = v8::Isolate::create_params();
    if config.heap_limit > 0 {
        create_params = create_params.heap_limits(0, config.heap_limit);
    }
    let mut isolate = v8::Isolate::new(create_params);

    v8::scope!(let scope, &mut isolate);
    let context = v8::Context::new(scope, Default::default());
    let mut scope = v8::ContextScope::new(scope, context);

    // 注入 Worker 全局环境
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

    // 执行 bootstrap
    v8_exec!(scope, bootstrap);

    // 执行初始化脚本
    if !init_script.trim().is_empty() {
        v8_exec!(scope, &init_script);
    }

    // 排空初始化消息
    v8_drain!(scope, event_sender);

    // 命令循环
    while let Ok(cmd) = cmd_receiver.recv() {
        match cmd {
            WorkerCommand::Execute(code) => {
                v8_exec!(scope, &code);
                v8_drain!(scope, event_sender);
            }
            WorkerCommand::PostMessage(msg) => {
                let dispatch_code = format!("_dispatchMessage({})", json_safe_arg(&msg));
                v8_exec!(scope, &dispatch_code);
                v8_drain!(scope, event_sender);
            }
            WorkerCommand::Terminate => {
                break;
            }
        }
    }

    let _ = event_sender.send(WorkerEvent::Closed);
}

/// 将字符串转换为 JS 安全的参数（作为带引号的 JS 字符串字面量）。
fn json_safe_arg(s: &str) -> String {
    json_stringify(s)
}

/// 简易 JSON 字符串转义。
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
    fn test_worker_create_and_terminate() {
        let mut worker = WorkerRuntime::new("var x = 1;", SandboxConfig::default()).unwrap();
        assert_eq!(worker.state(), WorkerState::Running);
        assert!(worker.is_running());
        worker.terminate();
        assert_eq!(worker.state(), WorkerState::Terminated);
        assert!(!worker.is_running());
    }

    #[test]
    fn test_worker_init_post_message() {
        let mut worker = WorkerRuntime::new("postMessage('hello from worker');", SandboxConfig::default()).unwrap();

        let event = worker.recv_timeout(Duration::from_secs(5)).unwrap();
        match event {
            WorkerEvent::Message(msg) => assert_eq!(msg, "hello from worker"),
            other => panic!("Expected Message, got: {other:?}"),
        }
        worker.terminate();
    }

    #[test]
    fn test_worker_echo_message() {
        let mut worker = WorkerRuntime::new(
            "onmessage = function(e) { postMessage('echo: ' + e.data); };",
            SandboxConfig::default(),
        )
        .unwrap();

        std::thread::sleep(Duration::from_millis(200));
        worker.post_message("test").unwrap();

        let event = worker.recv_timeout(Duration::from_secs(5)).unwrap();
        match event {
            WorkerEvent::Message(msg) => assert_eq!(msg, "echo: test"),
            other => panic!("Expected Message, got: {other:?}"),
        }
        worker.terminate();
    }

    #[test]
    fn test_worker_stateful_counter() {
        let mut worker = WorkerRuntime::new(
            "var count = 0; onmessage = function(e) { count++; postMessage('count: ' + count); };",
            SandboxConfig::default(),
        )
        .unwrap();

        std::thread::sleep(Duration::from_millis(200));

        for i in 1..=3 {
            worker.post_message("msg").unwrap();
            let event = worker.recv_timeout(Duration::from_secs(5)).unwrap();
            match event {
                WorkerEvent::Message(msg) => assert_eq!(msg, format!("count: {i}")),
                other => panic!("Iteration {i}: Expected Message, got: {other:?}"),
            }
        }
        worker.terminate();
    }

    #[test]
    fn test_worker_execute_script() {
        let mut worker = WorkerRuntime::new("var result = '';", SandboxConfig::default()).unwrap();

        std::thread::sleep(Duration::from_millis(200));
        worker
            .execute_script("result = 'computed'; postMessage(result);")
            .unwrap();

        let event = worker.recv_timeout(Duration::from_secs(5)).unwrap();
        match event {
            WorkerEvent::Message(msg) => assert_eq!(msg, "computed"),
            other => panic!("Expected Message, got: {other:?}"),
        }
        worker.terminate();
    }

    #[test]
    fn test_worker_json_message() {
        let mut worker = WorkerRuntime::new(
            "onmessage = function(e) { postMessage(JSON.stringify({echo: e.data})); };",
            SandboxConfig::default(),
        )
        .unwrap();

        std::thread::sleep(Duration::from_millis(200));
        worker.post_message("hello").unwrap();

        let event = worker.recv_timeout(Duration::from_secs(5)).unwrap();
        match event {
            WorkerEvent::Message(msg) => {
                assert!(msg.contains("echo"), "Message: {msg}");
                assert!(msg.contains("hello"), "Message: {msg}");
            }
            other => panic!("Expected Message, got: {other:?}"),
        }
        worker.terminate();
    }

    #[test]
    fn test_worker_complex_computation() {
        let mut worker = WorkerRuntime::new(
            "onmessage = function(e) { var n = parseInt(e.data); var sum = 0; for (var i = 0; i <= n; i++) sum += i; postMessage(String(sum)); };",
            SandboxConfig::default(),
        ).unwrap();

        std::thread::sleep(Duration::from_millis(200));
        worker.post_message("100").unwrap();

        let event = worker.recv_timeout(Duration::from_secs(5)).unwrap();
        match event {
            WorkerEvent::Message(msg) => assert_eq!(msg, "5050"),
            other => panic!("Expected Message, got: {other:?}"),
        }
        worker.terminate();
    }

    #[test]
    fn test_multiple_workers_isolated() {
        let mut w1 = WorkerRuntime::new(
            "var id = 'w1'; onmessage = function() { postMessage(id); };",
            SandboxConfig::default(),
        )
        .unwrap();
        let mut w2 = WorkerRuntime::new(
            "var id = 'w2'; onmessage = function() { postMessage(id); };",
            SandboxConfig::default(),
        )
        .unwrap();

        std::thread::sleep(Duration::from_millis(300));
        w1.post_message("ping").unwrap();
        w2.post_message("ping").unwrap();

        let e1 = w1.recv_timeout(Duration::from_secs(5)).unwrap();
        let e2 = w2.recv_timeout(Duration::from_secs(5)).unwrap();

        match e1 {
            WorkerEvent::Message(msg) => assert_eq!(msg, "w1"),
            other => panic!("w1: Expected Message, got: {other:?}"),
        }
        match e2 {
            WorkerEvent::Message(msg) => assert_eq!(msg, "w2"),
            other => panic!("w2: Expected Message, got: {other:?}"),
        }
        w1.terminate();
        w2.terminate();
    }

    #[test]
    fn test_worker_custom_config() {
        let config = SandboxConfig {
            heap_limit: 16 * 1024 * 1024,
            timeout_ms: 5000,
            persistent_context: false,
        };
        let mut worker = WorkerRuntime::new("postMessage('ok');", config).unwrap();

        let event = worker.recv_timeout(Duration::from_secs(5)).unwrap();
        match event {
            WorkerEvent::Message(msg) => assert_eq!(msg, "ok"),
            other => panic!("Expected Message, got: {other:?}"),
        }
        worker.terminate();
    }

    #[test]
    fn test_terminated_rejects_message() {
        let mut worker = WorkerRuntime::new("var x = 1;", SandboxConfig::default()).unwrap();
        worker.terminate();
        assert!(worker.post_message("test").is_err());
    }

    #[test]
    fn test_terminated_rejects_script() {
        let mut worker = WorkerRuntime::new("var x = 1;", SandboxConfig::default()).unwrap();
        worker.terminate();
        assert!(worker.execute_script("1+1").is_err());
    }

    #[test]
    fn test_double_terminate() {
        let mut worker = WorkerRuntime::new("var x = 1;", SandboxConfig::default()).unwrap();
        worker.terminate();
        worker.terminate();
        assert_eq!(worker.state(), WorkerState::Terminated);
    }

    #[test]
    fn test_worker_no_handler_no_crash() {
        let mut worker = WorkerRuntime::new("var x = 42;", SandboxConfig::default()).unwrap();

        std::thread::sleep(Duration::from_millis(200));
        // 没有 onmessage 处理器，不应崩溃
        worker.post_message("test").unwrap();
        worker.terminate();
    }

    #[test]
    fn test_worker_runtime_debug() {
        let worker = WorkerRuntime::new("var x = 1;", SandboxConfig::default()).unwrap();
        let debug = format!("{worker:?}");
        assert!(debug.contains("WorkerRuntime"));
        assert!(debug.contains("Running"));
    }

    #[test]
    fn test_worker_state_debug() {
        assert_eq!(format!("{:?}", WorkerState::Running), "Running");
        assert_eq!(format!("{:?}", WorkerState::Terminated), "Terminated");
        assert_eq!(format!("{:?}", WorkerState::Initializing), "Initializing");
    }

    #[test]
    fn test_worker_event_debug_clone() {
        let msg = WorkerEvent::Message("test".to_string());
        assert!(format!("{msg:?}").contains("Message"));
        let cloned = msg.clone();
        match cloned {
            WorkerEvent::Message(s) => assert_eq!(s, "test"),
            _ => panic!("Expected Message"),
        }

        let err = WorkerEvent::Error("fail".to_string());
        assert!(format!("{err:?}").contains("Error"));

        let closed = WorkerEvent::Closed;
        assert!(format!("{closed:?}").contains("Closed"));
    }
}
