//! 渲染进程 JS 线程 — V8 与页面渲染分离。

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use zero_engine::{
    AsyncResolver, DomMutation, ElementFromPointBridge, ElementFromPointCache, FetchBridge, FetchHandler, FetchRequest,
    FetchResponse, HandleSelectorMap, LayoutRectSnapshot, RectBridge, TimerBridge, generate_js_dom_shim,
    make_dom_html_rect_handler, new_element_from_point_cache, new_handle_selector_map, new_layout_rect_snapshot,
    register_dom_callbacks,
};
use zero_net::{FetchPriority, HttpMethod, HttpRequest, ResourceLoader, ResourceRequest};
use zero_script_sandbox::{
    ModuleRegistry, SandboxConfig, build_module_runtime_prelude, compile_dependency_iife, compile_module_script,
    extract_module_import_specifiers,
};

use crate::ipc_service_worker::ServiceWorkerIpcClient;

// P7b：15s 在真实站点长回调上误杀（html5test.co 出分后站点回调单次 execute >15s 被
// 看门狗强停 → pageerror「Execution timeout」+ 回调副作用截断，修复前旅程窗口 7 跑
// 5 命中、全数据集 5/8；Chrome 同页无硬杀）。30s 保留死循环防护（真死循环 tab 仍可
// 恢复）同时覆盖实测长尾。
const TAB_JS_EXEC_TIMEOUT_MS: u64 = 30_000;
const TAB_JS_CHANNEL_TIMEOUT: Duration = Duration::from_millis(TAB_JS_EXEC_TIMEOUT_MS + 5_000);

/// t2-pb1 fix#5：worker 空闲轮询间隔——普通通道空闲时以本间隔醒来检查优先通道
/// （`recv_timeout` 轮询环）。队列流动时优先命令在命令边界即时处理；仅空闲时最多
/// 延迟一个间隔（导航里程碑派发对此不敏感）。20 次/秒的空转开销可忽略。
const WORKER_IDLE_POLL: Duration = Duration::from_millis(50);

/// 看门狗超时错误附加脚本上下文（长度+头部片段）——页面回调超时（P7b 家族）定性
/// 需要脚本身份；`ScriptError` 本体不携带脚本信息（沙箱边界隔离），只能在 worker
/// 侧错误转字符串时附加。非超时错误原样透传（不污染既有错误文本消费方）。
fn annotate_timeout_error(message: &str, script: &str) -> String {
    if !message.contains("Execution timeout") {
        return message.to_string();
    }
    let head: String = script.chars().take(80).collect();
    format!("{message} [script_len={} head={head:?}]", script.len())
}

/// P1a gBCR kill-switch：默认 on；`ZW_REAL_RECT=0` 关闭 RectBridge（`__zw_getBoundingClientRect`
/// 不注册 → shim 回落零 rect = 当前行为，零回归）。snapshot 为空 / identity 未命中同样回落零 rect。
/// P1a Slice 2b：亦用作 observer host-tick 的 kill-switch（gBCR 关 → rect 恒零 → tick 无意义）。
pub(crate) fn real_rect_enabled() -> bool {
    !matches!(std::env::var("ZW_REAL_RECT").as_deref(), Ok("0"))
}

/// P1a 事件循环 slice 1（R2713b）：帧驱动 rAF kill-switch。默认 off——shim rAF 走同步 stub
///（reftest 兼容，零默认行为变更）；`ZW_RAF_FRAME_DRIVEN=1` 开启：shim rAF 注册队列，render 后
/// `tick_observers` 调 `__zw_raf_tick` 派发。详见 p1a-event-loop-raf-slice-design-2026-08-05.md。
pub(crate) fn raf_frame_driven_enabled() -> bool {
    matches!(std::env::var("ZW_RAF_FRAME_DRIVEN").as_deref(), Ok("1"))
}

type ScriptFn = Arc<dyn Fn(&str) -> Result<String, String> + Send + Sync>;
type ModuleFn = Arc<dyn Fn(&str, &str, &[(String, String)]) -> Result<String, String> + Send + Sync>;

enum JsWorkerCommand {
    Execute {
        script: String,
        reply: Sender<Result<String, String>>,
        /// t2-pb1 F1：文档代际序号（提交顺序单调递增）——ResetDocumentState 清队按
        /// `seq > reset_seq` 判定新文档命令，不再按「Reset 处理时刻」误清新文档队列。
        seq: u64,
    },
    ExecuteModule {
        source: String,
        url: String,
        deps: Vec<(String, String)>,
        reply: Sender<Result<String, String>>,
        /// t2-pb1 F1：文档代际序号（同 [`JsWorkerCommand::Execute`]）。
        seq: u64,
    },
    SetDomSnapshot {
        html: String,
        url: String,
        /// t2-pb1 fix#21：应用完成回执（None = fire-and-forget，优先通道快照用）。
        reply: Option<Sender<()>>,
        /// t2-pb1 F1：文档代际序号（同 [`JsWorkerCommand::Execute`]）。
        seq: u64,
    },
    /// P1b S1：跨线程异步回调 resolve（marshal channel）。任意线程经
    /// [`RendererJsWorker::async_resolver`] 投递 (id, result)，JS worker 收到后调
    /// `sandbox.resolve_async_callback`（执行 shim 的 `__zwResolveCallback` resolve Promise）。
    ResolveAsyncCallback {
        id: String,
        result: String,
    },
    /// P1b S3：注入 fetch handler（renderer 在 WebView 初始化后发送；测试用合成 handler）。
    SetFetchHandler {
        handler: FetchHandler,
    },
    /// media-playback M2c 后续：注入播放器注册表（镜像 browser tab_js_worker 同名命令
    /// ——多进程路径的 `__zwVideoBridge` 宿主桥一致性；renderer 主循环在 WebView 初始化
    /// 后发送）。
    SetVideoPlayers {
        registry: std::sync::Arc<std::sync::Mutex<zero_webview::video_registry::VideoPlayerRegistry>>,
        /// M3 切片 2（D4 获点名）：宿主泵时钟（renderer 主循环节拍 store）——桥 play
        /// 的 nowMs=0 翻译为泵时钟现值（registry play 锚与泵 tick 同源；tab_worker
        /// 泵时钟注入同款，扩批 XXV 的原点错位缺陷在 renderer 路径的消除）。
        pump_clock: Option<std::sync::Arc<std::sync::atomic::AtomicU64>>,
    },
    SetWebAudio {
        registry: std::sync::Arc<std::sync::Mutex<zero_webview::webaudio_registry::WebAudioRegistry>>,
    },
    ResetDocumentState {
        reply: Sender<()>,
        /// t2-pb1 F1：本次复位的代际界——提交序 `seq` 更大的命令是新文档工作，清队存活。
        seq: u64,
    },
    /// 视口提示（renderer 真实窗口尺寸）：快照换代后 shim `innerWidth/innerHeight` 缺省
    /// 1280x800 与真实视口不一致——首次 install 后按 hint 校正（幂等 guard，仅在失配时
    /// 调 `__zw_user_resize`）。CDP 面 PW fullPage 尺寸测量（scrollWidth 族）依赖真值。
    SetViewportHint {
        width: u32,
        height: u32,
    },
    DispatchIndexedDbConnectionEvent {
        connection_id: u64,
        old_version: u64,
        new_version: Option<u64>,
        reply: Sender<Result<(), String>>,
    },
    Shutdown,
}

/// S14：fetch 观测记录 `(phase, seq, url, method, status)`——phase 0=request /
/// 1=response / 2=finished|failed；seq 三阶段关联。
pub type FetchObservedRecord = (u8, u64, String, String, u16, u64);

/// 渲染进程 JS worker 句柄。
pub struct RendererJsWorker {
    cmd_tx: Sender<JsWorkerCommand>,
    /// t2-pb1 fix#9：优先通道发送端——宿主发起的自动化求值与 DOM 快照成对经此入队
    /// （`set_dom_snapshot_priority` + `execute_script_direct_priority`），不排在页面
    /// 回调积压之后。
    prio_tx: Sender<JsWorkerCommand>,
    /// t2-pb1 F1：文档代际序号源——文档类命令提交时在此分配单调 seq（宿主侧顺序即
    /// 代际顺序；ResetDocumentState 携带自己的 seq 作清队界）。
    cmd_seq: Arc<AtomicU64>,
    /// S11：page console 输出队列（worker 回调推入，runtime drain）。
    console_logs: Arc<std::sync::Mutex<Vec<(String, String, String)>>>,
    /// R-baidu2/P3：未捕获脚本错误队列（page_scripts 推入，runtime drain 后
    /// 经 IPC `ScriptError` 转发 browser/headless——`Runtime.exceptionThrown` 源）。
    script_errors: Arc<std::sync::Mutex<Vec<zero_protocol::message::ScriptErrorParams>>>,
    /// S16：document.write 落定信号队列（worker 回调推入，runtime drain）。
    doc_write_settled: Arc<std::sync::Mutex<Vec<()>>>,
    /// S11：宿主媒体上下文（matchMedia 求值的用户偏好源）。
    media_ctx: Arc<std::sync::Mutex<zero_css_parser::media_query::MediaContext>>,
    /// S14：renderer fetch 观测队列（worker 观测 handler 推入，runtime drain）。
    fetch_observed: Arc<std::sync::Mutex<Vec<FetchObservedRecord>>>,
    join: Option<JoinHandle<()>>,
    executor: ScriptFn,
    /// t2-pb1 fix#5：优先执行器——页面生命周期派发（`execute_script_direct_priority`）
    /// 专用通道，worker 在命令边界优先于普通 FIFO 队列消费。
    prio_executor: ScriptFn,
    module_executor: ModuleFn,
    mutations: Arc<std::sync::Mutex<Vec<DomMutation>>>,
    /// P1a gBCR：共享 layout-rect snapshot——renderer 主循环 render 后填充，
    /// js_worker 的 RectBridge handler 读取（经 identity→NodeId 解析后查 rect）。
    rect_snapshot: LayoutRectSnapshot,
    /// P1a gBCR path A：持久 handle→唯一选择器映射——生产 apply 路径（`apply_recorded_mutations`）
    /// merge 进此 map，js_worker 的 RectBridge handler 读它解析 handle-identity（`__n{n}`）。
    handle_selector_map: HandleSelectorMap,
    /// P1a elementFromPoint：共享 hit-test 缓存槽——renderer 主循环 render 后 swap 最新
    /// `Arc<HitTestCache>`，js_worker 的 `ElementFromPointBridge` 读它求 `(x,y)` 命中元素。
    element_from_point_cache: ElementFromPointCache,
    /// R2949 FontFace.load() 请求队列——`__zw_load_font` 回调（worker 线程）push，renderer 主循环
    /// drain 后 fetch_get 字节 + load_font/register/set_resolver + async_resolver.resolve 解析 Promise。
    font_loads: Arc<std::sync::Mutex<Vec<zero_engine::FontLoadRequest>>>,
    /// R3058 JS 发起跨文档导航请求队列——`__zw_request_navigate` 回调（worker 线程）push，renderer
    /// 主循环 drain 后 handle_navigate（fetch 新文档 + 重载）。location.href=/assign/replace 跨文档触发。
    navigations: Arc<std::sync::Mutex<Vec<String>>>,
    /// R3254-M7'：页面 JS `focus()`/`blur()` 变更队列——`apply_recorded_mutations` 从 mutations
    /// 分离后 push（主线程），renderer 事件循环在事务边界 drain 同步 retained 焦点状态。
    focus_changes: Arc<std::sync::Mutex<Vec<Option<String>>>>,
    /// 已由 JS worker resolve、等待 renderer 执行 microtask checkpoint 的异步回调。
    async_callbacks_ready: Arc<AtomicBool>,
    #[cfg(test)]
    execution_count: Arc<AtomicU64>,
}

impl RendererJsWorker {
    /// 启动 JS 专用线程。
    #[allow(dead_code)] // 独立 worker 测试使用本地 owner；生产 renderer 注入 browser IPC handler。
    pub fn spawn(renderer_id: u64) -> Self {
        let indexed_db_handler =
            zero_page_runtime::indexed_db_handler(Arc::new(std::sync::Mutex::new(zero_storage::StorageManager::new())));
        Self::spawn_with_indexed_db_handler(renderer_id, indexed_db_handler)
    }

    pub(crate) fn spawn_with_indexed_db_handler(
        renderer_id: u64,
        indexed_db_handler: zero_engine::IndexedDbHandler,
    ) -> Self {
        Self::spawn_with_handlers(renderer_id, indexed_db_handler, None)
    }

    pub(crate) fn spawn_with_handlers(
        renderer_id: u64,
        indexed_db_handler: zero_engine::IndexedDbHandler,
        service_worker_client: Option<ServiceWorkerIpcClient>,
    ) -> Self {
        let mutations: Arc<std::sync::Mutex<Vec<DomMutation>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
        let rect_snapshot = new_layout_rect_snapshot();
        let handle_selector_map = new_handle_selector_map();
        let element_from_point_cache = new_element_from_point_cache();
        // R2949 FontFace.load() 桥——queue 与 runtime 共享，bridge 移入 worker 线程注册 __zw_load_font。
        let font_bridge = zero_engine::FontLoadBridge::new();
        let font_loads = font_bridge.queue();
        // R3058 JS 跨文档导航桥——queue 与 runtime 共享，bridge 移入 worker 线程注册 __zw_request_navigate。
        let nav_bridge = zero_engine::NavigationBridge::new();
        let navigations = nav_bridge.queue();
        let focus_changes: Arc<std::sync::Mutex<Vec<Option<String>>>> = Arc::default();
        // S11（cdp-protocol value-only console 面）：page console 输出队列——worker 回调
        // 推入，runtime 主循环 drain → browser/headless（`Runtime.consoleAPICalled`）。
        let console_logs: Arc<std::sync::Mutex<Vec<(String, String, String)>>> = Arc::default();
        let script_errors: Arc<std::sync::Mutex<Vec<zero_protocol::message::ScriptErrorParams>>> = Arc::default();
        let script_errors_for_worker = Arc::clone(&script_errors);
        let console_logs_for_worker = Arc::clone(&console_logs);
        // S14（cdp-protocol network.events）：renderer fetch 观测队列——**唯一实例在本
        // spawn 内创建**，worker 侧观测 handler 推入、runtime 经 accessor 持同 Arc drain
        // → browser/headless（`Network.*` 事件源）。S13 教训：双实例 Arc 错接致事件丢失。
        let fetch_observed: Arc<std::sync::Mutex<Vec<FetchObservedRecord>>> = Arc::default();
        // S16（cdp-protocol page.setContent）：document.write 写周期落定队列——worker 侧
        // `__zw_document_write_settled` 回调推入，runtime 主循环 drain → browser/headless
        // （`Page.loadEventFired` 族重发事件源）。
        let doc_write_settled: Arc<std::sync::Mutex<Vec<()>>> = Arc::default();
        let doc_write_settled_for_worker = Arc::clone(&doc_write_settled);
        // S11（emulation.media）：宿主媒体上下文共享 cell——renderer SetColorScheme/
        // SetMediaType/SetViewport 更新；`__zw_match_media` 重注册（后注册者胜）后
        // prefers-color-scheme 等用户偏好进 matchMedia 求值。
        let media_ctx: Arc<std::sync::Mutex<zero_css_parser::media_query::MediaContext>> = Arc::new(
            std::sync::Mutex::new(zero_css_parser::media_query::MediaContext::new(0.0, 0.0)),
        );
        let media_ctx_for_worker = Arc::clone(&media_ctx);
        let async_callbacks_ready = Arc::new(AtomicBool::new(false));
        #[cfg(test)]
        let execution_count = Arc::new(AtomicU64::new(0));
        let (cmd_tx, cmd_rx) = mpsc::channel();
        // t2-pb1 fix#5：优先通道——页面生命周期派发专用（见 js_worker_main 分派环注释）。
        let (prio_tx, prio_rx) = mpsc::channel();
        // t2-pb1 F1：文档代际序号源（struct 方法与三个执行器闭包共享同一计数器）。
        let cmd_seq = Arc::new(AtomicU64::new(0));
        let seq_for_exec = Arc::clone(&cmd_seq);
        let seq_for_module = Arc::clone(&cmd_seq);
        let seq_for_prio = Arc::clone(&cmd_seq);
        let prio_tx_for_struct = prio_tx.clone();
        let cmd_for_exec = cmd_tx.clone();
        let cmd_for_module = cmd_tx.clone();
        let cmd_for_worker = cmd_tx.clone();
        let mutations_for_worker = Arc::clone(&mutations);
        let rect_snapshot_for_worker = Arc::clone(&rect_snapshot);
        let handle_selector_map_for_worker = Arc::clone(&handle_selector_map);
        let element_from_point_cache_for_worker = Arc::clone(&element_from_point_cache);
        let async_callbacks_ready_for_worker = Arc::clone(&async_callbacks_ready);
        #[cfg(test)]
        let execution_count_for_worker = Arc::clone(&execution_count);

        let join = thread::Builder::new()
            .name(format!("renderer-js-{}", renderer_id))
            // R5000 live 侧同类加固（2026-10-02 slice20 实证）：本线程承载页面脚本 eval，
            // shim/站点脚本深递归帧贴近 Rust 默认 2MiB 线程栈边际——baidu 首页脚本管线
            // 实测 `thread 'renderer-js-N' has overflowed its stack`（SIGABRT，整 renderer
            // 随崩）。测试侧 R5000 已用 32MiB 缓解（Makefile RUST_MIN_STACK）；真修
            //（递归帧瘦身）仍归渲染流域碰头账。
            .stack_size(32 * 1024 * 1024)
            .spawn(move || {
                js_worker_main(
                    cmd_rx,
                    cmd_for_worker,
                    prio_rx,
                    console_logs_for_worker,
                    script_errors_for_worker,
                    doc_write_settled_for_worker,
                    media_ctx_for_worker,
                    mutations_for_worker,
                    rect_snapshot_for_worker,
                    handle_selector_map_for_worker,
                    element_from_point_cache_for_worker,
                    font_bridge,
                    nav_bridge,
                    indexed_db_handler,
                    service_worker_client,
                    async_callbacks_ready_for_worker,
                    #[cfg(test)]
                    execution_count_for_worker,
                )
            })
            .expect("spawn renderer js worker");

        let executor: ScriptFn = Arc::new(move |script: &str| {
            let (reply_tx, reply_rx) = mpsc::channel();
            cmd_for_exec
                .send(JsWorkerCommand::Execute {
                    script: script.to_string(),
                    reply: reply_tx,
                    seq: seq_for_exec.fetch_add(1, Ordering::Relaxed),
                })
                .map_err(|e| e.to_string())?;
            reply_rx
                .recv_timeout(TAB_JS_CHANNEL_TIMEOUT)
                .map_err(|e| e.to_string())?
        });

        let module_executor: ModuleFn = Arc::new(move |source: &str, url: &str, deps: &[(String, String)]| {
            let (reply_tx, reply_rx) = mpsc::channel();
            cmd_for_module
                .send(JsWorkerCommand::ExecuteModule {
                    source: source.to_string(),
                    url: url.to_string(),
                    deps: deps.to_vec(),
                    reply: reply_tx,
                    seq: seq_for_module.fetch_add(1, Ordering::Relaxed),
                })
                .map_err(|e| e.to_string())?;
            reply_rx
                .recv_timeout(TAB_JS_CHANNEL_TIMEOUT)
                .map_err(|e| e.to_string())?
        });

        // t2-pb1 fix#5：优先执行器——与 `executor` 同命令语义（Execute + reply），
        // 仅入队通道不同（worker 命令边界优先消费）。
        let prio_executor: ScriptFn = Arc::new(move |script: &str| {
            let (reply_tx, reply_rx) = mpsc::channel();
            prio_tx
                .send(JsWorkerCommand::Execute {
                    script: script.to_string(),
                    reply: reply_tx,
                    seq: seq_for_prio.fetch_add(1, Ordering::Relaxed),
                })
                .map_err(|e| e.to_string())?;
            reply_rx
                .recv_timeout(TAB_JS_CHANNEL_TIMEOUT)
                .map_err(|e| e.to_string())?
        });

        Self {
            cmd_tx,
            prio_tx: prio_tx_for_struct,
            cmd_seq,
            join: Some(join),
            executor,
            prio_executor,
            module_executor,
            console_logs,
            script_errors,
            doc_write_settled,
            media_ctx,
            fetch_observed,
            mutations,
            rect_snapshot,
            handle_selector_map,
            element_from_point_cache,
            font_loads,
            navigations,
            focus_changes,
            async_callbacks_ready,
            #[cfg(test)]
            execution_count,
        }
    }

    /// 供 WebView 注入的外部脚本执行器（T4 脚本桥接统一后评估是否保留）。
    #[allow(dead_code)]
    pub fn executor(&self) -> ScriptFn {
        Arc::clone(&self.executor)
    }

    /// 执行 ES module（含依赖注册表）。
    pub fn execute_module(&self, source: &str, url: &str, deps: &[(String, String)]) -> Result<String, String> {
        (self.module_executor)(source, url, deps)
    }

    /// 在 JS 线程执行脚本（不经 WebView 包装）。
    pub fn execute_script_direct(&self, script: &str) -> Result<String, String> {
        (self.executor)(script)
    }

    /// t2-pb1 fix#8：有界等待执行——与 [`Self::execute_script_direct`] 同命令语义（普通
    /// FIFO 通道，保证运行在已入队异步回调 resolve 之后），但等待 reply 有上限。
    /// 页面回调积压（bilibili timer 臂级联单窗 ~16s）时 checkpoint 不阻塞 renderer 主循环：
    /// 超时即放弃本轮执行机会（滞留命令是空脚本 no-op，worker 照常消费；ready 旗标由
    /// worker 继续处理的回调重新置位，下一轮重试），主循环回到 recv 保持导航 IPC 响应。
    pub fn execute_script_direct_bounded(&self, script: &str, timeout: Duration) -> Result<String, String> {
        let (reply_tx, reply_rx) = mpsc::channel();
        self.cmd_tx
            .send(JsWorkerCommand::Execute {
                script: script.to_string(),
                reply: reply_tx,
                seq: self.cmd_seq.fetch_add(1, Ordering::Relaxed),
            })
            .map_err(|e| e.to_string())?;
        match reply_rx.recv_timeout(timeout) {
            Ok(result) => result,
            Err(_) => Err("js worker execute wait timeout".to_string()),
        }
    }

    /// t2-pb1 fix#5：优先队列执行——页面生命周期派发（DOMContentLoaded/load、资源 settle
    /// 事件族）专用入口。worker 在命令边界优先于普通 FIFO 队列消费，使导航里程碑派发不被
    /// 脚本阶段遗留的 promise/timer 续体级联压住（bilibili 导航 15s 超时根因）。仅生命周期
    /// 派发使用；普通脚本/timer/回调仍走 FIFO 通道，相互相对顺序不变。
    pub fn execute_script_direct_priority(&self, script: &str) -> Result<String, String> {
        (self.prio_executor)(script)
    }

    /// t2-pb1 fix#12：fire-and-forget 优先提交——脚本入优先队列执行，但调用方不等待结果。
    /// 适用于全部 best-effort 派发（生命周期/资源/字体/脚本元素级事件、observer tick 等，
    /// 返回值被忽略或仅告警）：页面回调单臂可达 28-30s（bilibili 实测），即使是优先队列
    /// 同步往返，主循环也要等当前臂跑完才拿 reply，期间 Navigate IPC 饿死。reply 接收端
    /// 即刻丢弃——worker 照常执行并尝试回信（发送失败静默），入队边界与 FIFO/优先语义
    /// 与同步版完全一致（同通道提交顺序 = 执行顺序，如字体批处理先于 font settle）。
    pub fn submit_script_priority(&self, script: &str) -> Result<(), String> {
        let (reply_tx, reply_rx) = mpsc::channel();
        drop(reply_rx);
        self.prio_tx
            .send(JsWorkerCommand::Execute {
                script: script.to_string(),
                reply: reply_tx,
                seq: self.cmd_seq.fetch_add(1, Ordering::Relaxed),
            })
            .map_err(|e| e.to_string())
    }

    /// t2-pb1 fix#15：优先通道执行 + 有界等待，超时把 reply 通道交还调用方挂起续答。
    /// 脚本已入优先队列照常执行（结果晚至）；worker 忙于长臂（bilibili timer 回调
    /// 28-30s）时调用方主循环至多等 `timeout` 即可放行导航等 IPC，回复由调用方轮询
    /// `rx` 补答。通道关闭（导航复位清队 fix#11 / worker 退出）→ `Ok(Err)`。
    pub fn execute_script_priority_deferrable(
        &self,
        script: &str,
        timeout: std::time::Duration,
    ) -> Result<Result<String, String>, mpsc::Receiver<Result<String, String>>> {
        let (reply_tx, reply_rx) = mpsc::channel();
        if let Err(e) = self.prio_tx.send(JsWorkerCommand::Execute {
            script: script.to_string(),
            reply: reply_tx,
            seq: self.cmd_seq.fetch_add(1, Ordering::Relaxed),
        }) {
            return Ok(Err(format!("js worker prio send failed: {e}")));
        }
        match reply_rx.recv_timeout(timeout) {
            Ok(result) => Ok(result),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Err(reply_rx),
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => Ok(Err("js worker dropped reply".to_string())),
        }
    }

    /// 注入真实视口尺寸提示（shim `innerWidth/innerHeight` 缺省 1280x800 的校正源；
    /// renderer 启动与 `SetViewport` 时各发一次，worker 在快照换代后按需校正）。
    pub fn set_viewport_hint(&self, width: u32, height: u32) {
        let _ = self.cmd_tx.send(JsWorkerCommand::SetViewportHint { width, height });
    }

    /// S11：宿主媒体上下文 cell（`SetColorScheme`/`SetMediaType`/`SetViewport` 更新，
    /// `__zw_match_media` 求值消费）。
    pub fn media_ctx(&self) -> Arc<std::sync::Mutex<zero_css_parser::media_query::MediaContext>> {
        Arc::clone(&self.media_ctx)
    }

    /// S14：fetch 观测队列句柄（runtime 持同 Arc；观测 handler 包装经此推入）。
    pub fn fetch_observed_queue(&self) -> Arc<std::sync::Mutex<Vec<FetchObservedRecord>>> {
        Arc::clone(&self.fetch_observed)
    }

    /// S14：原子取出 fetch 观测记录（`(phase, seq, url, method, status)`），供 runtime 主
    /// 循环转发 browser/headless（`Network.*` 事件源）。
    pub fn take_fetch_observed(&self) -> Vec<FetchObservedRecord> {
        self.fetch_observed
            .lock()
            .map(|mut q| std::mem::take(&mut *q))
            .unwrap_or_default()
    }

    /// R-baidu2/P3：推入一条未捕获脚本错误（页面脚本执行失败时由 page_scripts 调用）。
    pub fn push_script_error(&self, params: zero_protocol::message::ScriptErrorParams) {
        if let Ok(mut q) = self.script_errors.lock() {
            q.push(params);
        }
    }

    /// R-baidu2/P3：原子取出未捕获脚本错误，供 runtime 主循环转发 browser/headless
    /// （`Runtime.exceptionThrown` 事件源）。
    pub fn take_script_errors(&self) -> Vec<zero_protocol::message::ScriptErrorParams> {
        self.script_errors
            .lock()
            .map(|mut q| std::mem::take(&mut *q))
            .unwrap_or_default()
    }

    /// S11：原子取出 page console 输出（`(level, text, args_json)`），供 runtime 主循环
    /// 转发 browser/headless（`Runtime.consoleAPICalled` 事件源）。
    pub fn take_console_logs(&self) -> Vec<(String, String, String)> {
        self.console_logs
            .lock()
            .map(|mut q| std::mem::take(&mut *q))
            .unwrap_or_default()
    }

