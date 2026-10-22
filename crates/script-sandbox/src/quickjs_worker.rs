//! QuickJS Web Worker 运行时（对应 V8 的 worker.rs）。
//!
//! QuickJS Runtime 非 Send（Rc），但 Worker 在独立线程内创建自己的 Runtime
//! （不跨线程），通过通道与主线程通信（同 V8 Worker）。
//!
//! 提供 Dedicated Worker：独立线程 + 持久 QuickJS Context + postMessage/onmessage。

use crate::threaded_runtime::ThreadedRuntimeCore;
use crate::{SandboxConfig, ScriptError, WORKER_BOOTSTRAP_JS};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};

/// Rust 侧定时器轮条目（同 worker.rs，bootstrap 定时器家族 arm 时登记）。
struct WorkerTimer {
    /// 下次到期时刻。
    due: std::time::Instant,
    /// 重复周期毫秒数；`None` 为一次性定时器（setTimeout），到期即摘除。
    period_ms: Option<u64>,
    /// 创建时 nesting level（bootstrap arm op 第 5 元）；interval 重臂钳制按
    /// `nesting + 已重臂代数` 逐代判定（spec 重复分支在 task 内重跑 initialisation）。
    nesting: u64,
}

/// 定时器最小间隔 1ms（同 worker.rs：0 周期 setInterval 防命令循环忙转）。
/// 嵌套钳制（DC-1）：创建 delay 由 bootstrap JS 按创建 nesting 钳；interval 重臂按
/// [`worker::worker_timer_period_at`] 逐代钳——floor 保留为钳制未覆盖场景的忙转底线。
// https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html#timer-initialisation-steps
fn worker_timer_delay(ms: u64) -> std::time::Duration {
    std::time::Duration::from_millis(ms.max(1))
}

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

/// Dedicated Worker 运行时（QuickJS 实现）。
///
/// 在独立线程中运行 QuickJS Runtime + 持久 Context，通过通道与主线程通信。
/// Worker 的 JS 环境在所有脚本执行间保持状态一致。
pub struct WorkerRuntime {
    core: ThreadedRuntimeCore<WorkerCommand, WorkerEvent>,
}

impl WorkerRuntime {
    /// 创建新的 Dedicated Worker（QuickJS）。
    ///
    /// Worker 在独立线程中创建自己的 QuickJS Runtime + Context 并执行初始化脚本。
    pub fn new(script: &str, config: SandboxConfig) -> Result<Self, ScriptError> {
        Self::new_with_canvas_drain(script, config, None)
    }

    /// 创建 Dedicated Worker 并挂 canvas 操作排空回调（S1b canvas 桥，同 worker.rs
    /// v8 侧语义）：回调在命令循环间隙收 `JSON.stringify(_workerCanvasOps)` 原文。
    /// `None` 时操作读回即丢弃（无桥路径不累积）。
    pub fn new_with_canvas_drain(
        script: &str,
        config: SandboxConfig,
        canvas_drain: Option<crate::WorkerCanvasDrainHandle>,
    ) -> Result<Self, ScriptError> {
        let script = script.to_string();

        let core = ThreadedRuntimeCore::spawn(
            "zero-quickjs-worker",
            "worker",
            move |cmd_receiver, event_sender, worker_terminate_flag| {
                quickjs_worker_thread_fn(
                    script,
                    config,
                    cmd_receiver,
                    event_sender,
                    worker_terminate_flag,
                    canvas_drain,
                );
            },
        )?;

        Ok(Self { core })
    }

