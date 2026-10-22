//! # zero-script-sandbox
//!
//! 扩展/用户脚本引擎（V8/QuickJS feature gate）。
//!
//! 提供JavaScript脚本执行沙箱，用于扩展脚本、用户脚本和自动化脚本。
//! 通过feature gate选择后端引擎：
//! - `v8`（默认推荐）— 使用 v8 crate 绑定 V8 引擎
//! - `quickjs` — 使用rquickjs绑定QuickJS引擎

#![warn(missing_docs)]

#[cfg(feature = "v8")]
mod v8_runtime;

#[cfg(feature = "v8")]
mod dom_bindings;

// threaded_runtime / service_worker 是引擎无关的线程+通道封装，仅 create_engine
// 按特性选择引擎。无引擎构建保留类型与协议（供 page-runtime / browser 进程引用
// SandboxConfig、ServiceWorkerEvent 等类型），运行时创建在 service_worker.rs 的
// no-engine create_engine 中降级为 EngineUnavailable——引擎代码不会被链接进来。
mod threaded_runtime;

mod service_worker;

pub use service_worker::*;

#[cfg(feature = "v8")]
mod worker;

// js-dom R84：模块声明移至下方 not(v8) 门控区（union 态 dead-code）。
mod es_module;

// P6 import map：仅依赖 serde_json/url，与引擎 feature 无关，无条件编译。
mod import_map;

#[cfg(feature = "v8")]
pub use v8_runtime::*;

#[cfg(feature = "v8")]
pub use worker::*;

// js-dom R84：quickjs 的 WorkerRuntime/WorkerEvent 与 v8 worker.rs 同名——v8+quickjs
// 组合态（workspace 单 cargo 调用 feature 并集）`pub use` 两份同名类型 = ambiguous
// name 编译断（a8d5a22d 引入，阻 make test）。消费方（webview worker 接线）用 v8
// 语义；quickjs-only 构建不受影响（not(v8) 才编译本模块——门控 re-export 在 union
// 态会留 dead code 告警）。
#[cfg(all(feature = "quickjs", not(feature = "v8")))]
mod quickjs_worker;

#[cfg(all(feature = "quickjs", not(feature = "v8")))]
pub use quickjs_worker::*;

pub use es_module::*;

pub use import_map::{ImportMap, ImportMapResolution};

#[cfg(feature = "quickjs")]
mod quickjs_runtime;

// js-dom R84：QuickJSSandbox 等沙箱类型无 v8 同名冲突（es_module 经 crate::QuickJSSandbox
// 消费），保持无条件 re-export；仅 quickjs_worker（WorkerRuntime/WorkerEvent 同名）按
// not(v8) 门控。
#[cfg(feature = "quickjs")]
pub use quickjs_runtime::*;

/// 脚本执行错误类型。
#[derive(Debug, thiserror::Error)]
pub enum ScriptError {
    /// 脚本编译错误（语法错误）。
    #[error("Compile error: {0}")]
    CompileError(String),
    /// 脚本运行时错误。
    #[error("Runtime error: {0}")]
    RuntimeError(String),
    /// 脚本超时。
    #[error("Execution timeout: {0}")]
    Timeout(String),
    /// 沙箱未初始化。
    #[error("Sandbox not initialized")]
    NotInitialized,
    /// 无效输入。
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    /// 引擎不可用。
    #[error("Engine unavailable: {0}")]
    EngineUnavailable(String),
}

/// 脚本执行结果。
#[derive(Debug, Clone)]
pub struct ScriptResult {
    /// 脚本返回值的字符串表示。
    pub value: String,
    /// 脚本执行耗时（毫秒）。
    pub execution_time_ms: f64,
}

/// Long Task 判定阈值（毫秒）——Long Tasks API §2「long task」定义：任务持续
/// ≥ 50ms 即 long task。host 侧经 [`Sandbox::set_long_task_observer`] 订阅超阈值
/// 执行并生成 `PerformanceLongTaskTiming` 条目（performance-api-compat M2-S1）。
///
/// https://w3c.github.io/longtasks/#dfn-long-task
pub const LONG_TASK_THRESHOLD_MS: f64 = 50.0;