    /// S16：原子取出 document.write 落定信号数（本 tick 内 close() 次数），供 runtime
    /// 主循环转发 browser/headless（`Page.loadEventFired` 族重发事件源）。
    pub fn take_document_write_settled(&self) -> usize {
        self.doc_write_settled
            .lock()
            .map(|mut q| {
                let n = q.len();
                q.clear();
                n
            })
            .unwrap_or(0)
    }

    /// 脚本执行前更新 DOM HTML 快照与页面 URL。
    pub fn set_dom_snapshot(&self, html: &str, url: &str) {
        let (reply_tx, reply_rx) = std::sync::mpsc::channel();
        if self
            .cmd_tx
            .send(JsWorkerCommand::SetDomSnapshot {
                html: html.to_string(),
                url: url.to_string(),
                reply: Some(reply_tx),
                seq: self.cmd_seq.fetch_add(1, Ordering::Relaxed),
            })
            .is_err()
        {
            return;
        }
        // t2-pb1 fix#21：有界等待快照应用——后续优先通道派发（fix#12 生命周期
        // fire-and-forget）依赖快照已生效：body onload 反射按页 URL 去重，抢跑一次
        // （快照尚在普通通道、worker 忙于 bootstrap/旧臂）即整页反射被去重丢弃，
        // <body onload> 永不触发（r2946 单测实证；无脚本页生产窗口同源）。同
        // reset_document_state 的有界等待惯用法；worker 忙臂时超时放行不阻塞导航，
        // 此时快照已排普通队列，后续普通往返天然保持先快照后脚本的顺序。
        let _ = reply_rx.recv_timeout(std::time::Duration::from_millis(250));
    }

    /// t2-pb1 fix#9：优先通道快照——与 [`Self::set_dom_snapshot`] 同命令，入优先队列。
    /// 供自动化求值路径与 `execute_script_direct_priority` 成对使用（同通道 FIFO 保持
    /// 快照先于执行的顺序），使宿主求值不被页面回调积压（普通队列）挡住。
    pub fn set_dom_snapshot_priority(&self, html: &str, url: &str) {
        let _ = self.prio_tx.send(JsWorkerCommand::SetDomSnapshot {
            html: html.to_string(),
            url: url.to_string(),
            reply: None,
            seq: self.cmd_seq.fetch_add(1, Ordering::Relaxed),
        });
    }

    /// 清除前一 document 留下的异步页面状态。
    ///
    /// renderer 保持同一个 JS worker，但导航不能让旧 document 的 timer、DOM mutation 或
    /// deferred navigation 在新 document 中继续执行。
    pub fn reset_document_state(&self) {
        if let Ok(mut mutations) = self.mutations.lock() {
            mutations.clear();
        }
        // R-baidu3：drain 代际递增——查询视图增量链（query_view_entry prev_base）
        // 只在本批（无 drain）内成立。
        zero_engine::js_dom_bridge::bump_mut_drain_gen();
        if let Ok(mut loads) = self.font_loads.lock() {
            loads.clear();
        }
        if let Ok(mut navigations) = self.navigations.lock() {
            navigations.clear();
        }
        if let Ok(mut focus_changes) = self.focus_changes.lock() {
            focus_changes.clear();
        }
        // slice18（site-compat baidu 建议链 /sugrec）：handle→selector 表同为文档域状态——
        // handle/listener store 随导航销毁重建，陈旧条目使宿主元素事件反查
        //（`__zw_handle_for_selector`）命中上一文档的死 handle 键 → onload IDL 静默不达。
        // webview 侧同表文档换代即清（`publish_forward_handle_map(None)`），此处补齐。
        // https://html.spec.whatwg.org/multipage/browsers.html#navigate
        if let Ok(mut map) = self.handle_selector_map.lock() {
            map.clear();
        }
        self.async_callbacks_ready.store(false, Ordering::Release);
        // t2-pb1 fix#11：复位命令走优先队列（跳过页面回调积压；同通道 FIFO 保证新文档
        // 的优先命令排在其后）+ 有界等待——页面回调饱和 / 单臂 30s 级（bilibili timer
        // 实测）时同步 reply 等待曾把导航初始化整体挂住（二跳 ERR_FAILED 根因链末级）。
        // 超时放弃等待：复位命令滞留优先队列照常执行（worker 端清队 + reset_context），
        // 主循环继续导航流程（新文档取回/解析不依赖 worker）。
        let (reply_tx, reply_rx) = mpsc::channel();
        if self
            .prio_tx
            .send(JsWorkerCommand::ResetDocumentState {
                reply: reply_tx,
                seq: self.cmd_seq.fetch_add(1, Ordering::Relaxed),
            })
            .is_ok()
        {
            let _ = reply_rx.recv_timeout(Duration::from_millis(250));
        }
    }

    /// 脚本执行期间记录的 DOM 变更（由 `__zw_*` 回调写入）。
    pub fn mutations(&self) -> Arc<std::sync::Mutex<Vec<DomMutation>>> {
        Arc::clone(&self.mutations)
    }

    /// 清空 DOM 变更队列并递增 drain 代际——「脚本执行前归账清零」的统一入口
    /// （`drain ⇒ bump_mut_drain_gen` 不变式，PR #33 返修确立、R-baidu3
    /// `reset_document_state` 同款）：查询视图增量链（callbacks.rs `prev_base` /
    /// VIEW_DOC_CACHE）以「队列只增长 + drain 代际未变」为前提，旁路 bump 的
    /// clear 会让缓存的 `key.count` 与实际队列脱钩——automation eval 竞窗实测
    /// renderer panic（siteopt t2-pb3nm 集成验收，`mut_guard[prev_count..count]`
    /// 越界）。凡要清队列一律走本方法，禁止直接 `.lock().clear()`。
    pub fn clear_mutations_fresh(&self) {
        // 中毒锁强制清（into_inner）——与被替换调用点原语义一致（PR #77 审查 D3）：
        // 毒锁跳过 clear 会让残留 mutations 被 drain 后 apply 到错误页。
        self.mutations.lock().unwrap_or_else(|e| e.into_inner()).clear();
        zero_engine::js_dom_bridge::bump_mut_drain_gen();
    }

    /// R2949 FontFace.load() 请求队列句柄——`__zw_load_font` 回调（worker 线程）push，renderer 主循环
    /// drain 后处理（fetch_get 字节 + load_font/register/set_resolver + async_resolver.resolve）。
    pub fn pending_font_loads(&self) -> Arc<std::sync::Mutex<Vec<zero_engine::FontLoadRequest>>> {
        Arc::clone(&self.font_loads)
    }

    /// R3058 JS 跨文档导航请求队列句柄——`__zw_request_navigate` 回调（worker 线程）push，renderer
    /// 主循环 drain 后 handle_navigate（fetch 新文档 + 重载）。
    pub fn pending_navigations(&self) -> Arc<std::sync::Mutex<Vec<String>>> {
        Arc::clone(&self.navigations)
    }

    /// R3254-M7'：页面 JS `focus()`/`blur()` 变更队列句柄——主线程 drain 后同步焦点状态。
    pub fn focus_changes(&self) -> Arc<std::sync::Mutex<Vec<Option<String>>>> {
        Arc::clone(&self.focus_changes)
    }

    /// P1a gBCR：共享 layout-rect snapshot 句柄——renderer 主循环 render 后经
    /// `fill_layout_rect_snapshot` 填充，js_worker 的 RectBridge handler 读取。
    pub fn rect_snapshot(&self) -> LayoutRectSnapshot {
        Arc::clone(&self.rect_snapshot)
    }

    /// P1a gBCR path A：持久 handle→唯一选择器映射句柄——生产 apply 路径
    /// （`page_scripts::apply_recorded_mutations`）merge 进此 map，js_worker 的 RectBridge
    /// handler 读它解析 handle-identity（`__n{n}`，createElement 元素）。
    pub fn handle_selector_map(&self) -> HandleSelectorMap {
        Arc::clone(&self.handle_selector_map)
    }

    /// P1a elementFromPoint：共享 hit-test 缓存槽句柄——renderer 主循环 render 后 swap 最新
    /// `Arc<HitTestCache>`，js_worker 的 `ElementFromPointBridge` 读它求 `(x,y)` 命中元素。
    pub fn element_from_point_cache(&self) -> ElementFromPointCache {
        Arc::clone(&self.element_from_point_cache)
    }

    /// 返回异步回调 resolver（P1b S1）。克隆供跨线程异步完成方（fetch host / 定时器）持有，
    /// `resolver.resolve(id, result)` 经 cmd channel marshal 回 JS worker 线程，由 worker 调
    /// `sandbox.resolve_async_callback` resolve 对应 Promise。
    #[allow(dead_code)] // 未来 setTimeout / MutationObserver 跨线程完成方消费（S5/S2）。
    pub fn async_resolver(&self) -> AsyncResolver {
        let tx = Arc::new(std::sync::Mutex::new(self.cmd_tx.clone()));
        AsyncResolver::new(move |id, result| {
            let _ = tx.lock().unwrap().send(JsWorkerCommand::ResolveAsyncCallback {
                id: id.to_string(),
                result: result.to_string(),
            });
        })
    }

    /// 领取一次已就绪异步回调的 microtask checkpoint 执行机会。
    ///
    /// JS worker 仅在实际处理 `ResolveAsyncCallback` 后置位，避免 renderer 在空闲时反复
    /// 编译空脚本。worker 串行执行命令，因此 checkpoint 执行期间新处理的回调会重新置位。
    pub fn take_pending_async_callbacks(&self) -> bool {
        self.async_callbacks_ready.swap(false, Ordering::AcqRel)
    }

    #[cfg(test)]
    pub fn execution_count_for_test(&self) -> u64 {
        self.execution_count.load(Ordering::Acquire)
    }

    /// P1b S3：注入 fetch handler（renderer 在 WebView 初始化后调用；测试用合成实现）。
    /// `__zw_fetch` 回调读此 handler 抓取后 resolve Promise。
    pub fn set_fetch_handler(&self, handler: FetchHandler) {
        let _ = self.cmd_tx.send(JsWorkerCommand::SetFetchHandler { handler });
    }

    /// media-playback M2c 后续：注入播放器注册表（renderer 主循环 WebView 初始化后调用；
    /// worker 注册 `__zwVideoBridge` 宿主桥——镜像 browser tab_js_worker，多进程路径
    /// 与 tabworker 路径的媒体播放真值面一致）。
    pub fn set_video_players(
        &self,
        registry: std::sync::Arc<std::sync::Mutex<zero_webview::video_registry::VideoPlayerRegistry>>,
        pump_clock: Option<std::sync::Arc<std::sync::atomic::AtomicU64>>,
    ) {
        let _ = self
            .cmd_tx
            .send(JsWorkerCommand::SetVideoPlayers { registry, pump_clock });
    }

    /// media-audio M3：注入 Web Audio 注册表（镜像 browser tab_js_worker 同名方法；
    /// worker 注册 `__zwWA*` 宿主桥——多进程路径 AudioContext 最小面 NullSink 可观测）。
    pub fn set_webaudio(
        &self,
        registry: std::sync::Arc<std::sync::Mutex<zero_webview::webaudio_registry::WebAudioRegistry>>,
    ) {
        let _ = self.cmd_tx.send(JsWorkerCommand::SetWebAudio { registry });
    }

    pub fn dispatch_indexed_db_connection_event(
        &self,
        connection_id: u64,
        old_version: u64,
        new_version: Option<u64>,
    ) -> Result<(), String> {
        let (reply_tx, reply_rx) = mpsc::channel();
        self.cmd_tx
            .send(JsWorkerCommand::DispatchIndexedDbConnectionEvent {
                connection_id,
                old_version,
                new_version,
                reply: reply_tx,
            })
            .map_err(|error| error.to_string())?;
        reply_rx
            .recv_timeout(TAB_JS_CHANNEL_TIMEOUT)
            .map_err(|error| error.to_string())?
    }

    /// 关闭 JS 线程。
    pub fn shutdown(&mut self) {
        let _ = self.cmd_tx.send(JsWorkerCommand::Shutdown);
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}

impl Drop for RendererJsWorker {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// js-dom R386：worker 沙箱装原生 DOM 绑定（[`js_worker_main`] bootstrap 调用）。
///
/// 经 `Sandbox::install_native_bindings*` escape-hatch 进入持久 Context 安装
/// `dom_bindings`/`quickjs_dom_bindings`（与 webview `install_native_dom_bindings` 同一
/// 生产路径形态）。worker 无 live Document，从 `dom_html` 快照 re-parse——后续快照换代
/// 由 [`refresh_worker_native_dom_source`] 刷新 DOM 源（与 tab_js_worker 镜像）。
fn install_worker_native_dom_bindings(
    sandbox: &mut dyn zero_script_sandbox::Sandbox,
    dom_html: &std::sync::Mutex<String>,
) {
    let html = dom_html.lock().map(|s| s.clone()).unwrap_or_default();
    #[cfg(feature = "v8")]
    {
        let installed = sandbox.install_native_bindings(Box::new(move |scope, ctx| {
            zero_engine::dom_bindings::install_dom_bindings_from_html(scope, ctx, &html);
        }));
        if !installed {
            tracing::debug!("renderer worker: native DOM bindings install unavailable (non-persistent context)");
        }
    }
    #[cfg(all(feature = "quickjs", not(feature = "v8")))]
    {
        let installed = sandbox.install_native_bindings_quickjs(Box::new(move |ctx| {
            zero_engine::quickjs_dom_bindings::install_dom_bindings_quickjs_from_html(ctx, &html);
        }));
        if !installed {
            tracing::debug!("renderer worker: QuickJS native DOM bindings install unavailable");
        }
    }
}

/// js-dom R386：快照换代刷新 worker 原生绑定的 DOM 源（`SetDomSnapshot`/`ResetDocumentState`
/// 消费方调用；镜像 tab_js_worker 同名 helper）。
///
/// 同代际（绑定全局已装）走 refresh-only 快路径（仅换 DOM 源，**不重跑全局注册**——
/// quickjs 全量 install 会重挂 `globalThis.Event` 等 JS 胶水构造器覆盖 shim 同名全局，
/// shim `_dispatchWithBubble` 读 native 实例缺失的 `_defaultPrevented` 恒 true →
/// form.reset() 的 preventDefault 失效；R386 renderer 测试实证）。跨代际
/// （`reset_context` 后 context 重建、全局工厂丢失）重新全量 install。
fn refresh_worker_native_dom_source(
    sandbox: &mut dyn zero_script_sandbox::Sandbox,
    html: &str,
    native_installed: &mut bool,
) {
    #[cfg(feature = "v8")]
    {
        if *native_installed {
            // 同代际：全局工厂/模板在位，仅刷新 DOM 源（re-parse 快照 → Rc 交换）。
            zero_engine::dom_bindings::refresh_dom_source_from_html(html);
            return;
        }
        let html_owned = html.to_string();
        let installed = sandbox.install_native_bindings(Box::new(move |scope, ctx| {
            zero_engine::dom_bindings::install_dom_bindings_from_html(scope, ctx, &html_owned);
        }));
        *native_installed = installed;
    }
    #[cfg(all(feature = "quickjs", not(feature = "v8")))]
    {
        if *native_installed {
            zero_engine::quickjs_dom_bindings::refresh_quickjs_dom_source_from_html(html);
            return;
        }
        let html_owned = html.to_string();
        let installed = sandbox.install_native_bindings_quickjs(Box::new(move |ctx| {
            zero_engine::quickjs_dom_bindings::install_dom_bindings_quickjs_from_html(ctx, &html_owned);
        }));
        *native_installed = installed;
    }
}

/// t2-pb1 F1：导航复位清队的存活判定——命令提交代际（seq）晚于本次复位（`seq >
/// reset_seq`）即为新文档工作，存活；早于复位的文档类命令（Execute/ExecuteModule/
/// SetDomSnapshot/更早的 Reset 之前的同类）随旧文档丢弃。ResolveAsyncCallback 一律
/// 丢弃：晚至的旧页回调解析在新 context 无对应 id（t8 返修既有语义；新页回调解析
/// 恰入队的极端窗与其一同牺牲，与复位前行为一致）。其余为跨文档配置/生命周期命令
/// （handler 注入、注册表、视口提示、IndexedDb 事件、Shutdown），一律存活。
fn survives_document_reset(cmd: &JsWorkerCommand, reset_seq: u64) -> bool {
    match cmd {
        JsWorkerCommand::Execute { seq, .. }
        | JsWorkerCommand::ExecuteModule { seq, .. }
        | JsWorkerCommand::SetDomSnapshot { seq, .. }
        | JsWorkerCommand::ResetDocumentState { seq, .. } => *seq > reset_seq,
        JsWorkerCommand::ResolveAsyncCallback { .. } => false,
        _ => true,
    }
}

/// t7 诊断基座：js worker 命令成本普查（`ZW_JS_WORKER_CENSUS` 启用）。
/// 按命令类型累计「执行次数 / 累计耗时 / 单次最大耗时」，窗口 ≥5s 输出一行，
/// 用于把稳态满核分解为可排序的成本桶。纯观测：默认关闭，关闭时每命令仅一次
/// bool 判断；开启时每命令开销为两次 `Instant::now` + 一次哈希增量。
/// 输出三态：值 `1`/`true` → tracing INFO（renderer stderr 在多进程下进有界 tail
/// 缓冲，平时不可见）；空串与 `0`/`false`/`off`（任意大小写）→ 关闭；其余非空值
/// 视为路径 → 追加写该文件（诊断采集用侧信道）。
struct WorkerCensus {
    enabled: bool,
    sink: Option<std::path::PathBuf>,
    window_start: std::time::Instant,
    buckets: HashMap<&'static str, (u64, u128, u128)>,
}

impl WorkerCensus {
    fn new() -> Self {
        let value = zero_runtime_config::optional_string("ZW_JS_WORKER_CENSUS");
        let truthy = value
            .as_deref()
            .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
        // falsy 值与 truthy 值都不当作 sink 路径——否则 `CENSUS=0` 意外启用文件
        // 写、`CENSUS=1` 会在 cwd 留下名为 `1` 的垃圾文件（PR90 复核发现）。
        let sink = value
            .filter(|v| !truthy && !(v == "0" || v.eq_ignore_ascii_case("false") || v.eq_ignore_ascii_case("off")))
            .map(std::path::PathBuf::from);
        Self {
            enabled: truthy || sink.is_some(),
            sink,
            window_start: std::time::Instant::now(),
            buckets: HashMap::new(),
        }
    }

    fn kind(cmd: &JsWorkerCommand) -> &'static str {
        match cmd {
            JsWorkerCommand::Execute { .. } => "Execute",
            JsWorkerCommand::ExecuteModule { .. } => "ExecuteModule",
            JsWorkerCommand::SetDomSnapshot { .. } => "SetDomSnapshot",
            // 响应派发按 payload 规模分桶——区分「少量巨大响应」与「每个响应都贵」。
            JsWorkerCommand::ResolveAsyncCallback { result, .. } => {
                if result.len() < 1_000 {
                    "ResolveAsyncCallback[<1KB]"
                } else if result.len() < 10_000 {
                    "ResolveAsyncCallback[1-10KB]"
                } else if result.len() < 100_000 {
                    "ResolveAsyncCallback[10-100KB]"
                } else {
                    "ResolveAsyncCallback[>100KB]"
                }
            }
            JsWorkerCommand::SetFetchHandler { .. } => "SetFetchHandler",
            JsWorkerCommand::SetVideoPlayers { .. } => "SetVideoPlayers",
            JsWorkerCommand::SetWebAudio { .. } => "SetWebAudio",
            JsWorkerCommand::ResetDocumentState { .. } => "ResetDocumentState",
            JsWorkerCommand::SetViewportHint { .. } => "SetViewportHint",
            JsWorkerCommand::DispatchIndexedDbConnectionEvent { .. } => "DispatchIndexedDbConnectionEvent",
            JsWorkerCommand::Shutdown => "Shutdown",
        }
    }
}

/// 借用普查表，在作用域结束（该命令处理完成）时入账；窗口期满即输出并重置。
/// 输出按累计耗时降序——饱和大头一眼可见；busy% = Σ执行耗时 / 墙钟。
struct WorkerCensusGuard<'a> {
    census: &'a mut WorkerCensus,
    kind: &'static str,
    started: std::time::Instant,
}

impl WorkerCensusGuard<'_> {
    fn report(census: &mut WorkerCensus) {
        let wall = census.window_start.elapsed();
        if wall < std::time::Duration::from_secs(5) || census.buckets.is_empty() {
            return;
        }
        let mut rows: Vec<(&'static str, (u64, u128, u128))> = census.buckets.drain().collect();
        rows.sort_by_key(|(_, (_, total, _))| std::cmp::Reverse(*total));
        let mut total_ns: u128 = 0;
        let mut parts: Vec<String> = Vec::with_capacity(rows.len());
        for (name, (n, total, max)) in rows {
            total_ns += total;
            parts.push(format!(
                "{name}{{n={n},t={}ms,max={}ms}}",
                total / 1_000_000,
                max / 1_000_000
            ));
        }
        let busy = 100 * total_ns / wall.as_nanos().max(1);
        let line = format!("worker busy={busy}% wall={}ms {}", wall.as_millis(), parts.join(" "));
        tracing::info!(target: "js_worker_census", "{line}");
        if let Some(path) = &census.sink
            && let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path)
        {
            use std::io::Write as _;
            let _ = writeln!(f, "{line}");
        }
        census.window_start = std::time::Instant::now();
    }
}

impl Drop for WorkerCensusGuard<'_> {
    fn drop(&mut self) {
        let elapsed = self.started.elapsed();
        let entry = self.census.buckets.entry(self.kind).or_insert((0, 0, 0));
        entry.0 += 1;
        entry.1 += elapsed.as_nanos();
        entry.2 = entry.2.max(elapsed.as_nanos());
        Self::report(self.census);
    }
}

