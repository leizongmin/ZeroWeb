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
//! S1a 范围：spawn/postMessage/terminate/事件派发/Closed 收尾；S1d 补导航清理
//! （世代共享槽推进 → 全表 terminate + patch 终末 destroy，`teardown_for_navigation`）。

use std::collections::HashMap;
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use zero_engine::RectLookupHandler;
use zero_engine::js_dom_bridge::{CanvasRegistry, canvas_context_full_rgba, canvas_context_op};
use zero_protocol::transport::PipeTransport;
use zero_protocol::{IpcChannel, IpcMessage, IpcMessageKind};
use zero_script_sandbox::{SandboxConfig, WorkerCanvasDrain, WorkerCanvasDrainHandle, WorkerEvent, WorkerRuntime};

use crate::compositor_publish_thread::SharedWriter;
use crate::js_worker::PrioritySubmitter;

/// 命令通道闲置时的轮询间隔（仅影响 worker 事件派发延迟；service_worker_host 同值）。
const COMMAND_POLL_INTERVAL: Duration = Duration::from_millis(8);

/// spawn 同步回执等待上限（js_worker 脚本线程阻塞在 `__zw_worker_host_op('spawn')`
/// 内——WorkerRuntime::new 含 isolate 初始化毫秒级返回；兜底防宿主线程死亡悬挂
/// 脚本线程，超时 shim 回落 same-realm worker）。
const SPAWN_REPLY_TIMEOUT: Duration = Duration::from_secs(5);

/// S1b canvas 桥（RFC §2.2）：worker 线程本地 registry 排空态。
///
/// 每个 worker 一份（闭包内 `Arc` 持有，drain 回调在 worker 命令循环间隙执行）：
/// 像素只存 worker 本地 registry（无 paint 路径——同 realm 幻影注册语义修除，
/// 呈现面属 S1c patch 管线）；`prov` 为 bootstrap 侧临时 ctx id → registry 实际
/// ctx id 映射（`newCtx` 分配时机在排空侧，JS 侧先以自增 id 记 op）。
#[derive(Default)]
struct WorkerCanvasState {
    registry: CanvasRegistry,
    prov: HashMap<u64, u64>,
}

/// 宿主绑定表值：surface 键 → worker 本地 ctx 临时 id（页面 `transferControlToOffscreen`
/// 占位的 `_zwXferKey`；S1c patch 注册按此把呈现面接到元素矩形）。
type CanvasBindings = Arc<Mutex<HashMap<String, u64>>>;

/// M3-S1c patch 呈现面共享模板（install 期构造；per-worker spawn 实例化
/// [`PatchSink`]——绑定簿/消息序号每 worker 独立，writer/世代/矩形查询共享）。
struct PatchFace {
    /// stdout 直写句柄（与 publish 线程/service worker 客户端同 Arc）。
    writer: SharedWriter,
    /// patch 所属 surface（= renderer_id，`FramePublishState` 同源）。
    surface_id: u64,
    /// 当前导航世代共享槽（runtime 导航路径 store；Register lazy 首拍读取）。
    navigation_epoch: Arc<AtomicU64>,
    /// 源元素矩形查询（selector/handle → 文档空间 CSS px；gBCR 同源 handler）。
    rect_handler: RectLookupHandler,
}

impl PatchFace {
    fn sink_for(&self) -> PatchSink {
        PatchSink::new(
            self.writer.clone(),
            self.surface_id,
            Arc::clone(&self.navigation_epoch),
            Arc::clone(&self.rect_handler),
        )
    }
}

/// M3-S1c patch 呈现数据面发送器（RFC §2.3）：worker 线程经 SharedWriter 直写
/// stdout → browser 独立排队中继 → compositor `PatchSurface` 贴片。数据面全程
/// 不触 renderer 主线程（DC-4 (a) 结构性保证——主线程被页面脚本阻塞时 patch 仍
/// 直达）。每 worker 一份；`ZW_WORKER_CANVAS_PATCH` flag 关时不构造（现行语义
/// 逐字节）。
struct PatchSink {
    /// 只写通道（4B len + bincode 载荷帧协议；SharedWriter flush 单次锁持有写
    /// 完整帧——帧原子性契约同 compositor_publish_thread）。
    outbound: Mutex<PipeTransport<io::Empty, SharedWriter>>,
    surface_id: u64,
    navigation_epoch: Arc<AtomicU64>,
    rect_handler: RectLookupHandler,
    /// 绑定簿：`_zwXferKey`（'xfer-N'）→ 注册簿记（drain 回调 flush 与宿主终末
    /// destroy 并发于同 worker 线程序——锁只作跨表一致性）。
    state: Mutex<PatchSinkState>,
}