/// Long task 观察器：单次 `execute` 墙钟耗时（毫秒，≥ [`LONG_TASK_THRESHOLD_MS`])
/// 通知。观察器内**不得**重入沙箱（在本线程 execute 返回前同步调用）——宿主侧
/// 惯用法是把耗时推入共享槽，任务边界处统一排空发射（webview `drain_long_task_entries`）。
pub type LongTaskObserver = std::sync::Arc<dyn Fn(f64) + Send + Sync>;

/// 沙箱配置。
#[derive(Debug, Clone, Default)]
pub struct SandboxConfig {
    /// 堆内存上限（字节），0表示无限制。
    pub heap_limit: usize,
    /// 初始堆大小（字节），0表示 V8 默认（按系统内存推导）。
    ///
    /// V8 isolate 创建时会按初始堆大小预提交内存；嵌入式场景（WebView 页面
    /// 轻 JS）设小可显著降低常驻内存（RSS）。堆按需增长，上限仍由 `heap_limit`
    /// 控制，JS 语义不变。仅 V8 后端使用，QuickJS 忽略。
    pub initial_heap_size: usize,
    /// 脚本执行超时（毫秒），0表示无超时。
    pub timeout_ms: u64,
    /// 复用 V8 Context 以减少启动开销（默认 false）。
    ///
    /// 启用后，首次 execute 时创建的 Context 会被缓存复用，
    /// 避免每次执行都重新引导所有 JS 内置对象（Object/Array/Function 等）。
    /// 适用于 WebView 等需要频繁执行脚本的场景。
    ///
    /// 注意：启用后多次 execute 之间的 JS 状态不再隔离（变量会保留）。
    pub persistent_context: bool,
}

/// 根据 [`SandboxConfig`] 计算 V8 堆限制参数 `(initial, max)`，`None` 表示
/// 使用 V8 全部默认值（不调用 `heap_limits`）。
///
/// V8 要求 `initial <= max`（`SetHeapLimits` CHECK，违反即致命崩溃）。当
/// `heap_limit = 0`（无上限）但设置了 `initial_heap_size` 时，max 取 4GB
/// 显式上限——V8 默认堆上限量级，实际不会触发，仅满足 CHECK。
///
/// R3397：`SandboxConfig` 是公共嵌入 API（[`crate::SandboxConfig`]），嵌入方可能传
/// `initial_heap_size > 4GB`（默认 max）而 `heap_limit = 0`，此时 `initial > max`
/// 会触发 V8 `CHECK` **致命崩溃整个进程**（abort，非 panic 可捕获）。此处在校验
/// `initial <= max` 并把越界 `initial` 钳到 `max`（信任边界输入校验，CLAUDE.md「安全措施/
/// 输入校验不可简化」）。本仓内置 WebView config（initial=128MB / heap_limit=0）不受影响，
/// 此为公共 API 防崩溃硬化。
#[cfg(feature = "v8")]
pub(crate) fn v8_heap_limits(config: &SandboxConfig) -> Option<(usize, usize)> {
    if config.initial_heap_size == 0 && config.heap_limit == 0 {
        return None;
    }
    let max = if config.heap_limit > 0 {
        config.heap_limit
    } else {
        4 * 1024 * 1024 * 1024
    };
    // R3397：钳制 initial 不超过 max，防 V8 SetHeapLimits CHECK 致命崩溃。
    let initial = config.initial_heap_size.min(max);
    Some((initial, max))
}

/// S1b canvas 桥：worker canvas 操作排空回调——收 `JSON.stringify(_workerCanvasOps)`
/// 原文（newCtx/op/resize/bind 条目数组），由宿主侧求值进 worker 线程本地
/// CanvasRegistry（renderer worker_host 经 `canvas_context_op` 纯栅格核）。
pub type WorkerCanvasDrain = Box<dyn FnMut(&str) + Send>;

