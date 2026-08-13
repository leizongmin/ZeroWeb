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
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

/// `terminate()` join 的墙上墙钟上限。即便 worker 卡死（看门狗/中断未及生效），
/// 主线程也不应被无限阻塞——超时后 detach（泄漏 JoinHandle，worker 线程退出时
/// 由 OS 回收）。R3399：worker 死循环致 Drop/terminate 永久挂死主线程（页面提供
/// 恶意 worker 脚本 → DoS）的根因修复兜底。
const TERMINATE_JOIN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// Worker 线程接收的命令。
enum WorkerCommand {
    /// 执行脚本代码。
    Execute(String),
    /// 接收来自主线程的消息（模拟 postMessage）。
    PostMessage(String),
    /// 终止 Worker。
    Terminate,
}

/// 超时看门狗消息（seq 协议，同 [`crate::v8_runtime`] 的 SEC-13 看门狗）：
/// `script.run` 前 Arm（截止 = timeout_ms），后 Disarm（仅当 seq 匹配），沙箱销毁停线程 Stop。
#[cfg(feature = "v8")]
enum WorkerWatchdogMsg {
    /// 装载：截止时长 + isolate 句柄（到期调 terminate_execution）。
    Arm {
        seq: u64,
        timeout_ms: u64,
        handle: v8::IsolateHandle,
    },
    /// 撤除（仅当 seq 匹配当前装载项）。
    Disarm { seq: u64 },
    /// 停看门狗线程。
    Stop,
}

/// 持久超时看门狗线程主循环（同 `v8_runtime::watchdog_main` 语义）：
/// 阻塞等待直到收到消息或装载的截止到期 → `terminate_execution`。
/// R3399：worker 每次 `script.run` 走此看门狗，page-supplied 死循环到期被中断，
/// worker 线程得以返回 recv 循环消费 Terminate 命令正常退出。
#[cfg(feature = "v8")]
fn worker_watchdog_main(rx: Receiver<WorkerWatchdogMsg>) {
    let mut armed: Option<(u64, std::time::Instant, v8::IsolateHandle)> = None;
    loop {
        let wait = match &armed {
            Some((_, deadline, _)) => deadline.saturating_duration_since(std::time::Instant::now()),
            None => std::time::Duration::MAX,
        };
        match rx.recv_timeout(wait) {
            Ok(WorkerWatchdogMsg::Arm {
                seq,
                timeout_ms,
                handle,
            }) => {
                armed = Some((
                    seq,
                    std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms),
                    handle,
                ));
            }
            Ok(WorkerWatchdogMsg::Disarm { seq }) => {
                if armed.as_ref().is_some_and(|(s, _, _)| *s == seq) {
                    armed = None;
                }
            }
            Ok(WorkerWatchdogMsg::Stop) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if let Some((_, _, handle)) = armed.take() {
                    handle.terminate_execution();
                }
            }
        }
    }
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