/// patch 绑定簿。
#[derive(Default)]
struct PatchSinkState {
    next_msg_id: u64,
    patches: HashMap<String, PatchBinding>,
    /// 终末栅栏（S1d）：destroy_all 置位后 flush 不再发送——宿主导航清理与 worker
    /// 线程 drain 并发时，滞后 Update 不得复活已 Destroy 的贴片（新世代帧的
    /// cleanup_stale_patches 不清同世代项，栅栏是唯一确定收口）。
    torn_down: bool,
}

/// 单 patch 簿记。`registered` = compositor 侧已建贴片（首个 Update 时 lazy
/// Register——bind 时序在页面脚本期先于首帧渲染，彼时布局快照未冻结，矩形查询
/// 必空）；`epoch` = Register 时导航世代快照（后续 Update 携带同世代——导航换代
/// 后宿主导航清理发 Destroy，compositor 帧推进侧另有 cleanup_stale_patches 兜底）。
struct PatchBinding {
    patch_id: u64,
    sel: String,
    registered: bool,
    epoch: u64,
}

impl PatchSink {
    fn new(
        writer: SharedWriter,
        surface_id: u64,
        navigation_epoch: Arc<AtomicU64>,
        rect_handler: RectLookupHandler,
    ) -> Self {
        Self {
            outbound: Mutex::new(PipeTransport::new(io::empty(), writer)),
            surface_id,
            navigation_epoch,
            rect_handler,
            state: Mutex::new(PatchSinkState::default()),
        }
    }

    /// bind op 落簿（worker 侧复活路径；仅记簿不发送——Register lazy 到首个 Update）。
    /// patch_id = 键序号（'xfer-N'，页面 realm `__zwXferSeq` 全局自增，同 renderer 内唯一）。
    fn bind(&self, key: &str, sel: &str) {
        let Some(patch_id) = key.strip_prefix("xfer-").and_then(|n| n.parse::<u64>().ok()) else {
            return;
        };
        let Ok(mut st) = self.state.lock() else { return };
        st.patches.insert(
            key.to_string(),
            PatchBinding {
                patch_id,
                sel: sel.to_string(),
                registered: false,
                epoch: 0,
            },
        );
    }

    /// drain 批落定后冲刷 patch Update：touched 键 lazy Register（矩形查询 None
    /// 跳过保持未注册——布局快照未就绪时下批重试）+ Update（worker 本地 registry
    /// 全幅 RGBA 读回，f16 画布 ×255 量化——`canvas_context_full_rgba`）。
    fn flush_updates(
        &self,
        touched_cids: &[u64],
        registry: &CanvasRegistry,
        prov: &HashMap<u64, u64>,
        bindings: &CanvasBindings,
    ) {
        let Ok(mut st) = self.state.lock() else { return };
        if st.torn_down {
            return;
        }
        let Ok(table) = bindings.lock() else { return };
        for (key, &cid) in table.iter() {
            if !touched_cids.contains(&cid) {
                continue;
            }
            let (patch_id, registered, sel) = match st.patches.get(key) {
                Some(b) => (b.patch_id, b.registered, b.sel.clone()),
                None => continue,
            };
            // lazy Register：首拍查源元素矩形（文档空间 CSS px；布局快照 render 后
            // 填充并冻结——见 RectBridge handler 语义）。查询 None 跳过本键（保持
            // 未注册，下批重试；Update 亦不发送——注册前 compositor 侧无贴片）。
            if !registered {
                let Some((x, y, width, height)) = (self.rect_handler)(&sel) else {
                    continue;
                };
                let epoch = self.navigation_epoch.load(Ordering::Relaxed);
                if let Some(b) = st.patches.get_mut(key) {
                    b.registered = true;
                    b.epoch = epoch;
                }
                send_patch_msg(
                    &mut st,
                    &self.outbound,
                    IpcMessageKind::CompositorPatchRegister {
                        surface_id: self.surface_id,
                        navigation_epoch: epoch,
                        patch_id,
                        x,
                        y,
                        width,
                        height,
                    },
                );
            }
            let epoch = st.patches.get(key).map_or(0, |b| b.epoch);
            let Some(real) = prov.get(&cid) else { continue };
            let Some((width, height, rgba)) = canvas_context_full_rgba(registry, *real) else {
                continue;
            };
            send_patch_msg(
                &mut st,
                &self.outbound,
                IpcMessageKind::CompositorPatchUpdate {
                    surface_id: self.surface_id,
                    navigation_epoch: epoch,
                    patch_id,
                    width,
                    height,
                    rgba,
                },
            );
        }
    }