    /// 向 Worker 发送消息（模拟主线程 postMessage）。
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
    /// R3399：置终止标志（worker interrupt handler 据此返 true 中断死循环）后发 Terminate
    /// 命令，再限时 join——确保即便 worker 卡在 page-supplied 死循环也能及时退出；超时则
    /// detach（绝不无限阻塞调用线程，Drop 也走此路径）。QuickJS 对称 V8 worker 修复。
    pub fn terminate(&mut self) {
        self.core.terminate(WorkerCommand::Terminate, || {});
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

/// Worker 线程入口：body 退出（正常/terminate/`close()`）后发 [`WorkerEvent::Closed`]。
/// 对称 worker.rs 的 Closed 单出口收敛。
fn quickjs_worker_thread_fn(
    init_script: String,
    config: SandboxConfig,
    cmd_receiver: Receiver<WorkerCommand>,
    event_sender: Sender<WorkerEvent>,
    terminate_flag: Arc<AtomicBool>,
    canvas_drain: Option<crate::WorkerCanvasDrainHandle>,
) {
    quickjs_worker_thread_body(
        init_script,
        config,
        cmd_receiver,
        event_sender.clone(),
        terminate_flag,
        canvas_drain,
    );
    let _ = event_sender.send(WorkerEvent::Closed);
}

/// Worker 线程主循环（QuickJS 实现）。
///
/// 创建 QuickJS Runtime + 持久 Context，注入 Worker bootstrap，执行初始化脚本，
/// 然后循环处理命令（Execute/PostMessage/Terminate）+ 定时器轮。
///
/// R3399：`terminate_flag` 经 `set_interrupt_handler` 注册——terminate() 置位后，
/// QuickJS 解释器周期性调 interrupt handler 返 true → 抛 uncatchable 异常中断
/// page-supplied 死循环，worker 线程得以退出 recv 循环（对称 V8 worker 修复）。
fn quickjs_worker_thread_body(
    init_script: String,
    config: SandboxConfig,
    cmd_receiver: Receiver<WorkerCommand>,
    event_sender: Sender<WorkerEvent>,
    terminate_flag: Arc<AtomicBool>,
    canvas_drain: Option<crate::WorkerCanvasDrainHandle>,
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

    // Worker 全局环境（消息队列 + 定时器 + close，V8/QuickJS 共享 bootstrap）
    let bootstrap = WORKER_BOOTSTRAP_JS;

    // 执行 bootstrap（宿主供给，不上报）+ 初始化脚本（page-supplied，上报）+
    // 排空初始化消息与定时器操作
    let mut timers: HashMap<u64, WorkerTimer> = HashMap::new();
    let mut init_close = false;
    ctx.with(|mut ctx| {
        quickjs_exec(&mut ctx, bootstrap, &event_sender, &terminate_flag, false);
        if !init_script.trim().is_empty() {
            quickjs_exec(&mut ctx, &init_script, &event_sender, &terminate_flag, true);
        }
        quickjs_drain(&mut ctx, &event_sender);
        init_close = quickjs_drain_timer_ops(&mut ctx, &mut timers);
        quickjs_drain_canvas_ops(&mut ctx, &canvas_drain);
    });
    if init_close {
        return; // bootstrap close()：wrapper 发 Closed
    }

    // 命令循环 + 定时器轮（到期定时器在命令循环间隙服务，同 worker.rs）。
    loop {
        let now = std::time::Instant::now();
        let due_ids: Vec<u64> = timers
            .iter()
            .filter(|(_, timer)| timer.due <= now)
            .map(|(id, _)| *id)
            .collect();
        for id in due_ids {
            let period_ms = timers.get(&id).and_then(|timer| timer.period_ms);
            let mut close_requested = false;
            ctx.with(|mut ctx| {
                quickjs_exec(
                    &mut ctx,
                    &format!("_workerTimerFire({id})"),
                    &event_sender,
                    &terminate_flag,
                    true,
                );
                quickjs_drain(&mut ctx, &event_sender);
                close_requested = quickjs_drain_timer_ops(&mut ctx, &mut timers);
                quickjs_drain_canvas_ops(&mut ctx, &canvas_drain);
            });
            // interval 重臂（one-shot 摘除；callback 内 clear 已移除条目则不重臂）。
            // 重臂钳制按创建 nesting + 代数逐代判定（同 worker.rs，spec 同口径）。
            match period_ms {
                Some(period_ms) => {
                    if let Some(timer) = timers.get_mut(&id) {
                        timer.nesting += 1;
                        let period = crate::worker_timer_period_at(timer.nesting, period_ms);
                        timer.due = std::time::Instant::now() + worker_timer_delay(period);
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
                let mut close_requested = false;
                ctx.with(|mut ctx| {
                    quickjs_exec(&mut ctx, &code, &event_sender, &terminate_flag, true);
                    quickjs_drain(&mut ctx, &event_sender);
                    close_requested = quickjs_drain_timer_ops(&mut ctx, &mut timers);
                    quickjs_drain_canvas_ops(&mut ctx, &canvas_drain);
                });
                if close_requested {
                    return; // wrapper 发 Closed
                }
            }
            WorkerCommand::PostMessage(msg) => {
                let mut close_requested = false;
                ctx.with(|mut ctx| {
                    let dispatch = format!("_dispatchMessage({})", json_stringify(&msg));
                    quickjs_exec(&mut ctx, &dispatch, &event_sender, &terminate_flag, true);
                    quickjs_drain(&mut ctx, &event_sender);
                    close_requested = quickjs_drain_timer_ops(&mut ctx, &mut timers);
                    quickjs_drain_canvas_ops(&mut ctx, &canvas_drain);
                });
                if close_requested {
                    return; // wrapper 发 Closed
                }
            }
            WorkerCommand::Terminate => break,
        }
    }
}

/// 执行脚本，page-supplied 代码异常（eval Err）时发 `WorkerEvent::Error`，worker 继续运行。
/// terminate 中断（interrupt handler 抛 uncatchable 异常）不是页面错误，不报。
fn quickjs_exec(
    ctx: &mut rquickjs::Ctx,
    code: &str,
    sender: &Sender<WorkerEvent>,
    terminate_flag: &AtomicBool,
    report: bool,
) {
    if let Err(_e) = ctx.eval::<rquickjs::Value, _>(code.to_string()) {
        if report && !terminate_flag.load(Ordering::Acquire) {
            let _ = sender.send(WorkerEvent::Error(quickjs_exception_message(ctx)));
        } else {
            // 异常须消费（ctx.catch 取走 pending exception），不跨条目滞留。
            let _ = ctx.catch();
        }
    }
}

/// 提取当前 pending 异常的消息文本：rquickjs `Error::Exception` 的 Display 只有
/// "Exception generated by QuickJS"（不含 message）——用 `ctx.catch()` 取真实
/// error.name/message（同 quickjs_runtime 的 2026-08-08 修复口径）。消费该异常。
fn quickjs_exception_message(ctx: &rquickjs::Ctx) -> String {
    let caught = ctx.catch();
    if let Some(text) = caught.as_string() {
        return text.to_string().unwrap_or_else(|_| "throw (non-string)".to_string());
    }
    if let Some(object) = caught.as_object() {
        let name = object
            .get::<_, Option<String>>("name")
            .ok()
            .flatten()
            .unwrap_or_default();
        let message = object
            .get::<_, Option<String>>("message")
            .ok()
            .flatten()
            .unwrap_or_default();
        if !name.is_empty() || !message.is_empty() {
            return format!("{name}: {message}");
        }
    }
    "Exception generated by QuickJS".to_string()
}

/// 排空 bootstrap 定时器操作队列 `_workerTimerOps`（arm/cancel/close，保持提交序），
/// 填充 Rust 定时器轮。元素以 `String(op)` 序列化（`"arm,1,50,50"` 形态）读回。
/// 返回是否遇到 `close()`（bootstrap close 语义：worker 退出，调用方发 Closed）。
fn quickjs_drain_timer_ops(ctx: &mut rquickjs::Ctx, timers: &mut HashMap<u64, WorkerTimer>) -> bool {
    let mut close_requested = false;
    let count: i32 = ctx.eval::<i32, _>("_workerTimerOps.length".to_string()).unwrap_or(0);
    for i in 0..count {
        let raw: String = ctx
            .eval::<String, _>(format!("String(_workerTimerOps[{i}])"))
            .unwrap_or_default();
        let parts: Vec<&str> = raw.split(',').collect();
        match parts.as_slice() {
            ["arm", id, delay, period, level] => {
                if let Ok(id) = id.parse::<u64>() {
                    // 延迟/周期按 f64 截断（页面常写 `1000 / 60` 这类分数周期）。
                    let delay = delay.parse::<f64>().unwrap_or(0.0).max(0.0) as u64;
                    let period = period.parse::<f64>().unwrap_or(0.0).max(0.0) as u64;
                    let nesting = level.parse::<u64>().unwrap_or(0);
                    let due = std::time::Instant::now() + worker_timer_delay(delay);
                    let period_ms = (period > 0).then_some(period);
                    timers.insert(
                        id,
                        WorkerTimer {
                            due,
                            period_ms,
                            nesting,
                        },
                    );
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
    let _ = ctx.eval::<rquickjs::Value, _>("_workerTimerOps = []".to_string());
    close_requested
}

/// 排空 bootstrap canvas 操作队列 `_workerCanvasOps`（S1b canvas 桥，同 worker.rs
/// v8 侧语义）：整体 `JSON.stringify` 读回交宿主排空回调；无回调时读回即丢弃。
fn quickjs_drain_canvas_ops(ctx: &mut rquickjs::Ctx, drain: &Option<crate::WorkerCanvasDrainHandle>) {
    let json: String = ctx
        .eval::<String, _>("JSON.stringify(_workerCanvasOps)".to_string())
        .unwrap_or_default();
    if json.len() > 2
        && let Some(d) = drain.as_ref()
        && let Ok(mut guard) = d.lock()
    {
        guard(&json);
    }
    let _ = ctx.eval::<rquickjs::Value, _>("_workerCanvasOps = []".to_string());
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

    // ── S1a：QuickJS worker 定时器（对称 V8 worker 测试） ──

    #[test]
    fn test_quickjs_worker_set_timeout_fires() {
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
    fn test_quickjs_worker_set_interval_repeats_until_cleared() {
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
        std::thread::sleep(Duration::from_millis(250));
        while let Some(WorkerEvent::Message(_)) = worker.try_recv() {}
        assert!(worker.try_recv().is_none(), "clearInterval 后定时器仍触发");
        worker.terminate();
    }

    #[test]
    fn test_quickjs_worker_timer_callback_error_keeps_interval() {
        let mut worker = WorkerRuntime::new(
            "var n = 0; setInterval(function() { n++; if (n === 1) { throw new Error('tick-boom'); } postMessage('tick' + n); }, 50);",
            SandboxConfig::default(),
        )
        .unwrap();
        let event = worker.recv_timeout(Duration::from_secs(5)).unwrap();
        match event {
            WorkerEvent::Error(msg) => assert!(msg.contains("tick-boom"), "Error 消息: {msg}"),
            other => panic!("Expected Error, got: {other:?}"),
        }
        let event = worker.recv_timeout(Duration::from_secs(5)).unwrap();
        match event {
            WorkerEvent::Message(msg) => assert_eq!(msg, "tick2"),
            other => panic!("Expected Message after error, got: {other:?}"),
        }
        worker.terminate();
    }

    #[test]
    fn test_quickjs_worker_close_emits_closed() {
        let mut worker = WorkerRuntime::new("var x = 1;", SandboxConfig::default()).unwrap();
        std::thread::sleep(Duration::from_millis(200));
        worker.execute_script("close();").unwrap();
        let event = worker.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(matches!(event, WorkerEvent::Closed), "Expected Closed, got: {event:?}");
    }

    #[test]
    fn test_quickjs_worker_script_error_emits_error_and_stays_alive() {
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
        worker.post_message("probe").unwrap();
        let event = worker.recv_timeout(Duration::from_secs(5)).unwrap();
        match event {
            WorkerEvent::Message(msg) => assert_eq!(msg, "echo:probe"),
            other => panic!("Expected Message after error, got: {other:?}"),
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

    // ── S1b：worker canvas 排空（对称 V8 worker 测试——bootstrap `_workerCanvasOps`
    // → Rust drain 回调，QuickJS 命令循环间隙排空）。

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
    fn test_quickjs_worker_canvas_ops_drain_constructor_path() {
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
        // 打断排空序列，drain 丢批）。
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
    fn test_quickjs_worker_canvas_ops_drain_transfer_revive_binds_surface_key() {
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
    fn test_quickjs_worker_nested_timer_chain_clamped_and_completes_dc1() {
        // DC-1 端到端（QuickJS 真线程，对称 V8 worker 测试）：10 代零延迟嵌套链在钳制下
        // 完成，第 7 代起每代 ≥4ms（墙钟下限断言——修前每代 1ms floor，10 代 ~10ms）。
        let mut worker = WorkerRuntime::new(
            "globalThis.__gens = 0;\
             globalThis.__t0 = Date.now();\
             globalThis.__step = function() {\
               globalThis.__gens++;\
               if (globalThis.__gens < 10) { setTimeout(globalThis.__step, 0); }\
               else { postMessage('gens:' + __gens + ',ms:' + (Date.now() - __t0)); } };\
             setTimeout(globalThis.__step, 0);",
            SandboxConfig::default(),
        )
        .unwrap();
        let event = worker.recv_timeout(Duration::from_secs(5)).unwrap();
        match event {
            WorkerEvent::Message(msg) => {
                assert!(msg.starts_with("gens:10"), "钳制下 10 代链须完成: {msg}");
                let ms: u64 = msg.split("ms:").nth(1).unwrap_or("0").parse().unwrap_or(0);
                assert!(ms >= 14, "第 7 代起 ≥4ms 钳制生效（修前 ~10×1ms）: {msg}");
            }
            other => panic!("Expected Message, got: {other:?}"),
        }
        worker.terminate();
    }
}