/// 线程共享句柄形态（worker 线程命令循环间隙调用）。
pub type WorkerCanvasDrainHandle = std::sync::Arc<std::sync::Mutex<WorkerCanvasDrain>>;

/// Worker bootstrap（V8/QuickJS worker 共享，注入 worker 全局环境）。
///
/// 五块 JS 面（Rust 侧经排空协议消费，见 worker.rs / quickjs_worker.rs）：
/// - 全局别名：`self` = 全局对象（worker 脚本惯用 `self.onmessage`/`self.postMessage` 访问面）。
/// - 消息：`postMessage` 入 `_workerMessageQueue`，`_dispatchMessage` 派发给 `onmessage`；
///   入站 `__zwXferCanvas` 标记复活为 worker 本地 OffscreenCanvas（S1b canvas 桥）。
/// - 定时器：`setTimeout`/`setInterval`/`clearTimeout`/`clearInterval` 只把操作（arm/cancel）
///   记入 `_workerTimerOps`，到期在 worker 线程命令循环间隙由 Rust 轮驱动 `_workerTimerFire`。
///   isolate 无事件循环，纯 JS 不可自举定时器。
/// - canvas：`OffscreenCanvas` 2D 写路径只把操作记入 `_workerCanvasOps`
///   （newCtx/op/resize/bind，S1b 排空协议），宿主侧经 `canvas_context_op` 求值进
///   worker 线程本地 registry；读路径（getImageData 等）排空模型不可同步返回暂缺。
/// - 关闭：`close()` 记 `close` 操作，Rust 侧发 `WorkerEvent::Closed` 并退出线程
///   （https://html.spec.whatwg.org/multipage/workers.html#dom-dedicatedworkerglobalscope-close）。
pub(crate) const WORKER_BOOTSTRAP_JS: &str = r#"
    var _workerMessageQueue = [];
    // self = worker 全局对象自身，脚本经 self.onmessage/self.postMessage 访问消息面
    //（https://html.spec.whatwg.org/multipage/workers.html#dom-workerglobalscope-self）。
    var self = globalThis;
    var onmessage = null;
    var postMessage = function(data) {
        _workerMessageQueue.push(typeof data === 'string' ? data : JSON.stringify(data));
    };
    var _workerCanvasOps = [];
    var _workerCanvasSeq = 0;
    var _workerTimerSeq = 0;
    // ── worker 2D ctx 代理（S1b）：写路径记 op，样式 setter 记 set-* op 并存 JS 镜像；
    // 读路径（measureText/getImageData 等）排空模型不可同步返回暂缺（FIXME(S1c)：呈现
    // 管线落地后按需补读路径）。方法名与 canvas_context_op op 名 1:1（同主线程 shim）。
    var _zwWorkerMakeCtx2d = function(cid) {
        var target = {};
        var pushOp = function(name) {
            return function() {
                var args = [];
                for (var i = 0; i < arguments.length; i++) { args.push(arguments[i]); }
                _workerCanvasOps.push(['op', cid, name, args]);
            };
        };
        var drawOps = ['beginPath', 'closePath', 'moveTo', 'lineTo', 'arc', 'fill', 'stroke',
            'fillText', 'strokeText', 'fillRect', 'strokeRect', 'clearRect',
            'quadraticCurveTo', 'bezierCurveTo', 'ellipse', 'arcTo', 'roundRect', 'rect',
            'clip', 'save', 'restore', 'translate', 'rotate', 'scale',
            'setTransform', 'transform', 'resetTransform', 'reset'];
        for (var di = 0; di < drawOps.length; di++) {
            target[drawOps[di]] = pushOp(drawOps[di]);
        }
        var styleProps = { fillStyle: 'setFillStyle', strokeStyle: 'setStrokeStyle',
            globalAlpha: 'setGlobalAlpha', lineWidth: 'setLineWidth',
            lineJoin: 'setLineJoin', lineCap: 'setLineCap',
            shadowColor: 'setShadowColor', shadowBlur: 'setShadowBlur',
            shadowOffsetX: 'setShadowOffsetX', shadowOffsetY: 'setShadowOffsetY',
            globalCompositeOperation: 'setCompositeOperation' };
        for (var sp in styleProps) {
            Object.defineProperty(target, sp, { set: pushOp(styleProps[sp]), enumerable: true, configurable: true });
        }
        return target;
    };
    var _zwOffscreenInit = function(oc, w, h, presetCid) {
        var ctx = null;
        var cid = presetCid;
        oc._zwCid = 0;
        Object.defineProperty(oc, 'width', {
            get: function() { return w; },
            set: function(v) {
                if (typeof v !== 'number' || !isFinite(v) || v < 0) { return; }
                w = v | 0;
                if (ctx) { _workerCanvasOps.push(['resize', cid, w, h]); }
            },
            enumerable: true, configurable: true
        });
        Object.defineProperty(oc, 'height', {
            get: function() { return h; },
            set: function(v) {
                if (typeof v !== 'number' || !isFinite(v) || v < 0) { return; }
                h = v | 0;
                if (ctx) { _workerCanvasOps.push(['resize', cid, w, h]); }
            },
            enumerable: true, configurable: true
        });
        oc.getContext = function(type) {
            if (String(type) !== '2d') { return null; }
            if (ctx) { return ctx; }
            // worker 本地 registry 侧 ctx 尚不存在——无论构造还是复活路径，首个
            // getContext 都要记 newCtx（宿主排空侧分配真 id 并建 prov 映射）。
            if (!cid) { cid = ++_workerCanvasSeq; }
            _workerCanvasOps.push(['newCtx', cid, w, h]);
            oc._zwCid = cid;
            ctx = _zwWorkerMakeCtx2d(cid);
            return ctx;
        };
        oc.transferToImageBitmap = function() {
            // 排空模型读路径不可同步返回（FIXME(S1c)：呈现管线落地后按需补）。
            throw new Error('transferToImageBitmap: not supported in threaded worker canvas bridge');
        };
        return oc;
    };
    // OffscreenCanvas（https://html.spec.whatwg.org/multipage/canvas.html#the-offscreencanvas-interface）
    // ——worker 侧像素只存 worker 线程本地 registry（无 paint 路径，呈现面属 S1c patch 管线）。
    var OffscreenCanvas = function(width, height) {
        if (!(this instanceof OffscreenCanvas)) { return new OffscreenCanvas(width, height); }
        _zwOffscreenInit(this,
            (typeof width === 'number' && width >= 0) ? (width | 0) : 300,
            (typeof height === 'number' && height >= 0) ? (height | 0) : 150, 0);
    };
    // 入站 transfer 复活（S1b）：页面宿主分支 postMessage 把转移的 OffscreenCanvas 序列化为
    // `__zwXferCanvas` 标记（{key, w, h}），这里深走 payload 复活为本地 OffscreenCanvas 并记
    // bind op（宿主绑定表：surface 键 → worker 本地 ctx——S1c patch 呈现消费）。
    var _zwWorkerReviveXfer = function(v) {
        if (!v || typeof v !== 'object') { return v; }
        if (Array.isArray(v)) {
            for (var ai = 0; ai < v.length; ai++) { v[ai] = _zwWorkerReviveXfer(v[ai]); }
            return v;
        }
        var marker = v.__zwXferCanvas;
        if (marker) {
            var cid = ++_workerCanvasSeq;
            var oc = _zwOffscreenInit(Object.create(OffscreenCanvas.prototype),
                (typeof marker.w === 'number' && marker.w >= 0) ? (marker.w | 0) : 300,
                (typeof marker.h === 'number' && marker.h >= 0) ? (marker.h | 0) : 150, cid);
            oc._zwXferKey = String(marker.key || '');
            oc._zwSel = String(marker.sel || '');
            // S1c：bind 第 4 元携带源选择器（patch Register lazy 首拍查元素矩形）。
            _workerCanvasOps.push(['bind', cid, oc._zwXferKey, oc._zwSel]);
            return oc;
        }
        for (var k in v) {
            if (k === '__zwXferCanvas') { continue; }
            v[k] = _zwWorkerReviveXfer(v[k]);
        }
        return v;
    };
    var _dispatchMessage = function(data) {
        if (typeof onmessage === 'function') {
            onmessage({ data: _zwWorkerReviveXfer(data) });
        }
    };
    var _workerTimers = {};
    var _workerTimerOps = [];
    // DC-1（2026-10-22，background-resource-lifecycle goal）：嵌套 timer 最小延迟钳制，
    // 与页面 shim 同款任务链模型——创建时闭包携带 level（nesting level），fire 时计数
    // 绝对赋值 level+1（每次 fire 独立 execute，绝对赋值天然自洽）；创建时按 level 钳
    // delay（>5 且 <4 → 4）。interval 重复分支的重臂在宿主 Rust 轮（重臂钳制按
    // 创建 level + 已 tick 代数逐代判定，见 worker_timer_period_at）。
    // https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html#timer-initialisation-steps
    var _workerTimerNesting = 0;
    var _workerTimerSet = function(callback, delay, period) {
        if (typeof callback !== 'function') {
            return 0;
        }
        var id = ++_workerTimerSeq;
        var level = _workerTimerNesting;
        if (level > 5 && typeof delay === 'number' && delay < 4) {
            delay = 4;
        }
        _workerTimers[id] = function() {
            _workerTimerNesting = level + 1;
            callback();
            _workerTimerNesting = 0;
        };
        _workerTimerOps.push(['arm', id, typeof delay === 'number' && delay > 0 ? delay : 0, period, level]);
        return id;
    };
    var setTimeout = function(callback, delay) {
        return _workerTimerSet(callback, delay, 0);
    };
    var setInterval = function(callback, delay) {
        return _workerTimerSet(callback, delay, delay);
    };
    var clearTimeout = function(id) {
        _workerTimerOps.push(['cancel', typeof id === 'number' ? id : 0, 0, 0]);
    };
    var clearInterval = function(id) {
        _workerTimerOps.push(['cancel', typeof id === 'number' ? id : 0, 0, 0]);
    };
    var _workerTimerFire = function(id) {
        var callback = _workerTimers[id];
        if (typeof callback === 'function') {
            callback();
        }
    };
    var close = function() {
        _workerTimerOps.push(['close', 0, 0, 0]);
    };