    /// worker 终末（Terminate/Closed/导航清理）：全部已注册 patch 发 Destroy
    /// （compositor 冻结位图贴片摘除）并置终末栅栏（滞后 flush 静默）。未注册
    /// binding 无 compositor 侧状态——跳过。
    fn destroy_all(&self) {
        let pending = {
            let Ok(mut st) = self.state.lock() else { return };
            st.torn_down = true;
            let pending: Vec<(u64, u64)> = st
                .patches
                .values()
                .filter(|b| b.registered)
                .map(|b| (b.patch_id, b.epoch))
                .collect();
            st.patches.clear();
            pending
        };
        let Ok(mut out) = self.outbound.lock() else { return };
        for (patch_id, navigation_epoch) in pending {
            let _ = out.send(IpcMessage {
                id: 0, // fire-and-forget：browser 转发 compositor 不回执，id 不参与匹配
                kind: IpcMessageKind::CompositorPatchDestroy {
                    surface_id: self.surface_id,
                    navigation_epoch,
                    patch_id,
                },
            });
        }
    }
}

/// patch 消息直发（fire-and-forget——browser 独立排队中继，无回执）。写失败静默：
/// 连接断开即整 renderer 退出路径（SharedWriter flush 失败清缓冲语义同 publish 线程）。
fn send_patch_msg(
    st: &mut PatchSinkState,
    outbound: &Mutex<PipeTransport<io::Empty, SharedWriter>>,
    kind: IpcMessageKind,
) {
    let msg = IpcMessage {
        id: st.next_msg_id,
        kind,
    };
    st.next_msg_id += 1;
    if let Ok(mut out) = outbound.lock() {
        let _ = out.send(msg);
    }
}

/// 构造 worker canvas 排空回调（spawn 时每 worker 一份）：
/// 返回 (回调句柄, 共享态——测试/S1c 观测用, 宿主绑定表, S1c patch 发送器)。
/// `patch_face` = None（flag 关/桥未装）时 patch 面零参与——返回 None。
#[allow(clippy::type_complexity)]
fn make_canvas_drain(
    patch_face: Option<&PatchFace>,
) -> (
    WorkerCanvasDrainHandle,
    Arc<Mutex<WorkerCanvasState>>,
    CanvasBindings,
    Option<Arc<PatchSink>>,
) {
    let state = Arc::new(Mutex::new(WorkerCanvasState::default()));
    let bindings: CanvasBindings = Arc::new(Mutex::new(HashMap::new()));
    let drain_state = state.clone();
    let drain_bindings = bindings.clone();
    let patch_sink = patch_face.map(|face| Arc::new(face.sink_for()));
    let drain_patch = patch_sink.clone();
    let drain: WorkerCanvasDrain = Box::new(move |json: &str| {
        // 回调跑在 worker 命令循环线程；锁仅在单批操作求值期持有（页面主线程
        // 不触 worker 本地 registry——S1c patch 数据面亦从 worker 线程读取）。
        if let Ok(mut st) = drain_state.lock() {
            let patch = drain_patch.as_deref();
            let touched = apply_worker_canvas_ops(&mut st, &drain_bindings, patch, json);
            if let Some(sink) = &drain_patch {
                sink.flush_updates(&touched, &st.registry, &st.prov, &drain_bindings);
            }
        }
    });
    (Arc::new(Mutex::new(drain)), state, bindings, patch_sink)
}

