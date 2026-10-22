//! Web Worker 运行时 — 在独立线程中执行 JS 脚本。
//!
//! 提供基本的 Dedicated Worker 实现：
//! - 每个 Worker 在独立的 OS 线程中运行自己的 V8 持久上下文
//! - 通过通道（channel）进行 postMessage/onMessage 通信
//! - 支持 terminate 强制终止
//!
//! 与 V8Sandbox 不同，Worker 使用持久化的 V8 Context，
//! 使得脚本状态在多次执行间保持一致。

use crate::threaded_runtime::ThreadedRuntimeCore;
use crate::{SandboxConfig, ScriptError, WORKER_BOOTSTRAP_JS};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

/// Rust 侧定时器轮条目（bootstrap `setTimeout`/`setInterval` 家族 arm 时登记）。
struct WorkerTimer {
    /// 下次到期时刻。
    due: std::time::Instant,
    /// 重复周期毫秒数；`None` 为一次性定时器（setTimeout），到期即摘除。
    period_ms: Option<u64>,
}

/// 定时器最小间隔 1ms：0 延迟与 0 周期的 `setInterval` 若零等待重臂会让命令循环忙转。
/// HTML 定时器按 nesting level 的 ≥4ms 钳制未实现。
// FIXME(spec): https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html#timer-initialisation-steps
fn worker_timer_delay(ms: u64) -> std::time::Duration {
    std::time::Duration::from_millis(ms.max(1))
}

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
    /// 引擎无关的线程、通道与 bounded shutdown 核。
    core: ThreadedRuntimeCore<WorkerCommand, WorkerEvent>,
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
        Self::new_with_canvas_drain(script, config, None)
    }

    /// 创建 Dedicated Worker 并挂 canvas 操作排空回调（S1b canvas 桥）。
    ///
    /// 回调在 worker 线程命令循环间隙收到 `JSON.stringify(_workerCanvasOps)`
    /// 原文（newCtx/op/resize/bind 条目），由宿主侧求值进 worker 线程本地
    /// CanvasRegistry（renderer worker_host 经 `canvas_context_op` 纯栅格核）。
    /// `None` 时操作读回即丢弃（service worker 等无桥路径不累积）。
    ///
    /// # 参数
    ///
    /// - `script` — Worker 初始化时执行的脚本代码
    /// - `config` — V8 沙箱配置（堆限制等）
    /// - `canvas_drain` — canvas 操作排空回调（S1b；无桥路径传 None）
    pub fn new_with_canvas_drain(
        script: &str,
        config: SandboxConfig,
        canvas_drain: Option<crate::WorkerCanvasDrainHandle>,
    ) -> Result<Self, ScriptError> {
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
        let core = ThreadedRuntimeCore::spawn(
            "zero-worker",
            "worker",
            move |cmd_receiver, event_sender, worker_terminate_flag| {
                // js-dom R84：本模块整文件 cfg(feature = "v8")（lib.rs `#[cfg(feature = "v8")]
                // mod worker`），QuickJS 走 quickjs_worker.rs 的独立 spawn 路径——旧
                // `#[cfg(feature = "quickjs")]` 分支只在 v8+quickjs 同时启用时编译，
                // 调 8 参 v8 签名却传 5 参（a8d5a22d 引入，workspace 组合态编译断，
                // 阻 make test）。v8-only 语义不变：直接调 v8 变体。
                worker_thread_fn(
                    script,
                    config,
                    cmd_receiver,
                    event_sender,
                    handle_tx,
                    wd_tx_worker,
                    timeout_ms,
                    worker_terminate_flag,
                    canvas_drain,
                );
            },
        )?;

        // R3399：取回 worker isolate handle（用于强制中断）。worker 线程创建 isolate
        // 后立即发送；这里限时等待（极端慢机器上可能尚未就绪 → terminate 兜底靠
        // 看门狗超时 + bounded join + detach，语义仍安全）。
        #[cfg(feature = "v8")]
        let isolate_handle = handle_rx.recv_timeout(std::time::Duration::from_secs(5)).ok();

        Ok(Self {
            core,
            #[cfg(feature = "v8")]
            isolate_handle,
            #[cfg(feature = "v8")]
            watchdog_tx: Some(wd_tx),
        })
    }

    /// 向 Worker 发送消息（模拟主线程 postMessage）。
    ///
    /// 消息以 JSON 字符串形式传递，Worker 端通过 `onmessage` 回调接收。
    pub fn post_message(&mut self, message: &str) -> Result<(), ScriptError> {
        if self.core.is_terminated() {
            return Err(ScriptError::InvalidInput(
                "Cannot post message to terminated worker".into(),
            ));
        }
        self.core
            .send(WorkerCommand::PostMessage(message.to_string()))
            .map_err(|_| ScriptError::RuntimeError("Worker thread disconnected".into()))
    }

    /// 向 Worker 发送要执行的额外脚本。
    pub fn execute_script(&mut self, code: &str) -> Result<(), ScriptError> {
        if self.core.is_terminated() {
            return Err(ScriptError::InvalidInput(
                "Cannot execute script on terminated worker".into(),
            ));
        }
        self.core
            .send(WorkerCommand::Execute(code.to_string()))
            .map_err(|_| ScriptError::RuntimeError("Worker thread disconnected".into()))
    }

    /// 尝试接收 Worker 发出的事件（非阻塞）。
    pub fn try_recv(&self) -> Option<WorkerEvent> {
        self.core.try_recv()
    }

    /// 阻塞等待接收 Worker 发出的事件。
    pub fn recv(&self) -> Result<WorkerEvent, ScriptError> {
        self.core
            .recv()
            .map_err(|_| ScriptError::RuntimeError("Worker channel closed".into()))
    }

    /// 带超时地接收 Worker 事件。
    pub fn recv_timeout(&self, timeout: std::time::Duration) -> Result<WorkerEvent, ScriptError> {
        self.core.recv_timeout(timeout).map_err(|e| match e {
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
        let isolate_handle = self.isolate_handle.clone();
        let watchdog_tx = self.watchdog_tx.clone();
        self.core.terminate(WorkerCommand::Terminate, move || {
            if let Some(handle) = isolate_handle {
                handle.terminate_execution();
            }
            if let Some(tx) = watchdog_tx {
                let _ = tx.send(WorkerWatchdogMsg::Stop);
            }
        });
    }

    /// 获取 Worker 当前状态。
    pub fn state(&self) -> WorkerState {
        if self.core.is_terminated() {
            WorkerState::Terminated
        } else {
            WorkerState::Running
        }
    }

    /// Worker 是否仍在运行。
    pub fn is_running(&self) -> bool {
        !self.core.is_terminated()
    }

    /// 工作线程是否已退出（含 panic——panic 绕过单出口 `Closed` 发送）。宿主据此
    /// 探测异常退出补错误上报（S1d），不静默悬挂。
    pub fn thread_finished(&self) -> bool {
        self.core.is_thread_finished()
    }
}

impl Drop for WorkerRuntime {
    fn drop(&mut self) {
        self.terminate();
    }
}

impl std::fmt::Debug for WorkerRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkerRuntime").field("state", &self.state()).finish()
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
///
/// S1a（DC-4 场景 d）：`$report` 为 true 时，page-supplied 代码异常（编译错/运行时抛出）
/// 经 TryCatch 提取消息发 `WorkerEvent::Error`——worker 不因此退出（每次执行独立）；
/// 看门狗 terminate_execution 的终止异常不是页面错误，不报（同 `v8_runtime` 判定顺序）。
macro_rules! v8_exec {
    ($scope:expr, $code:expr, $ctx:expr, $sender:expr, $report:expr) => {{
        let scope = &mut $scope;
        let ctx = &mut $ctx;
        // R3399：终止标志置位时（terminate() 已调）直接跳过执行，让 worker 回 recv 循环。
        if ctx.terminate_flag.load(std::sync::atomic::Ordering::Acquire) {
            return;
        }
        v8::tc_scope!(let try_catch, scope);
        let Some(v8_code) = v8::String::new(try_catch, $code) else {
            return;
        };
        let script = v8::Script::compile(try_catch, v8_code, None);
        if try_catch.has_caught() || script.is_none() {
            if $report {
                let msg = crate::v8_try_catch_message!(try_catch);
                let _ = $sender.send(WorkerEvent::Error(msg));
            }
            return;
        }
        // R3399：看门狗 Arm（timeout_ms > 0 时）。guard Drop 时 Disarm（seq 匹配才撤）。
        let _wd_guard = ctx.arm();
        let _ = script.unwrap().run(try_catch);
        drop(_wd_guard);
        if $report && try_catch.has_caught() && !try_catch.has_terminated() {
            let msg = crate::v8_try_catch_message!(try_catch);
            let _ = $sender.send(WorkerEvent::Error(msg));
        }
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

/// 排空 bootstrap 定时器操作队列 `_workerTimerOps`（arm/cancel/close，保持提交序），
/// 填充 Rust 定时器轮。元素以 `String(op)` 序列化（`"arm,1,50,50"` 形态）读回。
/// 返回是否遇到 `close()`（bootstrap close 语义：worker 退出，调用方发 Closed）。
macro_rules! v8_drain_timer_ops {
    ($scope:expr, $timers:expr) => {{
        let scope = &mut $scope;
        let timers = $timers;
        let mut close_requested = false;
        let Some(len_code) = v8::String::new(scope, "_workerTimerOps.length") else {
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
        let Ok(count) = len_str.to_rust_string_lossy(scope).parse::<usize>() else {
            return;
        };
        for i in 0..count {
            let get_code = format!("String(_workerTimerOps[{i}])");
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
            let raw = str_val.to_rust_string_lossy(scope);
            let parts: Vec<&str> = raw.split(',').collect();
            match parts.as_slice() {
                ["arm", id, delay, period] => {
                    if let Ok(id) = id.parse::<u64>() {
                        // 延迟/周期按 f64 截断（页面常写 `1000 / 60` 这类分数周期）。
                        let delay = delay.parse::<f64>().unwrap_or(0.0).max(0.0) as u64;
                        let period = period.parse::<f64>().unwrap_or(0.0).max(0.0) as u64;
                        let due = std::time::Instant::now() + worker_timer_delay(delay);
                        let period_ms = (period > 0).then_some(period);
                        timers.insert(id, WorkerTimer { due, period_ms });
                    }
                }
                ["cancel", id, ..] => {
                    if let Ok(id) = id.parse::<u64>() {
                        timers.remove(&id);
                    }
                }
                ["close", ..] => {
                    close_requested = true;
                }
                _ => {}
            }
        }
        if let Some(clear_code) = v8::String::new(scope, "_workerTimerOps = []") {
            if let Some(clear_script) = v8::Script::compile(scope, clear_code, None) {
                let _ = clear_script.run(scope);
            }
        }
        close_requested
    }};
}

/// 排空 bootstrap canvas 操作队列 `_workerCanvasOps`（S1b canvas 桥）：整体
/// `JSON.stringify` 读回交宿主排空回调（renderer worker_host 求值进 worker 线程本地
/// CanvasRegistry，条目 newCtx/op/resize/bind）；无回调（`None`，如 service worker
/// 等无桥路径）时读回即丢弃防累积。调用位与定时器排空同点（命令循环间隙）。
macro_rules! v8_drain_canvas_ops {
    ($scope:expr, $drain:expr) => {{
        let scope = &mut $scope;
        let mut json = String::new();
        if let Some(code) = v8::String::new(scope, "JSON.stringify(_workerCanvasOps)") {
            if let Some(script) = v8::Script::compile(scope, code, None) {
                if let Some(result) = script.run(scope) {
                    if let Some(s) = result.to_string(scope) {
                        json = s.to_rust_string_lossy(scope);
                    }
                }
            }
        }
        if json.len() > 2 {
            if let Some(drain) = $drain.as_ref() {
                if let Ok(mut d) = drain.lock() {
                    d(&json);
                }
            }
        }
        if let Some(clear_code) = v8::String::new(scope, "_workerCanvasOps = []") {
            if let Some(clear_script) = v8::Script::compile(scope, clear_code, None) {
                let _ = clear_script.run(scope);
            }
        }
    }};
}

/// Worker 线程入口：body 退出（正常/terminate/`close()`/执行中断）后发 [`WorkerEvent::Closed`]。
///
/// S1a：Closed 收敛到唯一出口——body 内所有早退路径（含 [`v8_exec!`] 失败路径）不再各自
/// 补发，避免遗漏（旧实现 page-supplied 脚本编译错会静默杀死线程且无任何事件）。
#[allow(clippy::too_many_arguments)] // 线程入口：命令/事件通道 + R3399 中断/看门狗/标志 + S1b canvas 排空
fn worker_thread_fn(
    init_script: String,
    config: SandboxConfig,
    cmd_receiver: Receiver<WorkerCommand>,
    event_sender: Sender<WorkerEvent>,
    handle_tx: Sender<v8::IsolateHandle>,
    wd_tx: Sender<WorkerWatchdogMsg>,
    timeout_ms: u64,
    terminate_flag: Arc<AtomicBool>,
    canvas_drain: Option<crate::WorkerCanvasDrainHandle>,
) {
    worker_thread_body(
        init_script,
        config,
        cmd_receiver,
        event_sender.clone(),
        handle_tx,
        wd_tx,
        timeout_ms,
        terminate_flag,
        canvas_drain,
    );
    let _ = event_sender.send(WorkerEvent::Closed);
}

/// Worker 线程主循环。
///
/// 使用持久化的 V8 Context，使得脚本状态在多次执行间保持一致。
///
/// R3399：`handle_tx` 在 isolate 创建后立即回传 thread-safe handle（主线程据此强制
/// 中断）；`wd_tx` + `timeout_ms` 构建看门狗上下文，每次 `script.run` 经
/// [`v8_exec!`] Arm/Disarm；`terminate_flag` 供宏查询 terminate() 是否已发。
#[allow(clippy::too_many_arguments)] // 线程入口：命令/事件通道 + R3399 中断/看门狗/标志 + S1b canvas 排空
fn worker_thread_body(
    init_script: String,
    config: SandboxConfig,
    cmd_receiver: Receiver<WorkerCommand>,
    event_sender: Sender<WorkerEvent>,
    handle_tx: Sender<v8::IsolateHandle>,
    wd_tx: Sender<WorkerWatchdogMsg>,
    timeout_ms: u64,
    terminate_flag: Arc<AtomicBool>,
    canvas_drain: Option<crate::WorkerCanvasDrainHandle>,
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

    // 注入 Worker 全局环境（消息队列 + 定时器 + close，V8/QuickJS 共享）
    let bootstrap = WORKER_BOOTSTRAP_JS;

    // R3399：执行上下文（看门狗 + 终止标志）。与主线程 terminate_flag 共享同一 Arc。
    let mut exec_ctx = WorkerExecCtx {
        isolate_handle,
        wd_tx,
        seq: 0,
        timeout_ms,
        terminate_flag,
    };

    // 执行 bootstrap（宿主供给代码，异常不上报 Error 事件）
    v8_exec!(scope, bootstrap, exec_ctx, event_sender, false);

    // 执行初始化脚本（page-supplied worker body，异常上报 Error 事件）
    if !init_script.trim().is_empty() {
        v8_exec!(scope, &init_script, exec_ctx, event_sender, true);
    }

    // 排空初始化消息与定时器操作
    v8_drain!(scope, event_sender);
    let mut timers: HashMap<u64, WorkerTimer> = HashMap::new();
    if v8_drain_timer_ops!(scope, &mut timers) {
        return; // bootstrap close()：wrapper 发 Closed
    }
    v8_drain_canvas_ops!(scope, canvas_drain);

    // 命令循环 + 定时器轮：到期定时器在命令循环间隙服务（isolate 无事件循环，
    // 纯 JS 不可自举定时器）；无定时器时退化为阻塞 recv（与旧实现同语义）。
    loop {
        let now = std::time::Instant::now();
        let due_ids: Vec<u64> = timers
            .iter()
            .filter(|(_, timer)| timer.due <= now)
            .map(|(id, _)| *id)
            .collect();
        for id in due_ids {
            let period_ms = timers.get(&id).and_then(|timer| timer.period_ms);
            let fire_code = format!("_workerTimerFire({id})");
            v8_exec!(scope, &fire_code, exec_ctx, event_sender, true);
            v8_drain!(scope, event_sender);
            let close_requested = v8_drain_timer_ops!(scope, &mut timers);
            v8_drain_canvas_ops!(scope, canvas_drain);
            // interval 重臂（one-shot 摘除；callback 内 clear 已移除条目则不重臂）。
            match period_ms {
                Some(period_ms) => {
                    if let Some(timer) = timers.get_mut(&id) {
                        timer.due = std::time::Instant::now() + worker_timer_delay(period_ms);
                    }
                }
                None => {
                    timers.remove(&id);
                }
            }
            if close_requested {
                return; // wrapper 发 Closed
            }
        }

        let next_due = timers.values().map(|timer| timer.due).min();
        let cmd = match next_due {
            Some(due) => {
                let now = std::time::Instant::now();
                if due <= now {
                    continue;
                }
                match cmd_receiver.recv_timeout(due - now) {
                    Ok(cmd) => cmd,
                    Err(mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
            None => match cmd_receiver.recv() {
                Ok(cmd) => cmd,
                Err(_) => break,
            },
        };

        match cmd {
            WorkerCommand::Execute(code) => {
                v8_exec!(scope, &code, exec_ctx, event_sender, true);
                v8_drain!(scope, event_sender);
                if v8_drain_timer_ops!(scope, &mut timers) {
                    return; // wrapper 发 Closed
                }
                v8_drain_canvas_ops!(scope, canvas_drain);
            }
            WorkerCommand::PostMessage(msg) => {
                let dispatch_code = format!("_dispatchMessage({})", json_safe_arg(&msg));
                v8_exec!(scope, &dispatch_code, exec_ctx, event_sender, true);
                v8_drain!(scope, event_sender);
                if v8_drain_timer_ops!(scope, &mut timers) {
                    return; // wrapper 发 Closed
                }
                v8_drain_canvas_ops!(scope, canvas_drain);
            }
            WorkerCommand::Terminate => {
                break;
            }
        }
    }
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
    fn test_worker_self_global_alias() {
        // worker 全局必须有 self（WorkerGlobalScope.self）——规范 worker 脚本以
        // self.onmessage / self.postMessage 收发，缺失时脚本首行即抛 ReferenceError。
        let mut worker = WorkerRuntime::new(
            "self.onmessage = function(e) { self.postMessage('self: ' + e.data); };",
            SandboxConfig::default(),
        )
        .unwrap();

        std::thread::sleep(Duration::from_millis(200));
        worker.post_message("hi").unwrap();

        let event = worker.recv_timeout(Duration::from_secs(5)).unwrap();
        match event {
            WorkerEvent::Message(msg) => assert_eq!(msg, "self: hi"),
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

    // ── S1a：worker 定时器（setTimeout/setInterval/clear*，命令循环间隙服务） ──

    #[test]
    fn test_worker_set_timeout_fires() {
        let mut worker = WorkerRuntime::new(
            "setTimeout(function() { postMessage('timeout-fired'); }, 50);",
            SandboxConfig::default(),
        )
        .unwrap();
        let event = worker.recv_timeout(Duration::from_secs(5)).unwrap();
        match event {
            WorkerEvent::Message(msg) => assert_eq!(msg, "timeout-fired"),
            other => panic!("Expected Message, got: {other:?}"),
        }
        worker.terminate();
    }

    #[test]
    fn test_worker_set_interval_repeats_until_cleared() {
        let mut worker = WorkerRuntime::new(
            "var n = 0; var id = setInterval(function() { n++; postMessage('tick' + n); }, 50);",
            SandboxConfig::default(),
        )
        .unwrap();
        for i in 1..=3 {
            let event = worker.recv_timeout(Duration::from_secs(5)).unwrap();
            match event {
                WorkerEvent::Message(msg) => assert_eq!(msg, format!("tick{i}")),
                other => panic!("tick {i}: Expected Message, got: {other:?}"),
            }
        }
        worker.execute_script("clearInterval(id);").unwrap();
        // 已到期的 tick 可能仍在事件通道，先排空再观察静默窗。
        std::thread::sleep(Duration::from_millis(250));
        while let Some(WorkerEvent::Message(_)) = worker.try_recv() {}
        assert!(worker.try_recv().is_none(), "clearInterval 后定时器仍触发");
        worker.terminate();
    }

    #[test]
    fn test_worker_clear_timeout_prevents_fire() {
        let mut worker = WorkerRuntime::new(
            "var t = setTimeout(function() { postMessage('fired'); }, 500);
             onmessage = function(e) { postMessage('echo:' + e.data); };",
            SandboxConfig::default(),
        )
        .unwrap();
        std::thread::sleep(Duration::from_millis(200));
        worker.execute_script("clearTimeout(t);").unwrap();
        // 越过原 500ms 到期点：若清除未生效，此处必然收到 'fired'。
        std::thread::sleep(Duration::from_millis(400));
        worker.post_message("probe").unwrap();
        let event = worker.recv_timeout(Duration::from_secs(5)).unwrap();
        match event {
            WorkerEvent::Message(msg) => assert_eq!(msg, "echo:probe"),
            other => panic!("Expected echo Message, got: {other:?}"),
        }
        assert!(worker.try_recv().is_none(), "clearTimeout 后定时器仍触发");
        worker.terminate();
    }

    #[test]
    fn test_worker_timer_callback_error_keeps_interval() {
        let mut worker = WorkerRuntime::new(
            "var n = 0; setInterval(function() { n++; if (n === 1) { throw new Error('tick-boom'); } postMessage('tick' + n); }, 50);",
            SandboxConfig::default(),
        )
        .unwrap();
        // 第一次 tick 抛错 → Error 事件。
        let event = worker.recv_timeout(Duration::from_secs(5)).unwrap();
        match event {
            WorkerEvent::Error(msg) => assert!(msg.contains("tick-boom"), "Error 消息: {msg}"),
            other => panic!("Expected Error, got: {other:?}"),
        }
        // interval 不因回调异常中断：后续 tick 正常。
        let event = worker.recv_timeout(Duration::from_secs(5)).unwrap();
        match event {
            WorkerEvent::Message(msg) => assert_eq!(msg, "tick2"),
            other => panic!("Expected Message after error, got: {other:?}"),
        }
        worker.terminate();
    }

    #[test]
    fn test_worker_close_emits_closed() {
        let mut worker = WorkerRuntime::new("var x = 1;", SandboxConfig::default()).unwrap();
        std::thread::sleep(Duration::from_millis(200));
        worker.execute_script("close();").unwrap();
        let event = worker.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(matches!(event, WorkerEvent::Closed), "Expected Closed, got: {event:?}");
    }

    // ── S1a：page-supplied 脚本异常 → Error 事件（DC-4 场景 d），worker 不因此退出 ──

    #[test]
    fn test_worker_script_error_emits_error_and_stays_alive() {
        let mut worker = WorkerRuntime::new(
            "onmessage = function(e) { postMessage('echo:' + e.data); };",
            SandboxConfig::default(),
        )
        .unwrap();
        std::thread::sleep(Duration::from_millis(200));
        worker.execute_script("throw new Error('boom');").unwrap();
        let event = worker.recv_timeout(Duration::from_secs(5)).unwrap();
        match event {
            WorkerEvent::Error(msg) => assert!(msg.contains("boom"), "Error 消息: {msg}"),
            other => panic!("Expected Error, got: {other:?}"),
        }
        // 抛错后 worker 仍存活：消息处理继续工作。
        worker.post_message("probe").unwrap();
        let event = worker.recv_timeout(Duration::from_secs(5)).unwrap();
        match event {
            WorkerEvent::Message(msg) => assert_eq!(msg, "echo:probe"),
            other => panic!("Expected Message after error, got: {other:?}"),
        }
        worker.terminate();
    }

    #[test]
    fn test_worker_init_script_compile_error_emits_error_and_closed() {
        // 初始化脚本语法错：Error 事件 + 线程退出（Closed），不再静默死亡。
        let worker = WorkerRuntime::new("function broken( {", SandboxConfig::default()).unwrap();
        let mut saw_error = false;
        let mut saw_closed = false;
        for _ in 0..2 {
            match worker.recv_timeout(Duration::from_secs(5)) {
                Ok(WorkerEvent::Error(msg)) => saw_error = msg.contains("broken") || !msg.is_empty(),
                Ok(WorkerEvent::Closed) => saw_closed = true,
                Ok(other) => panic!("Unexpected event: {other:?}"),
                Err(e) => panic!("recv failed: {e}"),
            }
        }
        assert!(saw_error, "初始化编译错未报 Error 事件");
        assert!(saw_closed, "初始化编译错后未发 Closed");
    }

    // ── S1b：worker canvas 排空（bootstrap `_workerCanvasOps` → Rust drain 回调） ──

    /// 收集 drain 批次直到出现 `needle`（超时 panic——命令循环间隙排空，延迟毫秒级）。
    fn wait_canvas_drain(drained: &std::sync::Arc<std::sync::Mutex<Vec<String>>>, needle: &str) -> String {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            let all = drained.lock().unwrap().join("");
            if all.contains(needle) {
                return all;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "canvas drain 未出现 {needle}，got: {all}"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    #[test]
    fn test_worker_canvas_ops_drain_constructor_path() {
        let drained: std::sync::Arc<std::sync::Mutex<Vec<String>>> = Default::default();
        let sink = drained.clone();
        let drain: crate::WorkerCanvasDrain = Box::new(move |json: &str| {
            sink.lock().unwrap().push(json.to_string());
        });
        let mut worker = WorkerRuntime::new_with_canvas_drain(
            "var oc = new OffscreenCanvas(64, 32);\
             var ctx = oc.getContext('2d');\
             ctx.fillStyle = '#ff0000';\
             ctx.fillRect(0, 0, 64, 32);\
             postMessage('drawn');",
            SandboxConfig::default(),
            Some(std::sync::Arc::new(std::sync::Mutex::new(drain))),
        )
        .unwrap();
        // 先等 drain 回调（init 排空点晚于消息事件入队——先 recv 再 terminate 会
        // 用 terminate_execution 打断排空序列，drain 丢批）。
        let all = wait_canvas_drain(&drained, "fillRect");
        let event = worker.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(
            matches!(&event, WorkerEvent::Message(m) if m == "drawn"),
            "got: {event:?}"
        );
        worker.terminate();
        assert!(all.contains("\"newCtx\""), "构造路径应排空 newCtx，got: {all}");
        assert!(all.contains("setFillStyle"), "样式 setter 应记 op，got: {all}");
    }

    #[test]
    fn test_worker_canvas_ops_drain_transfer_revive_binds_surface_key() {
        let drained: std::sync::Arc<std::sync::Mutex<Vec<String>>> = Default::default();
        let sink = drained.clone();
        let drain: crate::WorkerCanvasDrain = Box::new(move |json: &str| {
            sink.lock().unwrap().push(json.to_string());
        });
        let mut worker = WorkerRuntime::new_with_canvas_drain(
            // 模拟 shim spawn prologue 的 `_dispatchMessage` JSON.parse 补丁（真线程\
            // 流水线：宿主 post JSON 字符串 → prologue 解析 → bootstrap `_zwDm` 复活\
            // `__zwXferCanvas` 标记 → onmessage）——裸 WorkerRuntime 消息面以字符串\
            // 为载体，无补丁则 revive 不达。\
            "var _zwDm = _dispatchMessage;\
             _dispatchMessage = function (raw) { _zwDm(typeof raw === 'string' ? JSON.parse(raw) : raw); };\
             onmessage = function (e) {\
               var ctx = e.data.getContext('2d');\
               ctx.fillRect(0, 0, 64, 32);\
               postMessage('bound:' + e.data.width + 'x' + e.data.height + ':'\
                 + (e.data instanceof OffscreenCanvas));\
             };",
            SandboxConfig::default(),
            Some(std::sync::Arc::new(std::sync::Mutex::new(drain))),
        )
        .unwrap();
        std::thread::sleep(Duration::from_millis(200));
        worker
            .post_message(r#"{"__zwXferCanvas":{"key":"xfer-9","w":64,"h":32}}"#)
            .unwrap();
        // 先等 drain 回调（PostMessage 排空点晚于消息事件入队——同上竞态规避）。
        let all = wait_canvas_drain(&drained, "\"bind\"");
        let event = worker.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(
            matches!(&event, WorkerEvent::Message(m) if m == "bound:64x32:true"),
            "复活对象应为本地 OffscreenCanvas（尺寸保真），got: {event:?}"
        );
        worker.terminate();
        // bind 与 newCtx/fillRect 应共享同一复活 cid（宿主排空侧 prov 映射闭环——
        // 复活路径 getContext 也记 newCtx）。
        let ops: Vec<serde_json::Value> = drained
            .lock()
            .unwrap()
            .iter()
            .filter_map(|b| serde_json::from_str::<Vec<serde_json::Value>>(b).ok())
            .flatten()
            .collect();
        let bind_cid = ops
            .iter()
            .find(|op| op[0] == "bind" && op[2] == "xfer-9")
            .map(|op| op[1].as_u64().unwrap())
            .expect("复活应记 bind op");
        assert!(
            ops.iter()
                .any(|op| op[0] == "newCtx" && op[1].as_u64() == Some(bind_cid)),
            "复活路径 getContext 应记同 cid newCtx，ops: {all}"
        );
        assert!(
            ops.iter()
                .any(|op| op[0] == "op" && op[1].as_u64() == Some(bind_cid) && op[2] == "fillRect"),
            "绘制 op 应引用复活 cid，ops: {all}"
        );
    }

    /// S1c：`__zwXferCanvas` 标记 `sel` 字段随复活进 bind 第 4 元（patch Register
    /// lazy 首拍查源元素矩形的数据源；旧 3 元标记兼容——sel 空串）。
    #[test]
    fn test_worker_canvas_xfer_revive_carries_source_selector() {
        let drained: std::sync::Arc<std::sync::Mutex<Vec<String>>> = Default::default();
        let sink = drained.clone();
        let drain: crate::WorkerCanvasDrain = Box::new(move |json: &str| {
            sink.lock().unwrap().push(json.to_string());
        });
        let mut worker = WorkerRuntime::new_with_canvas_drain(
            "var _zwDm = _dispatchMessage;\
             _dispatchMessage = function (raw) { _zwDm(typeof raw === 'string' ? JSON.parse(raw) : raw); };\
             onmessage = function (e) { postMessage('ok'); };",
            SandboxConfig::default(),
            Some(std::sync::Arc::new(std::sync::Mutex::new(drain))),
        )
        .unwrap();
        std::thread::sleep(Duration::from_millis(200));
        worker
            .post_message(r##"{"__zwXferCanvas":{"key":"xfer-4","sel":"#cv","w":8,"h":8}}"##)
            .unwrap();
        let _ = wait_canvas_drain(&drained, "\"bind\"");
        worker.terminate();
        let ops: Vec<serde_json::Value> = drained
            .lock()
            .unwrap()
            .iter()
            .filter_map(|b| serde_json::from_str::<Vec<serde_json::Value>>(b).ok())
            .flatten()
            .collect();
        let bind = ops.iter().find(|op| op[0] == "bind").expect("复活应记 bind op");
        assert_eq!(bind.as_array().unwrap().len(), 4, "bind 应含 sel 第 4 元: {bind}");
        assert_eq!(bind[3].as_str(), Some("#cv"));
    }

    #[test]
    fn test_worker_canvas_ops_without_drain_discarded() {
        // None drain：canvas op 读后即弃（service worker 等无宿主排空面不堆积）。
        let mut worker = WorkerRuntime::new(
            "var oc = new OffscreenCanvas(8, 8);\
             oc.getContext('2d').fillRect(0, 0, 8, 8);\
             postMessage('ok');",
            SandboxConfig::default(),
        )
        .unwrap();
        let event = worker.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(
            matches!(&event, WorkerEvent::Message(m) if m == "ok"),
            "无 drain 时 canvas op 不应阻塞脚本，got: {event:?}"
        );
        worker.terminate();
    }

    #[test]
    fn test_worker_thread_finished_detects_panic_bypassing_closed() {
        // S1d 异常退出探测：drain 回调 panic 在工作线程体内展开——绕过线程入口的
        // 单出口 `Closed` 发送（`worker_thread_fn` 尾部），事件通道断连。宿主侧
        // `try_recv()` 对 Empty/Disconnected 同归 None（ThreadedRuntimeCore::try_recv
        // `.ok()` 折叠），`thread_finished()` 是唯一的异常退出判据——宿主据此补
        // 错误上报，不静默悬挂（renderer worker_host pump_events 消费）。
        let drain: crate::WorkerCanvasDrain = Box::new(|_json: &str| {
            panic!("expected: drain panic probe");
        });
        let worker = WorkerRuntime::new_with_canvas_drain(
            // init 脚本记一条 canvas op——drain 仅在 op 批非空时回调（`json.len() > 2`），
            // 空批不触发则 panic 探针永不达。
            "var oc = new OffscreenCanvas(8, 8); oc.getContext('2d').fillRect(0, 0, 8, 8);",
            SandboxConfig::default(),
            Some(std::sync::Arc::new(std::sync::Mutex::new(drain))),
        )
        .unwrap();
        // init 排空点触发 panic（isolate handle 先于脚本执行回传，构造器不阻塞）。
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while !worker.thread_finished() {
            assert!(std::time::Instant::now() < deadline, "panic 线程未被探测为 finished");
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(
            worker.try_recv().is_none(),
            "异常退出无 Closed 事件（否则非单出口绕过）"
        );
    }
}