#[allow(clippy::too_many_arguments)] // Thread-owned bridges are explicit at the single worker bootstrap boundary.
fn js_worker_main(
    cmd_rx: Receiver<JsWorkerCommand>,
    cmd_tx: Sender<JsWorkerCommand>,
    prio_rx: Receiver<JsWorkerCommand>,
    console_logs_for_worker: Arc<std::sync::Mutex<Vec<(String, String, String)>>>,
    script_errors_for_worker: Arc<std::sync::Mutex<Vec<zero_protocol::message::ScriptErrorParams>>>,
    doc_write_settled_for_worker: Arc<std::sync::Mutex<Vec<()>>>,
    media_ctx_for_worker: Arc<std::sync::Mutex<zero_css_parser::media_query::MediaContext>>,
    mutations: Arc<std::sync::Mutex<Vec<DomMutation>>>,
    rect_snapshot: LayoutRectSnapshot,
    handle_selector_map: HandleSelectorMap,
    element_from_point_cache: ElementFromPointCache,
    font_bridge: zero_engine::FontLoadBridge,
    nav_bridge: zero_engine::NavigationBridge,
    indexed_db_handler: zero_engine::IndexedDbHandler,
    service_worker_client: Option<ServiceWorkerIpcClient>,
    async_callbacks_ready: Arc<AtomicBool>,
    #[cfg(test)] execution_count: Arc<AtomicU64>,
) {
    let js_config = SandboxConfig {
        persistent_context: true,
        timeout_ms: TAB_JS_EXEC_TIMEOUT_MS,
        ..Default::default()
    };
    // js-dom R386：v8+quickjs 组合态（workspace feature 并集）下 `js_config` 双 move——
    // v8 分支 clone（镜像 tab_js_worker R84 同款修法；CI 单 feature 矩阵掩盖组合态编译断）。
    #[cfg(feature = "v8")]
    let mut sandbox: Box<dyn zero_script_sandbox::Sandbox> =
        Box::new(zero_script_sandbox::V8Sandbox::with_config(js_config.clone()).expect("V8 sandbox init"));
    #[cfg(all(feature = "quickjs", not(feature = "v8")))]
    let mut sandbox: Box<dyn zero_script_sandbox::Sandbox> =
        Box::new(zero_script_sandbox::QuickJSSandbox::with_config(js_config).expect("QuickJS sandbox init"));
    let dom_html: Arc<std::sync::Mutex<String>> = Arc::new(std::sync::Mutex::new(String::new()));
    let page_url: Arc<std::sync::Mutex<String>> = Arc::new(std::sync::Mutex::new(String::from("about:blank")));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<zero_engine::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(zero_engine::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut *sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // js-dom R100/R145 identity 桥（renderer 侧）：`__zw_handle_for_selector` selector→handle
    // 反查——createElement/cloneNode 产物（__zwHandle 锚定 listener store）在宿主按 selector
    // 派发（R2944 元素级 load/error、宿主事件）时经此命中 handle key。镜像 webview
    // `register_identity_bridge_callback`。worker 的 `handle_selector_map` 是 handle→selector
    // **正置**表（webview selector_handle_map 的倒置镜像，生产方 =
    // page_scripts::apply_recorded_mutations 的 handle_selectors merge）——反查按值匹配
    //（表随 createElement 数量线性，宿主派发低频，O(n) 扫描可接受）。
    // 已知边界：多个 handle 映射到同一 selector 时，HashMap 迭代序不定 → 命中任意一个
    //（webview 正置表为 last-write-wins，语义不同但同属「多孪生元素未定义锚定」；如需
    // 确定性，须在 merge 时维护 selector→handle 索引）。
    // 缺此注册时 renderer 宿主派发对动态创建元素恒 miss。
    {
        let sel_map = Arc::clone(&handle_selector_map);
        sandbox.register_callback(
            "__zw_handle_for_selector",
            Box::new(move |args: &[String]| -> String {
                let sel = args.first().map(String::as_str).unwrap_or("");
                sel_map
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .iter()
                    .find(|(_, mapped_sel)| mapped_sel.as_str() == sel)
                    .map(|(handle, _)| handle.clone())
                    .unwrap_or_default()
            }),
        );
    }
    // S11（cdp-protocol value-only console 面）：覆盖引擎的 `__zw_console_log`（后注册者
    // 胜——execute 边界按注册序 re-bind 全局），tracing 行为保持 + 逐条推入共享队列供
    // renderer 主循环 drain → browser/headless（`Runtime.consoleAPICalled` 事件源）。
    {
        let console_queue = Arc::clone(&console_logs_for_worker);
        sandbox.register_callback(
            "__zw_console_log",
            Box::new(move |args: &[String]| -> String {
                let level = args.first().cloned().unwrap_or_else(|| "log".into());
                let text = args.get(1).cloned().unwrap_or_default();
                let args_json = args.get(2).cloned().unwrap_or_else(|| "[]".into());
                match level.as_str() {
                    "error" => tracing::error!("[console] {text}"),
                    "warn" => tracing::warn!("[console] {text}"),
                    "info" | "log" | "table" => tracing::info!("[console] {text}"),
                    _ => tracing::debug!("[console.{level}] {text}"),
                }
                if let Ok(mut q) = console_queue.lock() {
                    q.push((level, text, args_json));
                    if q.len() > 512 {
                        let drop = q.len() - 512;
                        q.drain(..drop);
                    }
                }
                String::new()
            }),
        );
    }
    // S16（cdp-protocol page.setContent）：`__zw_document_write_settled`——shim document.close()
    // 应用写内容后调用（spec：close() 解析结束触发 load 生命周期）；推入共享队列供 runtime
    // drain → browser/headless 重发 `Page.loadEventFired` 族（PW setContent 等待新 load）。
    {
        let settled_queue = Arc::clone(&doc_write_settled_for_worker);
        sandbox.register_callback(
            "__zw_document_write_settled",
            Box::new(move |_args: &[String]| -> String {
                tracing::debug!("document.write cycle settled");
                if let Ok(mut q) = settled_queue.lock() {
                    q.push(());
                }
                String::new()
            }),
        );
    }
    // `__zw_match_media` 宿主媒体上下文重注册（覆盖 register_dom_callbacks 的缺省 Light 版）——
    // renderer SetColorScheme/SetMediaType/SetViewport 更新共享 cell，matchMedia 求值即得
    // 用户偏好真值（CDP Emulation 面；S11 emulation.media）。
    {
        let media_ctx = Arc::clone(&media_ctx_for_worker);
        sandbox.register_callback(
            "__zw_match_media",
            Box::new(move |args: &[String]| -> String {
                let query = args.first().map(String::as_str).unwrap_or("");
                let width = args.get(1).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                let height = args.get(2).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                zero_engine::match_media_to_json_ctx(query, width, height, &media_ctx)
            }),
        );
    }
    // js-dom R386（DC-1 多进程生产路径收口）：worker 沙箱装原生 DOM 绑定——镜像 webview
    // `install_native_dom_bindings`（R384 default-on），使 renderer worker 的页面 JS↔DOM
    // 桥不再只走 polyfill 字符串桥。worker 的 DOM 真相是 `dom_html` 快照字符串（live
    // `Rc<RefCell<Document>>` 属 renderer 进程 webview 线程，不能跨线程），故从快照
    // re-parse（与 RectBridge handler 的确定性同源）。快照换代经 `SetDomSnapshot`/
    // `ResetDocumentState` 消费方刷新。
    install_worker_native_dom_bindings(&mut *sandbox, &dom_html);
    let indexed_db_bridge = zero_engine::IndexedDbBridge::new(indexed_db_handler);
    indexed_db_bridge.register(&mut *sandbox, &page_url);
    if let Some(client) = service_worker_client {
        client.register_callbacks(&mut *sandbox);
    }
    register_module_compile_callback(&mut *sandbox);
    // P1a gBCR（Slice 1）：RectBridge 注 `__zw_getBoundingClientRect(identity)` 同步回调。
    // handler 解析 identity(selector) → NodeId（fresh-parse dom_html，与渲染管线确定性一致）
    // → 查 rect_snapshot。kill-switch `ZW_REAL_RECT=0` 关闭（回落零 rect = 当前行为，零回归）。
    if real_rect_enabled() {
        let rect_bridge = RectBridge::new();
        rect_bridge.register(&mut *sandbox);
        rect_bridge.set_handler(make_dom_html_rect_handler(
            Arc::clone(&dom_html),
            Arc::clone(&rect_snapshot),
            Arc::clone(&handle_selector_map),
        ));
    }
    // P1a elementFromPoint：`ElementFromPointBridge` 注 `__zw_elementFromPoint(x, y)` 同步回调。
    // 回调锁内 clone `Arc<HitTestCache>`（renderer render 后 swap 进共享槽）→ `hit_test_element`
    // + `selector_from_element_hit` → 稳定选择器。未注入 cache / 无命中 → 空串（shim 返 null）。
    let element_from_point_bridge = ElementFromPointBridge::new(element_from_point_cache);
    element_from_point_bridge.register(&mut *sandbox);
    // P1b S1/S3：AsyncResolver（跨线程 resolve Promise）+ FetchBridge（__zw_fetch 注册 +
    // handler cell）。fetch_bridge 经 SetFetchHandler 命令在 WebView 初始化后注入生产 handler
    // （chicken-and-egg——js_worker spawn 早于 WebView）；未注入时 __zw_fetch resolve 错误 Promise。
    let resolver = AsyncResolver::new({
        let tx = Arc::new(std::sync::Mutex::new(cmd_tx));
        move |id, result| {
            let _ = tx.lock().unwrap().send(JsWorkerCommand::ResolveAsyncCallback {
                id: id.to_string(),
                result: result.to_string(),
            });
        }
    });
    let fetch_bridge = FetchBridge::new(resolver.clone());
    fetch_bridge.register(&mut *sandbox);
    // R2949 FontFace.load() 桥——__zw_load_font 回调 push 请求到共享队列（runtime drain 后 fetch+register+resolve）。
    font_bridge.register(&mut *sandbox);
    // R3058 JS 跨文档导航桥——__zw_request_navigate 回调 push URL 到共享队列（runtime drain 后 handle_navigate）。
    nav_bridge.register(&mut *sandbox);
    // P1b S5：TimerBridge 注 __zw_setTimeout——shim setTimeout/setInterval 真实延迟
    // （子线程 sleep + resolver.resolve → __zwResolveCallback 调用 JS 回调）。
    let timer_bridge = TimerBridge::new(resolver);
    timer_bridge.register(&mut *sandbox);
    // R2713b：帧驱动 rAF kill-switch——execute shim 前注入 globalThis.__ZW_RAF_FRAME_DRIVEN
    //（shim 据此分支 rAF 同步 stub / 帧驱动）。默认 OFF 不注入 → shim `|| false` → 同步 stub。
    if raf_frame_driven_enabled() {
        let _ = sandbox.execute("globalThis.__ZW_RAF_FRAME_DRIVEN = true;");
    }
    // js-dom R386：原生绑定 install 踪迹（bootstrap install 置 true）。renderer worker 的
    // `ResetDocumentState` 走 `sandbox.reset_context()`（context 销毁重建）→ 置 false，
    // 下一快照换代全量重 install。
    let mut native_installed = true;
    let shim = generate_js_dom_shim();
    if let Err(e) = sandbox.execute(shim) {
        tracing::error!("JS DOM shim init failed: {e}");
    }
    // t2-pb1 fix#19：动态外链脚本由宿主通路取回（runtime execute_new_dynamic_scripts/
    // tick_dynamic_scripts——no-cors 脚本语义 + 元素事件派发）；shim R387b 的页面 fetch
    //（cors 语义）分支让位，避免双执行/双事件（part04 R387b 读此旗标；reset 重建后重置，
    // 见 ResetDocumentState 臂。browser 单进程路径不置旗标，R387b 照常）。
    let _ = sandbox.execute("globalThis.__zwHostDynamicScripts = true;");
    let mut viewport_hint: (u32, u32) = (0, 0);

    // t2-pb1 fix#5：命令分派环——优先通道（页面生命周期派发）在每条命令边界先于普通
    // 队列被处理。真实浏览器中 DOMContentLoaded/load 在解析/资源 settle 即派发，不被脚本
    // 发起的异步续体推迟；本 worker 原先单 FIFO 通道，bilibili 脚本阶段遗留的 promise/
    // timer 续体级联（单臂最长 2.6s）把已入队的 DCL Execute 压 7-25s → 导航 15s 超时
    // ERR_FAILED。仅调序不并行：生命周期派发仍在本 worker 串行执行。
    // fix#18：被下方「醒来再查」搁置的普通命令——优先通道清空后立即补跑。
    let mut held_normal: Option<JsWorkerCommand> = None;
    // t2-pb1 F1：复位臂存活的滞留命令本地续派队列。不经通道回送（fix#20 的自馈送
    // 教训在此结构性免疫），且通道回送会让复位臂执行期间新到的命令插队到回送命令
    // 之前、破坏快照+执行的成对序——本地队列保持原相对序，下一轮分派最先消费。
    let mut retained_after_reset: VecDeque<JsWorkerCommand> = VecDeque::new();
    let mut census = WorkerCensus::new();
    loop {
        let next_cmd = 'dispatch: loop {
            // F1：复位臂存活的滞留命令按原序先于一切新到命令续派。
            if let Some(cmd) = retained_after_reset.pop_front() {
                break 'dispatch Some(cmd);
            }
            // 优先命令先取（每轮一条；循环回到顶部即继续清空）。
            if let Ok(cmd) = prio_rx.try_recv() {
                break 'dispatch Some(cmd);
            }
            if let Some(cmd) = held_normal.take() {
                break 'dispatch Some(cmd);
            }
            match cmd_rx.recv_timeout(WORKER_IDLE_POLL) {
                Ok(cmd) => {
                    // t2-pb1 fix#18：recv 醒来后必须再查一次优先通道。原实现只在阻塞前
                    // 查 prio（check-then-block 竞态窗口）：fix#12 把生命周期派发改为
                    // fire-and-forget 后，紧随优先提交到达的普通命令（单测探针、页面
                    // 后续脚本）可越过 DCL/load 先行执行——r2943 类单测探针读到
                    // 「提交顺序即执行顺序」破坏后的旧状态，四例齐失败。搁置普通命令
                    // 一轮，下一轮循环优先队列清空后在此处补跑。
                    if let Ok(prio_cmd) = prio_rx.try_recv() {
                        held_normal = Some(cmd);
                        break 'dispatch Some(prio_cmd);
                    }
                    break 'dispatch Some(cmd);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => continue 'dispatch,
                Err(mpsc::RecvTimeoutError::Disconnected) => break 'dispatch None,
            }
        };
        let Some(cmd) = next_cmd else { break };
        // t7 诊断基座：命令处理全程计时，Drop 入账（关闭时零开销跳过）。
        let _census = census.enabled.then(|| WorkerCensusGuard {
            census: &mut census,
            kind: WorkerCensus::kind(&cmd),
            started: std::time::Instant::now(),
        });
        match cmd {
            JsWorkerCommand::Execute { script, reply, .. } => {
                #[cfg(test)]
                execution_count.fetch_add(1, Ordering::Relaxed);
                let full = format!("__zw_begin_script && __zw_begin_script();\n{script}");
                let exec_started = std::time::Instant::now();
                let result = sandbox
                    .execute(&full)
                    .map(|r| r.value)
                    .map_err(|e| annotate_timeout_error(&e.to_string(), &script));
                // P7b 可观测性：长执行（>1s）记耗时与脚本长度——页面回调超时定性
                // 需要执行面数据（此前 ScriptErrorParams 只有页面 URL 无脚本上下文）。
                if exec_started.elapsed() >= std::time::Duration::from_secs(1) {
                    tracing::warn!(
                        target: "js_worker",
                        elapsed_ms = exec_started.elapsed().as_millis() as u64,
                        script_len = script.len(),
                        "slow js execute"
                    );
                }
                // R-baidu2/P3：未捕获脚本错误统一在此汇出（页面脚本/定时器/事件回调
                // 的异常都经某次 execute 的 Err 冒出）→ `Runtime.exceptionThrown`。
                if let Err(ref message) = result {
                    let source = page_url.lock().map(|mut g| std::mem::take(&mut *g)).unwrap_or_default();
                    if let Ok(mut q) = script_errors_for_worker.lock() {
                        q.push(zero_protocol::message::ScriptErrorParams {
                            text: message.clone(),
                            source,
                            line_number: 0,
                            column_number: 0,
                        });
                    }
                }
                // R-baidu2/P3 slice-2：排空未捕获异常报告（promise-reject 回调收集）。
                for (text, line, column) in sandbox.take_uncaught_reports() {
                    let source = page_url.lock().map(|g| g.clone()).unwrap_or_default();
                    if let Ok(mut q) = script_errors_for_worker.lock() {
                        q.push(zero_protocol::message::ScriptErrorParams {
                            text,
                            source,
                            line_number: line,
                            column_number: column,
                        });
                    }
                }
                let _ = reply.send(result);
            }
            JsWorkerCommand::ExecuteModule {
                source,
                url,
                deps,
                reply,
                ..
            } => {
                let result = execute_module_in_sandbox(&mut *sandbox, &source, &url, &deps);
                let _ = reply.send(result);
            }
            JsWorkerCommand::SetDomSnapshot { html, url, reply, .. } => {
                // 视口提示校正：shim 缺省 innerWidth/innerHeight 1280x800 与真实视口失配时
                // （首次 install 后必失配）按 hint 校正（幂等——匹配即 no-op，零事件噪声）。
                if viewport_hint.0 > 0 && viewport_hint.1 > 0 {
                    let guard = format!(
                        "if (typeof __zw_user_resize === 'function' && (globalThis.innerWidth !== {w} || globalThis.innerHeight !== {h})) __zw_user_resize({w}, {h});",
                        w = viewport_hint.0,
                        h = viewport_hint.1
                    );
                    let _ = sandbox.execute(&guard);
                }
                // slice18（site-compat baidu 建议链 /sugrec，R-baidu8 接管收尾）：renderer 上下文
                // 声明动态 src 脚本单点归属宿主管线——shim R387b 页面 fetch 通道（cors 语义）
                // 整体跳过：no-cors classic script 在 cors 语义下恒败误派元素 error（跨域无
                // ACAO CDN 脚本，AMD 加载器常态），同源则与 PendingDynamicScripts（no-cors
                // IPC 取回，tick_dynamic_scripts）双通道双执行。幂等；每快照换代重设
                //（reset_context 销毁重建后由下一快照重新置位，见本 arm 首行执行序）。
                // https://html.spec.whatwg.org/multipage/scripting.html#fetch-a-classic-script
                let _ = sandbox.execute("globalThis.__zwHostOwnsDynamicScripts = true;");
                // P1a form input：URL 变化（导航）→ 清 shim value 缓存，防跨页同选择器 stale value。
                let url_changed = page_url.lock().map(|u| *u != url).unwrap_or(true);
                if let Ok(mut snap) = dom_html.lock() {
                    // js-dom R386：快照换代同步刷新原生绑定 DOM 源（native 路径与 polyfill
                    // 桥同源读到新快照；首代 bootstrap 已 install → 走 refresh 快路径）。
                    refresh_worker_native_dom_source(&mut *sandbox, &html, &mut native_installed);
                    *snap = html;
                }
                // R358/R3243：就地换代（Arc 被回调捕获不可换装）→ bump 宿主视图缓存代际，
                // 防同 count 查询命中换代前解析的视图文档/备忘。
                zero_engine::js_dom_bridge::bump_dom_view_gen();
                // R358（js-dom M1）：快照换代即清 JS 侧 pending 记账（与 tab_js_worker 同款）。
                // slice33（RP-3 跨文档残影）：清算必须先于下方 slice27 named access 重装——
                // 旧序（重装 → 清算）把新装入 `_zwLiveCollections` 的集合随即抹除，注册表
                // 恒空，slice32 live 维护（childList/id·name 变异同步 + dead 冻结）在 renderer
                // 永不生效，集合成员冻结在安装时点视图上（残影放大器：安装后任何 stale
                // 成员都无人再校正）。换代先清、重装后注册，集合才真正 live。
                let _ = sandbox.execute("__zw_reset_pending_state && __zw_reset_pending_state();");
                // slice27（site-compat baidu 建议链 su 注册，2026-10-04）：**快照落地即同步
                // Window named access**。shim 的 install 自调用仅发生在 shim eval——bootstrap
                // 时 dom_html 恒空、reset 时读到的是**上一文档**快照——首载快照落地后无人
                // 注册，`window.su`/`window.kw` 永不出现（活体 B 臂 702336f67 首载实测，
                // baidu 建议链静默死）。此处于 `*snap = html` + 视图换代后执行注册，并维护
                // 「本特性已注册面」登记（`__zwNamedAccessInstalled`，随 reset_context 重建
                // 归零）：换代后消失的 id 元素全局回收（不留上一文档悬挂元素）；新 context
                // 首快照（登记未建、页面脚本未跑）全量扫除元素全局后重注；脚本自建
                // 同名全局不被遮蔽（spec：脚本 own property 位于 WindowProperties 命名属性
                // 层之下）。与 shim eval 自调用互补，幂等；每次快照换代各执行一遍。
                // slice28（RP-1）：`__zw_collect_ids` 同收 name 面（embed/form/img/
                // object 非空 name；iframe 委托 shim R139）——本处 cur 登记/换代回收/
                // install 全链路对 name 面自动生效，无须另改（登记口径 = collect 返回全集）。
                // https://html.spec.whatwg.org/multipage/window-object.html#named-access-on-the-window-object
                let _ = sandbox.execute(
                    r#"(function () {
  if (typeof __zw_collect_ids !== 'function') return;
  var ids = __zw_collect_ids();
  var cur = {};
  if (ids) {
    var parts = ids.split('|');
    for (var i = 0; i < parts.length; i++) {
      var id = parts[i];
      if (id && /^[A-Za-z_$][A-Za-z0-9_$]*$/.test(id)) cur[id] = true;
    }
  }
  // slice30（RP-1）：named access 安装值判别——元素（nodeType 1）或多命中集合
  //（HTMLCollection，__zwHC 内部标记）。换代回收/换代登记两口径同扩：集合无
  // nodeType，修前不在回收扫描内 → 跨换代悬挂。
  var _isNA = function (v) {
    if (!v || typeof v !== 'object') return false;
    if (v.nodeType === 1) return true;
    try { return typeof v.__zwHC === 'function'; } catch (e) { return false; }
  };
  var installed = globalThis.__zwNamedAccessInstalled;
  // slice30（RP-1）：集合值换代重装——登记在案的 named access 所有值中，①集合值
  //（多命中安装）随快照换代必删，交由 install 重装（成员随新快照刷新；形态切换
  // 多命中↔单命中同此闭合——旧集合不删则 install 的「不覆盖已存在全局」守卫跳过，
  // 残留旧形态）；②仍在多命中清单的名重装（集合成员刷新）。单命中元素值保持
  // slice27 口径（选择器 wrapper 再解析，不重装）。脚本自有全局（无登记）不动。
  // slice32（RP-3）：同代内集合 live 语义已落地（shim liveSpec 设施：childList/
  // id·name 属性变异同步维护 + 旧集合重装即 dead 冻结）；FIXME(live-collection)
  // 收窄至**动态取值面**——安装时点不存在的名（脚本后建）window.N 不解析
  //（WindowProperties 每读动态查找在数据属性安装全局上不可达，RP-3 池后续）。
  try {
    var mnames30 = typeof __zw_collect_ids_multi === 'function' ? __zw_collect_ids_multi() : '';
    var multiNow = {};
    if (mnames30) {
      var mp30 = mnames30.split('|');
      for (var mi30 = 0; mi30 < mp30.length; mi30++) if (mp30[mi30]) multiNow[mp30[mi30]] = true;
    }
    for (var k30 in cur) {
      if (!(installed && installed[k30])) continue;
      var ex30;
      // slice37（NPO 收口）：安装值读面 = __zwNAGet（wired：backing own「本面登记值」
      // 口径——globalThis 读经原型链也能解析，但脚本 expando 不得混入；quickjs 不接链：
      // globalThis own raw 读，slice36 口径，引擎分叉见 part05 接线处申报）。
      try { ex30 = __zwNAGet(k30); } catch (e30) { continue; }
      if (!_isNA(ex30)) continue;
      if (!(ex30.nodeType === 1) || multiNow[k30]) {
        try { __zwNADelete(k30); } catch (e30d) {}
      }
    }
  } catch (e30m) {}
  if (!installed) {
    // 新 context（reset 后首快照）：快照先于页面脚本执行，本 context 尚无脚本自建
    // 全局——shim eval 自调用此刻登记的「上一文档」元素全局全部回收（否则跨站
    // 导航/重载会把旧页元素残留进新文档，且 id 撞车时遮蔽新页注册）。
    // slice37（NPO 收口）：枚举面 = __zwNAOwnKeys（wired：backing own keys——原
    // for-in globalThis 在 NPO 化后枚举不到安装值；quickjs：getOwnPropertyNames
    // globalThis own——较 slice36 的 for-in（仅可枚举）为安全方向放宽，多覆盖
    // 非可枚举 own 残留，快照先于页面脚本故无越界回收面）。
    var ks37 = typeof __zwNAOwnKeys === 'function' ? __zwNAOwnKeys() : [];
    for (var ki37 = 0; ki37 < ks37.length; ki37++) {
      var k = ks37[ki37];
      if (cur[k]) continue;
      var v0;
      try { v0 = __zwNAGet(k); } catch (e) { continue; }
      if (_isNA(v0)) {
        try { __zwNADelete(k); } catch (e) {}
      }
    }
  } else {
    for (var k in installed) {
      if (cur[k]) continue;
      var old;
      try { old = __zwNAGet(k); } catch (e) { continue; }
      if (_isNA(old)) {
        try { __zwNADelete(k); } catch (e) {}
      }
    }
  }
  if (typeof __zwInstallNamedAccess === 'function') __zwInstallNamedAccess();
  var next = {};
  for (var id in cur) {
    var v;
    try { v = __zwNAGet(id); } catch (e) { continue; }
    if (_isNA(v)) next[id] = true;
  }
  globalThis.__zwNamedAccessInstalled = next;
})()"#,
                );
                if let Ok(mut u) = page_url.lock() {
                    *u = url;
                }
                if url_changed {
                    let _ = sandbox.execute("__zw_reset_form_state && __zw_reset_form_state();");
                    // R3059：导航 → 清旧页 _hist_entries（pushState/hash-setter 残留），新页 location.href
                    // 读 page_url fallback（= 新文档 url），history.length=1。闭合 SPA-then-redirect stale。
                    let _ = sandbox.execute("__zw_reset_history && __zw_reset_history();");
                    // P1a gBCR path A：导航 → 旧页 handle 在新页无效，清 handle→selector map
                    // （apply 路径会在新页 createElement 时重新 merge）。
                    if let Ok(mut map) = handle_selector_map.lock() {
                        map.clear();
                    }
                }
                // t2-pb1 fix#21：快照应用完成回执（set_dom_snapshot 有界等待此应答）。
                if let Some(reply) = reply {
                    let _ = reply.send(());
                }
            }
            JsWorkerCommand::ResolveAsyncCallback { id, result } => {
                // P1b S1：跨线程 marshal 到此——在 JS worker 线程调 resolve_async_callback
                // （执行 shim 的 __zwResolveCallback resolve Promise）。
                sandbox.resolve_async_callback(&id, &result);
                async_callbacks_ready.store(true, Ordering::Release);
            }
            JsWorkerCommand::SetFetchHandler { handler } => {
                // P1b S3：注入 fetch handler（renderer 在 WebView 初始化后发送）。
                fetch_bridge.set_handler(handler);
            }
            JsWorkerCommand::SetVideoPlayers { registry, pump_clock } => {
                // M2c 后续：注册宿主桥回调族 + 注入 __zwVideoBridge JS 门面（镜像
                // browser tab_js_worker 同名分支——多进程路径媒体播放真值面）。
                // M3 切片 2（D4）：pump_clock 注入——桥 play 锚与 renderer 主循环
                // 泵 tick 同源（扩批 XXV 原点错位缺陷的 renderer 路径消除）。
                zero_webview::video_registry::register_video_bridge_callbacks(
                    &mut *sandbox,
                    registry,
                    None,
                    pump_clock,
                );
            }
            JsWorkerCommand::SetWebAudio { registry } => {
                // media-audio M3：注册 Web Audio 宿主桥（`__zwWA*` 回调族——多进程
                // 路径 AudioContext 最小面 NullSink 可观测，镜像 browser 路径）。
                zero_webview::webaudio_registry::register_webaudio_bridge_callbacks(&mut *sandbox, registry);
            }
            JsWorkerCommand::SetViewportHint { width, height } => {
                viewport_hint = (width, height);
                let guard = format!(
                    "if (typeof __zw_user_resize === 'function' && (globalThis.innerWidth !== {width} || globalThis.innerHeight !== {height})) __zw_user_resize({width}, {height});"
                );
                let _ = sandbox.execute(&guard);
            }
            JsWorkerCommand::ResetDocumentState { reply, seq: reset_seq } => {
                // t2-pb1 fix#11：导航复位清队——旧文档残留的命令随 context 重建一并丢弃
                // （real browser：新文档的任务队列不继承旧文档队列；被丢弃 Execute 的
                // reply 通道随之关闭，调用方即刻得到错误而非排队悬挂）。
                // t2-pb1 F1（首轮缺陷审查 2026-10-02）：清队按命令**提交代际**（seq）判定，
                // 不按「Reset 处理时刻」——Reset 滞留长臂之后时，复位提交**之后**入队的新
                // 文档快照/脚本/生命周期命令排在 Reset 之后，按时刻清队会把它们当残留丢弃
                // （连 reply 一起），新文档 JS 整体静默死亡（video 页 ready:false/videoCount:0
                // 证据吻合）。存活判定见 `survives_document_reset`；丢弃即 drop 整条命令，
                // 其 reply 发送端随之关闭（fix#21 等待方即刻放行）。
                while let Ok(cmd) = prio_rx.try_recv() {
                    if survives_document_reset(&cmd, reset_seq) {
                        retained_after_reset.push_back(cmd);
                    }
                }
                // fix#18：搁置中的普通命令同属复位时刻的旧队列，按同代际判定续派或丢弃。
                if let Some(cmd) = held_normal.take()
                    && survives_document_reset(&cmd, reset_seq)
                {
                    retained_after_reset.push_back(cmd);
                }
                while let Ok(cmd) = cmd_rx.try_recv() {
                    if survives_document_reset(&cmd, reset_seq) {
                        retained_after_reset.push_back(cmd);
                    }
                }
                // https://html.spec.whatwg.org/multipage/browsing-the-web.html#navigate
                // A cross-document navigation creates a new global object. Keeping the
                // renderer worker is an implementation detail, not page-visible state.
                sandbox.reset_context();
                // t8 返修（defect-r1 D1）：reset 竞态窗加固——主线程 `reset_document_state`
                // 已先清一次队列，但 worker 串行处理下，旧页脚本/timer 回调
                // （ResolveAsyncCallback 的 microtask checkpoint）可能在其之后、本 arm 之前
                // 继续写入；本 arm 执行时旧页执行已全部结束，此处再清一次，确保跨导航残留
                // mutation 不被新文档的 tick/drain apply（跨文档污染）。
                // https://html.spec.whatwg.org/multipage/browsing-the-web.html#navigate
                if let Ok(mut pending) = mutations.lock() {
                    pending.clear();
                }
                // js-dom R386：QuickJS context 丢弃即释放其对象——绑定线程局部
                // （NODE_OBJECTS 等 Persistent）持已释放对象引用（webview Drop R3334/R74
                // 同族悬垂），须随 context 重建清空；下一快照换代全量重 install。
                #[cfg(all(feature = "quickjs", not(feature = "v8")))]
                zero_engine::quickjs_dom_bindings::reset_quickjs_state();
                // js-dom R386：context 重建后全局工厂丢失，标记失效——下一快照换代
                // 全量重 install（V8 线程局部缓存仍属本 Isolate，复用安全）。
                native_installed = false;
                if raf_frame_driven_enabled() {
                    let _ = sandbox.execute("globalThis.__ZW_RAF_FRAME_DRIVEN = true;");
                }
                if let Err(e) = sandbox.execute(generate_js_dom_shim()) {
                    tracing::error!("JS DOM shim reinit failed after document reset: {e}");
                }
                // t2-pb1 fix#19：context 重建即重置宿主动态脚本旗标（见 bootstrap 处注释）。
                let _ = sandbox.execute("globalThis.__zwHostDynamicScripts = true;");
                async_callbacks_ready.store(false, Ordering::Release);
                let _ = reply.send(());
            }
            JsWorkerCommand::DispatchIndexedDbConnectionEvent {
                connection_id,
                old_version,
                new_version,
                reply,
            } => {
                let new_version = new_version.map_or_else(|| "null".to_string(), |version| version.to_string());
                let script = format!(
                    "globalThis.__zw_idb_connection_event && \
                     globalThis.__zw_idb_connection_event({connection_id}, {old_version}, {new_version});"
                );
                let result = sandbox.execute(&script).map(|_| ()).map_err(|error| error.to_string());
                let _ = reply.send(result);
            }
            JsWorkerCommand::Shutdown => {
                // js-dom R386：worker 退出前清原生绑定线程局部（镜像 webview Drop
                // R3334/R74——QuickJS Runtime/V8 Isolate 随沙箱销毁，线程局部残留
                // 指向已释放对象；本进程后续 worker 线程仍需干净 DOM 源）。
                #[cfg(feature = "v8")]
                zero_engine::dom_bindings::reset_native_state();
                #[cfg(all(feature = "quickjs", not(feature = "v8")))]
                zero_engine::quickjs_dom_bindings::reset_quickjs_state();
                break;
            }
        }
    }
}