"#;

/// interval 重臂钳制（DC-1，2026-10-22；spec timer initialisation steps 重复分支）：
/// re-arm 发生于本 tick task 内，nesting level = 创建 level + 已 tick 代数；
/// level > 5 且周期 < 4 → 4。与页面 shim 第 6 tick 起 ≥4ms 同语义。
/// V8/QuickJS worker 轮共用（两模块 feature 互斥，放无门控层）。
// https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html#timer-initialisation-steps
pub(crate) fn worker_timer_period_at(nesting: u64, period_ms: u64) -> u64 {
    if nesting > 5 && period_ms < 4 { 4 } else { period_ms }
}

/// 脚本沙箱抽象 trait — `V8Sandbox` 和 `QuickJSSandbox` 都实现。
///
/// 调用方用 `Box<dyn Sandbox>` 持有引擎无关的沙箱实例（cfg 选 V8/QuickJS）。
/// `register_callback` 用 `Box<dyn Fn>`（非泛型）以支持 trait object 动态分发。
pub trait Sandbox {
    /// 执行 JavaScript 代码，返回字符串结果。
    fn execute(&mut self, code: &str) -> Result<ScriptResult, ScriptError>;
    /// 执行 JavaScript 代码，返回 JSON 字符串结果（`JSON.stringify` 包装）。
    fn execute_json(&mut self, code: &str) -> Result<ScriptResult, ScriptError>;
    /// 注册宿主回调，挂为全局函数 `name`（JS 调 `name(...)` 触发 Rust 闭包）。
    /// 须在 `execute` 之前调用。回调参数为 JS 参数的字符串数组，返回字符串。
    #[allow(clippy::type_complexity)]
    fn register_callback(&mut self, name: &str, callback: Box<dyn Fn(&[String]) -> String + Send + Sync>);
    /// P1b S1 异步回调 resolve（方案 A，RFC `p1b-rfc-2026-07-25.md` v0.3）：Rust 异步
    /// 完成后调此方法，在沙箱中执行 JS 全局 `__zwResolveCallback(id, result)`，由 JS
    /// 侧 pending 表 resolve 对应 Promise。`id`/`result` 按 JS 字符串字面量安全转义防注入。
    ///
    /// **前置**：JS 侧须先注入 `__zwResolveCallback` + pending 表（dom_bridge 负责）；
    /// 未注入时 V8 实现防御性 no-op。
    ///
    /// **默认 no-op**：QuickJS 后端降级（RFC v0.3 V8-first——异步/对象绑定仅 V8 可行，
    /// QuickJS 保持同步）。后续切片接通 `tab_js_worker` marshal channel 后，跨线程异步
    /// 完成（net fetch / setTimeout）经 marshal 回 JS worker 线程再调本方法。
    fn resolve_async_callback(&mut self, _id: &str, _result: &str) {}
    /// t-p103 D2-1 批决议：与逐条调 [`Sandbox::resolve_async_callback`] 语义相同
    /// （按 `items` 顺序逐条 `__zwResolveCallback(id, result)`），但合并为尽量少的
    /// 脚本求值。promise 密集页面（fetch/定时器续体链）下每跳一次完整 shim 求值
    /// 会把 JS worker 顶满（实测 github 5s 窗 ~2 万跳、worker busy=99%，eval/
    /// 定时器/帧发布全部饿死）；批处理把 2 万次求值缩为数十次。
    ///
    /// **语义边界**：与逐条调 [`Sandbox::resolve_async_callback`] 精确一致——批内
    /// 保序（= FIFO 提交序）、每条后执行 microtask checkpoint（task 后整链排空
    /// 微任务的 spec 语义，返修确立：批末统一 drain 会让后一条 resolve 抢在前一
    /// 条的 `.then` 微任务前执行）、逐条错误隔离。
    ///
    /// **默认实现**：逐条回退调 [`Sandbox::resolve_async_callback`]（行为等价，
    /// 为无原生批实现的后端兜底）。
    fn resolve_async_callbacks_batch(&mut self, items: &[(String, String)]) {
        for (id, result) in items {
            self.resolve_async_callback(id, result);
        }
    }
    /// 设置脚本执行超时（毫秒），0 表示无超时。
    fn set_timeout_ms(&mut self, timeout_ms: u64);
    /// performance-api-compat M2-S1：注册 long task 观察器——单次 `execute` 墙钟
    /// ≥ [`LONG_TASK_THRESHOLD_MS`] 时以实际耗时（毫秒）调用。`None` 撤销。
    /// 本引擎的任务边界 = 一次 `execute`（页面脚本块 / timer 回调 / 宿主驱动任务）。
    ///
    /// **默认 no-op**：未注册即零开销（性能关键路径，`execute` 内仅一次 `Option` 判空）。
    fn set_long_task_observer(&mut self, _observer: Option<LongTaskObserver>) {}
    /// R-baidu2/P3 slice-2：取走自上次调用以来收集的未捕获异常报告
    /// （`(text, line_number, column_number)`；V8 = isolate message listener +
    /// promise-reject 回调；QuickJS 默认不支持，返回空）。无收集时返回空。
    fn take_uncaught_reports(&mut self) -> Vec<(String, u32, u32)> {
        Vec::new()
    }
    /// 重置上下文（清空 JS 状态）。
    fn reset_context(&mut self);
    /// 返回沙箱配置的引用。
    fn config(&self) -> &SandboxConfig;
    /// P1b S2 原生绑定安装 escape-hatch（RFC `p1b-v8-native-bindings-rfc.md` §6 S2）。
    ///
    /// 进入沙箱持久 V8 Context（与 `execute` 同 scope setup），在 raw scope + context 内
    /// 调用 `installer`（宿主侧经此安装 `ObjectTemplate`/`FunctionTemplate`/accessor 等原生
    /// 绑定，不经 String 桥）。仅 V8 后端实现（返 `true`）；QuickJS 降级 no-op（返 `false`）。
    ///
    /// **默认 no-op**：非 V8 后端或未实现时返 `false`（零回归）。
    #[cfg(feature = "v8")]
    #[allow(clippy::type_complexity)] // escape-hatch 闭包类型（镜像 register_callback 模式）
    fn install_native_bindings(
        &mut self,
        _installer: Box<dyn FnOnce(&mut v8::PinScope, v8::Local<v8::Context>)>,
    ) -> bool {
        false
    }