/// 求值 bootstrap canvas 操作 JSON（`[[op, …], …]`）进 worker 本地 registry。
/// 条目：`["newCtx", cid, w, h]`（分配 registry ctx 并记 prov 映射）、
/// `["op", cid, opName, args…]`（translate 后走 `canvas_context_op`——engine 纯栅格核
/// 零改动复用）、`["resize", cid, w, h]`、`["bind", cid, key[, sel]]`（宿主绑定表；
/// S1c 起第 4 元携带源选择器，旧 3 元形态兼容——sel 记空串，Register 恒跳过）。
/// 返回本批触碰的临时 cid（S1c patch flush 触发集——`op`/`resize` 落地才算）。
fn apply_worker_canvas_ops(
    state: &mut WorkerCanvasState,
    bindings: &CanvasBindings,
    patch: Option<&PatchSink>,
    json: &str,
) -> Vec<u64> {
    let Ok(ops) = serde_json::from_str::<Vec<serde_json::Value>>(json) else {
        return Vec::new();
    };
    let mut touched: Vec<u64> = Vec::new();
    for op in ops {
        let Some(parts) = op.as_array() else { continue };
        let kind = parts.first().and_then(|v| v.as_str()).unwrap_or("");
        let cid = parts.get(1).and_then(|v| v.as_u64());
        match kind {
            "newCtx" => {
                let (Some(cid), Some(w), Some(h)) = (cid, parts.get(2), parts.get(3)) else {
                    continue;
                };
                let real = canvas_context_op(
                    &mut state.registry,
                    "0",
                    "getContext2d",
                    &[json_arg_str(w), json_arg_str(h)],
                );
                if let Ok(real_id) = real.trim().parse::<u64>() {
                    state.prov.insert(cid, real_id);
                }
            }
            "resize" => {
                let (Some(cid), Some(w), Some(h)) = (cid, parts.get(2), parts.get(3)) else {
                    continue;
                };
                if let Some(real) = state.prov.get(&cid) {
                    canvas_context_op(
                        &mut state.registry,
                        &real.to_string(),
                        "resizeContext",
                        &[json_arg_str(w), json_arg_str(h)],
                    );
                    touched.push(cid);
                }
            }
            "op" => {
                let Some(cid) = cid else { continue };
                let Some(name) = parts.get(2).and_then(|v| v.as_str()) else {
                    continue;
                };
                let args: Vec<String> = parts
                    .get(3)
                    .and_then(|v| v.as_array())
                    .map(|a| a.iter().map(json_arg_str).collect())
                    .unwrap_or_default();
                if let Some(real) = state.prov.get(&cid) {
                    canvas_context_op(&mut state.registry, &real.to_string(), name, &args);
                    touched.push(cid);
                }
            }
            "bind" => {
                let Some(key) = parts.get(2).and_then(|v| v.as_str()) else {
                    continue;
                };
                // S1c：第 4 元源选择器（patch Register lazy 首拍查元素矩形）。
                let sel = parts.get(3).and_then(|v| v.as_str()).unwrap_or("");
                if let (Ok(mut table), Some(cid)) = (bindings.lock(), cid) {
                    table.insert(key.to_string(), cid);
                }
                if let Some(sink) = patch {
                    sink.bind(key, sel);
                }
            }
            _ => {}
        }
    }
    touched
}