/// R2923 fetch 完整化：生产 fetch handler——经 `zero_net::ResourceLoader` 发起真实 HTTP 请求，
/// 支持全方法（GET/POST/PUT/DELETE/PATCH/HEAD/OPTIONS）、请求头、请求体，返 [`FetchResponse`]（status/
/// status_text/headers/body）。renderer 进程直接联网（与 browser `tab_js_worker::default_fetch_handler`
/// 同实现）。GET 行为零回归（method 默认 GET、body=None）。
///
/// `FetchBridge::register` 在**子线程**调本 handler（等待加载器结果时阻塞子线程，非 JS worker），故 JS worker
/// 不在 fetch 期间冻结。Response 对象 spec-compliance 由 shim `_makeResponseFromWire` 在 JS 侧包装。
pub fn default_fetch_handler() -> FetchHandler {
    Arc::new(|req: &FetchRequest| {
        let method = match req.method.to_ascii_uppercase().as_str() {
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            "PATCH" => HttpMethod::Patch,
            "HEAD" => HttpMethod::Head,
            "OPTIONS" => HttpMethod::Options,
            _ => HttpMethod::Get,
        };
        let http_req = HttpRequest {
            method,
            url: req.url.clone(),
            headers: req.headers.clone(),
            // R3020：二进制 body 优先（Blob/FormData multipart 字节保真）；否则文本 body → 字节。
            body: req
                .body_bytes
                .clone()
                .or_else(|| req.body.as_ref().map(|b| b.as_bytes().to_vec())),
        };
        let resp = ResourceLoader::shared()
            .submit_http(http_req, FetchPriority::MEDIUM)
            .recv()
            .map_err(|_| "fetch loader worker exited".to_string())?
            .map_err(|e| format!("fetch send: {e}"))?;
        Ok(FetchResponse {
            status: resp.status_code,
            status_text: status_reason(resp.status_code).to_string(),
            headers: resp.headers,
            body: String::from_utf8_lossy(&resp.body).to_string(),
            // R3021：携原始字节——bridge 对非 UTF-8 body 经 byte-wire 传 JS（response.blob()/arrayBuffer() 保真）。
            body_bytes: Some(resp.body),
        })
    })
}

/// 常见 HTTP 状态码 → 标准原因短语（供 `response.statusText`）；未知 → "OK"。
fn status_reason(code: u16) -> &'static str {
    match code {
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "OK",
    }
}

fn execute_module_in_sandbox(
    sandbox: &mut dyn zero_script_sandbox::Sandbox,
    source: &str,
    url: &str,
    deps: &[(String, String)],
) -> Result<String, String> {
    let mut registry = ModuleRegistry::new();
    for (spec, src) in deps {
        registry.register(spec, src);
    }
    let prelude = build_module_runtime_prelude(&registry).map_err(|e| e.to_string())?;
    let transformed = compile_module_script(source, url, &registry).map_err(|e| e.to_string())?;
    let full = format!("{prelude}\n{transformed}");
    sandbox.execute(&full).map(|r| r.value).map_err(|e| e.to_string())
}

fn register_module_compile_callback(sandbox: &mut dyn zero_script_sandbox::Sandbox) {
    // 静态模块依赖由主线程 prefetch + collect_module_deps 经 IPC 加载；动态 import 经 ResourceLoader。
    let runtime_iifes: Arc<std::sync::Mutex<HashMap<String, String>>> = Arc::new(std::sync::Mutex::new(HashMap::new()));

    sandbox.register_callback(
        "__zw_compile_module",
        Box::new(move |args| {
            if args.is_empty() {
                return String::new();
            }
            let spec = &args[0];
            let parent = args.get(1).map(String::as_str).unwrap_or("about:blank");
            let url = zero_engine::resolve_document_url(parent, spec);

            if let Ok(cache) = runtime_iifes.lock() {
                if let Some(iife) = cache.get(&url) {
                    return iife.clone();
                }
                if let Some(iife) = cache.get(spec) {
                    return iife.clone();
                }
            }

            let fetch = |u: &str| -> Result<String, String> {
                let response = if zero_net::is_file_url(u) {
                    zero_net::HttpClient::new().get(u).map_err(|e| e.to_string())?
                } else {
                    ResourceLoader::shared()
                        .submit(ResourceRequest::get(u, FetchPriority::HIGH).with_destination("script"))
                        .recv()
                        .map_err(|_| "module loader worker exited".to_string())?
                        .map_err(|e| format!("module fetch: {e}"))?
                };
                Ok(String::from_utf8_lossy(&response.body).into_owned())
            };

            let mut registry = HashMap::new();
            let src = match fetch(&url) {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!("fetch module {url}: {e}");
                    return String::new();
                }
            };
            if let Err(e) = collect_module_deps(&fetch, &url, &src, &mut registry) {
                tracing::warn!("module deps {url}: {e}");
                return String::new();
            }
            let mut reg = ModuleRegistry::new();
            for (spec, body) in &registry {
                reg.register(spec, body);
            }
            let iife = match compile_dependency_iife(&url, &reg) {
                Ok(i) => i,
                Err(e) => {
                    tracing::warn!("compile module {url}: {e}");
                    return String::new();
                }
            };
            if let Ok(mut cache) = runtime_iifes.lock() {
                cache.insert(url, iife.clone());
            }
            iife
        }),
    );
}

/// 递归抓取模块依赖图（specifier URL → 源码）。
pub fn collect_module_deps(
    fetch: &dyn Fn(&str) -> Result<String, String>,
    entry_url: &str,
    source: &str,
    registry: &mut HashMap<String, String>,
) -> Result<(), String> {
    if registry.contains_key(entry_url) {
        return Ok(());
    }
    registry.insert(entry_url.to_string(), source.to_string());
    for spec in extract_module_import_specifiers(source) {
        let dep_url = zero_engine::resolve_document_url(entry_url, &spec);
        if !registry.contains_key(&dep_url) {
            let dep_src = fetch(&dep_url)?;
            collect_module_deps(fetch, &dep_url, &dep_src, registry)?;
        }
    }
    Ok(())
}

/// renderer 的 JS worker 实现统一脚本执行器契约（T4）。
impl zero_page_runtime::JsExecutor for RendererJsWorker {
    fn set_dom_snapshot(&self, html: &str, url: &str) {
        self.set_dom_snapshot(html, url)
    }
    fn execute_script_direct(&self, script: &str) -> Result<String, String> {
        self.execute_script_direct(script)
    }
    fn execute_module(&self, source: &str, url: &str, deps: &[(String, String)]) -> Result<String, String> {
        self.execute_module(source, url, deps)
    }
    fn mutations(&self) -> Arc<std::sync::Mutex<Vec<zero_engine::DomMutation>>> {
        self.mutations()
    }
    // t2-pb1 fix#13：用户交互路径走优先通道 + 有界等待（快照与脚本同通道 FIFO 防倒置；
    // 超时脚本滞留优先队列照常执行，结果晚至）。详见 trait 方法文档与分派环注释。
    fn set_dom_snapshot_priority(&self, html: &str, url: &str) {
        RendererJsWorker::set_dom_snapshot_priority(self, html, url)
    }
    fn execute_script_priority_bounded(&self, script: &str, timeout: std::time::Duration) -> Result<String, String> {
        let (reply_tx, reply_rx) = mpsc::channel();
        self.prio_tx
            .send(JsWorkerCommand::Execute {
                script: script.to_string(),
                reply: reply_tx,
                seq: self.cmd_seq.fetch_add(1, Ordering::Relaxed),
            })
            .map_err(|e| e.to_string())?;
        match reply_rx.recv_timeout(timeout) {
            Ok(result) => result,
            Err(_) => Err("js worker execute wait timeout".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// R342（siteopt t2-pb3nm）：clear_mutations_fresh 的 drain⇒bump 不变式钉——
    /// 队列清空同时 MUT_DRAIN_GEN 前进（查询视图增量链的「只增长+代际未变」前提
    /// 靠本方法维持；automation eval 旁路 bump 的 clear 曾致 callbacks.rs 切片竞窗
    /// renderer panic）。
    #[test]
    fn clear_mutations_fresh_bumps_drain_gen_r342() {
        let worker = RendererJsWorker::spawn(6142);
        worker
            .mutations()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(zero_engine::DomMutation::SetAttr {
                selector: "#x".to_string(),
                name: "a".to_string(),
                value: "b".to_string(),
            });
        assert_eq!(worker.mutations().lock().unwrap().len(), 1, "前置：队列 1 条");
        let gen_before = zero_engine::js_dom_bridge::MUT_DRAIN_GEN.load(std::sync::atomic::Ordering::Relaxed);
        worker.clear_mutations_fresh();
        let gen_after = zero_engine::js_dom_bridge::MUT_DRAIN_GEN.load(std::sync::atomic::Ordering::Relaxed);
        assert_eq!(worker.mutations().lock().unwrap().len(), 0, "清空语义");
        // R342 返修（PR #77 审查 D2）：MUT_DRAIN_GEN 是进程级全局原子，libtest 并行
        // 测试可在两 load 之间 bump——断言「必须前进」而非恰好 +1（免 flake；对
        // 「禁 bump」变异同等灵敏：不变式本体即前进）。
        assert!(gen_after > gen_before, "clear 必须 bump drain 代际（不变式本体）");
    }

    /// js-dom R386（DC-1 多进程生产路径）：RendererJsWorker 沙箱装原生 DOM 绑定——
    /// `__zw_native_*` 工厂在 worker context 可用，且读 `set_dom_snapshot` 快照 +
    /// 换代刷新。镜像 tab_js_worker `tab_js_worker_native_bindings_installed_r386`。
    #[test]
    fn renderer_js_worker_native_bindings_installed_r386() {
        let mut worker = RendererJsWorker::spawn(61);
        worker.set_dom_snapshot(
            "<html><body><div id='main' class='c'>t</div></body></html>",
            "about:blank",
        );
        assert_eq!(
            worker
                .execute_script_direct("typeof __zw_native_element_for_id")
                .unwrap(),
            "function",
            "worker 沙箱须装原生绑定工厂"
        );
        assert_eq!(
            worker
                .execute_script_direct("__zw_native_element_for_id('main').nodeType")
                .unwrap(),
            "1",
            "native 工厂读快照 DOM 元素"
        );
        assert_eq!(
            worker
                .execute_script_direct("__zw_native_element_for_id('main').tagName")
                .unwrap(),
            "DIV"
        );
        // 快照换代刷新 DOM 源。
        worker.set_dom_snapshot("<html><body><p id='next'>n</p></body></html>", "about:blank");
        assert_eq!(
            worker
                .execute_script_direct("__zw_native_element_for_id('next').tagName")
                .unwrap(),
            "P",
            "快照换代后 native DOM 源刷新"
        );
        // ResetDocumentState 销毁重建 context → 下一快照换代全量重 install。
        worker.reset_document_state();
        worker.set_dom_snapshot("<html><body><span id='post'>s</span></body></html>", "about:blank");
        assert_eq!(
            worker
                .execute_script_direct("__zw_native_element_for_id('post').tagName")
                .unwrap(),
            "SPAN",
            "reset_context 后下一快照重 install 原生绑定"
        );
        worker.shutdown();
    }

    // slice27（site-compat baidu 建议链 su 注册，2026-10-04）：**Window named access
    // 须随快照落地注册**。spec：文档树内带 id 的元素可作 `window.<id>` 裸标识符访问，
    // 且随文档换代更新。缺陷形态（活体 B/702336f67 首载实测）：shim 的 install 自调用
    // 仅发生在 shim eval（bootstrap 时 dom_html 恒空；reset 时读到的是**上一文档**的
    // 快照）——首载快照落地后无人注册 → `window.su`/`window.kw` 永不出现，baidu 建议
    // 链静默死。钉：快照落地后 id 元素立即可裸访问；页面脚本自建同名全局不被遮蔽
    //（spec：脚本 own property 位于 WindowProperties 之下）；文档换代后消失的 id 全局
    // 被回收（不留上一文档悬挂元素）。
    // https://html.spec.whatwg.org/multipage/window-object.html#named-access-on-the-window-object
    #[test]
    fn renderer_js_worker_named_access_registers_after_snapshot_s27() {
        let mut worker = RendererJsWorker::spawn(63);
        // 首载：快照落地即注册（修复前此处 undefined——install 只在 shim eval 跑过、
        // 当时 dom_html 尚空）。
        worker.set_dom_snapshot(
            "<html><body><div id='s27target'></div><span id='s27gone'></span>\
             <div id='s27-not-ident'></div></body></html>",
            "https://example.test/",
        );
        assert_eq!(
            worker.execute_script_direct("typeof globalThis.s27target").unwrap(),
            "object",
            "快照落地后 id 元素须可裸标识符访问（spec named access）"
        );
        // 负控制 1：页面脚本自建同名全局不被遮蔽（装后设置——own property 优先）。
        worker
            .execute_script_direct("globalThis.s27target = 'page-owned'")
            .unwrap();
        // 负控制 2：非标识符形态 id 不注册（spec 仅合法标识符可裸访问）。
        // slice28（RP-2 补强）：此前仅注释宣称、无断言（幽灵注释）——落断言。
        assert_eq!(
            worker
                .execute_script_direct("String(globalThis['s27-not-ident'] === undefined)")
                .unwrap(),
            "true",
            "非标识符 id（含连字符）不注册为全局"
        );
        // 换代：s27target id 保留、s27gone 移除、s27added 新增（slice28 RP-2 正向注册
        // 断言——换代新增 id 须随快照落地出现）、脚本自有全局保留。
        worker.set_dom_snapshot(
            "<html><body><div id='s27target'></div><div id='s27added'></div></body></html>",
            "https://example.test/",
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.s27target)").unwrap(),
            "page-owned",
            "脚本自建同名全局不被 named access 覆盖"
        );
        assert_eq!(
            worker.execute_script_direct("typeof globalThis.s27added").unwrap(),
            "object",
            "换代新增 id 元素随快照落地注册（空洞 A）"
        );
        assert_eq!(
            worker
                .execute_script_direct("String(globalThis.s27gone === undefined)")
                .unwrap(),
            "true",
            "换代后消失的 id 元素全局被回收（不留上一文档悬挂元素）"
        );
        // 二次换代到空 id 文档：注册面清空，重建 context 后（reset）仍随快照恢复。
        worker.reset_document_state();
        worker.set_dom_snapshot("<html><body></body></html>", "https://example.test/");
        assert_eq!(
            worker.execute_script_direct("typeof globalThis.s27target").unwrap(),
            "undefined",
            "reset 后无 id 文档不残留命名属性"
        );
        worker.shutdown();
    }

    // slice28（site-compat Window named access name 属性面，RP-1）：name 面注册钉。
    // spec named access 除 id 面外覆盖 embed/form/img/object 四元素的非空
    // name 内容属性（树序合并；iframe 属 navigable target name 源，委托 shim R139
    // contentWindow 注册，不入本面）；快照落地注册须同达 name 面，换代回收口径与
    // id 面一致（不留上一文档悬挂 name 全局）。修前形态：name-only 元素不可裸访问
    //（collect 仅收 [id]，RP-1 缺陷轮 S1）。
    // https://html.spec.whatwg.org/multipage/window-object.html#named-access-on-the-window-object
    #[test]
    fn renderer_js_worker_named_access_name_face_registers_s28() {
        let mut worker = RendererJsWorker::spawn(64);
        worker.set_dom_snapshot(
            "<html><body>\
             <form name='s28form'></form>\
             <input name='s28q'>\
             <img id='s28img' name='s28imgname'>\
             <iframe name='s28fr'></iframe>\
             </body></html>",
            "https://example.test/",
        );
        assert_eq!(
            worker.execute_script_direct("typeof globalThis.s28form").unwrap(),
            "object",
            "name 面注册：form name= 可裸访问（修前 undefined）"
        );
        assert_eq!(
            worker.execute_script_direct("typeof globalThis.s28imgname").unwrap(),
            "object",
            "name+id 并存元素两名字均可裸访问"
        );
        assert_eq!(
            worker.execute_script_direct("typeof globalThis.s28img").unwrap(),
            "object",
            "id 面不回归（同页 id 元素照常注册）"
        );
        // iframe name= 不入快照注册面（spec：iframe 属 child navigable target name 源
        // ——shim R139 已以 contentWindow 值注册 named iframe 全局；本面若以元素先占
        // 名会压制 R139 注册，值类型倒退）。断言形态=非元素（R139 触发时序不定：
        // load 派发早则已注册 contentWindow[object Object]、晚则 undefined——两者都
        // 是「未以元素占名」的正确边界形态）。
        assert_eq!(
            worker
                .execute_script_direct("String(!!(window.s28fr && window.s28fr.nodeType === 1))")
                .unwrap(),
            "false",
            "iframe name 不被本面以元素占名（R139 contentWindow 委托保持）"
        );
        assert_eq!(
            worker
                .execute_script_direct("String(globalThis.s28q === undefined)")
                .unwrap(),
            "true",
            "负面：input 非 name-able 元素不注册"
        );
        // 换代：s28form 移除 → name 全局回收；保留 name 元素持续可用。
        worker.set_dom_snapshot(
            "<html><body><img id='s28img' name='s28imgname'></body></html>",
            "https://example.test/",
        );
        assert_eq!(
            worker
                .execute_script_direct("String(globalThis.s28form === undefined)")
                .unwrap(),
            "true",
            "换代后消失的 name 元素全局被回收（不留悬挂 name 全局）"
        );
        assert_eq!(
            worker.execute_script_direct("typeof globalThis.s28imgname").unwrap(),
            "object",
            "换代保留的 name 元素全局持续可用"
        );
        worker.shutdown();
    }

    // slice30（RP-1 同名多命中 HTMLCollection 面，2026-10-04）：同名多命中时 spec 要求
    // 返 HTMLCollection——WindowProperties 命名属性取值算法：唯一 named object 返元素
    // 本身，多命中返以文档为根、含全部同名 named object 的 HTMLCollection（树序）。
    // named object = 文档树内带 id 元素 + embed/form/img/object 非空 name 元素（同元素
    // id/name 同值只算一个）。修前形态：getElementById/querySelector 均首命中 → 单元素
    //（slice28 缺陷轮 I-1 定性、slice29 testeff I-6 核实守卫空缺）。
    // https://html.spec.whatwg.org/multipage/window-object.html#named-access-on-the-window-object
    // https://webidl.spec.whatwg.org/#WindowProperties
    #[test]
    fn renderer_js_worker_named_access_multi_match_collection_s30() {
        let mut worker = RendererJsWorker::spawn(65);
        worker.set_dom_snapshot(
            "<html><body>\
             <img name='s30mm' data-mark='img-a'>\
             <img name='s30mm' data-mark='img-b'>\
             <div id='s30mm' data-mark='div-c'></div>\
             </body></html>",
            "https://example.test/",
        );
        // 多命中 → HTMLCollection（修前：id 面查无 → name 面 querySelector 首命中单元素）。
        assert_eq!(
            worker
                .execute_script_direct("String(window.s30mm instanceof window.HTMLCollection)")
                .unwrap(),
            "true",
            "同名多命中须返 HTMLCollection（修前单元素）"
        );
        assert_eq!(
            worker.execute_script_direct("String(window.s30mm.length)").unwrap(),
            "3",
            "集合长度 = named object 命中数（2 img name 面 + 1 div id 面）"
        );
        // 树序 + 成员 identity（data-mark 判别，不依赖 tag 启发）。
        assert_eq!(
            worker
                .execute_script_direct(
                    "window.s30mm[0].getAttribute('data-mark') + '|' + \
                     window.s30mm[1].getAttribute('data-mark') + '|' + \
                     window.s30mm[2].getAttribute('data-mark')"
                )
                .unwrap(),
            "img-a|img-b|div-c",
            "集合成员按树序排列（spec named objects tree order）"
        );
        // namedItem 接口成员可用（HTMLCollection 专有，id/name 首匹配）。
        assert_eq!(
            worker
                .execute_script_direct("String(window.s30mm.namedItem('s30mm') === window.s30mm[0])")
                .unwrap(),
            "true",
            "多命中集合具 namedItem 接口成员（首匹配）"
        );
        worker.shutdown();
    }

    // slice30 单命中回归守卫：唯一 named object 时仍返元素本身（RP-1 不改单命中语义
    // ——slice27/28 修复面保持）。id 面 + name 面各一例。
    #[test]
    fn renderer_js_worker_named_access_single_match_element_regression_s30() {
        let mut worker = RendererJsWorker::spawn(66);
        worker.set_dom_snapshot(
            "<html><body>\
             <div id='s30single'></div>\
             <form name='s30f'></form>\
             </body></html>",
            "https://example.test/",
        );
        assert_eq!(
            worker
                .execute_script_direct("String(window.s30single && window.s30single.nodeType === 1)")
                .unwrap(),
            "true",
            "单命中 id 元素仍为元素（回归守卫）"
        );
        assert_eq!(
            worker
                .execute_script_direct("String(window.s30f && window.s30f.nodeType === 1)")
                .unwrap(),
            "true",
            "单命中 name 元素仍为元素（回归守卫）"
        );
        assert_eq!(
            worker
                .execute_script_direct("String(window.s30single instanceof window.HTMLCollection)")
                .unwrap(),
            "false",
            "单命中不被误装为集合（元素形态保持）"
        );
        worker.shutdown();
    }

    // slice30 负控 + 换代回收：① 非 name-able 元素（input）多命中不注册（spec name 面
    // 仅 embed/form/img/object）；② name 面多命中 collection 形态；③ 换代多命中→单命中
    // 集合回收、元素重装；④ 名消失后 collection 全局同在回收口径（修前 collection 无
    // nodeType 标记、不在换代回收扫描内——悬挂全局）。
    #[test]
    fn renderer_js_worker_named_access_multi_negative_and_reclaim_s30() {
        let mut worker = RendererJsWorker::spawn(67);
        worker.set_dom_snapshot(
            "<html><body>\
             <input name='s30q'><input name='s30q'>\
             <img name='s30m1'><img name='s30m1'>\
             </body></html>",
            "https://example.test/",
        );
        assert_eq!(
            worker
                .execute_script_direct("String(window.s30q === undefined)")
                .unwrap(),
            "true",
            "负面：input 非 name-able，多命中也不注册"
        );
        assert_eq!(
            worker
                .execute_script_direct(
                    "String(window.s30m1 instanceof window.HTMLCollection && window.s30m1.length === 2)"
                )
                .unwrap(),
            "true",
            "name 面多命中（双 form/img 同名）为 collection"
        );
        // 换代：多命中 → 单命中：collection 回收、单元素重装（生命周期跟快照）。
        worker.set_dom_snapshot("<html><body><img name='s30m1'></body></html>", "https://example.test/");
        assert_eq!(
            worker
                .execute_script_direct("String(window.s30m1 && window.s30m1.nodeType === 1)")
                .unwrap(),
            "true",
            "多命中转单命中后为元素（集合不残留）"
        );
        // 再换代：名消失 → 全局回收（collection 与元素同口径，不留悬挂）。
        worker.set_dom_snapshot(
            "<html><body><img name='s30m1'><img name='s30m1'></body></html>",
            "https://example.test/",
        );
        worker.set_dom_snapshot("<html><body><div></div></body></html>", "https://example.test/");
        assert_eq!(
            worker
                .execute_script_direct("String(window.s30m1 === undefined)")
                .unwrap(),
            "true",
            "换代后消失的多命中名全局回收（collection 在回收口径内）"
        );
        worker.shutdown();
    }

    // slice32（RP-3）renderer 面 live 钉：重装（登记·回收链路等价形态：删全局 +
    // __zwInstallNamedAccess）产出的 live 集合同代内接棒维护——appendChild 即时 +1。
    // 原 PD（ledger 在快照落地时点安装的集合对同代脚本 childList 变异聋化——probe 实证
    // 集合存在、__zwHC 可读、注册表空）已由 slice33 Fix A 收口（`__zw_reset_pending_state`
    // 先于 slice27 重装，注册表存活，见 SetDomSnapshot 臂次序注释），快照臂面由 s34 钉
    // `renderer_js_worker_snapshot_collection_live_mutation_s34` 常驻直断；本钉保留
    // 手工重装形态的证立价值（修前无 liveSpec：重装后 append 仍恒 2）。
    #[test]
    fn renderer_js_worker_named_access_reinstall_collection_live_s32() {
        let mut worker = RendererJsWorker::spawn(68);
        worker.set_dom_snapshot(
            "<html><body><div id='s32lv'></div><div id='s32lv'></div></body></html>",
            "https://example.test/",
        );
        assert_eq!(
            worker.execute_script_direct("String(window.s32lv.length)").unwrap(),
            "2",
            "安装时点多命中集合（slice30 基线）"
        );
        // 重装：删全局 + install（renderer 换代登记·重装链路的同代缩影）→ live 集合。
        worker
            .execute_script_direct("delete globalThis.s32lv; __zwInstallNamedAccess();")
            .unwrap();
        worker
            .execute_script_direct(
                "window.__s32add = document.createElement('div');\
             window.__s32add.setAttribute('id', 's32lv');\
             document.body.appendChild(window.__s32add);",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(window.s32lv.length)").unwrap(),
            "3",
            "重装集合同代内 live：appendChild 即时 +1（修前静态恒 2）"
        );
        worker
            .execute_script_direct("document.body.removeChild(window.__s32add);")
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(window.s32lv.length)").unwrap(),
            "2",
            "removeChild 即时 -1（live）"
        );
        worker.shutdown();
    }

    // slice34（RP-3 修复面钉债收口，slice33 testeff 评审 S-1）：**聋集合探针常驻钉**——
    // 快照臂（ledger 路径）装出的多命中集合同代内 live。slice33 Fix A（SetDomSnapshot 臂
    // `__zw_reset_pending_state` 先于 slice27 重装）的回退形态（重装 → 清算旧序）把新装入
    // `_zwLiveCollections` 的集合随即抹除（part05.js reset 内 `_zwLiveCollections.length = 0`），
    // 注册表恒空、slice32 live 维护在 renderer 永不生效：appendChild 后 len 恒 2（聋）。
    // 本钉以「set_dom_snapshot 装集合 → 同代脚本 appendChild → len 即时 +1」直断次序修复面
    //——s32 钉的手工重装形态（delete + __zwInstallNamedAccess）绕开快照臂、Fix A 回退下
    // 仍绿，与本钉互补。RED→GREEN 归档：diag/evidence/slice34/red-mutation/fixA-reset-order/。
    #[test]
    fn renderer_js_worker_snapshot_collection_live_mutation_s34() {
        let mut worker = RendererJsWorker::spawn(69);
        worker.set_dom_snapshot(
            "<html><body><div id='s34lc'>a</div><div id='s34lc'>b</div></body></html>",
            "https://example.test/",
        );
        assert_eq!(
            worker.execute_script_direct("String(window.s34lc.length)").unwrap(),
            "2",
            "安装时点多命中集合（快照臂 ledger 路径，slice30 基线）"
        );
        // 同代变异：appendChild → 集合即时 +1（Fix A 回退时注册表被 reset 抹除 → 恒 2 聋）。
        worker
            .execute_script_direct(
                "window.__s34add = document.createElement('div');\
             window.__s34add.setAttribute('id', 's34lc');\
             document.body.appendChild(window.__s34add);",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(window.s34lc.length)").unwrap(),
            "3",
            "快照臂集合同代 live：appendChild 即时 +1（Fix A 回退时恒 2 聋）"
        );
        assert_eq!(
            worker
                .execute_script_direct("String(window.s34lc[2] === window.__s34add)")
                .unwrap(),
            "true",
            "集合新成员即变异节点（identity，非重建副本）"
        );
        worker
            .execute_script_direct("document.body.removeChild(window.__s34add);")
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(window.s34lc.length)").unwrap(),
            "2",
            "removeChild 即时 -1（live 双向）"
        );
        worker.shutdown();
    }

