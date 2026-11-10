//! test262 通道真线程 worker 宿主（`testharness-test262` 命令专用）。
//!
//! `$262.agent` 面依赖真线程 Worker（provider broadcast/waitUntil 忙等自旋在同
//! realm 单线程必死锁）+ SAB 跨 isolate 管线（agent i32a 协调面经
//! `{"__zwSabRef"}` token 过桥）。结构与 `apps/renderer/src/worker_host.rs` 同构
//!（专用宿主线程持 `HashMap<u64, WorkerRuntime>`，命令/事件通道），裁剪 renderer
//! 专属面（canvas patch 呈现、导航世代、PrioritySubmitter——runner 单页单案顺序
//! 执行，事件泵接 testharness 探针循环）。
//!
//! 生命周期：每案 WebView 独立，案间由探针循环入口 `terminate_all()` 全表终结
//!（53k 案无线程/事件累积；上一案 agent 脚本的忙等自旋线程随之回收）。滞留
//! 派发脚本落新页 `_zwHostWorkers[id]` miss 无害（shim typeof 守卫）。

use std::collections::{HashMap, VecDeque};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use zero_script_sandbox::{SandboxConfig, WorkerEvent, WorkerRuntime};

/// 宿主命令（与 renderer HostCommand 同形，加全表终结臂）。
enum HostCommand {
    Spawn {
        script: String,
        reply: Sender<Result<u64, String>>,
    },
    PostMessage {
        id: u64,
        message: String,
    },
    Terminate {
        id: u64,
    },
    TerminateAll {
        reply: Sender<()>,
    },
}

/// spawn 同步回执超时（WorkerRuntime::new 含 isolate 初始化毫秒级返回；兜底防
/// 宿主线程死亡悬挂——renderer 同款）。
const SPAWN_REPLY_TIMEOUT: Duration = Duration::from_secs(5);

/// 事件泵轮询间隔（renderer COMMAND_POLL_INTERVAL 同量级；agent 协调面的
/// waitUntil 自旋在页面主线程，宿主泵只影响 worker→事件→派发脚本的延迟）。
const COMMAND_POLL_INTERVAL: Duration = Duration::from_millis(8);

/// 事件派发脚本队列（宿主线程产、探针循环消费）。Mutex<VecDeque> 而非 mpsc：
/// 消费端在 runner 主线程 thread_local 化开销不值，锁粒度毫秒级空闲。
static DISPATCH_QUEUE: std::sync::LazyLock<Mutex<VecDeque<String>>> =
    std::sync::LazyLock::new(|| Mutex::new(VecDeque::new()));

/// 宿主线程状态：worker 表 + id 分配源（renderer HostThread 裁剪版）。
struct HostThread {
    runtimes: HashMap<u64, WorkerRuntime>,
    next_id: u64,
}

impl HostThread {
    fn run(receiver: Receiver<HostCommand>) {
        let mut host = Self {
            runtimes: HashMap::new(),
            next_id: 1,
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
                // worker 看门狗关闭（SandboxConfig 默认 timeout_ms = 0）：agent 脚本
                // 的忙等自旋（waitUntil/broadcast）与 Atomics.wait 长等待是上游语义，
                // 不设脚本级超时；超界由案级 90s 看门狗与 terminate_all 收口。
                let result = WorkerRuntime::new_with_canvas_drain(&script, SandboxConfig::default(), None)
                    .map(|runtime| {
                        let id = self.next_id;
                        self.next_id += 1;
                        self.runtimes.insert(id, runtime);
                        id
                    })
                    .map_err(|e| e.to_string());
                let _ = reply.send(result);
            }
            HostCommand::PostMessage { id, message } => {
                if let Some(runtime) = self.runtimes.get_mut(&id) {
                    runtime.post_message(&message).ok();
                }
            }
            HostCommand::Terminate { id } => {
                if let Some(mut runtime) = self.runtimes.remove(&id) {
                    runtime.terminate();
                }
            }
            HostCommand::TerminateAll { reply } => {
                for (_, mut runtime) in self.runtimes.drain() {
                    runtime.terminate();
                }
                DISPATCH_QUEUE.lock().expect("dispatch queue poisoned").clear();
                let _ = reply.send(());
            }
        }
    }

    /// 事件泵：排空各 worker 事件转派发脚本入队。Closed 摘表收尾（renderer 同款；
    /// runner 页面不消费 closed——案级 terminate_all 已清表，页面 shim 无接收面）。
    fn pump_events(&mut self) {
        let ids: Vec<u64> = self.runtimes.keys().copied().collect();
        for id in ids {
            let mut closed = false;
            if let Some(runtime) = self.runtimes.get_mut(&id) {
                while let Some(event) = runtime.try_recv() {
                    match event {
                        WorkerEvent::Message(payload) => queue_dispatch(id, "message", &payload),
                        WorkerEvent::Error(message) => queue_dispatch(id, "error", &message),
                        WorkerEvent::Closed => {
                            closed = true;
                            break;
                        }
                    }
                }
            }
            if closed {
                self.runtimes.remove(&id);
            } else if self.runtimes.get(&id).is_some_and(WorkerRuntime::thread_finished) {
                // 异常退出探测（renderer 同款）：未见 Closed = worker 线程异常退出，
                // 补 error 派发 + 摘表，不静默悬挂。
                self.runtimes.remove(&id);
                queue_dispatch(id, "error", "worker thread exited unexpectedly");
            }
        }
    }
}

