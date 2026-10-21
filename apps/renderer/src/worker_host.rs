//! M3-S1a 页面真线程 worker 宿主（goal compositor-animation-continuity，RFC
//! `docs/goal/compositor-animation-continuity/m3-slice-worker-canvas.md` §2.1）：
//! renderer 侧 [`WorkerRuntime`] 集中持有 + 事件泵。
//!
//! 结构与 `service_worker_host.rs` 同构：专用宿主线程持 `HashMap<u64, WorkerRuntime>`；
//! 命令（spawn/postMessage/terminate）经进程级桥槽（engine `worker_bridge`，由
//! `ZW_THREADED_WORKERS` flag 门控注册的 `__zw_worker_host_op` 回调）自 js_worker
//! 脚本线程直接进入本线程（不经主循环——脚本线程同步等 spawn 回执，learning #24
//! 同款死锁规避）；事件泵 8ms 轮询排空各 worker 事件，经 [`PrioritySubmitter`] 以
//! fire-and-forget 优先脚本派发回页面（`__zw_worker_host_dispatch(id, type, payload)`
//! ——shim 侧 handler 派发）。主线程阻塞期事件在 js_worker 队列排队 = worker 事件
//! 任务粒度投递的固有语义（与同 realm shim 的 setTimeout(0) task 派发一致）。
//!
//! S1a 范围：spawn/postMessage/terminate/事件派发/Closed 收尾；导航清理（全表
//! terminate）归 S1d 生命周期批（master.md 批次切分）。

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use zero_script_sandbox::{SandboxConfig, WorkerEvent, WorkerRuntime};

use crate::js_worker::PrioritySubmitter;

/// 命令通道闲置时的轮询间隔（仅影响 worker 事件派发延迟；service_worker_host 同值）。
const COMMAND_POLL_INTERVAL: Duration = Duration::from_millis(8);

/// spawn 同步回执等待上限（js_worker 脚本线程阻塞在 `__zw_worker_host_op('spawn')`
/// 内——WorkerRuntime::new 含 isolate 初始化毫秒级返回；兜底防宿主线程死亡悬挂
/// 脚本线程，超时 shim 回落 same-realm worker）。
const SPAWN_REPLY_TIMEOUT: Duration = Duration::from_secs(5);

/// 宿主线程命令（js_worker 脚本线程经桥槽闭包投递）。
enum HostCommand {
    /// 创建 worker（`script` = shim 供给的 init 脚本：prologue + worker body）。
    /// reply 回十进制 id 或 `"err:…"`（shim 据此分流/回落 same-realm）。
    Spawn {
        script: String,
        reply: Sender<Result<u64, String>>,
    },
    /// 页面 → worker 消息（JSON 序列化载荷，shim `Worker.prototype.postMessage`
    /// host 分支产出；worker 侧 prologue `_dispatchMessage` JSON.parse 还原）。
    PostMessage { id: u64, message: String },
    /// 页面 `worker.terminate()`。
    Terminate { id: u64 },
}

/// 装填 worker 宿主：装桥槽（engine `set_worker_host_op`）+ 启动宿主线程。
/// `ZW_THREADED_WORKERS` flag 开时由 renderer 构造器调用一次（query_overrides
/// provider 注册同点）；flag 关不装填——`__zw_worker_host_op` 回调不注册，shim
/// `typeof` 守卫回落 same-realm worker（零行为）。
pub fn install(submitter: PrioritySubmitter) {
    let (command_tx, command_rx) = mpsc::channel::<HostCommand>();
    let op_tx = command_tx.clone();
    zero_engine::set_worker_host_op(Some(Arc::new(move |args: &[String]| -> String {
        host_op_dispatch(&op_tx, args)
    })));
    thread::Builder::new()
        .name("worker-runtime-host".into())
        .spawn(move || HostThread::run(command_rx, submitter))
        .expect("spawn worker runtime host thread");
}