    // slice34（ghost 面 gate 资产化，timeline2 12/12 配方的确定性核心常驻化）：跨文档
    // 换代集合无残影——gen1 快照装集合 → 跨文档导航 gen2（同 id 同位重建、内容换代）→
    // 集合成员整体随新快照刷新（无 gen1 残留成员）→ 换代后同代 append 仍 live。对应
    // timeline2 每轮「同位重建 → 失效 → 重装 → append 后无残影」链；webview 双表残影面
    //（handle 残端复活）由 webview.rs evict 单测 s34 钉守，本钉守 renderer 集合视图面。
    // 修前 Fix A 回退时 append 腿红（重装出的集合被 reset 抹除注册 → 聋化恒 2）。
    #[test]
    fn renderer_js_worker_cross_document_collection_no_ghost_s34() {
        let mut worker = RendererJsWorker::spawn(70);
        worker.set_dom_snapshot(
            "<html><body><div id='s34cd' data-phase='gen1'>x</div>\
             <div id='s34cd' data-phase='gen1'>y</div></body></html>",
            "https://example.test/doc1",
        );
        assert_eq!(
            worker.execute_script_direct("String(window.s34cd.length)").unwrap(),
            "2",
            "gen1 安装时点集合"
        );
        // 跨文档导航（url 换代）：同 id 同位重建、内容 gen2 → 集合随新快照重装。
        worker.set_dom_snapshot(
            "<html><body><div id='s34cd' data-phase='gen2'>x2</div>\
             <div id='s34cd' data-phase='gen2'>y2</div></body></html>",
            "https://example.test/doc2",
        );
        assert_eq!(
            worker.execute_script_direct("String(window.s34cd.length)").unwrap(),
            "2",
            "换代后集合长度随新快照（不累积旧文档成员）"
        );
        assert_eq!(
            worker
                .execute_script_direct(
                    "window.s34cd[0].getAttribute('data-phase') + '|' + \
                     window.s34cd[1].getAttribute('data-phase')"
                )
                .unwrap(),
            "gen2|gen2",
            "集合成员整体换代（无 gen1 残影成员）"
        );
        // 换代后同代 append 仍 live（重装出的集合接棒 live 维护）。
        worker
            .execute_script_direct(
                "window.__s34cd3 = document.createElement('div');\
             window.__s34cd3.setAttribute('id', 's34cd');\
             window.__s34cd3.setAttribute('data-phase', 'gen3');\
             document.body.appendChild(window.__s34cd3);",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(window.s34cd.length)").unwrap(),
            "3",
            "换代后集合 live：append 即时 +1（Fix A 回退时聋化恒 2）"
        );
        worker.shutdown();
    }

    // slice36（RP-3 动态名面）renderer 钉：脚本后建名（createElement + id + appendChild）
    // 访问时解析——同 execute 内（_mo_notify 注册面即时）与跨 execute（宿主批应用后，
    // `__zwNADynElsStore` 账本 + `__zwNamedAccessInstalled` 换代登记存活）；移除注销。
    // 修前脚本后建名恒 undefined（slice32 边界钉申报面）。
    #[test]
    fn renderer_js_worker_named_access_dynamic_resolve_s36() {
        let mut worker = RendererJsWorker::spawn(71);
        worker.set_dom_snapshot("<html><body></body></html>", "https://example.test/");
        // 脚本后建：pending 节点 appendChild → _mo_notify 注册（spec named property
        // visibility 每次访问对当前文档树求值，shim 以注册表增量维护近似）。
        worker
            .execute_script_direct(
                "window.__s36d = document.createElement('div');\
             window.__s36d.setAttribute('id', 's36dyn');\
             document.body.appendChild(window.__s36d);",
            )
            .unwrap();
        assert_eq!(
            worker
                .execute_script_direct("String(window.s36dyn === window.__s36d)")
                .unwrap(),
            "true",
            "同 execute 内访问时解析（修前恒 undefined）"
        );
        // 跨 execute（宿主批应用后）：动态安装值不因换代账本缺席而悬挂。
        assert_eq!(
            worker
                .execute_script_direct("String(window.s36dyn && window.s36dyn.nodeType === 1)")
                .unwrap(),
            "true",
            "跨 execute 仍解析（账本 + 换代登记存活）"
        );
        // 移除注销：removeChild → 真离树 → 全局回收（脚本自有改写不追删口径不变）。
        worker
            .execute_script_direct("document.body.removeChild(window.__s36d);")
            .unwrap();
        assert_eq!(
            worker
                .execute_script_direct("String(window.s36dyn === undefined)")
                .unwrap(),
            "true",
            "移除后名注销（spec：离树 named object 退出 WindowProperties）"
        );
        worker.shutdown();
    }

    // slice40（RP-3 残余②）：换文档重置全链路钉——同 context 快照换代（文档 A → 文档
    // B）后：旧文档静态名与脚本动态名全部回收（spec：document 替换后 named property
    // 按新文档重算，不留上一文档悬挂）、新文档静态名重装、动态注册面对新文档照常
    // 工作（不聋化）。engine 面（part38 named_access_reset_registry_functional_s40）
    // 锁 shim 换代钩子；本钉锁 js_worker SetDomSnapshot 臂全序（回收 → 重装 → 登记）。
    // https://html.spec.whatwg.org/multipage/window-object.html#named-access-on-the-window-object
    #[test]
    fn renderer_js_worker_named_access_document_reset_s40() {
        let mut worker = RendererJsWorker::spawn(79);
        // 文档 A：静态 id 名 + 脚本动态名。
        worker.set_dom_snapshot(
            "<html><body><div id='da40'></div></body></html>",
            "https://example.test/a",
        );
        worker
            .execute_script_direct(
                "window.__da40d = document.createElement('div');\
             window.__da40d.setAttribute('id', 'dyna40');\
             document.body.appendChild(window.__da40d);",
            )
            .unwrap();
        assert_eq!(
            worker
                .execute_script_direct("String(window.dyna40 === window.__da40d)")
                .unwrap(),
            "true",
            "文档 A 动态名解析（基线）"
        );
        assert_eq!(
            worker
                .execute_script_direct("String(window.da40 && window.da40.nodeType === 1)")
                .unwrap(),
            "true",
            "文档 A 静态名解析（基线）"
        );
        // 文档 B：快照换代（同 context，SPA 形态）。
        worker.set_dom_snapshot(
            "<html><body><div id='db40'></div></body></html>",
            "https://example.test/b",
        );
        assert_eq!(
            worker
                .execute_script_direct("String(typeof window.dyna40 === 'undefined')")
                .unwrap(),
            "true",
            "旧文档动态名跨换代回收（不留悬挂）"
        );
        assert_eq!(
            worker
                .execute_script_direct("String(typeof window.da40 === 'undefined')")
                .unwrap(),
            "true",
            "旧文档静态名跨换代回收（新文档无同名 id）"
        );
        assert_eq!(
            worker
                .execute_script_direct("String(window.db40 && window.db40.nodeType === 1)")
                .unwrap(),
            "true",
            "新文档静态名重装"
        );
        worker
            .execute_script_direct(
                "window.__db40d = document.createElement('div');\
             window.__db40d.setAttribute('id', 'dynb40');\
             document.body.appendChild(window.__db40d);",
            )
            .unwrap();
        assert_eq!(
            worker
                .execute_script_direct("String(window.dynb40 === window.__db40d)")
                .unwrap(),
            "true",
            "新文档动态注册面照常工作（换文档不聋化）"
        );
        worker.shutdown();
    }

    // slice18（site-compat baidu 建议链 /sugrec，R-baidu8 接管收尾）：SetDomSnapshot 置位
    // `__zwHostOwnsDynamicScripts`——动态 src 脚本单点归属宿主管线，shim R387b 页面 fetch
    // 通道（cors 语义）整体跳过，动态脚本归 PendingDynamicScripts no-cors 取回。
    // 负控制：无本置位时 R387b 对跨域 classic script 误派 error、同源与宿主 tick 双执行
    //（engine part25 r387b 两段钉双向覆盖）；reset_context 销毁重建 context 后下一快照
    // 须重新置位。
    // https://html.spec.whatwg.org/multipage/scripting.html#fetch-a-classic-script
    #[test]
    fn renderer_js_worker_snapshot_declares_host_owns_dynamic_scripts() {
        let mut worker = RendererJsWorker::spawn(62);
        worker.set_dom_snapshot("<html><body><div id='a'></div></body></html>", "about:blank");
        assert_eq!(
            worker
                .execute_script_direct("String(globalThis.__zwHostOwnsDynamicScripts === true)")
                .unwrap(),
            "true",
            "SetDomSnapshot 后宿主动态脚本所有权标志置位"
        );
        // ResetDocumentState 销毁重建 context → 下一快照换代重新置位。
        worker.reset_document_state();
        assert_eq!(
            worker
                .execute_script_direct("String(globalThis.__zwHostOwnsDynamicScripts === true)")
                .unwrap(),
            "false",
            "reset_context 后标志随 context 销毁（未置位态回到 R387b 原行为）"
        );
        worker.set_dom_snapshot("<html><body><span id='b'></span></body></html>", "about:blank");
        assert_eq!(
            worker
                .execute_script_direct("String(globalThis.__zwHostOwnsDynamicScripts === true)")
                .unwrap(),
            "true",
            "换代后标志重新置位（先于页面脚本就位）"
        );
        worker.shutdown();
    }