    /// js-dom goal M6 S0q：QuickJS 版原生绑定 escape-hatch（镜像 `install_native_bindings`
    /// 的 rquickjs 形态，DC-7 双引擎对等）。进入沙箱持久 QuickJS Context，在 `Ctx` 内调用
    /// `installer`（宿主侧经此安装 `Accessor`/`Function` 等原生绑定，不经 String 桥）。
    ///
    /// **默认 no-op**：非 QuickJS 后端或未实现时返 `false`（零回归）。
    #[cfg(feature = "quickjs")]
    fn install_native_bindings_quickjs(&mut self, _installer: Box<dyn FnOnce(&rquickjs::Ctx)>) -> bool {
        false
    }
}

#[cfg(all(test, feature = "v8"))]
mod tests {
    use super::*;

    // R3397：v8_heap_limits 须钳制 initial <= max，防 V8 SetHeapLimits CHECK 致命崩溃。
    #[test]
    fn v8_heap_limits_clamps_initial_to_max_r3397() {
        // 两零 → None（用 V8 默认）。
        let none_cfg = SandboxConfig {
            initial_heap_size: 0,
            heap_limit: 0,
            ..Default::default()
        };
        assert!(v8_heap_limits(&none_cfg).is_none());

        // initial > 默认 max(4GB) 且 heap_limit=0 → 须钳 initial 到 4GB（防 CHECK 崩溃）。
        let over_cfg = SandboxConfig {
            initial_heap_size: 8 * 1024 * 1024 * 1024, // 8GB > 4GB 默认 max
            heap_limit: 0,
            ..Default::default()
        };
        let (initial, max) = v8_heap_limits(&over_cfg).expect("应返回 Some");
        assert_eq!(max, 4 * 1024 * 1024 * 1024);
        assert!(
            initial <= max,
            "initial({initial}) 须钳到 <= max({max})，防 V8 CHECK 致命崩溃"
        );
        assert_eq!(initial, max, "8GB initial 须钳到 4GB max");

        // initial > 显式 heap_limit → 须钳到 heap_limit。
        let over_explicit = SandboxConfig {
            initial_heap_size: 512 * 1024 * 1024, // 512MB
            heap_limit: 256 * 1024 * 1024,        // 256MB
            ..Default::default()
        };
        let (initial, max) = v8_heap_limits(&over_explicit).expect("应返回 Some");
        assert_eq!(max, 256 * 1024 * 1024);
        assert!(initial <= max, "initial 须钳到显式 heap_limit");
        assert_eq!(initial, 256 * 1024 * 1024);

        // 合法配置不受影响（回归保护）：initial < max 原样。
        let ok_cfg = SandboxConfig {
            initial_heap_size: 128 * 1024 * 1024, // 128MB（内置 WebView config）
            heap_limit: 0,
            ..Default::default()
        };
        let (initial, max) = v8_heap_limits(&ok_cfg).expect("应返回 Some");
        assert_eq!(initial, 128 * 1024 * 1024);
        assert_eq!(max, 4 * 1024 * 1024 * 1024);
        assert!(initial < max);
    }
}