/// 桥槽闭包体（js_worker 脚本线程内执行）：spawn 同步等回执，post/terminate 投递
/// 即返回。op 面：`spawn(<init 脚本>)` → 十进制 id | `"err:…"`；`post(<id>,<载荷>)`；
/// `terminate(<id>)`。
fn host_op_dispatch(command_tx: &Sender<HostCommand>, args: &[String]) -> String {
    match args.first().map(String::as_str) {
        Some("spawn") => {
            let script = args.get(1).cloned().unwrap_or_default();
            let (reply_tx, reply_rx) = mpsc::channel();
            if command_tx
                .send(HostCommand::Spawn {
                    script,
                    reply: reply_tx,
                })
                .is_err()
            {
                return "err:host-gone".into();
            }
            match reply_rx.recv_timeout(SPAWN_REPLY_TIMEOUT) {
                Ok(Ok(id)) => id.to_string(),
                Ok(Err(reason)) => format!("err:{reason}"),
                Err(_) => "err:spawn-timeout".into(),
            }
        }
        Some("post") => {
            let Some(id) = args.get(1).and_then(|arg| arg.parse::<u64>().ok()) else {
                return "err:bad-id".into();
            };
            let message = args.get(2).cloned().unwrap_or_default();
            command_tx
                .send(HostCommand::PostMessage { id, message })
                .map(|_| "ok".to_string())
                .unwrap_or_else(|_| "err:host-gone".into())
        }
        Some("terminate") => {
            let Some(id) = args.get(1).and_then(|arg| arg.parse::<u64>().ok()) else {
                return "err:bad-id".into();
            };
            command_tx
                .send(HostCommand::Terminate { id })
                .map(|_| "ok".to_string())
                .unwrap_or_else(|_| "err:host-gone".into())
        }
        _ => "err:bad-op".into(),
    }
}

/// 宿主线程状态：worker 表 + id 分配源 + 事件派发提交器。
struct HostThread {
    runtimes: HashMap<u64, WorkerRuntime>,
    next_id: u64,
    submitter: PrioritySubmitter,
}

impl HostThread {
    fn run(receiver: Receiver<HostCommand>, submitter: PrioritySubmitter) {
        let mut host = Self {
            runtimes: HashMap::new(),
            next_id: 1,
            submitter,
        };
        loop {
            while let Ok(cmd) = receiver.try_recv() {
                host.handle_command(cmd);
            }
            host.pump_events();
            thread::sleep(COMMAND_POLL_INTERVAL);
        }
    }

    fn handle_command(&mut self, cmd: HostCommand) {
        match cmd {
            HostCommand::Spawn { script, reply } => {
                // 页面 worker 看门狗关闭（SandboxConfig 默认 timeout_ms = 0）：浏览器
                // 语义——worker 常驻至 terminate()/close()/导航清理（S1d），不在
                // config 层设脚本超时；异常 worker 由页面 terminate() 或 close() 收口。
                let result = WorkerRuntime::new(&script, SandboxConfig::default()).map(|runtime| {
                    let id = self.next_id;
                    self.next_id += 1;
                    self.runtimes.insert(id, runtime);
                    id
                });
                let _ = reply.send(result.map_err(|e| e.to_string()));
            }
            HostCommand::PostMessage { id, message } => {
                if let Some(runtime) = self.runtimes.get_mut(&id)
                    && let Err(e) = runtime.post_message(&message)
                {
                    tracing::warn!(id, error = %e, "worker post_message failed");
                }
            }
            HostCommand::Terminate { id } => {
                if let Some(mut runtime) = self.runtimes.remove(&id) {
                    runtime.terminate();
                }
            }
        }
    }

    /// 事件泵：排空各 worker 事件转页面派发。Closed 摘表收尾（worker 线程已退出，
    /// Drop 的 terminate() 对已退出线程即时 join）。
    fn pump_events(&mut self) {
        let ids: Vec<u64> = self.runtimes.keys().copied().collect();
        for id in ids {
            let mut events = Vec::new();
            let mut closed = false;
            if let Some(runtime) = self.runtimes.get_mut(&id) {
                while let Some(event) = runtime.try_recv() {
                    if matches!(event, WorkerEvent::Closed) {
                        closed = true;
                        break;
                    }
                    events.push(event);
                }
            }
            if closed {
                self.runtimes.remove(&id);
                self.dispatch(id, "closed", "");
                continue;
            }
            for event in events {
                match event {
                    WorkerEvent::Message(payload) => self.dispatch(id, "message", &payload),
                    WorkerEvent::Error(message) => self.dispatch(id, "error", &message),
                    WorkerEvent::Closed => unreachable!("closed handled above"),
                }
            }
        }
    }

    /// 事件派发回页面：`__zw_worker_host_dispatch(id, type, payload)` 优先脚本
    /// fire-and-forget。载荷以 JSON 表达式内嵌（JSON 是 JS 字面量子集）。
    fn dispatch(&self, id: u64, event_type: &str, payload: &str) {
        self.submitter.submit(&dispatch_script(id, event_type, payload));
    }
}