/// JSON 值 → `canvas_context_op` 串参（字符串去引号，数字原样——与主线程 shim
/// `__zw_canvas_op` 串参形态一致）。
fn json_arg_str(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        other => other.to_string(),
    }
}

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
///
/// S1c patch 呈现面（RFC §2.3）：`ZW_WORKER_CANVAS_PATCH` flag 开时构造
/// [`PatchFace`]（`writer`/`surface_id`/`navigation_epoch`/`rect_handler` 四注入），
/// per-worker sink 经 SharedWriter 直写 stdout；flag 关 = patch 面零参与（绑定簿
/// 不建、零消息写），现行语义逐字节。嵌套于 `ZW_THREADED_WORKERS` 之下——排空
/// 协议仅真线程 worker 存在。
pub fn install(
    submitter: PrioritySubmitter,
    writer: SharedWriter,
    surface_id: u64,
    navigation_epoch: Arc<AtomicU64>,
    rect_handler: RectLookupHandler,
) {
    let patch_face = zero_runtime_config::enabled_when_true("ZW_WORKER_CANVAS_PATCH").then(|| PatchFace {
        writer,
        surface_id,
        navigation_epoch: Arc::clone(&navigation_epoch),
        rect_handler,
    });
    let (command_tx, command_rx) = mpsc::channel::<HostCommand>();
    let op_tx = command_tx.clone();
    zero_engine::set_worker_host_op(Some(Arc::new(move |args: &[String]| -> String {
        host_op_dispatch(&op_tx, args)
    })));
    thread::Builder::new()
        .name("worker-runtime-host".into())
        .spawn(move || HostThread::run(command_rx, submitter, patch_face, navigation_epoch))
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

/// 宿主线程状态：worker 表 + id 分配源 + 事件派发提交器 + S1b 绑定表（worker id →
/// surface 键 → 本地 ctx 临时 id）。
struct HostThread {
    runtimes: HashMap<u64, WorkerRuntime>,
    next_id: u64,
    submitter: PrioritySubmitter,
    canvas_bindings: HashMap<u64, CanvasBindings>,
    /// M3-S1c：patch 呈现面共享模板（flag 关 = None——全链零参与）。
    patch_face: Option<PatchFace>,
    /// 每 worker patch 发送器（spawn 时实例化；Terminate/Closed 终末 destroy）。
    worker_patch_sinks: HashMap<u64, Arc<PatchSink>>,
    /// 导航世代共享槽（S1d：宿主观察世代推进做全表清理；与 PatchFace 同源）。
    navigation_epoch: Arc<AtomicU64>,
    /// 宿主已见世代（推进即触发一次导航清理）。
    last_seen_epoch: u64,
}

impl HostThread {
    fn run(
        receiver: Receiver<HostCommand>,
        submitter: PrioritySubmitter,
        patch_face: Option<PatchFace>,
        navigation_epoch: Arc<AtomicU64>,
    ) {
        let mut host = Self {
            runtimes: HashMap::new(),
            next_id: 1,
            submitter,
            canvas_bindings: HashMap::new(),
            patch_face,
            worker_patch_sinks: HashMap::new(),
            last_seen_epoch: navigation_epoch.load(Ordering::Relaxed),
            navigation_epoch,
        };
        loop {
            host.maybe_teardown_on_epoch_change();
            while let Ok(cmd) = receiver.try_recv() {
                host.handle_command(cmd);
            }
            host.pump_events();
            thread::sleep(COMMAND_POLL_INTERVAL);
        }
    }

    /// S1d 导航清理守卫：世代槽推进（navigate/load_html store 点）即全表终结——
    /// 文档卸载其 workers（HTML spec worker lifetime tied to document），滞留
    /// worker 的 patch 数据面随之终末 destroy（否则旧页 worker 可把新世代 patch
    /// 注册到新文档呈现面）。返回是否触发了清理（测试观察面）。
    fn maybe_teardown_on_epoch_change(&mut self) -> bool {
        let epoch = self.navigation_epoch.load(Ordering::Relaxed);
        if epoch == self.last_seen_epoch {
            return false;
        }
        self.last_seen_epoch = epoch;
        self.teardown_for_navigation();
        true
    }

    /// 导航换代全表清理：terminate 全部 worker、摘绑定表与 patch 发送器（已注册
    /// patch 发 Destroy）。不派发 `closed` 事件——所属文档正在卸载，页面侧无接收面。
    fn teardown_for_navigation(&mut self) {
        let runtimes = self.runtimes.len();
        let sinks = self.worker_patch_sinks.len();
        for (_, mut runtime) in self.runtimes.drain() {
            runtime.terminate();
        }
        self.canvas_bindings.clear();
        for (_, sink) in self.worker_patch_sinks.drain() {
            sink.destroy_all();
        }
        tracing::info!(workers = runtimes, patch_sinks = sinks, "导航换代清理 worker 宿主全表");
    }

    fn handle_command(&mut self, cmd: HostCommand) {
        match cmd {
            HostCommand::Spawn { script, reply } => {
                // 页面 worker 看门狗关闭（SandboxConfig 默认 timeout_ms = 0）：浏览器
                // 语义——worker 常驻至 terminate()/close()/导航清理（S1d），不在
                // config 层设脚本超时；异常 worker 由页面 terminate() 或 close() 收口。
                // S1b：每 worker 一份 canvas 排空回调（worker 线程本地 registry）+
                // 宿主绑定表登记（terminate/Closed 随摘）。
                let (canvas_drain, _state, bindings, patch_sink) = make_canvas_drain(self.patch_face.as_ref());
                let result =
                    WorkerRuntime::new_with_canvas_drain(&script, SandboxConfig::default(), Some(canvas_drain)).map(
                        |runtime| {
                            let id = self.next_id;
                            self.next_id += 1;
                            self.runtimes.insert(id, runtime);
                            self.canvas_bindings.insert(id, bindings);
                            if let Some(sink) = patch_sink {
                                self.worker_patch_sinks.insert(id, sink);
                            }
                            id
                        },
                    );
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
                self.canvas_bindings.remove(&id);
                // M3-S1c：worker 终末——已注册 patch 发 Destroy（compositor 贴片摘除；
                // HTML spec canvas 位图随宿主断连定格语义由 compositor detached 臂承接）。
                if let Some(sink) = self.worker_patch_sinks.remove(&id) {
                    sink.destroy_all();
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
                self.canvas_bindings.remove(&id);
                // M3-S1c：worker 退出终末 Destroy（Terminate 臂同款）。
                if let Some(sink) = self.worker_patch_sinks.remove(&id) {
                    sink.destroy_all();
                }
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
            .spawn(move || HostThread::run(command_rx, submitter, None, Arc::new(AtomicU64::new(0))))
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

    // ── S1b canvas 桥：排空协议求值（bootstrap `_workerCanvasOps` JSON → worker 本地
    // registry + 宿主绑定表，经 engine `canvas_context_op` 纯栅格核零改动复用）。

    /// newCtx 分配真 ctx（prov 映射）→ op 落 worker 本地 registry（像素读回验证）→
    /// bind 记宿主绑定表；读回用 `canvas_context_op` getImageData（wire `w:h;r,g,b,a`）。
    #[test]
    fn apply_worker_canvas_ops_allocates_context_and_binds_surface_key() {
        let state = Arc::new(Mutex::new(WorkerCanvasState::default()));
        let bindings: CanvasBindings = Arc::new(Mutex::new(HashMap::new()));
        apply_worker_canvas_ops(
            &mut state.lock().unwrap(),
            &bindings,
            None,
            r##"[["newCtx",1,64,32],["op",1,"setFillStyle",["#ff0000"]],["op",1,"fillRect",[0,0,64,32]],["bind",1,"xfer-1"]]"##,
        );
        let mut st = state.lock().unwrap();
        let real = st.prov.get(&1).copied().expect("newCtx 应建 prov 映射");
        let wire = canvas_context_op(
            &mut st.registry,
            &real.to_string(),
            "getImageData",
            &["0".into(), "0".into(), "1".into(), "1".into()],
        );
        assert_eq!(wire, "1:1;255,0,0,255", "fillRect 像素应落 worker 本地 registry");
        drop(st);
        assert_eq!(
            bindings.lock().unwrap().get("xfer-1"),
            Some(&1),
            "bind 应记宿主绑定表（surface 键 → 复活临时 cid）"
        );
    }

    // ── S1c patch 呈现数据面：lazy Register / Update 冲刷 / 终末 Destroy
    //（SharedWriter 捕获缓冲解帧验证 wire 形态）。

    /// 捕获式 writer（Arc 共享缓冲——解帧断言 patch 消息序列）。
    #[derive(Clone, Default)]
    struct CaptureBuf(Arc<Mutex<Vec<u8>>>);

    impl std::io::Write for CaptureBuf {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    /// 解帧捕获缓冲 → IpcMessage 序列并清空（4B len 前缀 + bincode 载荷，transport
    /// 同款；取走语义——分段断言互不串扰）。
    fn parse_frames(buf: &CaptureBuf) -> Vec<IpcMessage> {
        let bytes = std::mem::take(&mut *buf.0.lock().unwrap());
        let mut out = Vec::new();
        let mut pos = 0usize;
        while pos + 4 <= bytes.len() {
            let n = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
            pos += 4;
            out.push(zero_protocol::serialize::deserialize(&bytes[pos..pos + n]).unwrap());
            pos += n;
        }
        out
    }

    /// 矩形槽类型（测试用——可变矩形应答源）。
    type RectCell = Arc<Mutex<Option<(f32, f32, f32, f32)>>>;

    fn test_sink(rect: RectCell, epoch: u64) -> (PatchSink, CaptureBuf) {
        let buf = CaptureBuf::default();
        let writer = SharedWriter::new(Box::new(buf.clone())).0;
        let handler: RectLookupHandler = Arc::new(move |_identity: &str| *rect.lock().unwrap());
        (PatchSink::new(writer, 7, Arc::new(AtomicU64::new(epoch)), handler), buf)
    }

    /// lazy Register：首拍 Update 触发 Register（矩形 + 当代世代快照）+ Update
    ///（全幅 RGBA）；二拍只 Update（不重发 Register）；注册前（矩形 None）零发送。
    #[test]
    fn patch_sink_lazy_register_then_update_and_defer_on_missing_rect() {
        let rect = Arc::new(Mutex::new(None));
        let (sink, buf) = test_sink(Arc::clone(&rect), 5);
        let state = Arc::new(Mutex::new(WorkerCanvasState::default()));
        let bindings: CanvasBindings = Arc::new(Mutex::new(HashMap::new()));

        // bind（带 sel）+ newCtx + fillRect 同批——touched 命中但矩形未就绪 → 零发送。
        let touched = {
            let mut st = state.lock().unwrap();
            apply_worker_canvas_ops(
                &mut st,
                &bindings,
                Some(&sink),
                r##"[["bind",1,"xfer-1","#cv"],["newCtx",1,2,2],["op",1,"setFillStyle",["#ff0000"]],["op",1,"fillRect",[0,0,2,2]]]"##,
            )
        };
        let st = state.lock().unwrap();
        sink.flush_updates(&touched, &st.registry, &st.prov, &bindings);
        drop(st);
        assert!(parse_frames(&buf).is_empty(), "矩形未就绪：Register/Update 均不发送");

        // 矩形就绪 → Register{10,20,100,50, epoch=5} + Update{2x2 红}。
        *rect.lock().unwrap() = Some((10.0, 20.0, 100.0, 50.0));
        let touched = {
            let mut st = state.lock().unwrap();
            apply_worker_canvas_ops(&mut st, &bindings, Some(&sink), r#"[["op",1,"fillRect",[0,0,2,2]]]"#)
        };
        let st = state.lock().unwrap();
        sink.flush_updates(&touched, &st.registry, &st.prov, &bindings);
        drop(st);
        let frames = parse_frames(&buf);
        assert_eq!(frames.len(), 2, "首拍：Register + Update");
        let IpcMessageKind::CompositorPatchRegister {
            surface_id,
            navigation_epoch,
            patch_id,
            x,
            y,
            width,
            height,
        } = frames[0].kind
        else {
            panic!("首消息应为 Register，got {:?}", frames[0].kind);
        };
        assert_eq!((surface_id, navigation_epoch, patch_id), (7, 5, 1));
        assert_eq!((x, y, width, height), (10.0, 20.0, 100.0, 50.0));
        let IpcMessageKind::CompositorPatchUpdate {
            surface_id,
            navigation_epoch,
            patch_id,
            width,
            height,
            rgba,
        } = &frames[1].kind
        else {
            panic!("次消息应为 Update，got {:?}", frames[1].kind);
        };
        assert_eq!(
            (*surface_id, *navigation_epoch, *patch_id, *width, *height),
            (7, 5, 1, 2, 2)
        );
        assert_eq!(
            rgba.as_slice(),
            &[255u8, 0, 0, 255].repeat(4),
            "fillRect 2x2 红全幅读回"
        );

        // 二拍：只 Update（registered 已置位，不重发 Register）。
        let touched = {
            let mut st = state.lock().unwrap();
            apply_worker_canvas_ops(&mut st, &bindings, Some(&sink), r#"[["op",1,"fillRect",[0,0,2,2]]]"#)
        };
        let st = state.lock().unwrap();
        sink.flush_updates(&touched, &st.registry, &st.prov, &bindings);
        drop(st);
        let frames = parse_frames(&buf);
        assert_eq!(frames.len(), 1, "二拍只 Update");
        assert!(matches!(frames[0].kind, IpcMessageKind::CompositorPatchUpdate { .. }));
    }

    /// 终末 Destroy：已注册 patch 发 Destroy（Register 世代快照）；未注册（矩形
    /// 未就绪即终末，独立 sink 实例）零发送；destroy 后簿清空——再 destroy 零发送。
    #[test]
    fn patch_sink_destroy_sends_only_registered_then_clears() {
        let state = Arc::new(Mutex::new(WorkerCanvasState::default()));
        let bindings: CanvasBindings = Arc::new(Mutex::new(HashMap::new()));
        let script = r##"[["bind",1,"xfer-3","#cv"],["newCtx",1,1,1],["op",1,"setFillStyle",["#ff0000"]],["op",1,"fillRect",[0,0,1,1]]]"##;

        // 未注册即终末（矩形未就绪）：零 Destroy（无 compositor 侧状态）。
        let rect_none = Arc::new(Mutex::new(None));
        let (sink_none, buf_none) = test_sink(rect_none, 9);
        let touched = {
            let mut st = state.lock().unwrap();
            apply_worker_canvas_ops(&mut st, &bindings, Some(&sink_none), script)
        };
        let st = state.lock().unwrap();
        sink_none.flush_updates(&touched, &st.registry, &st.prov, &bindings);
        drop(st);
        sink_none.destroy_all();
        assert!(parse_frames(&buf_none).is_empty(), "未注册 patch 无 compositor 侧状态");

        // 注册后再终末：Destroy 携带 Register 世代快照（epoch=9）。
        let rect = Arc::new(Mutex::new(Some((0.0, 0.0, 30.0, 30.0))));
        let (sink, buf) = test_sink(rect, 9);
        let touched = {
            let mut st = state.lock().unwrap();
            apply_worker_canvas_ops(&mut st, &bindings, Some(&sink), script)
        };
        let st = state.lock().unwrap();
        sink.flush_updates(&touched, &st.registry, &st.prov, &bindings);
        drop(st);
        let frames = parse_frames(&buf);
        assert_eq!(frames.len(), 2, "注册拍：Register + Update");
        sink.destroy_all();
        let frames = parse_frames(&buf);
        assert_eq!(frames.len(), 1);
        let IpcMessageKind::CompositorPatchDestroy {
            surface_id,
            navigation_epoch,
            patch_id,
        } = frames[0].kind
        else {
            panic!("应为 Destroy，got {:?}", frames[0].kind);
        };
        assert_eq!((surface_id, navigation_epoch, patch_id), (7, 9, 3));
        // 簿已清空：再 destroy 零发送。
        sink.destroy_all();
        assert!(parse_frames(&buf).is_empty());
    }

    // ── S1d 生命周期：导航换代全表清理（epoch 守卫 + 终末 Destroy + 滞后 flush 栅栏）。

    /// 世代槽推进触发一次导航清理、同代不触发；清理清空全表并对已注册 patch 发
    /// Destroy（携 Register 世代快照）；清理后滞后 flush 静默（torn_down 栅栏——
    /// 宿主清理与 worker 线程 drain 并发时滞后 Update 不得复活已 Destroy 贴片）。
    #[test]
    fn teardown_for_navigation_clears_tables_and_fences_late_flush() {
        let js_worker = crate::js_worker::RendererJsWorker::spawn(2);
        let mut host = HostThread {
            runtimes: HashMap::new(),
            next_id: 1,
            submitter: js_worker.priority_submitter(),
            canvas_bindings: HashMap::new(),
            patch_face: None,
            worker_patch_sinks: HashMap::new(),
            last_seen_epoch: 1,
            navigation_epoch: Arc::new(AtomicU64::new(1)),
        };
        assert!(!host.maybe_teardown_on_epoch_change(), "同代不清理");

        // 注册态 sink（bind + 矩形就绪 flush → Register + Update）入宿主表。
        let state = Arc::new(Mutex::new(WorkerCanvasState::default()));
        let bindings: CanvasBindings = Arc::new(Mutex::new(HashMap::new()));
        let rect = Arc::new(Mutex::new(Some((0.0, 0.0, 30.0, 30.0))));
        let (sink, buf) = test_sink(rect, 1);
        let sink = Arc::new(sink);
        let touched = {
            let mut st = state.lock().unwrap();
            apply_worker_canvas_ops(
                &mut st,
                &bindings,
                Some(&sink),
                r##"[["bind",1,"xfer-4","#cv"],["newCtx",1,1,1],["op",1,"setFillStyle",["#ff0000"]],["op",1,"fillRect",[0,0,1,1]]]"##,
            )
        };
        {
            let st = state.lock().unwrap();
            sink.flush_updates(&touched, &st.registry, &st.prov, &bindings);
        }
        assert_eq!(parse_frames(&buf).len(), 2, "注册拍：Register + Update");
        host.worker_patch_sinks.insert(11, Arc::clone(&sink));
        host.canvas_bindings.insert(11, Arc::clone(&bindings));
        host.runtimes.insert(
            11,
            WorkerRuntime::new("onmessage = function () {};", SandboxConfig::default()).expect("test runtime"),
        );

        // 世代推进 → 清理：全表摘除 + Destroy（Register 世代快照 epoch=1）。
        host.navigation_epoch.store(2, Ordering::Relaxed);
        assert!(host.maybe_teardown_on_epoch_change(), "世代推进应触发清理");
        assert!(!host.maybe_teardown_on_epoch_change(), "同代二次检查不重复清理");
        assert!(host.runtimes.is_empty(), "worker 表清空");
        assert!(host.canvas_bindings.is_empty(), "绑定表清空");
        assert!(host.worker_patch_sinks.is_empty(), "patch 发送器表清空");
        let frames = parse_frames(&buf);
        assert_eq!(frames.len(), 1, "清理追加 Destroy（注册拍 2 帧已先排空）");
        let IpcMessageKind::CompositorPatchDestroy {
            surface_id,
            navigation_epoch,
            patch_id,
        } = frames[0].kind
        else {
            panic!("应为 Destroy，got {:?}", frames[0].kind);
        };
        assert_eq!((surface_id, navigation_epoch, patch_id), (7, 1, 4));

        // 滞后 flush（worker 线程与清理并发窗口）：栅栏后零发送。
        {
            let st = state.lock().unwrap();
            sink.flush_updates(&touched, &st.registry, &st.prov, &bindings);
        }
        assert_eq!(parse_frames(&buf).len(), 0, "终末栅栏后 flush 静默");
    }

    /// 防御面：未知临时 cid 的 op / resize 静默丢弃（复活前 op 不落 registry）、
    /// 畸形 JSON 与非数组条目忽略不 panic。
    #[test]
    fn apply_worker_canvas_ops_drops_unknown_cid_and_malformed_json() {
        let state = Arc::new(Mutex::new(WorkerCanvasState::default()));
        let bindings: CanvasBindings = Arc::new(Mutex::new(HashMap::new()));
        apply_worker_canvas_ops(
            &mut state.lock().unwrap(),
            &bindings,
            None,
            r#"[["op",7,"fillRect",[0,0,1,1]],["resize",7,10,10],"not-array",[123],{"x":1}]"#,
        );
        let st = state.lock().unwrap();
        assert!(st.prov.is_empty(), "未知 cid 不应建映射，got: {:?}", st.prov);
    }
}