    // slice18（site-compat baidu 建议链 /sugrec）：`handle_selector_map` 是**文档域**状态——
    // handle/listener store 随导航销毁重建，陈旧 handle→selector 条目使宿主元素事件
    //（R2944 script load/error 按 selector 反查 handle）经 `__zw_handle_for_selector`
    // find 有概率命中上一文档的死 handle 键 → listener store 无监听 → 派发落空。
    // webview 侧同表在文档换代时显式清空（`publish_forward_handle_map(None)`），renderer
    // 侧补齐同语义。已知边界：清表为必要非充分——活体（矩阵/复刻页）元素事件对目标
    // onload 的派送仍有不达成分（快照/apply 代际 × shim 视图一致性，slice18 汇报为
    // 独立待修缺口），本钉只锁「文档换代不留陈旧条目」这一不变式。
    // https://html.spec.whatwg.org/multipage/browsers.html#navigate
    #[test]
    fn renderer_js_worker_reset_document_state_clears_handle_selector_map() {
        use zero_engine::apply_mutations_to_html_with_handles;
        let mut worker = RendererJsWorker::spawn(64);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__el = document.createElement('div');\
                 document.body.appendChild(globalThis.__el);",
            )
            .unwrap();
        let recorded = worker.mutations().lock().unwrap().clone();
        let (_html1, handle_map) =
            apply_mutations_to_html_with_handles("<html><body></body></html>", &recorded).unwrap();
        assert_eq!(handle_map.len(), 1, "一个 createElement handle 映射");
        worker.handle_selector_map().lock().unwrap().extend(handle_map);
        assert_eq!(
            worker.handle_selector_map().lock().unwrap().len(),
            1,
            "merge 后表含本文档条目"
        );
        worker.reset_document_state();
        assert_eq!(
            worker.handle_selector_map().lock().unwrap().len(),
            0,
            "文档换代须清空 handle→selector 表（陈旧条目使宿主派发命中死键）"
        );
        worker.shutdown();
    }

    /// P1b S3 incr-d（镜像 browser tab_js_worker）：非阻塞 fetch 的 resolve 时机异步——
    /// 轮询 `globalThis.{key}` 直到非 undefined（或超时返当前值）。子线程抓取 → generous
    /// 超时下可靠（非 flaky）。
    fn wait_for_global(worker: &RendererJsWorker, key: &str, timeout_ms: u64) -> String {
        let start = std::time::Instant::now();
        let probe = format!("String(globalThis.{key})");
        loop {
            if let Ok(v) = worker.execute_script_direct(&probe)
                && v != "undefined"
            {
                return v;
            }
            if start.elapsed().as_millis() >= timeout_ms as u128 {
                return worker.execute_script_direct(&probe).unwrap_or_default();
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }

    // P1a Slice 2b：轮询直到 `globalThis.{key}` === want。observer tick 回调经 `_defer`
    // （queueMicrotask）在 execute 末尾 checkpoint drain；probe 本身触发 drain，故即便 drain
    // 跨 execute 也能在下一轮 probe 捕获。带超时兜底。
    fn wait_eq(worker: &RendererJsWorker, key: &str, want: &str, timeout_ms: u64) -> String {
        let start = std::time::Instant::now();
        let probe = format!("String(globalThis.{key})");
        loop {
            if let Ok(v) = worker.execute_script_direct(&probe)
                && v == want
            {
                return v;
            }
            if start.elapsed().as_millis() >= timeout_ms as u128 {
                return worker.execute_script_direct(&probe).unwrap_or_default();
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }

    #[test]
    fn renderer_js_worker_document_reset_drops_old_async_page_state() {
        let mut worker = RendererJsWorker::spawn(0);
        worker
            .execute_script_direct(
                "globalThis.__zw_pending.old_page_timer = function() {};\
                 globalThis.__zw_timer_trace = ['old'];\
                 globalThis.__zw_test_runner = { stale: true };\
                 globalThis.old_page_global = 42;\
                 let document_scoped_score = 1;",
            )
            .unwrap();

        worker.reset_document_state();

        assert_eq!(
            worker.execute_script_direct("typeof globalThis.__zw_pending").unwrap(),
            "object"
        );
        assert_eq!(
            worker
                .execute_script_direct("Object.keys(globalThis.__zw_pending).length")
                .unwrap(),
            "0"
        );
        assert_eq!(
            worker
                .execute_script_direct("typeof globalThis.__zw_timer_trace")
                .unwrap(),
            "undefined"
        );
        assert_eq!(
            worker
                .execute_script_direct("typeof globalThis.__zw_test_runner")
                .unwrap(),
            "undefined"
        );
        assert_eq!(
            worker
                .execute_script_direct("typeof globalThis.old_page_global")
                .unwrap(),
            "undefined"
        );
        assert_eq!(
            worker
                .execute_script_direct("let document_scoped_score = 1; document_scoped_score")
                .unwrap(),
            "1",
            "a refreshed document must be able to redeclare its top-level lexical bindings"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_async_resolver_delivers_cross_command() {
        // P1b S1（镜像 browser）：跨命令 marshal 验证。JS 建 pending Promise → 主线程经
        // async_resolver().resolve() 投递 ResolveAsyncCallback → worker FIFO 后于该命令的
        // 下一条 Execute 读到已 resolve 的 __result。
        let mut worker = RendererJsWorker::spawn(1);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        let init = worker.execute_script_direct(
            "new Promise(function(resolve){ globalThis.__zw_pending['r1'] = resolve; })
                 .then(function(v){ globalThis.__result = v; });",
        );
        assert!(init.is_ok(), "init script should succeed: {:?}", init.err());
        assert_eq!(
            worker.execute_script_direct("typeof globalThis.__result").unwrap(),
            "undefined"
        );
        let resolver = worker.async_resolver();
        resolver.resolve("r1", "delivered!");
        // FIFO：resolve 命令先于下一条 Execute 入队 → worker 先 resolve 后读。
        assert_eq!(
            worker.execute_script_direct("globalThis.__result").unwrap(),
            "delivered!"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_marks_resolved_async_callbacks_for_renderer_drain() {
        let mut worker = RendererJsWorker::spawn(2);
        assert!(!worker.take_pending_async_callbacks());

        worker.async_resolver().resolve("unknown-id", "value");
        let deadline = std::time::Instant::now() + Duration::from_secs(1);
        while !worker.take_pending_async_callbacks() && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }

        assert!(
            std::time::Instant::now() < deadline,
            "resolved callback should wake the renderer drain"
        );
        assert!(!worker.take_pending_async_callbacks(), "wake is consumed once");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_async_resolver_safe_for_unknown_id() {
        // 未知 id（无 pending resolver）经 shim 防御分支静默 no-op，不报错/不崩溃。
        let mut worker = RendererJsWorker::spawn(2);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker.async_resolver().resolve("nonexistent-id", "v");
        assert_eq!(worker.execute_script_direct("1 + 2").unwrap(), "3");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_async_resolver_usable_from_other_thread() {
        // AsyncResolver: Send（可 move 到子线程）+ Arc<Mutex> clone 跨线程工作
        // （仿真实 fetch host / 定时器跨线程完成）。
        let mut worker = RendererJsWorker::spawn(3);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "new Promise(function(r){ globalThis.__zw_pending['t1'] = r; })
                 .then(function(v){ globalThis.__result = v; });",
            )
            .unwrap();
        let resolver = worker.async_resolver();
        let handle = std::thread::spawn(move || {
            resolver.resolve("t1", "from-thread!");
        });
        handle.join().unwrap();
        assert_eq!(
            worker.execute_script_direct("globalThis.__result").unwrap(),
            "from-thread!"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_fetch_resolves_via_handler() {
        // P1b S3（镜像 browser）：fetch 经 __zw_fetch 回调 + handler 抓取 + resolver.resolve
        // 端到端。合成 handler 返 body:<url>；resolve Response 对象（r.text() 取 body）。
        let mut worker = RendererJsWorker::spawn(4);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker.set_fetch_handler(Arc::new(|req: &FetchRequest| {
            let mut response = FetchResponse::ok(format!("body:{}", req.url));
            response
                .headers
                .push(("Access-Control-Allow-Origin".to_string(), "*".to_string()));
            Ok(response)
        }));
        worker
            .execute_script_direct(
                "fetch('/hello').then(function(r){ return r.text(); })
                 .then(function(t){ globalThis.__result = t; });",
            )
            .unwrap();
        let r = wait_for_global(&worker, "__result", 1000);
        assert_eq!(r, "body:/hello");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_fetch_without_handler_resolves_error() {
        // 未注入 handler 时 __zw_fetch resolve 错误标记 → Response.ok=false（不悬挂）。
        let mut worker = RendererJsWorker::spawn(5);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct("fetch('/x').then(function(r){ globalThis.__result = r.ok ? 'OK' : 'ERR'; });")
            .unwrap();
        let r = wait_for_global(&worker, "__result", 1000);
        assert_eq!(r, "ERR");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_fetch_response_object_shape_and_json() {
        // P1b S3 incr-c（镜像 browser）：Response 对象 spec-compliance（ok/status/text()/json()）。
        let mut worker = RendererJsWorker::spawn(7);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker.set_fetch_handler(Arc::new(|_req: &FetchRequest| {
            let mut response = FetchResponse::ok("{\"key\":\"value\",\"n\":42}".to_string());
            response
                .headers
                .push(("Access-Control-Allow-Origin".to_string(), "*".to_string()));
            Ok(response)
        }));
        worker
            .execute_script_direct(
                "fetch('/j').then(function(r){
                   globalThis.__shape = r.ok + ':' + r.status;
                   return r.json();
                 }).then(function(o){ globalThis.__result = o.key + '/' + o.n; });",
            )
            .unwrap();
        assert_eq!(wait_for_global(&worker, "__shape", 1000), "true:200");
        assert_eq!(wait_for_global(&worker, "__result", 1000), "value/42");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_default_fetch_handler_real_http() {
        // P1b S3（镜像 browser）：生产 default_fetch_handler 经 net pool 真实 HTTP GET。
        // 本地 HTTP server（127.0.0.1）服务固定 body——不依赖外部网络。非阻塞：子线程 recv
        // （不冻结 JS worker）。全仓并发构建/字体加载会挤占 runtime，使用 15s 墙钟预算。
        use std::io::{Read, Write};
        use std::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind local server");
        let port = listener.local_addr().expect("local addr").port();
        let server = std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 1024];
                let _ = stream.read(&mut buf); // 丢弃请求行
                let body = "hello-from-renderer";
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(resp.as_bytes());
            }
        });
        let mut worker = RendererJsWorker::spawn(6);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker.set_fetch_handler(default_fetch_handler());
        let url = format!("http://127.0.0.1:{port}/data");
        worker
            .execute_script_direct(&format!(
                "fetch({:?}).then(function(r){{ return r.text(); }})
                 .then(function(t){{ globalThis.__result = t; }});",
                url
            ))
            .unwrap();
        let r = wait_for_global(&worker, "__result", 15_000);
        assert_eq!(r, "hello-from-renderer");
        worker.shutdown();
        let _ = server.join();
    }

    #[test]
    fn renderer_js_worker_settimeout_fires_after_real_delay() {
        // P1b S5（镜像 browser）：setTimeout 真实延迟。host __zw_setTimeout → 子线程 sleep
        // 后 resolve → __zwResolveCallback 调用回调。
        let mut worker = RendererJsWorker::spawn(8);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct("setTimeout(function(){ globalThis.__fired = 'yes'; }, 50);")
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("typeof globalThis.__fired").unwrap(),
            "undefined"
        );
        let r = wait_for_global(&worker, "__fired", 1000);
        assert_eq!(r, "yes");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_cleartimeout_cancels_callback() {
        // P1b S5（镜像 browser）：clearTimeout 删 pending → 回调永不触发。
        let mut worker = RendererJsWorker::spawn(9);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "var h = setTimeout(function(){ globalThis.__fired = 'yes'; }, 30);
                 clearTimeout(h);",
            )
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(150));
        assert_eq!(
            worker.execute_script_direct("typeof globalThis.__fired").unwrap(),
            "undefined"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_setinterval_repeats_then_clear() {
        // P1b S5（镜像 browser）：setInterval re-arm 重复触发；clearInterval 断链。
        let mut worker = RendererJsWorker::spawn(10);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__n = 0;
                 globalThis.__iv = setInterval(function(){ globalThis.__n++; }, 20);",
            )
            .unwrap();
        // 轮询等待 setInterval 至少触发 2 次（R2149：原固定 sleep 120ms 在 `make test` 全
        // workspace 并行负载下偶发 worker 线程饿死 → n1<2 false-fail；改条件式轮询
        // robust-to-starvation，1000ms 充分覆盖调度延迟）。镜像 browser 侧同名测试。
        let mut n1: u32 = 0;
        let poll_start = std::time::Instant::now();
        while n1 < 2 && poll_start.elapsed().as_millis() < 1000 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            n1 = worker
                .execute_script_direct("String(globalThis.__n)")
                .unwrap()
                .parse::<u32>()
                .unwrap_or(0);
        }
        assert!(n1 >= 2, "setInterval should repeat at least twice, got {n1}");
        worker.execute_script_direct("clearInterval(globalThis.__iv);").unwrap();
        // R3092：clearInterval 生效验证（robust-to-starvation）。原固定 120ms sleep + assert_eq!(n2,n1)
        // 在 make test 并行负载下撞上 clearInterval 前已调度的尾 tick（re-arm 模型：尾 setTimeout 回调
        // 已入队）→ n2=n1+1 false-fail。改轮询确认 n 收敛（3 次连续不变 = clearInterval 生效，容忍尾 tick）。
        let mut last = n1;
        let mut stable = 0u32;
        let conv = std::time::Instant::now();
        while conv.elapsed().as_millis() < 500 && stable < 3 {
            std::thread::sleep(std::time::Duration::from_millis(30));
            let n_now = worker
                .execute_script_direct("String(globalThis.__n)")
                .unwrap()
                .parse::<u32>()
                .unwrap_or(last);
            if n_now == last {
                stable += 1;
            } else {
                last = n_now;
                stable = 0;
            }
        }
        assert!(
            stable >= 3,
            "clearInterval 后 n 应收敛稳定（n1={n1}, last={last}, stable={stable}）"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_mutation_observer_childlist() {
        // P1b S2 incr1（镜像 browser）：MutationObserver handle-based，JS 创建子树。
        let mut worker = RendererJsWorker::spawn(11);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__seen = null;
                 var obs = new MutationObserver(function(records){
                   globalThis.__seen = records[0].type + ':' + records[0].addedNodes.length;
                 });
                 var root = document.createElement('div');
                 obs.observe(root, { childList: true });
                 root.appendChild(document.createElement('span'));",
            )
            .unwrap();
        let r = wait_for_global(&worker, "__seen", 1000);
        assert_eq!(r, "childList:1");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_mutation_observer_attributes() {
        // P1b S2 incr1（镜像 browser）：attributes 观测。
        let mut worker = RendererJsWorker::spawn(12);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__seen = null;
                 var obs = new MutationObserver(function(records){
                   globalThis.__seen = records[0].type + ':' + records[0].attributeName;
                 });
                 var el = document.createElement('div');
                 obs.observe(el, { attributes: true });
                 el.setAttribute('data-x', '1');",
            )
            .unwrap();
        let r = wait_for_global(&worker, "__seen", 1000);
        assert_eq!(r, "attributes:data-x");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_mutation_observer_disconnect() {
        // P1b S2 incr1（镜像 browser）：disconnect 后不再派发。
        let mut worker = RendererJsWorker::spawn(13);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__seen = null;
                 var obs = new MutationObserver(function(records){ globalThis.__seen = 'fired'; });
                 var el = document.createElement('div');
                 obs.observe(el, { attributes: true });
                 obs.disconnect();
                 el.setAttribute('data-x', '1');",
            )
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__seen)").unwrap(),
            "null"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_mutation_observer_existing_dom_attributes() {
        // P1b S2 incr2（镜像 browser）：观测现有 DOM attributes（selector 身份）。
        let mut worker = RendererJsWorker::spawn(14);
        worker.set_dom_snapshot("<html><body><div id='t'></div></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__seen = null;
                 var obs = new MutationObserver(function(records){
                   globalThis.__seen = records[0].type + ':' + records[0].attributeName;
                 });
                 var el = document.querySelector('#t');
                 obs.observe(el, { attributes: true });
                 el.setAttribute('data-x', '1');",
            )
            .unwrap();
        let r = wait_for_global(&worker, "__seen", 1000);
        assert_eq!(r, "attributes:data-x");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_mutation_observer_existing_dom_childlist() {
        // P1b S2 incr2（镜像 browser）：观测现有 DOM childList。
        let mut worker = RendererJsWorker::spawn(15);
        worker.set_dom_snapshot("<html><body><ul id='list'></ul></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__seen = null;
                 var obs = new MutationObserver(function(records){
                   globalThis.__seen = records[0].type + ':' + records[0].addedNodes.length;
                 });
                 var list = document.querySelector('#list');
                 obs.observe(list, { childList: true });
                 list.appendChild(document.createElement('li'));",
            )
            .unwrap();
        let r = wait_for_global(&worker, "__seen", 1000);
        assert_eq!(r, "childList:1");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_element_identity_stable_proxy() {
        // P1b S2 incr3（镜像 browser）：=== node identity——Proxy 缓存。
        let mut worker = RendererJsWorker::spawn(16);
        worker.set_dom_snapshot("<html><body><div id='t'></div></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "var a = document.querySelector('#t');
                 var b = document.querySelector('#t');
                 globalThis.__same = (a === b);
                 var c1 = document.createElement('div');
                 var c2 = document.createElement('div');
                 globalThis.__diff = (c1 !== c2);",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__same)").unwrap(),
            "true"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__diff)").unwrap(),
            "true"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_element_from_point_r2924() {
        // R2924 elementFromPoint：`document.elementFromPoint(x, y)` → 视口 (x,y) 命中的最深元素。
        // 注入合成 HitTestCache（root div + 子 p#inner），shim 经 `__zw_elementFromPoint` 求命中选择器
        // → `_wrapSelector` → `.tagName`（`_realTag` 经 `__zw_get_tag` 查 dom_html 真实 tag）。
        use std::sync::Arc;
        use zero_engine::{
            HitTestCache, HitTestCacheSnapshot, HitTestLayoutSnapshot, HitTestNodeSnapshot, node_id_from_u64,
        };
        let mut worker = RendererJsWorker::spawn(31);
        // dom_html 须含 #inner（`_realTag("#inner")` 经 `__zw_get_tag` 查它返 "p" → tagName "P"）。
        let html = "<html><body><div><p id='inner'>x</p></div></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        // 合成 HitTestCache：root div(0,0,800,600) + 子 p#inner(10,20,100,50)（坐标相对父内容区）。
        let id0 = node_id_from_u64(0); // Document 节点（非元素，落空时 hit_test_element 返 None）
        let id1 = node_id_from_u64(1); // div
        let id2 = node_id_from_u64(2); // p#inner
        let cache = HitTestCache::from_snapshot(HitTestCacheSnapshot {
            doc_root: id0,
            layout_root: HitTestLayoutSnapshot {
                node_id: Some(id1),
                x: 0.0,
                y: 0.0,
                width: 800.0,
                height: 600.0,
                reported: None,
                children: vec![HitTestLayoutSnapshot {
                    node_id: Some(id2),
                    x: 10.0,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    reported: None,
                    children: vec![],
                }],
            },
            nodes: vec![
                (
                    id1,
                    HitTestNodeSnapshot {
                        tag_name: "div".into(),
                        id: None,
                        class_name: None,
                        selector: "div".into(),
                        href: None,
                        src: None,
                    },
                ),
                (
                    id2,
                    HitTestNodeSnapshot {
                        tag_name: "p".into(),
                        id: Some("inner".into()),
                        class_name: None,
                        selector: "#inner".into(),
                        href: None,
                        src: None,
                    },
                ),
            ],
            parents: vec![(id2, id1)],
            hidden_nodes: Vec::new(),
            pe_none_nodes: Vec::new(),
        });
        *worker.element_from_point_cache().lock().unwrap() = Some(Arc::new(cache));
        worker
            .execute_script_direct(
                "var hit = document.elementFromPoint(50, 40);\
                 globalThis.__t1 = hit ? hit.tagName : 'null';\
                 var root = document.elementFromPoint(5, 5);\
                 globalThis.__t2 = root ? root.tagName : 'null';\
                 var miss = document.elementFromPoint(900, 900);\
                 globalThis.__t3 = miss ? miss.tagName : 'null';",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__t1)").unwrap(),
            "P",
            "(50,40) 命中最深子元素 p#inner"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__t2)").unwrap(),
            "DIV",
            "(5,5) 仅落在 root div 内（子外）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__t3)").unwrap(),
            "null",
            "(900,900) 落在所有元素外 → null（spec）"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_elements_from_point_r2925() {
        // R2925 elementsFromPoint（镜像 browser）：`document.elementsFromPoint(x, y)` → 视口 (x,y)
        // 处全部元素（绘制序，最前在前）。注入合成 HitTestCache（root div + 子 p#inner），断言命中栈。
        use std::sync::Arc;
        use zero_engine::{
            HitTestCache, HitTestCacheSnapshot, HitTestLayoutSnapshot, HitTestNodeSnapshot, node_id_from_u64,
        };
        let mut worker = RendererJsWorker::spawn(32);
        let html = "<html><body><div><p id='inner'>x</p></div></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        // 合成 HitTestCache：root div(0,0,800,600) + 子 p#inner(10,20,100,50)（坐标相对父内容区）。
        let id0 = node_id_from_u64(0);
        let id1 = node_id_from_u64(1);
        let id2 = node_id_from_u64(2);
        let cache = HitTestCache::from_snapshot(HitTestCacheSnapshot {
            doc_root: id0,
            layout_root: HitTestLayoutSnapshot {
                node_id: Some(id1),
                x: 0.0,
                y: 0.0,
                width: 800.0,
                height: 600.0,
                reported: None,
                children: vec![HitTestLayoutSnapshot {
                    node_id: Some(id2),
                    x: 10.0,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    reported: None,
                    children: vec![],
                }],
            },
            nodes: vec![
                (
                    id1,
                    HitTestNodeSnapshot {
                        tag_name: "div".into(),
                        id: None,
                        class_name: None,
                        selector: "div".into(),
                        href: None,
                        src: None,
                    },
                ),
                (
                    id2,
                    HitTestNodeSnapshot {
                        tag_name: "p".into(),
                        id: Some("inner".into()),
                        class_name: None,
                        selector: "#inner".into(),
                        href: None,
                        src: None,
                    },
                ),
            ],
            parents: vec![(id2, id1)],
            hidden_nodes: Vec::new(),
            pe_none_nodes: Vec::new(),
        });
        *worker.element_from_point_cache().lock().unwrap() = Some(Arc::new(cache));
        worker
            .execute_script_direct(
                "var list = document.elementsFromPoint(50, 40);\
                 globalThis.__n1 = list.length;\
                 globalThis.__f1 = list.length ? list[0].tagName : 'null';\
                 globalThis.__s1 = list.length > 1 ? list[1].tagName : 'null';\
                 globalThis.__n2 = document.elementsFromPoint(5, 5).length;\
                 globalThis.__n3 = document.elementsFromPoint(900, 900).length;",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__n1)").unwrap(),
            "2",
            "(50,40) 命中栈 = 2 元素（p#inner + root div）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__f1)").unwrap(),
            "P",
            "首元素 = p#inner（最前/最深）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__s1)").unwrap(),
            "DIV",
            "次元素 = root div（最后/最浅）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__n2)").unwrap(),
            "1",
            "(5,5) 仅 root div → 1 元素"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__n3)").unwrap(),
            "0",
            "(900,900) 落空 → 空数组"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_attach_shadow_r2926() {
        // R2926 attachShadow / shadowRoot（Tier 2 Web Components 地基，镜像 browser）：
        // `element.attachShadow({mode})` 返 ShadowRoot（nodeType 11 / nodeName '#shadow-root' /
        // mode / host）；shadowRoot getter（open 返 root / closed 返 null）；已挂载→抛 NotSupportedError；
        // 非法 mode→抛 TypeError。shadow root 复用 DocumentFragment handle（不渲染，fidelity defer）。
        let mut worker = RendererJsWorker::spawn(33);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "var host = document.createElement('div');\
                 var sr = host.attachShadow({ mode: 'open' });\
                 globalThis.__nt = sr.nodeType;\
                 globalThis.__nn = sr.nodeName;\
                 globalThis.__mode = sr.mode;\
                 globalThis.__host = (sr.host === host);\
                 globalThis.__sr = (host.shadowRoot === sr);\
                 var host2 = document.createElement('div');\
                 host2.attachShadow({ mode: 'closed' });\
                 globalThis.__closed = (host2.shadowRoot === null);\
                 var host3 = document.createElement('div');\
                 host3.attachShadow({ mode: 'open' });\
                 var threw = false;\
                 try { host3.attachShadow({ mode: 'open' }); } catch (e) { threw = true; }\
                 globalThis.__threw = threw;\
                 var host4 = document.createElement('div');\
                 var threwMode = false;\
                 try { host4.attachShadow({ mode: 'bad' }); } catch (e) { threwMode = true; }\
                 globalThis.__threwMode = threwMode;",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__nt)").unwrap(),
            "11",
            "ShadowRoot nodeType = 11"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__nn)").unwrap(),
            "#shadow-root",
            "ShadowRoot nodeName = '#shadow-root'"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__mode)").unwrap(),
            "open",
            "ShadowRoot.mode 反映 init.mode"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__host)").unwrap(),
            "true",
            "shadowRoot.host === 宿主元素（同一 proxy）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__sr)").unwrap(),
            "true",
            "element.shadowRoot（open）=== attachShadow 返回的 root"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__closed)").unwrap(),
            "true",
            "closed mode → element.shadowRoot === null（spec）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__threw)").unwrap(),
            "true",
            "已挂 shadow 的 host 再次 attachShadow → 抛异常（spec NotSupportedError）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__threwMode)").unwrap(),
            "true",
            "非法 mode → 抛异常（spec TypeError）"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_handle_children_registry_r2927() {
        // R2927 handle-children registry：容器 handle（shadow root / fragment）经 appendChild 记录子节点
        // → childNodes/firstChild/firstElementChild/childElementCount 可观察（旧实现 handle-only 恒返 []）。
        // shadow 构建模式（imperative custom element）自测解锁。removeChild 同步更新；fragment flatten
        // 进非容器父后清空（spec）。
        let mut worker = RendererJsWorker::spawn(34);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "var host = document.createElement('div');\
                 var sr = host.attachShadow({ mode: 'open' });\
                 var span1 = document.createElement('span');\
                 sr.appendChild(span1);\
                 globalThis.__cn1 = sr.childNodes.length;\
                 globalThis.__ff1 = (sr.firstChild === span1);\
                 globalThis.__fe1 = (sr.firstElementChild === span1);\
                 globalThis.__ec1 = sr.childElementCount;\
                 var tn = document.createTextNode('hi');\
                 sr.appendChild(tn);\
                 globalThis.__cn2 = sr.childNodes.length;\
                 globalThis.__ec2 = sr.childElementCount;\
                 sr.removeChild(span1);\
                 globalThis.__cn3 = sr.childNodes.length;\
                 var frag = document.createDocumentFragment();\
                 frag.appendChild(document.createElement('b'));\
                 globalThis.__fc = frag.childNodes.length;\
                 document.body.appendChild(frag);\
                 globalThis.__fc2 = frag.childNodes.length;",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__cn1)").unwrap(),
            "1",
            "appendChild 1 子 → childNodes 1"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ff1)").unwrap(),
            "true",
            "firstChild === span1"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__fe1)").unwrap(),
            "true",
            "firstElementChild === span1"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ec1)").unwrap(),
            "1",
            "childElementCount 1"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__cn2)").unwrap(),
            "2",
            "append textNode → childNodes 2"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ec2)").unwrap(),
            "1",
            "childElementCount 仍 1（text 过滤）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__cn3)").unwrap(),
            "1",
            "removeChild span1 → childNodes 1（剩 textNode）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__fc)").unwrap(),
            "1",
            "fragment appendChild → fragment.childNodes 1"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__fc2)").unwrap(),
            "0",
            "fragment flatten 进 body 后清空 → childNodes 0（spec）"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_handle_subtree_query_selector_r2928() {
        // R2928 handle 子树 querySelector/querySelectorAll——JS 端 registry 树搜索 + 客户端选择器匹配。
        // handle 元素（shadow root / createElement）无 sel，host `__zw_query_*_sub` 不可用 → registry DFS
        // + compound（tag/id/class/attr）/ 后代组合器 / 逗号列表 匹配。覆盖 Lit `sr.querySelector('#x')`
        // shadow 构建模式自测。querySelector 不穿透 shadow 边界（host.querySelector 查 light-DOM 子树）。
        let mut worker = RendererJsWorker::spawn(35);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "var host = document.createElement('div');\
                 var sr = host.attachShadow({ mode: 'open' });\
                 var wrap = document.createElement('div'); wrap.id = 'wrap'; wrap.className = 'outer';\
                 var btn = document.createElement('button'); btn.id = 'go';\
                 btn.className = 'btn primary'; btn.setAttribute('type', 'submit');\
                 var span = document.createElement('span'); span.className = 'label';\
                 wrap.appendChild(btn); wrap.appendChild(span); sr.appendChild(wrap);\
                 globalThis.__byTag = (sr.querySelector('button') === btn);\
                 globalThis.__byId = (sr.querySelector('#go') === btn);\
                 globalThis.__byClass = (sr.querySelector('.btn') === btn);\
                 globalThis.__compound = (sr.querySelector('button.btn') === btn);\
                 globalThis.__compound2 = (sr.querySelector('button.primary#go') === btn);\
                 globalThis.__desc = (sr.querySelector('div button') === btn);\
                 globalThis.__desc2 = (sr.querySelector('div span') === span);\
                 globalThis.__descClass = (sr.querySelector('div .primary') === btn);\
                 globalThis.__attr = (sr.querySelector('[type=submit]') === btn);\
                 globalThis.__comma = (sr.querySelector('button, span') === btn);\
                 globalThis.__allBtnSpan = sr.querySelectorAll('button, span').length;\
                 globalThis.__allClass = sr.querySelectorAll('.btn, .label').length;\
                 globalThis.__wildcard = (sr.querySelector('*') === wrap);\
                 globalThis.__nomatch = (sr.querySelector('input') === null);\
                 globalThis.__nomatchAll = sr.querySelectorAll('input').length;\
                 globalThis.__boundary = (host.querySelector('button') === null);\
                 var sec = document.createElement('section');\
                 var p = document.createElement('p'); p.id = 'p1';\
                 sec.appendChild(p);\
                 globalThis.__elQs = (sec.querySelector('#p1') === p);\
                 globalThis.__elQsTag = (sec.querySelector('p') === p);",
            )
            .unwrap();
        let cases = [
            ("__byTag", "true", "shadow querySelector('button') === btn（tag）"),
            ("__byId", "true", "shadow querySelector('#go') === btn（id）"),
            ("__byClass", "true", "shadow querySelector('.btn') === btn（class）"),
            ("__compound", "true", "shadow querySelector('button.btn')（复合）"),
            (
                "__compound2",
                "true",
                "shadow querySelector('button.primary#go')（多 class + id）",
            ),
            ("__desc", "true", "shadow querySelector('div button')（后代）"),
            ("__desc2", "true", "shadow querySelector('div span')（后代，另支）"),
            (
                "__descClass",
                "true",
                "shadow querySelector('div .primary')（后代 + class）",
            ),
            ("__attr", "true", "shadow querySelector('[type=submit]')（属性 =）"),
            (
                "__comma",
                "true",
                "shadow querySelector('button, span')（逗号列表，文档序首匹配）",
            ),
            (
                "__allBtnSpan",
                "2",
                "shadow querySelectorAll('button, span').length === 2",
            ),
            (
                "__allClass",
                "2",
                "shadow querySelectorAll('.btn, .label').length === 2",
            ),
            (
                "__wildcard",
                "true",
                "shadow querySelector('*') === wrap（通配，DFS 首元素）",
            ),
            ("__nomatch", "true", "shadow querySelector('input') === null（无匹配）"),
            (
                "__nomatchAll",
                "0",
                "shadow querySelectorAll('input').length === 0（无匹配）",
            ),
            (
                "__boundary",
                "true",
                "host.querySelector('button') === null（不穿透 shadow 边界）",
            ),
            (
                "__elQs",
                "true",
                "created element querySelector('#p1') === p（非容器 handle）",
            ),
            ("__elQsTag", "true", "created element querySelector('p') === p（tag）"),
        ];
        for (key, expect, msg) in cases {
            assert_eq!(
                worker
                    .execute_script_direct(&format!("String(globalThis.{key})"))
                    .unwrap(),
                expect,
                "{msg}"
            );
        }
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_range_mutation_ops_r2929() {
        // R2929 Range 变更操作：deleteContents/extractContents/insertNode/真实 cloneContents（既有 _makeRange
        // 这几项原为 defer / 仅文本）。经既有 mutation-emitting proxy（child.remove() → __zw_remove、
        // insertBefore/appendChild、cloneNode deep）真实变更，emit DomMutation。精确覆盖 start==end 元素容器的
        // offset 区间（selectNode/selectNodeContents 后），sel/handle 子均支持。
        // 验证：① fragment 内容（in-script 同步）；② apply_mutations_to_html → re-snapshot → 结构反映变更。
        use zero_engine::apply_mutations_to_html;
        let mut worker = RendererJsWorker::spawn(36);
        let html = "<html><body>\
                    <div id='cc'><span>A</span><span>B</span><span>C</span></div>\
                    <div id='ec'><span>A</span><span>B</span><span>C</span></div>\
                    <div id='dc'><span>A</span><span>B</span><span>C</span></div>\
                    <div id='ic'><p>0</p></div>\
                    </body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        worker
            .execute_script_direct(
                "var r1 = document.createRange(); r1.selectNodeContents(document.getElementById('cc'));\
                 var cc = r1.cloneContents();\
                 globalThis.__ccN = cc.childNodes.length;\
                 globalThis.__ccT = cc.childNodes[0].tagName;\
                 var r2 = document.createRange(); r2.selectNodeContents(document.getElementById('ec'));\
                 var ec = r2.extractContents();\
                 globalThis.__ecN = ec.childNodes.length;\
                 globalThis.__ecT = ec.childNodes[1].tagName;\
                 var r3 = document.createRange(); r3.selectNodeContents(document.getElementById('dc'));\
                 r3.deleteContents();\
                 var r4 = document.createRange(); r4.selectNodeContents(document.getElementById('ic'));\
                 r4.collapse(true);\
                 var ins = document.createElement('b');\
                 globalThis.__insRet = (r4.insertNode(ins) === ins);",
            )
            .unwrap();
        // ① fragment 内容（同步）
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ccN)").unwrap(),
            "3",
            "cloneContents fragment 3 子"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ccT)").unwrap(),
            "SPAN",
            "cloneContents [0].tagName SPAN（真实克隆非文本）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ecN)").unwrap(),
            "3",
            "extractContents fragment 3 子"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ecT)").unwrap(),
            "SPAN",
            "extractContents [1].tagName SPAN"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__insRet)").unwrap(),
            "true",
            "insertNode 返回插入的节点（spec）"
        );
        // ② apply mutations → re-snapshot → 结构反映变更（用 .children.length 测元素子数；`> *` 选择器
        // 在 host querySelectorAll 路径不稳定，避免）
        let recorded = worker.mutations().lock().unwrap().clone();
        let html1 = apply_mutations_to_html(html, &recorded).expect("apply range mutations");
        worker.set_dom_snapshot(&html1, "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__dcN = document.getElementById('dc').children.length;\
                 globalThis.__ecSrc = document.getElementById('ec').children.length;\
                 globalThis.__ccSrc = document.getElementById('cc').children.length;\
                 globalThis.__icN = document.getElementById('ic').children.length;\
                 globalThis.__icFirst = document.getElementById('ic').children[0].tagName;",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__dcN)").unwrap(),
            "0",
            "deleteContents 后 #dc 0 子"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ecSrc)").unwrap(),
            "0",
            "extractContents 后 #ec 0 子（内容移走）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ccSrc)").unwrap(),
            "3",
            "cloneContents 不改源 → #cc 仍 3 子"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__icN)").unwrap(),
            "2",
            "insertNode 后 #ic 2 子（b 插在 p 前）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__icFirst)").unwrap(),
            "B",
            "insertNode 插在首位 → #ic 首子为 B"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_range_surround_contents_r2930() {
        // R2930 Range surroundContents：selectNodeContents 包整元素内容（覆盖块延伸到容器末尾）→ clone 内容进
        // newParent + 逆序删原件 + appendChild newParent。headline 用法（rich-text wrap）。验证 apply 后结构：
        // 容器仅含 newParent（1 子），newParent 含克隆内容。
        use zero_engine::apply_mutations_to_html;
        let mut worker = RendererJsWorker::spawn(37);
        let html = "<html><body><div id='sc'><span>A</span><span>B</span></div></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        worker
            .execute_script_direct(
                "var r = document.createRange();\
                 r.selectNodeContents(document.getElementById('sc'));\
                 var wrap = document.createElement('div'); wrap.id = 'w';\
                 globalThis.__ret = r.surroundContents(wrap);",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ret)").unwrap(),
            "undefined",
            "surroundContents 返回 undefined（spec）"
        );
        let recorded = worker.mutations().lock().unwrap().clone();
        let html1 = apply_mutations_to_html(html, &recorded).expect("apply surround mutations");
        worker.set_dom_snapshot(&html1, "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__scN = document.getElementById('sc').children.length;\
                 globalThis.__scChildTag = document.getElementById('sc').children[0].tagName;\
                 globalThis.__scChildId = document.getElementById('sc').children[0].id;\
                 globalThis.__wN = document.getElementById('w').children.length;\
                 globalThis.__wFirst = document.getElementById('w').children[0].tagName;",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__scN)").unwrap(),
            "1",
            "surroundContents 后 #sc 仅 1 子（wrap）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__scChildTag)").unwrap(),
            "DIV",
            "#sc 唯一子为 wrap（DIV）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__scChildId)").unwrap(),
            "w",
            "#sc 子 id=w"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__wN)").unwrap(),
            "2",
            "wrap 含 2 个克隆子（原 span A/B）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__wFirst)").unwrap(),
            "SPAN",
            "wrap 首子为 SPAN"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_page_lifecycle_r2931() {
        // R2931 页面生命周期/分析簇：navigator.sendBeacon（卸载 beacon，accept-and-return-true）+
        // PageTransitionEvent 构造器 + pageshow 首次注册 _defer 派发（window + document 路径）。
        // PageTransitionEvent 是 modern non-createable interface，须通过构造器创建。
        let mut worker = RendererJsWorker::spawn(38);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__beacon1 = navigator.sendBeacon('/analytics', { x: 1 });\
                 globalThis.__beacon2 = navigator.sendBeacon();\
                 globalThis.__pt1 = new PageTransitionEvent('pageshow', { persisted: true }).persisted;\
                 globalThis.__pt2 = new PageTransitionEvent('pageshow').persisted;\
                 globalThis.__ev = (new PageTransitionEvent('').constructor === globalThis.PageTransitionEvent);\
                 globalThis.__ps = 'no'; globalThis.__ps2 = 'no';\
                 window.addEventListener('pageshow', function (e) {\
                   globalThis.__ps = e.type + ':' + String(e.persisted);\
                 });\
                 document.addEventListener('pageshow', function (e) {\
                   globalThis.__ps2 = e.type;\
                 });",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__beacon1)").unwrap(),
            "true",
            "sendBeacon(url, data) → true（accept）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__beacon2)").unwrap(),
            "false",
            "sendBeacon() 缺 url → false"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__pt1)").unwrap(),
            "true",
            "PageTransitionEvent persisted:true → persisted true"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__pt2)").unwrap(),
            "false",
            "PageTransitionEvent 默认 persisted false"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ev)").unwrap(),
            "true",
            "new PageTransitionEvent() 构造器匹配"
        );
        // pageshow 经首次注册 _defer 派发（execute 末 drain）；window + document 两路径 listener 均收。
        let ps = wait_eq(&worker, "__ps", "pageshow:false", 2000);
        assert_eq!(
            ps, "pageshow:false",
            "window pageshow listener 触发（type + persisted:false）"
        );
        let ps2 = wait_eq(&worker, "__ps2", "pageshow", 2000);
        assert_eq!(ps2, "pageshow", "document pageshow listener 亦触发（同一次 dispatch）");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_window_on_handlers_r2932() {
        // R2932 window IDL on-event handler：on* setter 经 _globalAddEventListener 注册为 listener（移除旧），
        // getter 返存储 fn；=null 移除。window.dispatchEvent 合成派发可触 handler。onpageshow 触发 R2931 派发。
        let mut worker = RendererJsWorker::spawn(39);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__c1 = 0; globalThis.__c2 = 0; globalThis.__id = 'no';\
                 globalThis.__null = 'no'; globalThis.__ps = 'no';\
                 function h1() { globalThis.__c1++; }\
                 function h2() { globalThis.__c2++; }\
                 window.onload = h1;\
                 globalThis.__id = (window.onload === h1);\
                 window.dispatchEvent(new Event('load'));\
                 window.onload = h2;\
                 window.dispatchEvent(new Event('load'));\
                 window.onload = null;\
                 globalThis.__null = (window.onload === null);\
                 window.dispatchEvent(new Event('load'));\
                 window.onpageshow = function (e) {\
                   globalThis.__ps = e.type + ':' + String(e.persisted);\
                 };",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__id)").unwrap(),
            "true",
            "window.onload = h1 → getter 返同一 fn（identity）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__c1)").unwrap(),
            "1",
            "dispatch load → h1 触发一次"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__c2)").unwrap(),
            "1",
            "重赋 onload=h2 → h2 触发一次（h1 已移除不再触发，c1 仍 1）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__null)").unwrap(),
            "true",
            "window.onload = null → getter 返 null（移除）"
        );
        // onpageshow 经 setter→_globalAddEventListener 触发 R2931 首次注册 _defer 派发。
        let ps = wait_eq(&worker, "__ps", "pageshow:false", 2000);
        assert_eq!(ps, "pageshow:false", "onpageshow setter 触发 pageshow 派发");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_element_on_handlers_r2933() {
        // R2933 element 级 IDL on-event handler：onclick/oninput setter 路由到 per-element listener store
        //（先于 set trap 末尾属性 fallthrough，否则 fn 被当字符串属性写）；getter 返存储 fn；=null 移除；
        // dispatchEvent 触发。parsed 元素（sel-based）+ created 元素（handle-based）均覆盖。
        let mut worker = RendererJsWorker::spawn(40);
        worker.set_dom_snapshot("<html><body><div id='d'></div></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "var d = document.getElementById('d');\
                 globalThis.__dc = 0; globalThis.__did = 'no'; globalThis.__dnull = 'no';\
                 function dh(e) { if (e && e.type === 'click') globalThis.__dc++; }\
                 d.onclick = dh;\
                 globalThis.__did = (d.onclick === dh);\
                 d.dispatchEvent(new Event('click'));\
                 d.onclick = null;\
                 globalThis.__dnull = (d.onclick === null);\
                 d.dispatchEvent(new Event('click'));\
                 var btn = document.createElement('button');\
                 globalThis.__bc = 0;\
                 function bh() { globalThis.__bc++; }\
                 btn.oninput = bh;\
                 globalThis.__bid = (btn.oninput === bh);\
                 btn.dispatchEvent(new Event('input'));",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__did)").unwrap(),
            "true",
            "parsed 元素 d.onclick = dh → getter 返同一 fn（identity）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__dc)").unwrap(),
            "1",
            "dispatchEvent click → onclick 触发一次（=null 后不再触发）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__dnull)").unwrap(),
            "true",
            "d.onclick = null → getter 返 null（移除）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__bid)").unwrap(),
            "true",
            "created 元素 btn.oninput = bh → getter 返同一 fn"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__bc)").unwrap(),
            "1",
            "created 元素 dispatchEvent input → oninput 触发"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_inline_html_handlers_r2934() {
        // R2934 inline HTML event handler：`<button onclick="...">` on* 属性编译为函数（new Function + with(this)
        // scope），on* getter 返编译 fn；dispatchEvent/click 触发；JS 设值覆盖 inline；无 inline 元素 onclick===null。
        let mut worker = RendererJsWorker::spawn(41);
        let html = "<html><body>\
                    <button id='b' onclick=\"globalThis.__inline='yes'\">\
                      <span id='s' onclick=\"globalThis.__tag=this.tagName\"></span>\
                    </button>\
                    <button id='b2' onclick=\"globalThis.__cfired='yes'\"></button>\
                    <div id='d'></div>\
                    </body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        worker
            .execute_script_direct(
                "var b = document.getElementById('b');\
                 globalThis.__typeof = typeof b.onclick;\
                 globalThis.__inline = 'no';\
                 b.dispatchEvent(new Event('click'));\
                 var s = document.getElementById('s');\
                 globalThis.__tag = 'no';\
                 s.dispatchEvent(new Event('click'));\
                 b.onclick = function () { globalThis.__js = 'yes'; };\
                 globalThis.__js = 'no';\
                 b.dispatchEvent(new Event('click'));\
                 globalThis.__cfired = 'no';\
                 document.getElementById('b2').click();\
                 var d = document.getElementById('d');\
                 globalThis.__disnull = (d.onclick === null);",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__typeof)").unwrap(),
            "function",
            "inline onclick → getter 返编译的 function"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__inline)").unwrap(),
            "yes",
            "dispatchEvent click → inline handler 触发"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__tag)").unwrap(),
            "SPAN",
            "inline handler with(this) scope → this.tagName === 'SPAN'"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__js)").unwrap(),
            "yes",
            "JS 覆盖 inline → JS handler 触发"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__cfired)").unwrap(),
            "yes",
            "click() 方法触发 inline handler"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__disnull)").unwrap(),
            "true",
            "无 inline 无 JS → onclick === null"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_inline_handler_ancestor_bubble_r2935() {
        // R2935 祖先 inline handler 冒泡触发：R2934 仅 target 阶段 ensure；补 capture/bubble 祖先阶段 →
        // <div onclick><button> 点 button 冒泡到 div 触发其 inline handler（this=祖先 currentTarget）。
        // 非 bubbles 事件不触发祖先 inline。
        let mut worker = RendererJsWorker::spawn(42);
        let html = "<html><body>\
                    <div id='outer' onclick=\"globalThis.__outer=this.id\">\
                      <div id='inner' onclick=\"globalThis.__inner=this.id\">\
                        <button id='btn'>x</button>\
                      </div>\
                    </div>\
                    <div id='o2' onclick=\"globalThis.__o2='yes'\"><button id='b2'>x</button></div>\
                    </body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__outer = 'no'; globalThis.__inner = 'no';\
                 document.getElementById('btn').dispatchEvent(new Event('click', { bubbles: true }));\
                 globalThis.__o2 = 'no';\
                 document.getElementById('b2').dispatchEvent(new Event('click', { bubbles: false }));",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__inner)").unwrap(),
            "inner",
            "祖先 inner inline handler 冒泡触发（this=inner）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__outer)").unwrap(),
            "outer",
            "祖父 outer inline handler 冒泡触发（this=outer）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__o2)").unwrap(),
            "no",
            "非 bubbles 事件不触发祖先 inline handler"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_clipboard_events_r2936() {
        // R2936 剪贴板事件：ClipboardEvent 构造器 + document.execCommand('copy') 派发 ClipboardEvent 到
        // document.activeElement（焦点元素，bubbles+cancelable）→ copy listener + oncopy handler + 冒泡到 window。
        // ClipboardEvent 是 modern non-createable interface，须通过构造器创建。
        let mut worker = RendererJsWorker::spawn(43);
        let html = "<html><body><input id='inp'></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__cd = new ClipboardEvent('copy', { clipboardData: 'dt' }).clipboardData;\
                 globalThis.__evc = (new ClipboardEvent('').constructor === globalThis.ClipboardEvent);\
                 var inp = document.getElementById('inp');\
                 inp.focus();\
                 globalThis.__copy = 'no';\
                 inp.addEventListener('copy', function (e) {\
                   globalThis.__copy = e.type + ':' + (e.constructor === globalThis.ClipboardEvent);\
                 });\
                 globalThis.__oncopy = 'no';\
                 inp.oncopy = function (e) { globalThis.__oncopy = e.type; };\
                 globalThis.__wcopy = 'no';\
                 window.addEventListener('copy', function (e) { globalThis.__wcopy = e.type; });\
                 document.execCommand('copy');",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__cd)").unwrap(),
            "dt",
            "ClipboardEvent clipboardData:'dt' → clipboardData 字段"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__evc)").unwrap(),
            "true",
            "new ClipboardEvent() 构造器匹配"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__copy)").unwrap(),
            "copy:true",
            "execCommand('copy') → 焦点元素 copy listener 触发（type + ClipboardEvent）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__oncopy)").unwrap(),
            "copy",
            "execCommand('copy') → oncopy handler 触发（R2933 on* 路由）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__wcopy)").unwrap(),
            "copy",
            "copy 事件冒泡到 window listener"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_drag_and_drop_r2937() {
        // R2937 Drag & Drop API：DataTransfer（setData/getData/types）+ DragEvent（extends MouseEvent +
        // dataTransfer）+ createEvent 注册。drag 事件类型经 generic addEventListener/ondrop（R2933）+ dispatchEvent
        // 触发。headless 无真拖拽源，但库 / drop handler 经合成 DragEvent + dataTransfer 读写 payload。
        let mut worker = RendererJsWorker::spawn(44);
        let html = "<html><body><div id='dz'></div></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        worker
            .execute_script_direct(
                "var dt = new DataTransfer();\
                 dt.setData('text/plain', 'hello');\
                 dt.setData('text/html', '<b>hi</b>');\
                 globalThis.__dt1 = dt.getData('text/plain');\
                 globalThis.__dt2 = dt.getData('text/html');\
                 globalThis.__dt3 = dt.getData('text/missing');\
                 globalThis.__types = dt.types.join(',');\
                 globalThis.__evc = (document.createEvent('DragEvent').constructor === globalThis.DragEvent);\
                 var dz = document.getElementById('dz');\
                 globalThis.__drop = 'no';\
                 dz.addEventListener('drop', function (e) {\
                   globalThis.__drop = e.type + ':' + e.dataTransfer.getData('text/plain') + ':'\
                     + (e.constructor === globalThis.DragEvent);\
                 });\
                 globalThis.__ondrop = 'no';\
                 dz.ondrop = function (e) { globalThis.__ondrop = e.type; };\
                 var dt2 = new DataTransfer();\
                 dt2.setData('text/plain', 'payload');\
                 dz.dispatchEvent(new DragEvent('drop', { dataTransfer: dt2, bubbles: true, cancelable: true }));",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__dt1)").unwrap(),
            "hello",
            "DataTransfer.setData/getData text/plain"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__dt2)").unwrap(),
            "<b>hi</b>",
            "DataTransfer.setData/getData text/html"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__dt3)").unwrap(),
            "",
            "getData 未设格式 → ''"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__types)").unwrap(),
            "text/plain,text/html",
            "DataTransfer.types（插入序）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__evc)").unwrap(),
            "true",
            "createEvent('DragEvent') 构造器匹配"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__drop)").unwrap(),
            "drop:payload:true",
            "dispatchEvent DragEvent('drop') → drop listener 触发（dataTransfer.getData + 构造器）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ondrop)").unwrap(),
            "drop",
            "ondrop handler 触发（R2933 on* 路由）"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_get_bounding_client_rect_real_rect() {
        // P1a gBCR path C：selector-identity 元素的 getBoundingClientRect 返真实 DOMRect。
        // shim `__zw_getBoundingClientRect(sel)` → handler fresh-parse dom_html → find_by_selector
        // → NodeId → 查 rect_snapshot。本测试用「同一 html fresh-parse」的 NodeId 填 snapshot
        // （模拟 renderer 主循环 render 后填充；NodeId 确定性由 engine 的
        // `test_node_id_determinism_across_fresh_parses` 保证 = 渲染管线会用同一 NodeId）。
        use zero_dom::parse_html;
        use zero_engine::{find_by_selector, node_id_to_u64};
        let mut worker = RendererJsWorker::spawn(18);
        let html = "<html><body><div id='t' style='width:100px;height:50px'>hi</div></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        // 填 snapshot：解析同一 html 取 #t 的 NodeId（= 渲染管线会用的 NodeId），插入其 rect。
        let doc = parse_html(html);
        let id_t = find_by_selector(&doc, "#t").expect("#t");
        let snap = worker.rect_snapshot();
        snap.lock()
            .unwrap()
            .insert(node_id_to_u64(id_t), (10.0, 20.0, 100.0, 50.0));
        // 读 gBCR：width/left/top 应反映 snapshot（rect 反映「上次 render」，此处 snapshot 即该 render）。
        worker
            .execute_script_direct(
                "var r = document.querySelector('#t').getBoundingClientRect();\
                 globalThis.__w = r.width; globalThis.__l = r.left; globalThis.__t = r.top;",
            )
            .unwrap();
        assert_eq!(worker.execute_script_direct("String(globalThis.__w)").unwrap(), "100");
        assert_eq!(worker.execute_script_direct("String(globalThis.__l)").unwrap(), "10");
        assert_eq!(worker.execute_script_direct("String(globalThis.__t)").unwrap(), "20");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_get_bounding_client_rect_empty_snapshot_zero() {
        // 零回归：snapshot 未填（无 render / 未命中）→ handler 返 None → shim 回落零 rect
        // （= 旧行为；作 reflow 触发器语义仍正确，返回值多被丢弃）。
        let mut worker = RendererJsWorker::spawn(19);
        worker.set_dom_snapshot("<html><body><div id='t'>hi</div></body></html>", "about:blank");
        worker
            .execute_script_direct("globalThis.__w = document.querySelector('#t').getBoundingClientRect().width;")
            .unwrap();
        assert_eq!(worker.execute_script_direct("String(globalThis.__w)").unwrap(), "0");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_get_bounding_client_rect_handle_identity_create_element() {
        // P1a gBCR path A：createElement 元素（JS 持 handle `__n{n}`，sel 空）的 getBoundingClientRect
        // 返真实 rect。流程模拟生产 apply 路径：脚本1 createElement+setId+append（记录 mutations）
        // → apply_mutations_to_html_with_handles 产出 handle→selector map → merge 进 worker 持久 map
        // → set_dom_snapshot 新 html（含已 append 元素）→ 填 snapshot → 脚本2 经 handle 测量返真实 rect。
        use zero_dom::parse_html;
        use zero_engine::{apply_mutations_to_html_with_handles, find_by_selector, node_id_to_u64};
        let mut worker = RendererJsWorker::spawn(22);
        let html0 = "<html><body id='b'></body></html>";
        worker.set_dom_snapshot(html0, "about:blank");
        // 脚本1：创建 div、设 id、append（handle 持于 globalThis.__el）。
        worker
            .execute_script_direct(
                "globalThis.__el = document.createElement('div');\
                 globalThis.__el.id = 'dyn';\
                 document.body.appendChild(globalThis.__el);",
            )
            .unwrap();
        // 模拟 apply_recorded_mutations：取记录的 mutations 应用到 html0，得新 html + handle→selector map。
        let recorded = worker.mutations().lock().unwrap().clone();
        let (html1, handle_map) = apply_mutations_to_html_with_handles(html0, &recorded).unwrap();
        assert!(
            html1.contains("<div id=\"dyn\">"),
            "createElement+setId+append 应产出 <div id=\"dyn\">，got: {html1}"
        );
        assert_eq!(handle_map.len(), 1, "唯一选择器映射应只含一个 createElement handle");
        assert_eq!(
            handle_map.values().next(),
            Some(&"#dyn".to_string()),
            "handle → #dyn（id 唯一）"
        );
        // merge map 进 worker 持久 map（= page_scripts::apply_recorded_mutations 的行为）。
        worker.handle_selector_map().lock().unwrap().extend(handle_map);
        // 更新 dom_html 为含已 append 元素的新 html（= 下一次 set_dom_snapshot）。
        worker.set_dom_snapshot(&html1, "about:blank");
        // 填 snapshot：fresh-parse html1 取 #dyn NodeId（= 渲染管线会用同一 NodeId）。
        let doc = parse_html(&html1);
        let id_dyn = find_by_selector(&doc, "#dyn").expect("#dyn in html1");
        worker
            .rect_snapshot()
            .lock()
            .unwrap()
            .insert(node_id_to_u64(id_dyn), (30.0, 40.0, 100.0, 50.0));
        // 脚本2：经 handle（sel 空）测量 → path A 解析 handle→#dyn→NodeId→snapshot rect。
        worker
            .execute_script_direct(
                "var r = globalThis.__el.getBoundingClientRect();\
                 globalThis.__w = r.width; globalThis.__l = r.left;",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__w)").unwrap(),
            "100",
            "handle-identity gBCR width 应反映 snapshot"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__l)").unwrap(),
            "30",
            "handle-identity gBCR left 应反映 snapshot"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_get_bounding_client_rect_handle_identity_ambiguous_tag() {
        // P1a gBCR path A + nth-child 结构路径：无 id/class 的 createElement 元素，文档已有同 tag
        // 元素（歧义）→ stable_selector 不唯一 → 回落 nth-child 结构路径 → 仍返真实 rect
        // （path A 限制①「tag-only 歧义→零 rect」的收尾）。
        use zero_dom::parse_html;
        use zero_engine::{apply_mutations_to_html_with_handles, find_by_selector, node_id_to_u64};
        let mut worker = RendererJsWorker::spawn(23);
        // body 已有一个 div（使新 div 的 "div" 选择器歧义）。
        let html0 = "<html><body id='b'><div>existing</div></body></html>";
        worker.set_dom_snapshot(html0, "about:blank");
        // 脚本1：创建无 id/class 的 div 并 append（歧义 tag）。
        worker
            .execute_script_direct(
                "globalThis.__el = document.createElement('div');\
                 document.body.appendChild(globalThis.__el);",
            )
            .unwrap();
        let recorded = worker.mutations().lock().unwrap().clone();
        let (html1, handle_map) = apply_mutations_to_html_with_handles(html0, &recorded).unwrap();
        assert_eq!(handle_map.len(), 1, "一个 createElement handle");
        let (handle, sel) = handle_map.iter().next().unwrap();
        let sel = sel.clone();
        assert!(
            sel.contains("nth-child"),
            "歧义 tag 应回落 nth-child 结构路径，got handle={handle} sel={sel}"
        );
        // merge + 更新 dom_html。
        worker.handle_selector_map().lock().unwrap().extend(handle_map);
        worker.set_dom_snapshot(&html1, "about:blank");
        // 用结构路径解析出该 handle 的 NodeId（= 渲染管线会用同一 NodeId），填 snapshot。
        let doc = parse_html(&html1);
        let id_el = find_by_selector(&doc, &sel).expect("结构路径须可解析");
        worker
            .rect_snapshot()
            .lock()
            .unwrap()
            .insert(node_id_to_u64(id_el), (5.0, 6.0, 80.0, 40.0));
        // 脚本2：经 handle 测量歧义元素 → 结构路径解析 → 真实 rect（非零）。
        worker
            .execute_script_direct(
                "var r = globalThis.__el.getBoundingClientRect();\
                 globalThis.__w = r.width; globalThis.__h = r.height;",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__w)").unwrap(),
            "80",
            "歧义 tag 经结构路径应返真实 width"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__h)").unwrap(),
            "40",
            "歧义 tag 经结构路径应返真实 height"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_select_value_read_and_setter() {
        // P1a select：<select>.value 读（选中 option 的 value）+ selectedIndex + option.selected，
        // + 编程设 select.value=x（SelectOption mutation，apply 后反映）。
        use zero_engine::apply_mutations_to_html;
        let mut worker = RendererJsWorker::spawn(24);
        // option b 默认 selected。
        let html = "<html><body><select id='s'>\
                    <option value='a'>A</option>\
                    <option value='b' selected>B</option>\
                    <option value='c'>C</option>\
                    </select></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        // 读：value='b'、selectedIndex=1、option b selected=true / a selected=false。
        worker
            .execute_script_direct(
                "var s = document.querySelector('#s');\
                 globalThis.__v = s.value;\
                 globalThis.__i = s.selectedIndex;\
                 globalThis.__sb = document.querySelector('#s > option:nth-of-type(2)').selected;\
                 globalThis.__sa = document.querySelector('#s > option:nth-of-type(1)').selected;",
            )
            .unwrap();
        assert_eq!(worker.execute_script_direct("String(globalThis.__v)").unwrap(), "b");
        assert_eq!(worker.execute_script_direct("String(globalThis.__i)").unwrap(), "1");
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__sb)").unwrap(),
            "true",
            "option b 应 selected"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__sa)").unwrap(),
            "false",
            "option a 应未 selected"
        );
        // 编程设 select.value='c'（记录 SelectOption mutation）→ apply → 反映。
        worker
            .execute_script_direct("document.querySelector('#s').value = 'c';")
            .unwrap();
        let recorded = worker.mutations().lock().unwrap().clone();
        let html1 = apply_mutations_to_html(html, &recorded).unwrap();
        worker.set_dom_snapshot(&html1, "about:blank");
        worker
            .execute_script_direct("globalThis.__v2 = document.querySelector('#s').value;")
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__v2)").unwrap(),
            "c",
            "setter 后 select.value 应为 c"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_query_selector_all_unique_identity() {
        // P1a querySelectorAll 唯一选择器：`querySelectorAll('option')` 每元素返唯一身份（nth-child
        // 结构路径），各 `.value`/`.selected` 读对（此前全返 "option"→全指向首个 option，读全错）。
        let mut worker = RendererJsWorker::spawn(25);
        let html = "<html><body><select id='s'>\
                    <option value='a'>A</option>\
                    <option value='b' selected>B</option>\
                    <option value='c'>C</option>\
                    </select></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        worker
            .execute_script_direct(
                "var opts = document.querySelectorAll('#s option');\
                 globalThis.__n = opts.length;\
                 globalThis.__vals = opts.map(function(o){ return o.value; }).join(',');\
                 globalThis.__sels = opts.map(function(o){ return o.selected ? '1':'0'; }).join(',');",
            )
            .unwrap();
        assert_eq!(worker.execute_script_direct("String(globalThis.__n)").unwrap(), "3");
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__vals)").unwrap(),
            "a,b,c",
            "各 option.value 应读对（唯一身份）"
        );
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__sels)").unwrap(),
            "0,1,0",
            "各 option.selected 应读对（b 选中）"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_select_options_collection() {
        // P1a select：`select.options` 集合（length/索引/value/selectedIndex）+
        // `select.selectedOptions`（选中 option 数组）。
        let mut worker = RendererJsWorker::spawn(26);
        let html = "<html><body><select id='s'>\
                    <option value='a'>A</option>\
                    <option value='b' selected>B</option>\
                    <option value='c'>C</option>\
                    </select></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        worker
            .execute_script_direct(
                "var s = document.querySelector('#s');\
                 globalThis.__len = s.options.length;\
                 globalThis.__v0 = s.options[0].value;\
                 globalThis.__v2 = s.options[2].value;\
                 globalThis.__ov = s.options.value;\
                 globalThis.__oi = s.options.selectedIndex;\
                 globalThis.__item = s.options.item(1).value;\
                 globalThis.__selN = s.selectedOptions.length;\
                 globalThis.__selV = s.selectedOptions[0].value;",
            )
            .unwrap();
        assert_eq!(worker.execute_script_direct("String(globalThis.__len)").unwrap(), "3");
        assert_eq!(worker.execute_script_direct("String(globalThis.__v0)").unwrap(), "a");
        assert_eq!(worker.execute_script_direct("String(globalThis.__v2)").unwrap(), "c");
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__ov)").unwrap(),
            "b",
            "options.value 应 = select.value（选中 b）"
        );
        assert_eq!(worker.execute_script_direct("String(globalThis.__oi)").unwrap(), "1");
        assert_eq!(worker.execute_script_direct("String(globalThis.__item)").unwrap(), "b");
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__selN)").unwrap(),
            "1",
            "selectedOptions 应含 1 个（b）"
        );
        assert_eq!(worker.execute_script_direct("String(globalThis.__selV)").unwrap(), "b");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_intersection_observer_intersecting() {
        // P1a Slice 2a：observe 视口内元素 → spec initial notification 派发，isIntersecting=true、
        // ratio≈1（target 完全在 viewport 内）。复用 gBCR：snapshot 填 #t rect，IO 经
        // `__zw_getBoundingClientRect(sel)` 算与 viewport 重叠（sel = `__zw_query_match('#t')` 返回值，
        // 与本测试 `find_by_selector(&doc, "#t")` 同 NodeId，见 gBCR test 既有验证）。
        use zero_dom::parse_html;
        use zero_engine::{find_by_selector, node_id_to_u64};
        let mut worker = RendererJsWorker::spawn(20);
        let html = "<html><body><div id='t' style='width:100px;height:50px'>hi</div></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        let doc = parse_html(html);
        let id_t = find_by_selector(&doc, "#t").expect("#t");
        worker
            .rect_snapshot()
            .lock()
            .unwrap()
            .insert(node_id_to_u64(id_t), (10.0, 20.0, 100.0, 50.0)); // 视口内（innerWidth/Height=1280/800）
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 var el = document.querySelector('#t');\
                 var obs = new IntersectionObserver(function(entries){\
                   var e = entries[0];\
                   globalThis.__seen = String(e.isIntersecting) + ':' + String(e.target === el)\
                     + ':' + (e.intersectionRatio > 0.99 ? 'full' : 'partial');\
                 });\
                 obs.observe(el);",
            )
            .unwrap();
        let r = wait_for_global(&worker, "__seen", 1000);
        assert_eq!(r, "true:true:full");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_intersection_observer_not_intersecting_initial() {
        // P1a Slice 2a：observe 视口外元素 → spec 仍派发 initial notification，isIntersecting=false、ratio=0。
        // （旧 shim 无 IO → `new IntersectionObserver` 抛 ReferenceError 中断脚本；本切片消除之。）
        use zero_dom::parse_html;
        use zero_engine::{find_by_selector, node_id_to_u64};
        let mut worker = RendererJsWorker::spawn(21);
        let html = "<html><body><div id='t' style='width:10px;height:10px'>hi</div></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        let doc = parse_html(html);
        let id_t = find_by_selector(&doc, "#t").expect("#t");
        worker
            .rect_snapshot()
            .lock()
            .unwrap()
            .insert(node_id_to_u64(id_t), (2000.0, 2000.0, 10.0, 10.0)); // 视口外（>1280/800）
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 var obs = new IntersectionObserver(function(entries){\
                   globalThis.__seen = String(entries[0].isIntersecting) + ':' + String(entries[0].intersectionRatio);\
                 });\
                 obs.observe(document.querySelector('#t'));",
            )
            .unwrap();
        let r = wait_for_global(&worker, "__seen", 1000);
        assert_eq!(r, "false:0");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_intersection_observer_disconnect() {
        // P1a Slice 2a：observe 后 disconnect（microtask 派发前）→ _targets 清空 → callback 不派发。
        use zero_dom::parse_html;
        use zero_engine::{find_by_selector, node_id_to_u64};
        let mut worker = RendererJsWorker::spawn(22);
        let html = "<html><body><div id='t' style='width:100px;height:50px'>hi</div></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        let doc = parse_html(html);
        let id_t = find_by_selector(&doc, "#t").expect("#t");
        worker
            .rect_snapshot()
            .lock()
            .unwrap()
            .insert(node_id_to_u64(id_t), (10.0, 20.0, 100.0, 50.0));
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 var obs = new IntersectionObserver(function(_entries){ globalThis.__seen = 'fired'; });\
                 obs.observe(document.querySelector('#t'));\
                 obs.disconnect();",
            )
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__seen)").unwrap(),
            "null"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_resize_observer_initial() {
        // P1a Slice 3：observe → spec initial notification 派发，contentRect.width/height 匹配
        // snapshot 尺寸（复用 gBCR：snapshot 填 #t rect，RO 经 `__zw_getBoundingClientRect(sel)` 读取）。
        // sel = `__zw_query_match('#t')` 返回值，与本测试 `find_by_selector(&doc, "#t")` 同 NodeId
        // （见 gBCR test 既有确定性验证）。
        use zero_dom::parse_html;
        use zero_engine::{find_by_selector, node_id_to_u64};
        let mut worker = RendererJsWorker::spawn(23);
        let html = "<html><body><div id='t' style='width:100px;height:50px'>hi</div></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        let doc = parse_html(html);
        let id_t = find_by_selector(&doc, "#t").expect("#t");
        worker
            .rect_snapshot()
            .lock()
            .unwrap()
            .insert(node_id_to_u64(id_t), (10.0, 20.0, 100.0, 50.0));
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 var el = document.querySelector('#t');\
                 var obs = new ResizeObserver(function(entries){\
                   var e = entries[0];\
                   globalThis.__seen = String(e.target === el) + ':' + String(e.contentRect.width)\
                     + 'x' + String(e.contentRect.height) + ':' + String(e.borderBoxSize[0].inlineSize);\
                 });\
                 obs.observe(el);",
            )
            .unwrap();
        let r = wait_for_global(&worker, "__seen", 1000);
        assert_eq!(r, "true:100x50:100");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_resize_observer_zero_fallback() {
        // P1a Slice 3：gBCR 未命中（snapshot 空）→ contentRect 为零，仍派发 initial notification（no-throw）。
        // （旧 shim 无 RO → `new ResizeObserver` 抛 ReferenceError 中断脚本；本切片消除之。）
        let mut worker = RendererJsWorker::spawn(24);
        worker.set_dom_snapshot("<html><body><div id='t'></div></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 var obs = new ResizeObserver(function(entries){\
                   globalThis.__seen = String(entries[0].contentRect.width) + 'x'\
                     + String(entries[0].contentRect.height);\
                 });\
                 obs.observe(document.querySelector('#t'));",
            )
            .unwrap();
        let r = wait_for_global(&worker, "__seen", 1000);
        assert_eq!(r, "0x0");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_resize_observer_disconnect() {
        // P1a Slice 3：observe 后 disconnect（microtask 派发前）→ _targets 清空 → callback 不派发。
        let mut worker = RendererJsWorker::spawn(25);
        worker.set_dom_snapshot("<html><body><div id='t'></div></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 var obs = new ResizeObserver(function(_entries){ globalThis.__seen = 'fired'; });\
                 obs.observe(document.querySelector('#t'));\
                 obs.disconnect();",
            )
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__seen)").unwrap(),
            "null"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_observer_tick_refires_on_size_change() {
        // P1a Slice 2b：observe（initial 派发，snapshot v1 100x50）→ 更新 snapshot（size 变化 200x80）
        // → `__zw_observers_tick` → RO size-diff 再次派发回调（__calls 1→2，__last 200x80）。
        // size 未变再 tick → 不派发（_lastSize 守，__calls 仍 2）。证明 host render-loop tick 机制。
        use zero_dom::parse_html;
        use zero_engine::{find_by_selector, node_id_to_u64};
        let mut worker = RendererJsWorker::spawn(26);
        let html = "<html><body><div id='t' style='width:100px;height:50px'>hi</div></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        let doc = parse_html(html);
        let id_t = find_by_selector(&doc, "#t").expect("#t");
        let snap = worker.rect_snapshot();
        snap.lock()
            .unwrap()
            .insert(node_id_to_u64(id_t), (0.0, 0.0, 100.0, 50.0)); // v1
        worker
            .execute_script_direct(
                "globalThis.__calls = 0;\
                 globalThis.__last = '';\
                 var obs = new ResizeObserver(function(entries){\
                   globalThis.__calls = (globalThis.__calls | 0) + 1;\
                   globalThis.__last = String(entries[0].contentRect.width) + 'x' + String(entries[0].contentRect.height);\
                 });\
                 obs.observe(document.querySelector('#t'));",
            )
            .unwrap();
        // initial 派发（microtask 在 execute 末尾 checkpoint drain；wait_eq probe 兜底）。
        assert_eq!(wait_eq(&worker, "__calls", "1", 1000), "1");
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__last)").unwrap(),
            "100x50"
        );
        // 更新 snapshot → size 变化 → tick 再次派发。
        snap.lock()
            .unwrap()
            .insert(node_id_to_u64(id_t), (0.0, 0.0, 200.0, 80.0));
        worker
            .execute_script_direct("if(globalThis.__zw_observers_tick)globalThis.__zw_observers_tick();")
            .unwrap();
        assert_eq!(wait_eq(&worker, "__calls", "2", 1000), "2");
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__last)").unwrap(),
            "200x80"
        );
        // size 未变再 tick → 不派发（_lastSize 守，__calls 仍 2）。
        worker
            .execute_script_direct("if(globalThis.__zw_observers_tick)globalThis.__zw_observers_tick();")
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(80));
        assert_eq!(worker.execute_script_direct("String(globalThis.__calls)").unwrap(), "2");
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_raf_tick_fires_frame_driven_callbacks() {
        // R2713b：renderer worker 帧驱动 rAF——__ZW_RAF_FRAME_DRIVEN=true 时 requestAnimationFrame
        // 注册延后到 __zw_raf_tick（renderer `tick_observers` 在 post-render 调 `__zw_raf_tick`；
        // 本测试 JS 侧直调验证 shim 在 renderer worker 上下文正确，env set_var 在并行测试下有竞态
        // 故 JS 侧注入 flag）。tick 前不 fire，tick 后按序 fire。
        let mut worker = RendererJsWorker::spawn(28);
        worker.set_dom_snapshot("<html><body><div id='t'>hi</div></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__ZW_RAF_FRAME_DRIVEN = true;\
                 globalThis.__count = 0;\
                 requestAnimationFrame(function(){ globalThis.__count++; });\
                 requestAnimationFrame(function(){ globalThis.__count++; });",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__count)").unwrap(),
            "0",
            "帧驱动：tick 前回调不应 fire"
        );
        worker
            .execute_script_direct("if(globalThis.__zw_raf_tick)globalThis.__zw_raf_tick(0);")
            .unwrap();
        assert_eq!(
            wait_eq(&worker, "__count", "2", 1000),
            "2",
            "tick 后按注册序 fire 两个回调"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_performance_now_available() {
        // R2769：renderer worker 上下文 performance.now() 可用（register_dom_callbacks 注册
        // __zw_performance_now，R2768 land）——证明 tick_observers 的
        // `__zw_raf_tick(performance.now())`（page_scripts.rs）真 ts 参数在 renderer 路径有效。
        let mut worker = RendererJsWorker::spawn(33);
        worker.set_dom_snapshot("<html><body></body></html>", "about:blank");
        assert_eq!(
            worker.execute_script_direct("typeof performance.now").unwrap(),
            "function",
            "performance.now 应为 function"
        );
        assert_eq!(
            worker.execute_script_direct("String(performance.now() >= 0)").unwrap(),
            "true",
            "performance.now() 非负（renderer 上下文真单调时钟）"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_intersection_observer_refires_on_threshold_cross() {
        // R2714：IO 持续跟踪（Slice 2b 已就绪——post-render `__zw_observers_tick` → IO `_schedule`
        // → `_crossed` threshold 越界 → 再派发）。observe（initial：target 在 root 外 ratio 0）→
        // 更新 snapshot（target 移入 root，ratio 跨 threshold 0.5）→ tick → 再派发（isIntersecting
        // false→true，__calls 1→2）。显式 root + threshold 0.5 使几何确定（不受 viewport 影响）。
        use zero_dom::parse_html;
        use zero_engine::{find_by_selector, node_id_to_u64};
        let mut worker = RendererJsWorker::spawn(29);
        let html = "<html><body><div id='root'><div id='t'>hi</div></div></body></html>";
        worker.set_dom_snapshot(html, "about:blank");
        let doc = parse_html(html);
        let id_root = find_by_selector(&doc, "#root").expect("#root");
        let id_t = find_by_selector(&doc, "#t").expect("#t");
        let snap = worker.rect_snapshot();
        // v1：root 200x200，target 在 root 外（1000,1000）→ ratio 0、isIntersecting false。
        snap.lock()
            .unwrap()
            .insert(node_id_to_u64(id_root), (0.0, 0.0, 200.0, 200.0));
        snap.lock()
            .unwrap()
            .insert(node_id_to_u64(id_t), (1000.0, 1000.0, 100.0, 100.0));
        worker
            .execute_script_direct(
                "globalThis.__calls = 0;\
                 globalThis.__intersecting = null;\
                 var obs = new IntersectionObserver(function(entries){\
                   globalThis.__calls = (globalThis.__calls | 0) + 1;\
                   globalThis.__intersecting = String(entries[0].isIntersecting);\
                 }, { root: document.querySelector('#root'), threshold: 0.5 });\
                 obs.observe(document.querySelector('#t'));",
            )
            .unwrap();
        // initial 派发（ratio 0，isIntersecting false）。
        assert_eq!(wait_eq(&worker, "__calls", "1", 1000), "1");
        assert_eq!(
            worker
                .execute_script_direct("String(globalThis.__intersecting)")
                .unwrap(),
            "false",
            "initial：target 在 root 外 → isIntersecting false"
        );
        // v2：target 移入 root（10,10）→ ratio 1.0 跨 threshold 0.5 → tick 再派发。
        snap.lock()
            .unwrap()
            .insert(node_id_to_u64(id_t), (10.0, 10.0, 100.0, 100.0));
        worker
            .execute_script_direct("if(globalThis.__zw_observers_tick)globalThis.__zw_observers_tick();")
            .unwrap();
        assert_eq!(wait_eq(&worker, "__calls", "2", 1000), "2");
        assert_eq!(
            worker
                .execute_script_direct("String(globalThis.__intersecting)")
                .unwrap(),
            "true",
            "tick 后 target 移入 root → isIntersecting true"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_form_text_input_updates_value_and_fires_input() {
        // P1a form input：`__zw_text_input(sel, ch)` 对 input 元素 append char 到 `.value`（缓存，
        // listener 立即可见新值）+ 派发 'input' 事件。`.value` lazy-init 自 value 属性（"ab"），
        // 注入 "c" → "abc"，input listener 读 `el.value` 见 "abc"（不滞后 mutation-apply）。
        // 非 input/textarea 目标 → no-op（不派发 input）。
        let mut worker = RendererJsWorker::spawn(27);
        worker.set_dom_snapshot("<html><body><input id='i' value='ab'></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 var el = document.querySelector('#i');\
                 el.addEventListener('input', function(_e){ globalThis.__seen = 'input:' + el.value; });\
                 __zw_text_input('#i', 'c');",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__seen)").unwrap(),
            "input:abc"
        );
        // 第二次注入 "d" → "abcd"（缓存跨 execute 存活，多键 typing 成立）。
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 __zw_text_input('#i', 'd');",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__seen)").unwrap(),
            "input:abcd"
        );
        // 非 input 目标（body）→ no-op，无 input 派发。
        worker
            .execute_script_direct(
                "globalThis.__seen2 = 'unchanged';\
                 __zw_text_input('body', 'x');",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__seen2)").unwrap(),
            "unchanged"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_form_textarea_newline_via_text_input() {
        // P1a form input：textarea 的 Enter 经 host 路由为 `__zw_text_input('#ta', '\n')`
        //（main.rs handle_keyboard_event：textarea Enter → 换行，非 submit）。验证 '\n' append 到
        // textarea value + 派发 'input'。修复前 textarea Enter 为 no-op（多行输入断裂）。
        let mut worker = RendererJsWorker::spawn(31);
        worker.set_dom_snapshot(
            "<html><body><textarea id='ta'>ab</textarea></body></html>",
            "about:blank",
        );
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 var el = document.querySelector('#ta');\
                 el.addEventListener('input', function(_e){ globalThis.__seen = 'input:' + el.value; });\
                 __zw_text_input('#ta', '\\n');",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__seen)").unwrap(),
            "input:ab\n"
        );
        // 再加 'c' → "ab\nc"（缓存跨 execute 存活，多行 typing 成立）。
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 __zw_text_input('#ta', 'c');",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__seen)").unwrap(),
            "input:ab\nc"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_form_text_delete_removes_last_char_and_fires_input() {
        // P1a form input 编辑互补：`__zw_text_delete(sel)` 删 value 末字符 + 派发 'input'。
        // "abcd" → backspace → "abc"，listener 见新值。空值 backspace → 无变化不派发（同 real browser）。
        let mut worker = RendererJsWorker::spawn(28);
        worker.set_dom_snapshot("<html><body><input id='i' value='abcd'></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 var el = document.querySelector('#i');\
                 el.addEventListener('input', function(_e){ globalThis.__seen = 'input:' + el.value; });\
                 __zw_text_delete('#i');",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__seen)").unwrap(),
            "input:abc"
        );
        // 再删 → "ab"（多键成立）。
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 __zw_text_delete('#i');",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__seen)").unwrap(),
            "input:ab"
        );
        // 删到空后再删 → 无 input 派发（__seen 保持 null）。
        worker.execute_script_direct("__zw_text_delete('#i');").unwrap(); // "a"
        worker.execute_script_direct("__zw_text_delete('#i');").unwrap(); // ""
        worker
            .execute_script_direct("globalThis.__seen = 'sentinel'; __zw_text_delete('#i');")
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__seen)").unwrap(),
            "sentinel"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_form_text_edit_respects_selection() {
        let mut worker = RendererJsWorker::spawn(32);
        worker.set_dom_snapshot("<html><body><input id='i' value='abcd'></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "var el = document.querySelector('#i');\
                 el.setSelectionRange(1, 3);\
                 __zw_text_input('#i', '中');\
                 globalThis.__edit = el.value + '|' + el.selectionStart + ':' + el.selectionEnd;",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__edit)").unwrap(),
            "a中d|2:2"
        );

        worker
            .execute_script_direct(
                "var el = document.querySelector('#i');\
                 el.setSelectionRange(1, 1);\
                 __zw_text_delete('#i');\
                 globalThis.__delete = el.value + '|' + el.selectionStart + ':' + el.selectionEnd;",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__delete)").unwrap(),
            "中d|0:0"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_form_backspace_deletes_one_unicode_scalar() {
        let mut worker = RendererJsWorker::spawn(33);
        worker.set_dom_snapshot("<html><body><input id='i' value='A😀中'></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "var el = document.querySelector('#i');\
                 el.setSelectionRange(3, 3);\
                 __zw_text_delete('#i');\
                 globalThis.__unicode = el.value + '|' + el.selectionStart + ':' + el.selectionEnd;",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__unicode)").unwrap(),
            "A中|1:1"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_form_submit_dispatches_submit_event() {
        // P1a form submit：apply_submit_on_enter 经 script_dispatch_dom_event(form_sel,"submit")
        // → 即 `__zw_dispatch_event(form_sel, 'submit', null)`。本 driving test 验证 submit 事件
        // 经 shim 派发命中 form 的 submit listener（form-resolution 由 engine 单测覆盖）。
        let mut worker = RendererJsWorker::spawn(29);
        worker.set_dom_snapshot(
            "<html><body><form id='f'><input id='i'></form></body></html>",
            "about:blank",
        );
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 document.querySelector('#f').addEventListener('submit', function(_e){\
                   globalThis.__seen = 'submit-fired';\
                 });\
                 __zw_dispatch_event('#f', 'submit', null);",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__seen)").unwrap(),
            "submit-fired"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_checkbox_checked_reflection_and_change_dispatch() {
        // P1a checkbox：`el.checked` getter 经 `__zw_has_attr` 反映 boolean 属性存在性；
        // change 事件经 shim 派发命中 listener（toggle 由 host `apply_toggle_checkbox` 覆盖，
        // engine 单测覆盖 RemoveAttr/has_attribute/is_checkbox）。
        let mut worker = RendererJsWorker::spawn(30);
        worker.set_dom_snapshot(
            "<html><body><input id='on' type='checkbox' checked><input id='off' type='checkbox'></body></html>",
            "about:blank",
        );
        // el.checked 反映存在性。
        assert_eq!(
            worker
                .execute_script_direct("String(document.querySelector('#on').checked)")
                .unwrap(),
            "true"
        );
        assert_eq!(
            worker
                .execute_script_direct("String(document.querySelector('#off').checked)")
                .unwrap(),
            "false"
        );
        // change 派发命中 listener（e.target.checked 读当前状态）。
        worker
            .execute_script_direct(
                "globalThis.__seen = null;\
                 document.querySelector('#off').addEventListener('change', function(e){\
                   globalThis.__seen = 'change:' + String(e.target.checked);\
                 });\
                 __zw_dispatch_event('#off', 'change', null);",
            )
            .unwrap();
        assert_eq!(
            worker.execute_script_direct("String(globalThis.__seen)").unwrap(),
            "change:false"
        );
        worker.shutdown();
    }

    #[test]
    fn renderer_js_worker_mutation_observer_property_set() {
        // P1b S2 incr3（镜像 browser）：property set（el.className='x'）触发 attributes 记录。
        let mut worker = RendererJsWorker::spawn(17);
        worker.set_dom_snapshot("<html><body><div id='t'></div></body></html>", "about:blank");
        worker
            .execute_script_direct(
                "globalThis.__seen = null;
                 var obs = new MutationObserver(function(records){
                   globalThis.__seen = records[0].type + ':' + records[0].attributeName;
                 });
                 var el = document.querySelector('#t');
                 obs.observe(el, { attributes: true });
                 el.className = 'active';",
            )
            .unwrap();
        let r = wait_for_global(&worker, "__seen", 1000);
        assert_eq!(r, "attributes:class");
        worker.shutdown();
    }

    #[test]
    fn timeout_error_annotation_includes_script_context() {
        // P7b：超时错误必须带脚本身份（长度+头部），否则页面回调超时无法定位。
        let m = annotate_timeout_error("Execution timeout: 30000ms", "detect(); // site callback");
        assert!(m.starts_with("Execution timeout: 30000ms"), "{m}");
        assert!(m.contains("script_len=26"), "{m}");
        assert!(m.contains("\"detect(); // site callback\""), "{m}");
    }

    #[test]
    fn non_timeout_error_passes_through_untouched() {
        // 非超时错误原样透传——不污染既有错误文本消费方。
        assert_eq!(
            annotate_timeout_error("ReferenceError: x is not defined", "x()"),
            "ReferenceError: x is not defined"
        );
    }

    #[test]
    fn tab_js_exec_timeout_pinned_with_channel_order() {
        // P7b 回归钉：阈值回退（如回 15s 误杀长回调）或 CHANNEL 次序被改走样
        // 必须在此变红——核心行为的常驻自动化防线（testval 首轮 B1/B6 缺口）。
        assert_eq!(TAB_JS_EXEC_TIMEOUT_MS, 30_000);
        assert_eq!(
            TAB_JS_CHANNEL_TIMEOUT,
            Duration::from_millis(TAB_JS_EXEC_TIMEOUT_MS + 5_000)
        );
    }

    /// t2-pb1 fix#15：可挂起优先执行——worker 空闲时有界等待内同步完成。
    #[test]
    fn execute_script_priority_deferrable_done_path() {
        let worker = RendererJsWorker::spawn(72);
        match worker.execute_script_priority_deferrable("1+2", std::time::Duration::from_secs(5)) {
            Ok(Ok(value)) => assert_eq!(value, "3"),
            other => panic!("worker 空闲时须同步完成，实际 {other:?}"),
        }
    }

    /// t2-pb1 fix#15：worker 长臂期间挂起（返回通道），臂结束后脚本照常执行、结果晚至。
    #[test]
    fn execute_script_priority_deferrable_parks_then_completes() {
        let worker = RendererJsWorker::spawn(73);
        let (busy_tx, busy_rx) = mpsc::channel();
        // 占住 worker 的长臂：稠密循环（v8 ~1-2s；quickjs 30s 上限截断亦不破坏断言）。
        worker
            .prio_tx
            .send(JsWorkerCommand::Execute {
                script: "var s=0;for(var i=0;i<5e8;i++)s+=i;String(s)".to_string(),
                reply: busy_tx,
                seq: 0,
            })
            .expect("长臂入队");
        std::thread::sleep(std::time::Duration::from_millis(120));
        // 门控：确认 worker 确在长臂中（1ms 有界等待必须挂起）。
        let rx = match worker.execute_script_priority_deferrable("6*7", std::time::Duration::from_millis(1)) {
            Err(rx) => rx,
            Ok(Ok(value)) => panic!("长臂期间应挂起，实际同步返回 {value}"),
            Ok(Err(e)) => panic!("长臂期间应挂起，实际错误 {e}"),
        };
        // 挂起脚本在臂后照常执行：先等长臂收尾（结果/30s 截断错误均可），再收晚至值。
        let _ = busy_rx.recv_timeout(Duration::from_secs(35));
        let value = rx
            .recv_timeout(Duration::from_secs(10))
            .expect("挂起脚本须在臂后执行")
            .expect("挂起脚本须成功");
        assert_eq!(value, "42");
    }

    /// t2-pb1 fix#15 + F1 语义修订（首轮缺陷审查 2026-10-02）：**复位提交之前**挂起的
    /// 优先通道求值在 prio FIFO 中先于 Reset 出队——臂结束后照常对旧文档（当时仍是
    /// 当前文档）执行并回值，不被复位取消；复位清队只吞排在其后的旧代际残留。复位
    /// 之后提交的脚本属新文档，复位不清（见
    /// [`reset_purge_keeps_commands_submitted_after_reset`]）；worker 退出（shutdown）
    /// 丢弃全部滞留 reply 的取消面不变。
    #[test]
    fn execute_script_priority_deferrable_before_reset_completes_not_cancelled() {
        let mut worker = RendererJsWorker::spawn(74);
        let (busy_tx, busy_rx) = mpsc::channel();
        worker
            .prio_tx
            .send(JsWorkerCommand::Execute {
                script: "var s=0;for(var i=0;i<5e8;i++)s+=i;String(s)".to_string(),
                reply: busy_tx,
                seq: 0,
            })
            .expect("长臂入队");
        std::thread::sleep(std::time::Duration::from_millis(120));
        // 复位提交之前挂起的求值：排长臂之后（prio FIFO），1ms 有界等待挂起。
        let rx = match worker.execute_script_priority_deferrable(
            "String(globalThis.__preNav = 'x') || globalThis.__preNav",
            std::time::Duration::from_millis(1),
        ) {
            Err(rx) => rx,
            _ => panic!("长臂期间应挂起"),
        };
        // 复位命令排挂起求值之后（prio FIFO）；reply 有界等待 250ms 在长臂期间超时，命令滞留队列。
        worker.reset_document_state();
        worker.shutdown();
        let _ = busy_rx.recv_timeout(Duration::from_secs(35));
        let value = rx
            .recv_timeout(Duration::from_secs(10))
            .expect("先于复位挂起的求值须在臂后照常执行")
            .expect("先于复位挂起的求值不被复位取消");
        assert_eq!(value, "x");
    }

    /// t2-pb1 F1 回归：Reset 滞留长臂之后时，**复位提交之后**入队的新文档快照+脚本
    /// 不得被清队。修复前按「Reset 处理时刻」清队——滞留 Reset 处理时把队列里排在
    /// 它后面的新文档命令连 reply 一起丢弃，新文档 JS 整体静默死亡（bilibili video 页
    /// ready:false/videoCount:0 证据吻合）。代际判定下快照先于脚本续派（成对序保持）。
    #[test]
    fn reset_purge_keeps_commands_submitted_after_reset() {
        let mut worker = RendererJsWorker::spawn(75);
        let (busy_tx, busy_rx) = mpsc::channel();
        // 长臂占住 worker（普通通道；复位/新文档命令排优先队列）。
        worker
            .cmd_tx
            .send(JsWorkerCommand::Execute {
                script: "var s=0;for(var i=0;i<5e8;i++)s+=i;String(s)".to_string(),
                reply: busy_tx,
                seq: 0,
            })
            .expect("长臂入队");
        std::thread::sleep(std::time::Duration::from_millis(120));
        // 复位提交（滞留优先队列；有界等待在长臂期间超时放行）。
        worker.reset_document_state();
        // 新文档工作在复位提交之后入队：快照 + 脚本（优先通道成对，fix#10 语义）。
        worker.set_dom_snapshot_priority("<html><body></body></html>", "https://example.test/");
        let rx = match worker.execute_script_priority_deferrable(
            "globalThis.__newdoc = (typeof document !== 'undefined' && document.body) ? 42 : 0; String(globalThis.__newdoc)",
            std::time::Duration::from_millis(1),
        ) {
            Err(rx) => rx,
            _ => panic!("长臂期间应挂起"),
        };
        worker.shutdown();
        let _ = busy_rx.recv_timeout(Duration::from_secs(35));
        let value = rx
            .recv_timeout(Duration::from_secs(10))
            .expect("复位后提交的新文档脚本须照常执行")
            .expect("新文档脚本须成功");
        assert_eq!(value, "42", "新文档快照+脚本须在滞留复位之后按序执行");
    }

    /// t2-pb1 fix#20 断言级回归：复位清队的滞留配置命令须跨复位保留并生效、复位后
    /// worker 须继续服务后续命令（修复前排空循环内向同通道重排队自馈送——Reset 时
    /// cmd_rx 含后续命令即 100% CPU 自旋，Shutdown 永不处理，join 挂死由包裹器杀树
    /// 才暴露；本测试给出断言级证据而非挂死形态）。
    #[test]
    fn reset_purge_retains_config_commands_and_worker_keeps_serving() {
        let mut worker = RendererJsWorker::spawn(76);
        let (busy_tx, busy_rx) = mpsc::channel();
        // 长臂占住 worker。
        worker
            .cmd_tx
            .send(JsWorkerCommand::Execute {
                script: "var s=0;for(var i=0;i<5e8;i++)s+=i;String(s)".to_string(),
                reply: busy_tx,
                seq: 0,
            })
            .expect("长臂入队");
        std::thread::sleep(std::time::Duration::from_millis(120));
        // 配置命令（视口提示）滞留普通队列——跨复位存活。
        worker.set_viewport_hint(1000, 700);
        worker.reset_document_state();
        // 复位后 worker 继续服务：快照换代使滞留视口提示生效（shim 缺省 1280x800 失配校正）。
        worker.set_dom_snapshot("<html><body></body></html>", "https://example.test/");
        let value = worker
            .execute_script_direct_bounded("String(globalThis.innerWidth)", Duration::from_secs(5))
            .expect("复位后 worker 须继续服务");
        assert_eq!(value, "1000", "滞留配置命令（视口提示）须跨复位保留并生效");
        worker.shutdown();
        let _ = busy_rx.recv_timeout(Duration::from_secs(35));
    }

    /// t2-pb1 T3（首轮测试有效性审查缺口，2026-10-02）：`__zwHostDynamicScripts`
    /// bootstrap 旗标黑盒钉——spawn 后置位（R387b shim 页面 fetch 让位 renderer 宿主
    /// 通路，part04 分支 gate 读取）；ResetDocumentState 重建 sandbox 后须重臂（缺此
    /// 重臂则首个导航后动态脚本双执行/双事件回归）。置位语义本身由 engine
    /// r387b2_host_dynamic_scripts_bootstrap_flag_skips_shim_fetch 钉。
    #[test]
    fn host_dynamic_scripts_flag_set_at_bootstrap_and_rearmed_after_reset() {
        let mut worker = RendererJsWorker::spawn(77);
        let value = worker
            .execute_script_direct_bounded(
                "String(globalThis.__zwHostDynamicScripts === true)",
                Duration::from_secs(5),
            )
            .expect("bootstrap 后 worker 可执行");
        assert_eq!(value, "true", "bootstrap 须置位 __zwHostDynamicScripts");
        worker.reset_document_state();
        let value = worker
            .execute_script_direct_bounded(
                "String(globalThis.__zwHostDynamicScripts === true)",
                Duration::from_secs(5),
            )
            .expect("复位后 worker 可执行");
        assert_eq!(value, "true", "复位重建 sandbox 后须重臂旗标");
        worker.shutdown();
    }

    // slice37（NPO 收口）：renderer 腿（真实快照臂路径，同一份 shim JS）named access
    // 语义钉，按 shim 自身引擎分叉（`__zwNPO.wired`——part05 接线处 FIXME 申报）分支：
    // wired（V8/Chrome 语义）= NPO 面——①named prop 不在 globalThis own（hasOwnProperty
    // 假）而 gsp 描述符 {w:true,e:false,c:true}；②gPN(npo) 不含 named prop；③window 级
    // delete 再读复活（slice36 FIXME ⑥ 收口面）；quickjs（不接链）= slice36 安装面——
    // globalThis own 非可枚举数据属性、delete 即清除（引擎 C 层 get_property 短路，
    // 如实申报）。共用面：for-in 可枚举守恒 + 换代快照回收重装。Chrome 154 对照：
    // diag/evidence/slice37/repro/。
    #[test]
    fn renderer_js_worker_npo_faces_s37() {
        let mut worker = RendererJsWorker::spawn(78);
        worker.set_dom_snapshot("<html><body><img name='s37r'></body></html>", "https://example.test/");
        let wired = worker
            .execute_script_direct("String(globalThis.__zwNPO && globalThis.__zwNPO.wired === true)")
            .unwrap()
            == "true";
        if wired {
            assert_eq!(
                worker
                    .execute_script_direct("String(Object.prototype.hasOwnProperty.call(globalThis, 's37r'))")
                    .unwrap(),
                "false",
                "named prop 不在 globalThis own（NPO 化本量）"
            );
            assert_eq!(
                worker
                    .execute_script_direct(
                        "var gsp=Object.getPrototypeOf(Object.getPrototypeOf(window));\
                         var d=Object.getOwnPropertyDescriptor(gsp,'s37r');\
                         String(d && d.writable===true && d.enumerable===false && d.configurable===true)"
                    )
                    .unwrap(),
                "true",
                "NPO gsp 描述符 writable/!enumerable/configurable（WebIDL §3.7.4.1）"
            );
            assert_eq!(
                worker
                    .execute_script_direct(
                        "var gsp=Object.getPrototypeOf(Object.getPrototypeOf(window));\
                         String(Object.getOwnPropertyNames(gsp).indexOf('s37r')===-1 && gsp.hasOwnProperty('s37r'))"
                    )
                    .unwrap(),
                "true",
                "gPN 不含 named prop 而 hasOwnProperty 真（Chrome 同款并存面）"
            );
            assert_eq!(
                worker
                    .execute_script_direct("delete window.s37r; String(window.s37r && window.s37r.nodeType === 1)")
                    .unwrap(),
                "true",
                "window 级 delete 后再读复活（FIXME ⑥ 收口，Chrome 同款）"
            );
        } else {
            // quickjs（引擎分叉申报）：slice36 安装面——own 非可枚举数据属性在位，
            // window 级 delete 命中 own 即清除（再读 undefined，无复活——NPO 层缺席）。
            assert_eq!(
                worker
                    .execute_script_direct(
                        "var d=Object.getOwnPropertyDescriptor(globalThis,'s37r');\
                         String(!!d && d.writable===true && d.enumerable===false && d.configurable===true && d.value.nodeType===1)"
                    )
                    .unwrap(),
                "true",
                "quickjs 安装面：globalThis own 非可枚举数据属性=元素（slice36 口径）"
            );
            assert_eq!(
                worker
                    .execute_script_direct("delete window.s37r; String(window.s37r === undefined)")
                    .unwrap(),
                "true",
                "quickjs：window 级 delete 命中 own 清除（slice36 口径，无 NPO 复活面）"
            );
        }
        assert_eq!(
            worker
                .execute_script_direct(
                    "String(typeof window.s37r === 'undefined' || (window.s37r && window.s37r.nodeType === 1))"
                )
                .unwrap(),
            "true",
            "named access 取值面在位（双腿：NPO 值/own 数据属性）"
        );
        assert_eq!(
            worker
                .execute_script_direct(
                    "var bad=[]; for (var k in window) { if (k==='s37r'||k==='__zwNPO'||k==='__zwNAGet'||k==='__zwNADelete'||k==='__zwNAOwnKeys') bad.push(k); } String(bad.join(','))"
                )
                .unwrap(),
            "",
            "for-in(window) 不暴露 named prop 与 NPO 内部全局（可枚举守恒）"
        );
        // 换代：同名换新元素——wired 走 NPO backing 重装 + 回收链路；quickjs 走
        // slice36 own 重装（快照换代同款，双腿同 JS 臂）。
        worker.set_dom_snapshot(
            "<html><body><img name='s37r' id='s37r2'></body></html>",
            "https://example.test/",
        );
        assert_eq!(
            worker
                .execute_script_direct(
                    "String(window.s37r && window.s37r.nodeType === 1 && window.s37r.id === 's37r2')"
                )
                .unwrap(),
            "true",
            "换代后 named prop 重装为新元素（回收链路双腿不失效）"
        );
        worker.shutdown();
    }
}