/// 构造事件派发脚本。Message 载荷为 bootstrap `JSON.stringify` 产物（合法 JSON），
/// 原样内嵌（`__zwStr` 标记保留给 shim 解包）；兜底把非 JSON 文本包成 JSON 字符串
/// 字面量（shim 对无 `__zwStr` 标记的字符串原样作 data 派发）。
fn dispatch_script(id: u64, event_type: &str, payload: &str) -> String {
    let payload_expr = serde_json::from_str::<serde_json::Value>(payload)
        .map(|value| value.to_string())
        .unwrap_or_else(|_| serde_json::to_string(payload).unwrap_or_else(|_| "null".into()));
    format!(
        "typeof __zw_worker_host_dispatch==='function'&&__zw_worker_host_dispatch({id},{event_type:?},{payload_expr});"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 桥 op 全链路（真 WorkerRuntime）：spawn 同步回 id → post 消息回显 → 事件泵
    /// 产出 message 派发脚本（经 PrioritySubmitter 落进真 js_worker，执行计数可观测）。
    #[test]
    fn host_op_round_trip_spawns_threaded_worker_and_dispatches_events() {
        let js_worker = crate::js_worker::RendererJsWorker::spawn(1);
        let executions_before = js_worker.execution_count_for_test();
        let (command_tx, command_rx) = mpsc::channel::<HostCommand>();
        let submitter = js_worker.priority_submitter();
        thread::Builder::new()
            .name("worker-host-test".into())
            .spawn(move || HostThread::run(command_rx, submitter))
            .unwrap();

        let id = host_op_dispatch(
            &command_tx,
            &[
                "spawn".into(),
                "onmessage = function(e) { postMessage({ echo: e.data }); };".into(),
            ],
        )
        .parse::<u64>()
        .expect("spawn 应回十进制 id");
        assert_eq!(
            "ok",
            host_op_dispatch(&command_tx, &["post".into(), id.to_string(), "\"ping\"".into()])
        );

        // 事件泵轮询（8ms）+ js_worker 执行，给足裕量后断言派发脚本已执行。
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while js_worker.execution_count_for_test() == executions_before && std::time::Instant::now() < deadline {
            thread::sleep(Duration::from_millis(20));
        }
        assert!(
            js_worker.execution_count_for_test() > executions_before,
            "worker 事件派发脚本未进 js_worker 执行"
        );
        assert_eq!(
            "ok",
            host_op_dispatch(&command_tx, &["terminate".into(), id.to_string()])
        );
    }

    /// 无效 op / 坏 id 的错误面（桥返回 err: 前缀，shim 回落 same-realm）。
    #[test]
    fn host_op_rejects_bad_ops_and_ids() {
        let (command_tx, _command_rx) = mpsc::channel::<HostCommand>();
        assert!(host_op_dispatch(&command_tx, &["frobnicate".into()]).starts_with("err:"));
        assert!(
            host_op_dispatch(&command_tx, &["post".into(), "not-a-number".into(), "null".into()]).starts_with("err:")
        );
    }

    /// spawn 在宿主线程缺席（通道关闭）时同步回 err:——shim 回落 same-realm 的触发面。
    #[test]
    fn host_op_reports_host_gone_when_receiver_dropped() {
        let (command_tx, command_rx) = mpsc::channel::<HostCommand>();
        drop(command_rx);
        let result = host_op_dispatch(&command_tx, &["spawn".into(), "var x = 1;".into()]);
        assert!(result.starts_with("err:"), "got: {result}");
    }

    /// 派发脚本形态：`typeof` 守卫 + JSON 载荷内嵌（`__zwStr` 标记原样保留给 shim 解包）。
    #[test]
    fn dispatch_script_embeds_json_payload_behind_typeof_guard() {
        let script = dispatch_script(3, "message", "{\"__zwStr\":1,\"v\":\"hi\"}");
        assert!(script.starts_with("typeof __zw_worker_host_dispatch==='function'&&"));
        assert!(script.contains("__zw_worker_host_dispatch(3,\"message\","));
        assert!(script.contains("\"__zwStr\":1"));
        // 非 JSON 文本兜底为字符串字面量（worker 侧 prologue 缺位的防御面）。
        let raw = dispatch_script(4, "error", "plain text");
        assert!(raw.contains("\"plain text\""));
    }
}