/// 限时 join worker 线程；超时则 detach（不阻塞调用线程）。
///
/// R3399：worker 在强制中断（`terminate_execution` / interrupt handler）后应能及时退出；
/// 此函数在 `TERMINATE_JOIN_TIMEOUT` 内 join，超时则让 JoinHandle 析构（Rust 不 join
/// 即 detach，线程由 OS 回收），确保 Drop/terminate **永不无限阻塞**主线程。
fn join_bounded_or_detach(handle: JoinHandle<()>) {
    let start = std::time::Instant::now();
    while start.elapsed() < TERMINATE_JOIN_TIMEOUT {
        if handle.is_finished() {
            let _ = handle.join();
            return;
        }
        // 轮询间隔：够短（快速收回已退出的 worker），够长（不占满 CPU）。
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    // 超时仍未退出：放弃 join（drop handle = detach）。worker 在 interrupt/看门狗
    // 作用下最终会退出；若 worker 因故真正卡死，单 worker 线程泄漏不影响主线程。
    // 实测：强制 terminate_execution 后死循环 worker 在 ~ms 级退出，远低于上限。
    drop(handle);
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
    /// V8 Isolate 句柄（跨线程 terminate_execution 用）。`terminate()` 经此强制中断
    /// 卡在 `script.run` 的死循环，使 worker 线程能退出 recv 循环响应 Terminate 命令。
    /// R3399：旧实现 `terminate()` 仅发 Terminate 命令后无条件 `join()`——worker 卡在
    /// 页面提供的死循环时命令永远不被消费，join 永久阻塞主线程（Drop 也走此路径 → DoS）。
    #[cfg(feature = "v8")]
    isolate_handle: Option<v8::IsolateHandle>,
    /// 持久看门狗线程发送端。每次 `script.run` 前 Arm（截止 = timeout_ms），后 Disarm；
    /// 到期调 `terminate_execution`。同 `v8_runtime.rs` 的 SEC-13 看门狗模式。
    #[cfg(feature = "v8")]
    watchdog_tx: Option<Sender<WorkerWatchdogMsg>>,
    /// 终止标志（worker 线程 poll）：`terminate()` 置 true，worker interrupt 回调据此
    /// 强制中断死循环。QuickJS 后端用 `set_interrupt_handler` 据此返 true。
    terminate_flag: Arc<AtomicBool>,
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

        // R3399：worker 线程创建 isolate 后回传其 thread-safe handle，供主线程
        // `terminate()` 跨线程 `terminate_execution` 强制中断死循环。
        #[cfg(feature = "v8")]
        let (handle_tx, handle_rx) = mpsc::channel::<v8::IsolateHandle>();
        // R3399：worker 持久看门狗（同 v8_runtime SEC-13）：每次 script.run 前 Arm，
        // timeout_ms 到期 terminate_execution。每 worker 一个常驻线程。
        // wd_tx 留主线程发 Stop（Drop 时停看门狗）；wd_tx_worker 供 worker 线程 Arm/Disarm。
        #[cfg(feature = "v8")]
        let timeout_ms = config.timeout_ms;
        #[cfg(feature = "v8")]
        let (wd_tx, wd_rx) = mpsc::channel::<WorkerWatchdogMsg>();
        #[cfg(feature = "v8")]
        let wd_tx_worker = wd_tx.clone();
        #[cfg(feature = "v8")]
        thread::Builder::new()
            .name("zero-worker-watchdog".to_string())
            .spawn(move || worker_watchdog_main(wd_rx))
            .map_err(|e| ScriptError::EngineUnavailable(format!("Failed to spawn worker watchdog: {e}")))?;

        let script = script.to_string();
        let terminate_flag = Arc::new(AtomicBool::new(false));
        let worker_terminate_flag = terminate_flag.clone();

        let handle = thread::Builder::new()
            .name("zero-worker".to_string())
            .spawn(move || {
                #[cfg(feature = "v8")]
                worker_thread_fn(
                    script,
                    config,
                    cmd_receiver,
                    event_sender,
                    handle_tx,
                    wd_tx_worker,
                    timeout_ms,
                    worker_terminate_flag,
                );
                #[cfg(feature = "quickjs")]
                worker_thread_fn(script, config, cmd_receiver, event_sender, worker_terminate_flag);
                #[cfg(not(any(feature = "v8", feature = "quickjs")))]
                {
                    let _ = (script, config, cmd_receiver, event_sender, worker_terminate_flag);
                }
            })
            .map_err(|e| ScriptError::EngineUnavailable(format!("Failed to spawn worker thread: {e}")))?;

        // R3399：取回 worker isolate handle（用于强制中断）。worker 线程创建 isolate
        // 后立即发送；这里限时等待（极端慢机器上可能尚未就绪 → terminate 兜底靠
        // 看门狗超时 + bounded join + detach，语义仍安全）。
        #[cfg(feature = "v8")]
        let isolate_handle = handle_rx.recv_timeout(std::time::Duration::from_secs(5)).ok();

        Ok(Self {
            cmd_sender,
            event_receiver,
            worker_handle: Some(handle),
            state: WorkerState::Running,
            #[cfg(feature = "v8")]
            isolate_handle,
            #[cfg(feature = "v8")]
            watchdog_tx: Some(wd_tx),
            terminate_flag,
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
    /// R3399：发送终止命令后**强制中断** worker 当前脚本执行（V8 `terminate_execution` /
    /// QuickJS interrupt handler），再**限时 join**——确保即便 worker 卡在 page-supplied
    /// 死循环也能及时退出；若限时内仍未结束则 detach（泄漏 JoinHandle，worker 线程在
    /// interrupt/看门狗作用下最终退出，由 OS 回收），**绝不无限阻塞调用线程**。已终止的
    /// Worker 不能再发送消息。
    pub fn terminate(&mut self) {
        if self.state == WorkerState::Terminated {
            return;
        }
        // 置终止标志：worker interrupt 回调 / QuickJS interrupt handler 据此中断死循环。
        self.terminate_flag.store(true, Ordering::Release);
        // 发 Terminate 命令（worker 从 recv 循环消费后正常退出）。
        let _ = self.cmd_sender.send(WorkerCommand::Terminate);
        // R3399：强制中断 worker 当前 `script.run`——死循环会被打断，worker 线程得以
        // 返回 recv 循环消费上面的 Terminate 命令。这是「不发命令就 join 死等」根因的
        // 正向修复（旧实现仅靠 worker 主动消费命令，卡死时命令永不被消费）。
        #[cfg(feature = "v8")]
        if let Some(handle) = &self.isolate_handle {
            handle.terminate_execution();
        }
        // 停看门狗（Drop 路径下 worker 已被强制中断，看门狗无需再跑）。
        #[cfg(feature = "v8")]
        if let Some(tx) = &self.watchdog_tx {
            let _ = tx.send(WorkerWatchdogMsg::Stop);
        }
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

// ── Worker 线程实现 ──

/// Worker 线程执行上下文：跨 `script.run` 持有 isolate 句柄、看门狗发送端、
/// timeout 与终止标志，供 [`v8_exec!`] 宏 Arm/Disarm 看门狗并查询终止标志。
///
/// R3399：worker 线程内多次脚本执行共享同一上下文（isolate 复用），看门狗 seq 每次 Arm 递增。
struct WorkerExecCtx {
    /// V8 Isolate 句柄（clone，IsolateHandle 内部 Arc，跨线程 terminate_execution）。
    isolate_handle: v8::IsolateHandle,
    /// 看门狗发送端（Arm/Disarm）。
    wd_tx: Sender<WorkerWatchdogMsg>,
    /// 每次 Arm 的 seq（递增，Disarm 按 seq 撤）。
    seq: u64,
    /// 脚本执行超时（毫秒），0 表示无超时（不 Arm）。
    timeout_ms: u64,
    /// 终止标志（主线程 terminate() 置位）。
    terminate_flag: Arc<AtomicBool>,
}

impl WorkerExecCtx {
    /// Arm 看门狗（timeout_ms > 0 时），返回 guard（Drop 时 Disarm 同 seq）。
    /// 每次 Arm 前 cancel_terminate_execution，清除上一轮可能残留的终止标记
    ///（terminate_execution 一次后 isolate 终止标记可能残留，不清会在下次 run 即时抛终止异常）。
    fn arm(&mut self) -> WorkerWatchdogGuard<'_> {
        self.isolate_handle.cancel_terminate_execution();
        let seq = self.seq;
        self.seq += 1;
        if self.timeout_ms > 0 {
            let _ = self.wd_tx.send(WorkerWatchdogMsg::Arm {
                seq,
                timeout_ms: self.timeout_ms,
                handle: self.isolate_handle.clone(),
            });
        }
        WorkerWatchdogGuard {
            wd_tx: &self.wd_tx,
            seq,
            armed: self.timeout_ms > 0,
        }
    }
}

/// 看门狗 Arm 的 RAII guard：Drop 时 Disarm（仅当 seq 匹配当前装载项）。
struct WorkerWatchdogGuard<'a> {
    wd_tx: &'a Sender<WorkerWatchdogMsg>,
    seq: u64,
    armed: bool,
}

impl Drop for WorkerWatchdogGuard<'_> {
    fn drop(&mut self) {
        if self.armed {
            let _ = self.wd_tx.send(WorkerWatchdogMsg::Disarm { seq: self.seq });
        }
    }
}

/// 在 V8 scope 中执行脚本（R3399：经看门狗 Arm/Disarm 包裹，死循环到期被中断）。
///
/// `$ctx` 为 [`WorkerExecCtx`]（隔离 handle + 看门狗发送端 + timeout_ms + 终止标志）：
/// timeout_ms > 0 时 Arm 看门狗，`script.run` 返回（含被 terminate_execution 中断）后
/// guard Drop 时 Disarm。
macro_rules! v8_exec {
    ($scope:expr, $code:expr, $ctx:expr) => {{
        let scope = &mut $scope;
        let ctx = &mut $ctx;
        // R3399：终止标志置位时（terminate() 已调）直接跳过执行，让 worker 回 recv 循环。
        if ctx.terminate_flag.load(std::sync::atomic::Ordering::Acquire) {
            return;
        }
        let Some(v8_code) = v8::String::new(scope, $code) else {
            return;
        };
        let Some(script) = v8::Script::compile(scope, v8_code, None) else {
            return;
        };
        // R3399：看门狗 Arm（timeout_ms > 0 时）。guard Drop 时 Disarm（seq 匹配才撤）。
        let _wd_guard = ctx.arm();
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
///
/// R3399：`handle_tx` 在 isolate 创建后立即回传 thread-safe handle（主线程据此强制
/// 中断）；`wd_tx` + `timeout_ms` 构建看门狗上下文，每次 `script.run` 经
/// [`v8_exec!`] Arm/Disarm；`terminate_flag` 供宏查询 terminate() 是否已发。
#[allow(clippy::too_many_arguments)] // 线程入口：命令/事件通道 + R3399 中断/看门狗/标志
fn worker_thread_fn(
    init_script: String,
    config: SandboxConfig,
    cmd_receiver: Receiver<WorkerCommand>,
    event_sender: Sender<WorkerEvent>,
    handle_tx: Sender<v8::IsolateHandle>,
    wd_tx: Sender<WorkerWatchdogMsg>,
    timeout_ms: u64,
    terminate_flag: Arc<AtomicBool>,
) {
    // 确保 V8 已初始化
    crate::v8_runtime::V8Sandbox::new().ok();

    // 创建 Isolate + 持久化 Context
    let mut create_params = v8::Isolate::create_params();
    // 初始堆大小可配置（默认 0 = V8 按系统内存推导）。合法组合见 v8_heap_limits。
    if let Some((initial, max)) = crate::v8_heap_limits(&config) {
        create_params = create_params.heap_limits(initial, max);
    }
    let mut isolate = v8::Isolate::new(create_params);

    // R3399：回传 isolate thread-safe handle（主线程 terminate() 据此强制中断死循环）。
    // 立即发送（isolate 存活期有效）；主线程限时接收。
    let isolate_handle = isolate.thread_safe_handle();
    let _ = handle_tx.send(isolate_handle.clone());

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

    // R3399：执行上下文（看门狗 + 终止标志）。与主线程 terminate_flag 共享同一 Arc。
    let mut exec_ctx = WorkerExecCtx {
        isolate_handle,
        wd_tx,
        seq: 0,
        timeout_ms,
        terminate_flag,
    };

    // 执行 bootstrap
    v8_exec!(scope, bootstrap, exec_ctx);

    // 执行初始化脚本
    if !init_script.trim().is_empty() {
        v8_exec!(scope, &init_script, exec_ctx);
    }

    // 排空初始化消息
    v8_drain!(scope, event_sender);

    // 命令循环
    while let Ok(cmd) = cmd_receiver.recv() {
        match cmd {
            WorkerCommand::Execute(code) => {
                v8_exec!(scope, &code, exec_ctx);
                v8_drain!(scope, event_sender);
            }
            WorkerCommand::PostMessage(msg) => {
                let dispatch_code = format!("_dispatchMessage({})", json_safe_arg(&msg));
                v8_exec!(scope, &dispatch_code, exec_ctx);
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
            ..Default::default()
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

    // ── R3399：page-supplied 死循环 worker 不应让 terminate()/Drop 永久挂死主线程 ──
    // 修复前：worker_thread_fn 无执行中断，terminate() 仅发命令后无条件 join → 卡死。
    // 修复后：terminate() 强制 terminate_execution + bounded join + detach 兜底。

    #[test]
    fn test_terminate_returns_for_infinite_loop_init_r3399() {
        // worker 初始化脚本 = 死循环（恶意 page-supplied worker 脚本）。
        let mut worker = WorkerRuntime::new("while (true) {}", SandboxConfig::default()).unwrap();
        // 等 worker 线程进入死循环。
        std::thread::sleep(Duration::from_millis(300));

        let start = std::time::Instant::now();
        // terminate() 须及时返回（远低于 TERMINATE_JOIN_TIMEOUT 的 5s 兜底）。
        worker.terminate();
        let elapsed = start.elapsed();
        assert_eq!(worker.state(), WorkerState::Terminated);
        assert!(
            elapsed < Duration::from_secs(4),
            "terminate() 挂死了主线程（耗时 {:?}），R3399 回归",
            elapsed
        );
    }

    #[test]
    fn test_drop_returns_for_infinite_loop_init_r3399() {
        // Drop 路径走 terminate()——同样不得永久阻塞。
        let worker = WorkerRuntime::new("while (true) {}", SandboxConfig::default()).unwrap();
        std::thread::sleep(Duration::from_millis(300));

        let start = std::time::Instant::now();
        drop(worker);
        let elapsed = start.elapsed();
        assert!(
            elapsed < Duration::from_secs(4),
            "Drop 触发的 terminate() 挂死了主线程（耗时 {:?}），R3399 回归",
            elapsed
        );
    }

    #[test]
    fn test_terminate_returns_for_infinite_loop_via_exec_r3399() {
        // worker 经 execute_script 投递死循环（非初始化脚本路径）。
        let mut worker = WorkerRuntime::new("var x = 0;", SandboxConfig::default()).unwrap();
        std::thread::sleep(Duration::from_millis(200));
        worker.execute_script("while (true) { x++; }").unwrap();
        std::thread::sleep(Duration::from_millis(300));

        let start = std::time::Instant::now();
        worker.terminate();
        let elapsed = start.elapsed();
        assert_eq!(worker.state(), WorkerState::Terminated);
        assert!(
            elapsed < Duration::from_secs(4),
            "execute_script 死循环路径 terminate() 挂死（耗时 {:?}），R3399 回归",
            elapsed
        );
    }

    #[test]
    fn test_watchdog_timeout_interrupts_loop_worker_still_alive_r3399() {
        // timeout_ms > 0：worker 死循环被看门狗到期中断后，worker 仍能响应后续命令
        //（未 terminate，isolate 终止标记被 arm() 的 cancel_terminate_execution 清除）。
        let config = SandboxConfig {
            timeout_ms: 300,
            ..Default::default()
        };
        let mut worker = WorkerRuntime::new("var seen = 0;", config).unwrap();
        std::thread::sleep(Duration::from_millis(200));
        // 投递死循环——看门狗 300ms 后 terminate_execution 中断它。
        worker.execute_script("while (true) {}").unwrap();
        std::thread::sleep(Duration::from_millis(700));
        // worker 应仍存活且能执行新脚本（终止标记已清除）。
        worker
            .execute_script("postMessage('recovered:' + (seen = seen + 1));")
            .unwrap();
        let event = worker.recv_timeout(Duration::from_secs(5)).unwrap();
        match event {
            WorkerEvent::Message(msg) => assert_eq!(msg, "recovered:1"),
            other => panic!("watchdog 中断后 worker 未恢复，got: {other:?}"),
        }
        let start = std::time::Instant::now();
        worker.terminate();
        assert!(
            start.elapsed() < Duration::from_secs(4),
            "恢复后的 worker terminate() 挂死，R3399 回归"
        );
    }
}