/// 事件派发脚本入队（`__zw_worker_host_dispatch(id, type, payload)` 优先形态，
/// renderer dispatch_script 同款——载荷 JSON 表达式内嵌）。
fn queue_dispatch(id: u64, event_type: &str, payload: &str) {
    let payload_expr = serde_json::from_str::<serde_json::Value>(payload)
        .map(|value| value.to_string())
        .unwrap_or_else(|_| serde_json::to_string(payload).unwrap_or_else(|_| "null".into()));
    let script = format!(
        "typeof __zw_worker_host_dispatch==='function'&&__zw_worker_host_dispatch({id},{event_type:?},{payload_expr});"
    );
    DISPATCH_QUEUE
        .lock()
        .expect("dispatch queue poisoned")
        .push_back(script);
}

/// 装填宿主（`testharness-test262` 命令路径调用一次）：进程级 worker host 桥 +
/// 宿主线程。桥闭包形态与 renderer `worker_host::install` 一致（engine
/// `__zw_worker_host_op` 回调读槽转发）。
pub fn install() {
    let (command_tx, command_rx) = channel::<HostCommand>();
    let op_tx = command_tx.clone();
    zero_engine::set_worker_host_op(Some(Arc::new(move |args: &[String]| -> String {
        host_op_dispatch(&op_tx, args)
    })));
    *HOST_COMMAND_TX.lock().expect("host command tx poisoned") = Some(command_tx);
    thread::Builder::new()
        .name("test262-worker-host".into())
        .spawn(move || HostThread::run(command_rx))
        .expect("spawn test262 worker host thread");
}

/// 桥槽闭包体（engine 回调线程内执行）：op 面 spawn/post/terminate（renderer
/// host_op_dispatch 同款）。
fn host_op_dispatch(command_tx: &Sender<HostCommand>, args: &[String]) -> String {
    match args.first().map(String::as_str) {
        Some("spawn") => {
            let script = args.get(1).cloned().unwrap_or_default();
            let (reply_tx, reply_rx) = channel();
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

/// 探针循环取走当前积压的派发脚本（调用方逐条 `execute_script`；队空返空 vec
/// 零开销——非 test262 通道/无 worker 页面无额外 execute）。
pub fn take_dispatch_scripts() -> Vec<String> {
    let mut queue = DISPATCH_QUEUE.lock().expect("dispatch queue poisoned");
    queue.drain(..).collect()
}

/// 全表终结（每案探针循环入口调用）：同步等宿主线程 terminate+join 全表、清空
/// 积压派发脚本（上一案 agent 自旋线程回收——不带此步则忙等 worker 线程跨案存续）。
/// 宿主未装填（非 test262 命令路径）零操作——探针循环对全通道共享，此处静默。
pub fn terminate_all() {
    let Some(tx) = HOST_COMMAND_TX.lock().expect("host command tx poisoned").clone() else {
        return;
    };
    let (reply_tx, reply_rx) = channel();
    if tx.send(HostCommand::TerminateAll { reply: reply_tx }).is_err() {
        return;
    }
    let _ = reply_rx.recv_timeout(SPAWN_REPLY_TIMEOUT);
}

/// 命令通道句柄（terminate_all 握手用；install 填装）。
static HOST_COMMAND_TX: std::sync::LazyLock<Mutex<Option<Sender<HostCommand>>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));
