//! V8引擎运行时实现。
//!
//! 封装 v8 crate，提供安全的JavaScript脚本执行沙箱。

use std::cell::RefCell;
use std::sync::{Arc, Once};

use crate::{SandboxConfig, ScriptError, ScriptResult};

/// 从 TryCatch 提取异常消息与堆栈（须在调用点展开以匹配具体 scope 类型）。
macro_rules! v8_try_catch_message {
    ($try_catch:expr) => {{
        let try_catch = $try_catch;
        if try_catch.has_caught() {
            if let Some(exception) = try_catch.exception() {
                let mut out = exception
                    .to_string(try_catch)
                    .map(|s| s.to_rust_string_lossy(try_catch))
                    .unwrap_or_else(|| "exception".to_string());
                if let Some(stack) = try_catch.stack_trace() {
                    if let Some(stack_s) = stack.to_string(try_catch) {
                        out.push('\n');
                        out.push_str(&stack_s.to_rust_string_lossy(try_catch));
                    }
                }
                out
            } else {
                "unknown error".to_string()
            }
        } else {
            "unknown error".to_string()
        }
    }};
}

/// 宿主注入的扁平字符串回调类型（JS-DOM-bridge：JS shim 经 __zw_* 回调操作宿主状态）。
///
/// 回调接收 JS 传入参数的字符串数组，返回字符串结果（写入 V8 ReturnValue）。
/// `Send + Sync + 'static` 以便能跨 V8 调用边界存于线程局部注册表。
pub type HostCallback = Arc<dyn Fn(&[String]) -> String + Send + Sync + 'static>;

// 宿主回调注册表（线程局部）。FunctionTemplate 回调须为 `Copy`
//（MapFnTo<FunctionCallback>），无法捕获 Arc 状态；故回调闭包存于此注册表，
// FunctionTemplate 经 builder().data(idx) 携带索引，fn 回调按 idx 查表调用。
thread_local! {
    static HOST_CALLBACKS: RefCell<Vec<HostCallback>> = RefCell::new(Vec::new());
}

/// V8 FunctionTemplate 回调：按 args.data() 的索引查 HOST_CALLBACKS 调用。
fn host_callback_invoke(scope: &mut v8::PinScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue) {
    let idx = args.data().integer_value(scope).unwrap_or(-1);
    if idx < 0 {
        return;
    }
    let n = args.length();
    let strs: Vec<String> = (0..n)
        .filter_map(|i| args.get(i).to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
        .collect();
    let result = HOST_CALLBACKS.with(|cbs| cbs.borrow().get(idx as usize).map(|cb| cb(&strs)).unwrap_or_default());
    if let Some(s) = v8::String::new(scope, &result) {
        rv.set(s.into());
    }
}

/// 将 Rust 字符串转为 JS 字符串字面量（含两端双引号），转义特殊字符防注入。
/// 供 [`V8Sandbox::resolve_async_callback`] 把 `id`/`result` 安全嵌入执行脚本。
fn js_string_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' | '\\' => {
                out.push('\\');
                out.push(c);
            }
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// V8平台初始化守卫（全局只初始化一次）。
static V8_INIT: Once = Once::new();

struct IsolateEnterGuard {
    isolate: *const v8::OwnedIsolate,
}

impl Drop for IsolateEnterGuard {
    fn drop(&mut self) {
        // SAFETY: The guard is created only after entering this isolate, and it
        // is dropped before the owned isolate is disposed.
        unsafe {
            (*self.isolate).exit();
        }
    }
}

/// 确保V8平台已初始化。
///
/// P1b S1：engine 的原生 DOM 绑定（`zero_engine::dom_bindings`）现直接依赖 v8 crate，
/// 需在自建 Isolate 前确保平台初始化。本函数经 `pub use v8_runtime::*` 公开（feature-gated v8），
/// 进程级 `Once` 防重复初始化（V8 平台初始化须全局一次）。
pub fn ensure_v8_initialized() {
    V8_INIT.call_once(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        // SAFETY: V8平台初始化在进程生命周期内只调用一次，
        // 且在所有Isolate创建之前完成。
        #[allow(unused_unsafe)]
        unsafe {
            v8::V8::initialize_platform(platform);
        }
        v8::V8::initialize();
    });
}

/// 超时看门狗消息（seq 协议）：execute 装载（Arm，携带截止时长与 isolate 句柄）、
/// guard Drop 撤除（Disarm，仅当 seq 匹配）、沙箱销毁停线程（Stop）。
enum WatchdogMsg {
    Arm {
        seq: u64,
        timeout_ms: u64,
        handle: v8::IsolateHandle,
    },
    Disarm {
        seq: u64,
    },
    Stop,
}

/// 持久超时看门狗线程主循环：阻塞等待直到（a）收到消息或（b）已装载的截止时刻
/// 到期 → `terminate_execution`。无装载时无限等待（`recv_timeout(Duration::MAX)`）。
///
/// 2026-08-10 重构：旧实现每次 execute spawn+join 一个 OS 线程，Windows 并行负载
/// （nextest 全量并发 + wait_for_global 5ms 轮询）下线程 churn 经 loader lock
/// 序列化致批量楔死（windows-x86_64 tab_js_worker ×4 120s 挂起 3/3 复现）。
/// 每 sandbox 一个常驻线程 + seq 协议无 churn；timeout_ms 由 Arm 携带支持
/// set_timeout_ms 动态变化。
fn watchdog_main(rx: std::sync::mpsc::Receiver<WatchdogMsg>) {
    let mut armed: Option<(u64, std::time::Instant, v8::IsolateHandle)> = None;
    loop {
        let wait = match &armed {
            Some((_, deadline, _)) => deadline.saturating_duration_since(std::time::Instant::now()),
            None => std::time::Duration::MAX,
        };
        match rx.recv_timeout(wait) {
            Ok(WatchdogMsg::Arm {
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
            Ok(WatchdogMsg::Disarm { seq }) => {
                if armed.as_ref().is_some_and(|(s, _, _)| *s == seq) {
                    armed = None;
                }
            }
            Ok(WatchdogMsg::Stop) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // 截止到期：终止执行（execute 侧 guard Drop 会按 seq 撤除/清理）。
                if let Some((_, _, handle)) = armed.take() {
                    handle.terminate_execution();
                }
            }
        }
    }
}

/// execute 超时装载守卫：Drop 时按 seq 撤除看门狗。成功与错误路径统一走 Drop
/// （旧实现脚本编译/运行出错提前返回时看门狗线程残留，5s 后误伤下一次执行）。
struct WatchdogGuard {
    tx: std::sync::mpsc::Sender<WatchdogMsg>,
    seq: u64,
}

impl Drop for WatchdogGuard {
    fn drop(&mut self) {
        let _ = self.tx.send(WatchdogMsg::Disarm { seq: self.seq });
    }
}

/// V8脚本沙箱 — 封装一个V8 Isolate和Context，提供安全的脚本执行环境。
///
/// # 生命周期
///
/// 1. 调用 [`V8Sandbox::new()`] 创建沙箱（内部初始化V8平台）
/// 2. 调用 [`V8Sandbox::execute()`] 执行脚本
/// 3. 沙箱释放时自动清理V8资源
///
/// # 性能优化
///
/// 当 `SandboxConfig::persistent_context` 为 `true` 时，首次 execute 创建的
/// V8 Context 会被缓存复用，避免每次执行都重新引导 JS 内置对象，显著降低执行开销。
/// 适用于 WebView 等需要频繁执行脚本的场景。
///
/// # 线程安全
///
/// V8 Isolate不是线程安全的。每个线程应创建独立的沙箱实例。
pub struct V8Sandbox {
    /// 缓存的 V8 Context（当 persistent_context 启用时复用）。
    cached_context: Option<v8::Global<v8::Context>>,
    /// V8 Isolate（拥有所有权）。
    isolate: Option<v8::OwnedIsolate>,
    /// 沙箱配置。
    config: SandboxConfig,
    /// 宿主注入的回调名 + 线程局部注册表索引（register_callback 注册），execute 时挂到全局对象。
    callbacks: Vec<(String, usize)>,
    /// SEC-13 持久超时看门狗（2026-08-10 重构）：常驻线程 + Arm/Disarm seq 协议，
    /// 替代每次 execute 创建/销毁 OS 线程（Windows 并行负载线程 churn 楔死修复）。
    watchdog_tx: Option<std::sync::mpsc::Sender<WatchdogMsg>>,
    watchdog_seq: u64,
    watchdog_join: Option<std::thread::JoinHandle<()>>,
}

impl V8Sandbox {
    /// 创建新的V8脚本沙箱。
    ///
    /// 首次调用时会初始化V8平台（全局一次性操作）。
    /// 使用默认配置。
    pub fn new() -> Result<Self, ScriptError> {
        Self::with_config(SandboxConfig::default())
    }

    /// 使用自定义配置创建V8脚本沙箱。
    pub fn with_config(config: SandboxConfig) -> Result<Self, ScriptError> {
        ensure_v8_initialized();

        let mut create_params = v8::Isolate::create_params();
        // 初始堆大小可配置（默认 0 = V8 按系统内存推导）；堆按需增长。上限与
        // 初始堆的合法组合见 v8_heap_limits（V8 要求 initial <= max）。
        if let Some((initial, max)) = crate::v8_heap_limits(&config) {
            create_params = create_params.heap_limits(initial, max);
        }

        // TEMP-DIAG（2026-08-10 windows tab_js_worker 挂起定位，定位后删除）：
        // 打印 isolate 创建耗时与调用线程信息。
        #[cfg(feature = "v8")]
        let t_diag = std::time::Instant::now();
        #[cfg(feature = "v8")]
        let t_diag_name = std::thread::current().name().unwrap_or("unnamed").to_string();

        let isolate = v8::Isolate::new(create_params);

        #[cfg(feature = "v8")]
        eprintln!(
            "[TEMP-DIAG] with_config: isolate created on thread '{t_diag_name}' in {:?}",
            t_diag.elapsed()
        );

        // SEC-13 持久看门狗（2026-08-10）：每 sandbox 一个常驻线程（execute 侧按
        // 当前 timeout_ms Arm），避免每次 execute spawn+join 的线程 churn。
        let (watchdog_tx, watchdog_rx) = std::sync::mpsc::channel();
        let watchdog_join = std::thread::Builder::new()
            .name("v8-sandbox-watchdog".to_string())
            .spawn(move || watchdog_main(watchdog_rx))
            .expect("spawn v8 sandbox watchdog");

        Ok(Self {
            isolate: Some(isolate),
            config,
            cached_context: None,
            callbacks: Vec::new(),
            watchdog_tx: Some(watchdog_tx),
            watchdog_seq: 0,
            watchdog_join: Some(watchdog_join),
        })
    }

    /// 注册宿主回调，挂为全局函数 `name`（JS-DOM-bridge：JS shim 调 `name(...)` 触发）。
    ///
    /// 回调闭包存入线程局部注册表（返回索引），`execute` 时按索引挂到当前 Context 的
    /// 全局对象。参数按字符串数组传入，返回字符串写入 JS 调用结果。
    /// 无 `register_callback` 调用时行为完全同今（零回归）。须在 `execute` 之前调用。
    #[allow(clippy::type_complexity)]
    pub fn register_callback(&mut self, name: &str, callback: Box<dyn Fn(&[String]) -> String + Send + Sync>) {
        let cb: HostCallback = Arc::from(callback);
        let idx = HOST_CALLBACKS.with(|cbs| {
            let mut cbs = cbs.borrow_mut();
            let idx = cbs.len();
            cbs.push(cb);
            idx
        });
        self.callbacks.push((name.to_string(), idx));
    }

    /// P1b S1 异步回调 resolve（方案 A）。Rust 异步完成后调此方法：执行 JS 全局
    /// `__zwResolveCallback(id, result)`，由 JS 侧 pending 表 resolve 对应 Promise。
    /// `execute` 末尾的 `perform_microtask_checkpoint` 会 drain 随后触发的 `.then`
    /// 微任务，故返回时 Promise 已 resolve。
    ///
    /// **防御**：JS 侧未注入 `__zwResolveCallback`（dom_bridge 未接通）时，外层
    /// `if(globalThis.__zwResolveCallback)` 守卫令本次 execute 的脚本结果为
    /// `undefined`，不报错（零回归）。`id`/`result` 经 [`js_string_literal`] 转义防注入。
    pub fn resolve_async_callback(&mut self, id: &str, result: &str) {
        let id_lit = js_string_literal(id);
        let result_lit = js_string_literal(result);
        let js = format!("if(globalThis.__zwResolveCallback){{__zwResolveCallback({id_lit},{result_lit});}}");
        let _ = self.execute(&js);
    }

    /// 设置脚本执行超时（毫秒）；0 表示无超时。
    pub fn set_timeout_ms(&mut self, timeout_ms: u64) {
        self.config.timeout_ms = timeout_ms;
    }

    /// 在沙箱中执行JavaScript脚本。
    ///
    /// # 参数
    ///
    /// - `code` — JavaScript源代码
    ///
    /// # 返回
    ///
    /// 返回脚本执行结果，包含返回值的字符串表示和执行耗时。
    pub fn execute(&mut self, code: &str) -> Result<ScriptResult, ScriptError> {
        if code.trim().is_empty() {
            return Err(ScriptError::InvalidInput("script is empty".into()));
        }

        let persistent = self.config.persistent_context;
        // SAFETY: cached_context 和 isolate 是不同的字段。
        // HandleScope 持有 isolate 的借用，不会触及 cached_context。
        // 原始指针允许我们在 isolate 被借用时访问 cached_context。
        let cached_ptr: *mut _ = &mut self.cached_context;

        let isolate = self.isolate.as_mut().ok_or(ScriptError::NotInitialized)?;
        // SAFETY: OwnedIsolate instances are entered on creation. Re-entering
        // around each execution makes multiple sandbox instances usable on the
        // same thread while preserving V8's stack-like enter/exit discipline.
        unsafe {
            isolate.enter();
        }
        let _enter_guard = IsolateEnterGuard { isolate };

        let start = std::time::Instant::now();

        // SEC-13: 强制执行 timeout——持久看门狗 Arm/Disarm（2026-08-10 重构：
        // 旧实现每次 execute spawn+join 一个 OS 线程，Windows 并行负载下线程
        // churn 楔死；guard Drop 统一撤除，顺带消除脚本出错提前返回时看门狗
        // 线程残留 5s 后 terminate_execution 误伤下一次执行的 latent bug）。
        let _timeout_guard: Option<WatchdogGuard> = if self.config.timeout_ms > 0 {
            let seq = self.watchdog_seq;
            self.watchdog_seq += 1;
            if let Some(tx) = &self.watchdog_tx {
                let _ = tx.send(WatchdogMsg::Arm {
                    seq,
                    timeout_ms: self.config.timeout_ms,
                    handle: isolate.thread_safe_handle(),
                });
                Some(WatchdogGuard { tx: tx.clone(), seq })
            } else {
                None
            }
        } else {
            None
        };

        v8::scope!(let hs, isolate);
        // SAFETY: cached_ptr 指向 self.cached_context，与 self.isolate 不重叠。
        // HandleScope 的借用仅涉及 isolate，不会修改 cached_context。
        let context = unsafe { resolve_context(persistent, cached_ptr, hs) };
        let mut ctx_scope = v8::ContextScope::new(hs, context);
        v8::tc_scope!(let try_catch, &mut ctx_scope);

        // 把宿主回调（register_callback 注册）挂到全局对象。无注册时为 no-op（零回归）。
        if !self.callbacks.is_empty() {
            let global = context.global(try_catch);
            for (name, idx) in &self.callbacks {
                let data = v8::Integer::new(try_catch, *idx as i32);
                let tmpl = v8::FunctionTemplate::builder(host_callback_invoke)
                    .data(data.into())
                    .build(try_catch);
                let Some(function) = tmpl.get_function(try_catch) else {
                    continue;
                };
                if let Some(key) = v8::String::new(try_catch, name) {
                    let _ = global.set(try_catch, key.into(), function.into());
                }
            }
        }

        // 编译脚本
        let code_str = v8::String::new(try_catch, code)
            .ok_or_else(|| ScriptError::InvalidInput("failed to create V8 string".into()))?;

        let script = v8::Script::compile(try_catch, code_str, None);
        if try_catch.has_caught() || script.is_none() {
            let msg = v8_try_catch_message!(try_catch);
            return Err(ScriptError::CompileError(msg));
        }
        let script = script.unwrap();

        // 执行脚本
        let result = script.run(try_catch);
        if try_catch.has_caught() || result.is_none() {
            // SEC-13：超时终止（看门狗 terminate_execution）——V8 终止异常在
            // TryCatch 表现为 caught 的 "null" 异常，须先于 RuntimeError 判定，
            // 报告为 Timeout（2026-08-10 持久看门狗重构时校正）。
            if try_catch.has_terminated() {
                try_catch.cancel_terminate_execution();
                return Err(ScriptError::Timeout(format!("{}ms", self.config.timeout_ms)));
            }
            if result.is_none() && !try_catch.has_caught() {
                try_catch.cancel_terminate_execution();
                return Err(ScriptError::Timeout(format!("{}ms", self.config.timeout_ms)));
            }
            let msg = v8_try_catch_message!(try_catch);
            return Err(ScriptError::RuntimeError(msg));
        }
        let result = result.unwrap();

        try_catch.perform_microtask_checkpoint();
        if try_catch.has_caught() {
            let msg = v8_try_catch_message!(try_catch);
            return Err(ScriptError::RuntimeError(msg));
        }

        // 转换结果为字符串
        let result_str = result
            .to_string(try_catch)
            .map(|s| s.to_rust_string_lossy(try_catch))
            .unwrap_or_default();

        let execution_time_ms = start.elapsed().as_secs_f64() * 1000.0;

        // 清理 timeout guard（Drop → Disarm 撤除看门狗；成功与错误路径统一）
        drop(_timeout_guard);

        Ok(ScriptResult {
            value: result_str,
            execution_time_ms,
        })
    }

    /// 在沙箱中执行JavaScript脚本并返回JSON字符串。
    pub fn execute_json(&mut self, code: &str) -> Result<ScriptResult, ScriptError> {
        if code.trim().is_empty() {
            return Err(ScriptError::InvalidInput("script is empty".into()));
        }

        let persistent = self.config.persistent_context;
        // SAFETY: 同 execute() 中的说明
        let cached_ptr: *mut _ = &mut self.cached_context;

        let isolate = self.isolate.as_mut().ok_or(ScriptError::NotInitialized)?;
        // SAFETY: See execute().
        unsafe {
            isolate.enter();
        }
        let _enter_guard = IsolateEnterGuard { isolate };

        let start = std::time::Instant::now();

        v8::scope!(let hs, isolate);
        let context = unsafe { resolve_context(persistent, cached_ptr, hs) };
        let mut ctx_scope = v8::ContextScope::new(hs, context);
        v8::tc_scope!(let try_catch, &mut ctx_scope);

        let code_str = v8::String::new(try_catch, code)
            .ok_or_else(|| ScriptError::InvalidInput("failed to create V8 string".into()))?;

        let script = v8::Script::compile(try_catch, code_str, None);
        if try_catch.has_caught() || script.is_none() {
            let msg = v8_try_catch_message!(try_catch);
            return Err(ScriptError::CompileError(msg));
        }
        let script = script.unwrap();

        let result = script.run(try_catch);
        if try_catch.has_caught() || result.is_none() {
            let msg = v8_try_catch_message!(try_catch);
            return Err(ScriptError::RuntimeError(msg));
        }
        let result = result.unwrap();

        try_catch.perform_microtask_checkpoint();
        if try_catch.has_caught() {
            let msg = v8_try_catch_message!(try_catch);
            return Err(ScriptError::RuntimeError(msg));
        }

        // 尝试JSON.stringify
        let json_str = Self::value_to_json_string(try_catch, result);

        let execution_time_ms = start.elapsed().as_secs_f64() * 1000.0;

        Ok(ScriptResult {
            value: json_str,
            execution_time_ms,
        })
    }
}

/// 获取或创建 V8 Context。
///
/// 当 `persistent` 为 true 时，首次创建的 Context 会被缓存到 `cached` 中复用。
/// 否则每次调用都创建新的 Context（保证执行间状态隔离）。
///
/// # Safety
///
/// `cached_ptr` 必须指向一个有效的、不与 `scope` 所借用的 isolate 重叠的
/// `Option<Global<Context>>`。调用方需确保两个引用不冲突。
unsafe fn resolve_context<'s>(
    persistent: bool,
    cached_ptr: *mut Option<v8::Global<v8::Context>>,
    scope: &mut v8::PinScope<'s, '_, ()>,
) -> v8::Local<'s, v8::Context> {
    let cached = unsafe { &mut *cached_ptr };
    if !persistent {
        return v8::Context::new(scope, Default::default());
    }

    // 尝试复用缓存的 Context
    if let Some(ref cached_ctx) = *cached {
        return v8::Local::new(scope, cached_ctx);
    }

    // 首次执行：创建并缓存 Context
    let context = v8::Context::new(scope, Default::default());
    *cached = Some(v8::Global::new(scope, context));
    context
}

impl V8Sandbox {
    /// 重置缓存的 V8 Context。
    ///
    /// 下次 execute 时会创建新的 Context。仅在 `persistent_context` 模式下有意义。
    pub fn reset_context(&mut self) {
        self.cached_context = None;
    }

    /// P1b S2 escape-hatch：进入持久 V8 Context，在 raw scope + context 内执行 `f`。
    ///
    /// 供 [`Sandbox::install_native_bindings`] 安装原生 DOM 绑定（`ObjectTemplate`/
    /// `FunctionTemplate`/accessor）——与 `execute` 共享同一持久 Context + scope setup
    /// （isolate.enter + `scope!` + `resolve_context` + `ContextScope`），故 `f` 安装的
    /// 全局对象/模板对后续 `execute` 可见。无 isolate / 未初始化时返 `None`。
    ///
    /// 镜像 [`V8Sandbox::execute`] 的 scope 进入（含 `IsolateEnterGuard` 的 enter/exit 配对）。
    fn with_context<R>(&mut self, f: impl FnOnce(&mut v8::PinScope, v8::Local<v8::Context>) -> R) -> Option<R> {
        let persistent = self.config.persistent_context;
        // SAFETY: 同 execute()——cached_context 与 isolate 为不同字段；raw ptr 借用
        // cached_context 不与 HandleScope 的 isolate 借用重叠。
        let cached_ptr: *mut _ = &mut self.cached_context;
        let isolate = self.isolate.as_mut()?;
        // SAFETY: enter/exit 配对（IsolateEnterGuard drop 时 exit），维持 V8 栈式纪律。
        unsafe {
            isolate.enter();
        }
        let _enter_guard = IsolateEnterGuard { isolate };
        v8::scope!(let hs, isolate);
        // SAFETY: cached_ptr 指向 self.cached_context，与 isolate 不重叠。
        let context = unsafe { resolve_context(persistent, cached_ptr, hs) };
        let mut ctx_scope = v8::ContextScope::new(hs, context);
        v8::tc_scope!(let scope, &mut ctx_scope);
        Some(f(scope, context))
    }

    /// 获取V8引擎版本号。
    pub fn v8_version() -> &'static str {
        ensure_v8_initialized();
        v8::V8::get_version()
    }

    /// 将V8值转换为JSON字符串。
    fn value_to_json_string(
        scope: &mut v8::PinnedRef<v8::TryCatch<v8::HandleScope>>,
        value: v8::Local<v8::Value>,
    ) -> String {
        let context = scope.get_current_context();
        let global = context.global(scope);

        let json_key = v8::String::new(scope, "JSON").unwrap();
        let json_val = global.get(scope, json_key.into());

        let Some(json_val) = json_val else {
            return value
                .to_string(scope)
                .map(|s| s.to_rust_string_lossy(scope))
                .unwrap_or_default();
        };

        if json_val.is_undefined() {
            return value
                .to_string(scope)
                .map(|s| s.to_rust_string_lossy(scope))
                .unwrap_or_default();
        }

        let Some(json_obj) = json_val.to_object(scope) else {
            return value
                .to_string(scope)
                .map(|s| s.to_rust_string_lossy(scope))
                .unwrap_or_default();
        };

        let stringify_key = v8::String::new(scope, "stringify").unwrap();
        let stringify_val = json_obj.get(scope, stringify_key.into());

        let Some(stringify_val) = stringify_val else {
            return value
                .to_string(scope)
                .map(|s| s.to_rust_string_lossy(scope))
                .unwrap_or_default();
        };

        if !stringify_val.is_function() {
            return value
                .to_string(scope)
                .map(|s| s.to_rust_string_lossy(scope))
                .unwrap_or_default();
        }

        let Ok(stringify_fn) = v8::Local::<v8::Function>::try_from(stringify_val) else {
            return value
                .to_string(scope)
                .map(|s| s.to_rust_string_lossy(scope))
                .unwrap_or_default();
        };

        let args = [value];
        let result = stringify_fn.call(scope, json_obj.into(), &args);

        result
            .and_then(|v| v.to_string(scope))
            .map(|s| s.to_rust_string_lossy(scope))
            .unwrap_or_else(|| {
                value
                    .to_string(scope)
                    .map(|s| s.to_rust_string_lossy(scope))
                    .unwrap_or_default()
            })
    }
}

impl Drop for V8Sandbox {
    fn drop(&mut self) {
        // 先停看门狗线程并 join（防其持 ThreadSafeHandle 对即将销毁的 isolate 调
        // terminate_execution），再释放 isolate 与 context。
        // TEMP-DIAG（2026-08-10 windows tab_js_worker 挂起定位，定位后删除）
        #[cfg(feature = "v8")]
        eprintln!("[TEMP-DIAG] V8Sandbox::drop: sending Stop");
        if let Some(tx) = &self.watchdog_tx {
            let _ = tx.send(WatchdogMsg::Stop);
        }
        if let Some(join) = self.watchdog_join.take() {
            // TEMP-DIAG（2026-08-10 windows tab_js_worker 挂起定位，定位后删除）
            #[cfg(feature = "v8")]
            eprintln!("[TEMP-DIAG] V8Sandbox::drop: joining watchdog");
            let _ = join.join();
        }
        // TEMP-DIAG（2026-08-10 windows tab_js_worker 挂起定位，定位后删除）
        #[cfg(feature = "v8")]
        eprintln!("[TEMP-DIAG] V8Sandbox::drop: watchdog joined, dropping isolate");
        self.cached_context = None;
        self.isolate = None;
        // TEMP-DIAG（2026-08-10 windows tab_js_worker 挂起定位，定位后删除）
        #[cfg(feature = "v8")]
        eprintln!("[TEMP-DIAG] V8Sandbox::drop: isolate dropped, drop complete");
    }
}

impl crate::Sandbox for V8Sandbox {
    fn execute(&mut self, code: &str) -> Result<ScriptResult, ScriptError> {
        V8Sandbox::execute(self, code)
    }
    fn execute_json(&mut self, code: &str) -> Result<ScriptResult, ScriptError> {
        V8Sandbox::execute_json(self, code)
    }
    fn register_callback(&mut self, name: &str, callback: Box<dyn Fn(&[String]) -> String + Send + Sync>) {
        V8Sandbox::register_callback(self, name, callback)
    }
    fn resolve_async_callback(&mut self, id: &str, result: &str) {
        V8Sandbox::resolve_async_callback(self, id, result);
    }
    fn set_timeout_ms(&mut self, timeout_ms: u64) {
        V8Sandbox::set_timeout_ms(self, timeout_ms)
    }
    fn reset_context(&mut self) {
        V8Sandbox::reset_context(self)
    }
    fn config(&self) -> &SandboxConfig {
        &self.config
    }
    #[allow(clippy::type_complexity)] // escape-hatch 闭包类型（镜像 register_callback 模式）
    fn install_native_bindings(
        &mut self,
        installer: Box<dyn FnOnce(&mut v8::PinScope, v8::Local<v8::Context>)>,
    ) -> bool {
        self.with_context(|scope, ctx| installer(scope, ctx)).is_some()
    }
}

impl Default for V8Sandbox {
    fn default() -> Self {
        Self::new().expect("V8 sandbox creation should not fail")
    }
}

impl std::fmt::Debug for V8Sandbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("V8Sandbox")
            .field("config", &self.config)
            .field("initialized", &self.isolate.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 创建与初始化 ──

    #[test]
    fn test_sandbox_new() {
        let sandbox = V8Sandbox::new();
        assert!(sandbox.is_ok(), "V8 sandbox creation should succeed");
    }

    #[test]
    fn test_sandbox_default() {
        let sandbox = V8Sandbox::default();
        let _ = &sandbox;
    }

    #[test]
    fn test_sandbox_debug_format() {
        let sandbox = V8Sandbox::new().unwrap();
        let debug = format!("{sandbox:?}");
        assert!(debug.contains("V8Sandbox"));
        assert!(debug.contains("initialized"));
    }

    #[test]
    fn test_sandbox_with_config() {
        let config = SandboxConfig {
            heap_limit: 32 * 1024 * 1024, // 32MB
            timeout_ms: 5000,
            persistent_context: false,
            ..Default::default()
        };
        let sandbox = V8Sandbox::with_config(config);
        assert!(sandbox.is_ok());
    }

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert_eq!(config.heap_limit, 0);
        assert_eq!(config.timeout_ms, 0);
        assert!(!config.persistent_context, "默认应使用每次创建新 Context");
    }

    #[test]
    fn test_v8_version() {
        let version = V8Sandbox::v8_version();
        assert!(!version.is_empty(), "V8 version should not be empty");
        assert!(version.contains('.'), "Version should contain dots");
    }

    // ── 脚本执行：正常路径 ──

    #[test]
    fn test_execute_simple_expression() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("1 + 1").unwrap();
        assert_eq!(result.value, "2");
        assert!(result.execution_time_ms >= 0.0);
    }

    #[test]
    fn test_execute_string_concat() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("'Hello' + ' ' + 'World'").unwrap();
        assert_eq!(result.value, "Hello World");
    }

    #[test]
    fn test_execute_variable_declaration() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("var x = 42; x").unwrap();
        assert_eq!(result.value, "42");
    }

    // ── 宿主回调（register_callback，JS-DOM-bridge Phase 0）──

    #[test]
    fn test_register_callback_invoked_from_js() {
        let mut sandbox = V8Sandbox::new().unwrap();
        sandbox.register_callback(
            "__zw_test",
            Box::new(|args| format!("echo:{}:{}", args.len(), args.join("|"))),
        );
        // JS 调用宿主回调，返回值成为脚本结果。
        let result = sandbox.execute("__zw_test('a', 'bb')").unwrap();
        assert_eq!(result.value, "echo:2:a|bb");
    }

    #[test]
    fn test_register_callback_persistent_context() {
        // persistent_context=true 时回调仍须在缓存的 Context 上生效。
        let config = SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).unwrap();
        sandbox.register_callback("__zw_greet", Box::new(|args| format!("hi {}", args[0])));
        let r1 = sandbox.execute("__zw_greet('world')").unwrap();
        assert_eq!(r1.value, "hi world");
        // 第二次 execute 复用缓存 Context，回调仍可用。
        let r2 = sandbox.execute("__zw_greet('again')").unwrap();
        assert_eq!(r2.value, "hi again");
    }

    // ── P1b S1 异步回调 resolve（方案 A）──

    #[test]
    fn test_resolve_async_callback_resolves_pending_promise() {
        // 方案 A 端到端：宿主回调同步返「回调 ID」，JS 端建 pending Promise；
        // Rust 异步完成后 resolve_async_callback 触发 __zwResolveCallback resolve。
        let config = SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).unwrap();
        // JS 侧 __zwResolveCallback + pending 表（生产由 dom_bridge 注入）。
        sandbox
            .execute(
                "globalThis.__zw_pending = {};
                 globalThis.__zwResolveCallback = function(id, result) {
                     var r = globalThis.__zw_pending[id];
                     if (r) { r(result); delete globalThis.__zw_pending[id]; }
                 };",
            )
            .unwrap();
        // 宿主回调返回调 ID（同步，方案 A）。
        sandbox.register_callback("__zw_start", Box::new(|args| format!("id:{}", args[0])));
        // JS：调宿主回调拿 ID，建 Promise 存 pending[id]，then 写全局 __result。
        sandbox
            .execute(
                "var id = __zw_start('7');
                 new Promise(function(resolve){ globalThis.__zw_pending[id] = resolve; })
                     .then(function(v){ globalThis.__result = v; });",
            )
            .unwrap();
        // resolve 前：Promise pending，__result 未设。
        let before = sandbox.execute("typeof globalThis.__result").unwrap();
        assert_eq!(before.value, "undefined");
        // Rust 异步完成 → resolve（execute 内 perform_microtask_checkpoint drain then）。
        sandbox.resolve_async_callback("id:7", "done-value");
        // resolve 后：__result 已被 .then 写入。
        let after = sandbox.execute("globalThis.__result").unwrap();
        assert_eq!(after.value, "done-value");
    }

    #[test]
    fn test_resolve_async_callback_safe_when_resolver_not_injected() {
        // 防御：JS 侧未注入 __zwResolveCallback（dom_bridge 未接通）时不报错（零回归）。
        let mut sandbox = V8Sandbox::new().unwrap();
        // 不注入 __zwResolveCallback，直接调 resolve_async_callback。
        sandbox.resolve_async_callback("id:x", "v");
        // 沙箱仍可用。
        let r = sandbox.execute("1 + 1").unwrap();
        assert_eq!(r.value, "2");
    }

    #[test]
    fn test_resolve_async_callback_escapes_injection() {
        // id/result 含双引号/反斜杠/代码片段时，js_string_literal 须转义，不得逃逸为 JS 代码。
        let config = SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).unwrap();
        sandbox
            .execute(
                "globalThis.__zw_pending = {};
                 globalThis.__zw_evil_called = false;
                 globalThis.__zwResolveCallback = function(id, result) {
                     globalThis.__zw_received = result;
                 };",
            )
            .unwrap();
        // 恶意 payload：试图闭合字符串注入 evil()。js_string_literal 须使其被当作纯字符串。
        sandbox.resolve_async_callback("a\", globalThis.__zw_evil_called = true, \"b", "x\"); evil");
        // evil 未执行（evil_called 仍 false），payload 作为字符串原样送达。
        let evil = sandbox.execute("globalThis.__zw_evil_called").unwrap();
        assert_eq!(evil.value, "false");
        let received = sandbox.execute("globalThis.__zw_received").unwrap();
        assert_eq!(received.value, "x\"); evil");
    }

    #[test]
    fn test_js_string_literal_escapes_special_chars() {
        // 直接单测转义器：常见注入向量与控制字符。
        assert_eq!(js_string_literal("plain"), "\"plain\"");
        assert_eq!(js_string_literal("a\"b"), "\"a\\\"b\"");
        assert_eq!(js_string_literal("a\\b"), "\"a\\\\b\"");
        assert_eq!(js_string_literal("a\nb"), "\"a\\nb\"");
        assert_eq!(js_string_literal("a\tb"), "\"a\\tb\"");
        // 控制字符（U+0001）转 。
        assert_eq!(js_string_literal("a\u{0001}b"), "\"a\\u0001b\"");
    }

    #[test]
    fn test_no_callback_zero_impact() {
        // 无 register_callback 时 execute 行为完全同今（零回归）。
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("6 * 7").unwrap();
        assert_eq!(result.value, "42");
    }

    #[test]
    fn test_execute_function_call() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("(function() { return 'success'; })()").unwrap();
        assert_eq!(result.value, "success");
    }

    #[test]
    fn test_execute_object_creation() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("({ name: 'test', value: 123 })").unwrap();
        // V8的object toString返回"[object Object]"
        assert!(
            result.value.contains("test") || result.value.contains("[object Object]"),
            "Expected object representation, got: {}",
            result.value
        );
    }

    #[test]
    fn test_execute_array() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("[1, 2, 3]").unwrap();
        assert_eq!(result.value, "1,2,3");
    }

    #[test]
    fn test_execute_boolean() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("true").unwrap();
        assert_eq!(result.value, "true");
    }

    #[test]
    fn test_execute_undefined() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("undefined").unwrap();
        assert_eq!(result.value, "undefined");
    }

    #[test]
    fn test_execute_null() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("null").unwrap();
        assert_eq!(result.value, "null");
    }

    #[test]
    fn test_execute_math() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("Math.PI").unwrap();
        let pi: f64 = result.value.parse().unwrap();
        assert!((pi - std::f64::consts::PI).abs() < 0.001);
    }

    #[test]
    fn test_execute_json_parse() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("JSON.parse('{\"a\":1}').a").unwrap();
        assert_eq!(result.value, "1");
    }

    #[test]
    fn test_execute_json_stringify() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("JSON.stringify({x: 1})").unwrap();
        assert_eq!(result.value, "{\"x\":1}");
    }

    #[test]
    fn test_execute_template_literal() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("`hello ${42}`").unwrap();
        assert_eq!(result.value, "hello 42");
    }

    #[test]
    fn test_execute_arrow_function() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("((x) => x * 2)(21)").unwrap();
        assert_eq!(result.value, "42");
    }

    #[test]
    fn test_execute_destructuring() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("const {a, b} = {a: 1, b: 2}; a + b").unwrap();
        assert_eq!(result.value, "3");
    }

    #[test]
    fn test_execute_spread_operator() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("const arr = [1, 2, 3]; [...arr, 4].length").unwrap();
        assert_eq!(result.value, "4");
    }

    #[test]
    fn test_execute_promise() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("Promise.resolve(42)").unwrap();
        assert!(result.value.contains("Promise") || result.value == "42");
    }

    #[test]
    fn test_execution_time_positive() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox
            .execute("var sum = 0; for (var i = 0; i < 1000; i++) sum += i; sum")
            .unwrap();
        assert_eq!(result.value, "499500");
        assert!(result.execution_time_ms >= 0.0);
    }

    #[test]
    fn test_execute_unicode() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("'こんにちは世界 🌍'").unwrap();
        assert!(result.value.contains("こんにちは"));
    }

    #[test]
    fn test_execute_class() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox
            .execute("class Point { constructor(x, y) { this.x = x; this.y = y; } } new Point(1, 2).x")
            .unwrap();
        assert_eq!(result.value, "1");
    }

    #[test]
    fn test_execute_let_const() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("let a = 10; const b = 20; a + b").unwrap();
        assert_eq!(result.value, "30");
    }

    #[test]
    fn test_execute_typeof() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("typeof 42").unwrap();
        assert_eq!(result.value, "number");
    }

    #[test]
    fn test_execute_typeof_string() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("typeof 'hello'").unwrap();
        assert_eq!(result.value, "string");
    }

    #[test]
    fn test_execute_typeof_function() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("typeof function(){}").unwrap();
        assert_eq!(result.value, "function");
    }

    #[test]
    fn test_execute_typeof_object() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("typeof {}").unwrap();
        assert_eq!(result.value, "object");
    }

    #[test]
    fn test_execute_array_methods() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("[1, 2, 3].map(x => x * 2).join(',')").unwrap();
        assert_eq!(result.value, "2,4,6");
    }

    #[test]
    fn test_execute_string_methods() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("'hello world'.split(' ').join('-')").unwrap();
        assert_eq!(result.value, "hello-world");
    }

    // ── 脚本执行：错误路径 ──

    #[test]
    fn test_execute_empty_script() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("");
        assert!(matches!(result, Err(ScriptError::InvalidInput(_))));
    }

    #[test]
    fn test_execute_whitespace_only() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("   ");
        assert!(matches!(result, Err(ScriptError::InvalidInput(_))));
    }

    #[test]
    fn test_execute_syntax_error() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("function(");
        assert!(matches!(result, Err(ScriptError::CompileError(_))));
    }

    #[test]
    fn test_execute_undefined_variable() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("nonExistentVariable");
        assert!(matches!(result, Err(ScriptError::RuntimeError(_))));
    }

    #[test]
    fn test_execute_type_error() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("null.toString()");
        assert!(matches!(result, Err(ScriptError::RuntimeError(_))));
    }

    #[test]
    fn test_execute_reference_error() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("x");
        assert!(matches!(result, Err(ScriptError::RuntimeError(_))));
    }

    #[test]
    fn test_execute_range_error() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("new Array(-1)");
        assert!(matches!(result, Err(ScriptError::RuntimeError(_))));
    }

    #[test]
    fn test_compile_error_has_message() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("function(");
        if let Err(ScriptError::CompileError(msg)) = result {
            assert!(!msg.is_empty(), "Compile error message should not be empty");
        } else {
            panic!("Expected CompileError");
        }
    }

    // ── execute_json ──

    #[test]
    fn test_execute_json_object() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute_json("({a: 1, b: 2})").unwrap();
        assert!(result.value.contains("\"a\""));
        assert!(result.value.contains("\"b\""));
    }

    #[test]
    fn test_execute_json_array() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute_json("[1, 2, 3]").unwrap();
        assert_eq!(result.value, "[1,2,3]");
    }

    #[test]
    fn test_execute_json_string() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute_json("'hello'").unwrap();
        assert_eq!(result.value, "\"hello\"");
    }

    #[test]
    fn test_execute_json_number() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute_json("42").unwrap();
        assert_eq!(result.value, "42");
    }

    // ── 多次执行 ──

    #[test]
    fn test_multiple_executions() {
        let mut sandbox = V8Sandbox::new().unwrap();

        let r1 = sandbox.execute("1 + 1").unwrap();
        assert_eq!(r1.value, "2");

        let r2 = sandbox.execute("'hello'").unwrap();
        assert_eq!(r2.value, "hello");

        let r3 = sandbox.execute("Math.sqrt(144)").unwrap();
        assert_eq!(r3.value, "12");
    }

    #[test]
    fn test_multiple_sandboxes() {
        let mut sandbox1 = V8Sandbox::new().unwrap();
        let mut sandbox2 = V8Sandbox::new().unwrap();

        let r1 = sandbox1.execute("42").unwrap();
        assert_eq!(r1.value, "42");

        let r2 = sandbox2.execute("'hello'").unwrap();
        assert_eq!(r2.value, "hello");
    }

    // ── 错误类型 ──

    #[test]
    fn test_script_error_display() {
        let err = ScriptError::CompileError("unexpected token".into());
        assert!(err.to_string().contains("Compile error"));
        assert!(err.to_string().contains("unexpected token"));

        let err = ScriptError::RuntimeError("type error".into());
        assert!(err.to_string().contains("Runtime error"));

        let err = ScriptError::Timeout("5s".into());
        assert!(err.to_string().contains("timeout"));

        let err = ScriptError::NotInitialized;
        assert!(err.to_string().contains("not initialized"));

        let err = ScriptError::InvalidInput("empty".into());
        assert!(err.to_string().contains("Invalid input"));

        let err = ScriptError::EngineUnavailable("v8".into());
        assert!(err.to_string().contains("Engine unavailable"));
    }

    #[test]
    fn test_script_result_clone() {
        let result = ScriptResult {
            value: "42".to_string(),
            execution_time_ms: 1.5,
        };
        let cloned = result.clone();
        assert_eq!(cloned.value, "42");
        assert_eq!(cloned.execution_time_ms, 1.5);
    }

    #[test]
    fn test_script_result_debug() {
        let result = ScriptResult {
            value: "hello".to_string(),
            execution_time_ms: 0.1,
        };
        let debug = format!("{result:?}");
        assert!(debug.contains("hello"));
    }

    #[test]
    fn test_sandbox_config_clone() {
        let config = SandboxConfig {
            heap_limit: 1024,
            timeout_ms: 100,
            persistent_context: false,
            ..Default::default()
        };
        let cloned = config.clone();
        assert_eq!(cloned.heap_limit, 1024);
        assert_eq!(cloned.timeout_ms, 100);
    }

    #[test]
    fn test_sandbox_config_debug() {
        let config = SandboxConfig {
            heap_limit: 2048,
            timeout_ms: 200,
            persistent_context: false,
            ..Default::default()
        };
        let debug = format!("{config:?}");
        assert!(debug.contains("2048"));
        assert!(debug.contains("200"));
    }

    // ── 状态隔离测试 ──

    #[test]
    /// 测试多次 execute 调用之间变量不泄漏（每次创建新 Context）。
    fn test_state_isolation_between_executions() {
        let mut sandbox = V8Sandbox::new().unwrap();
        // 第一次执行定义变量 x
        let r1 = sandbox.execute("var x = 42; x").unwrap();
        assert_eq!(r1.value, "42");
        // 第二次执行应无法访问 x（新 Context）
        let r2 = sandbox.execute("typeof x === 'undefined'");
        match r2 {
            Ok(result) => assert_eq!(
                result.value, "true",
                "variable from previous execution should not be visible"
            ),
            Err(ScriptError::RuntimeError(_)) => {} // 访问未定义变量抛出 ReferenceError 也是合法
            Err(e) => panic!("Unexpected error: {e}"),
        }
    }

    #[test]
    /// 测试多个独立沙箱之间状态完全隔离。
    fn test_state_isolation_between_sandboxes() {
        let mut sandbox1 = V8Sandbox::new().unwrap();
        let mut sandbox2 = V8Sandbox::new().unwrap();
        sandbox1.execute("var secret = 123").unwrap();
        let r2 = sandbox2.execute("typeof secret === 'undefined'");
        assert_eq!(r2.unwrap().value, "true");
    }

    // ── execute_json 边界测试 ──

    #[test]
    /// 测试 execute_json 处理嵌套对象。
    fn test_execute_json_nested_object() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute_json("({a: {b: {c: 1}}})").unwrap();
        assert!(result.value.contains("\"a\""));
        assert!(result.value.contains("\"b\""));
        assert!(result.value.contains("\"c\""));
        assert!(result.value.contains("1"));
    }

    #[test]
    /// 测试 execute_json 处理空对象。
    fn test_execute_json_empty_object() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute_json("({})").unwrap();
        assert_eq!(result.value, "{}");
    }

    #[test]
    /// 测试 execute_json 处理空数组。
    fn test_execute_json_empty_array() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute_json("([])").unwrap();
        assert_eq!(result.value, "[]");
    }

    #[test]
    /// 测试 execute_json 处理 null。
    fn test_execute_json_null() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute_json("null").unwrap();
        assert_eq!(result.value, "null");
    }

    #[test]
    /// 测试 execute_json 处理 boolean。
    fn test_execute_json_boolean() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let r1 = sandbox.execute_json("true").unwrap();
        assert_eq!(r1.value, "true");
        let r2 = sandbox.execute_json("false").unwrap();
        assert_eq!(r2.value, "false");
    }

    #[test]
    /// 测试 execute_json 处理 undefined（JSON 序列化为 undefined 通常是 undefined）。
    fn test_execute_json_undefined() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute_json("undefined").unwrap();
        // JSON.stringify(undefined) 返回 undefined（非字符串），value_to_json_string 应处理
        assert!(result.value == "null" || result.value == "undefined" || result.value.is_empty());
    }

    #[test]
    /// 测试 execute_json 对语法错误返回 CompileError。
    fn test_execute_json_syntax_error() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute_json("function(");
        assert!(matches!(result, Err(ScriptError::CompileError(_))));
    }

    #[test]
    /// 测试 execute_json 对空脚本返回 InvalidInput。
    fn test_execute_json_empty_script() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute_json("");
        assert!(matches!(result, Err(ScriptError::InvalidInput(_))));
    }

    #[test]
    /// 测试 execute_json 处理含特殊字符的字符串。
    fn test_execute_json_special_chars() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute_json("'hello\\nworld\\t!'").unwrap();
        assert!(result.value.contains("hello"));
    }

    // ── 大脚本与性能边界测试 ──

    #[test]
    /// 测试执行较大脚本（10000 次循环）正常返回。
    fn test_execute_large_loop() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let code = "var sum = 0; for (var i = 0; i < 10000; i++) { sum += i; } sum";
        let result = sandbox.execute(code).unwrap();
        assert_eq!(result.value, "49995000");
    }

    #[test]
    /// 测试执行生成大字符串的脚本。
    fn test_execute_large_string_concat() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let code = "var s = ''; for (var i = 0; i < 1000; i++) { s += 'a'; } s.length";
        let result = sandbox.execute(code).unwrap();
        assert_eq!(result.value, "1000");
    }

    // ── 更多 ES6+ 特性测试 ──

    #[test]
    /// 测试 Map 数据结构。
    fn test_execute_map() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox
            .execute("const m = new Map([[1,'a'],[2,'b']]); m.get(1)")
            .unwrap();
        assert_eq!(result.value, "a");
    }

    #[test]
    /// 测试 Set 数据结构。
    fn test_execute_set() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("const s = new Set([1,2,2,3]); s.size").unwrap();
        assert_eq!(result.value, "3");
    }

    #[test]
    /// 测试 Symbol。
    fn test_execute_symbol() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("typeof Symbol('test')").unwrap();
        assert_eq!(result.value, "symbol");
    }

    #[test]
    /// 测试 Proxy。
    fn test_execute_proxy() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox
            .execute("const p = new Proxy({}, { get: (t, k) => k === 'name' ? 'zero' : undefined }); p.name")
            .unwrap();
        assert_eq!(result.value, "zero");
    }

    #[test]
    /// 测试 async/await 语法（返回 Promise）。
    fn test_execute_async_await() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("(async () => 42)()").unwrap();
        // V8 可能输出 Promise 对象或 42
        assert!(result.value.contains("42") || result.value.contains("Promise"));
    }

    #[test]
    /// 测试默认参数。
    fn test_execute_default_parameters() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox.execute("(function(x = 10) { return x; })()").unwrap();
        assert_eq!(result.value, "10");
    }

    #[test]
    /// 测试 rest 参数。
    fn test_execute_rest_parameters() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox
            .execute("(function(...args) { return args.length; })(1,2,3)")
            .unwrap();
        assert_eq!(result.value, "3");
    }

    #[test]
    /// 测试 for...of 循环。
    fn test_execute_for_of() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox
            .execute("let sum = 0; for (const x of [1,2,3]) sum += x; sum")
            .unwrap();
        assert_eq!(result.value, "6");
    }

    #[test]
    /// 测试 Object.entries / Object.values / Object.keys。
    fn test_execute_object_static_methods() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let r1 = sandbox.execute("Object.keys({a:1,b:2}).length").unwrap();
        assert_eq!(r1.value, "2");
        let r2 = sandbox
            .execute("Object.values({a:1,b:2}).reduce((s,v)=>s+v,0)")
            .unwrap();
        assert_eq!(r2.value, "3");
        let r3 = sandbox.execute("Object.entries({x:10}).length").unwrap();
        assert_eq!(r3.value, "1");
    }

    #[test]
    /// 测试 Array.isArray / Array.from / Array.of。
    fn test_execute_array_static_methods() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let r1 = sandbox.execute("Array.isArray([1,2])").unwrap();
        assert_eq!(r1.value, "true");
        let r2 = sandbox.execute("Array.from('abc').length").unwrap();
        assert_eq!(r2.value, "3");
        let r3 = sandbox.execute("Array.of(1,2,3).length").unwrap();
        assert_eq!(r3.value, "3");
    }

    #[test]
    /// 测试 JSON.parse / JSON.stringify 可用。
    fn test_execute_json_builtins() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let r1 = sandbox.execute("JSON.parse('{\"a\":1}').a").unwrap();
        assert_eq!(r1.value, "1");
        let r2 = sandbox.execute("JSON.stringify({b:2})").unwrap();
        assert!(r2.value.contains("\"b\""));
    }

    #[test]
    /// 测试 Math 常用方法。
    fn test_execute_math_methods() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let r1 = sandbox.execute("Math.max(1, 5, 3)").unwrap();
        assert_eq!(r1.value, "5");
        let r2 = sandbox.execute("Math.min(1, 5, 3)").unwrap();
        assert_eq!(r2.value, "1");
        let r3 = sandbox.execute("Math.round(2.7)").unwrap();
        assert_eq!(r3.value, "3");
        let r4 = sandbox.execute("Math.abs(-42)").unwrap();
        assert_eq!(r4.value, "42");
    }

    /// 验证：V8Sandbox::with_config() with custom heap limit and timeout
    #[test]
    fn test_v8_sandbox_with_config_custom() {
        let custom_config = SandboxConfig {
            heap_limit: 64 * 1024 * 1024, // 64MB
            timeout_ms: 10000,
            persistent_context: false,
            ..Default::default()
        };
        let sandbox = V8Sandbox::with_config(custom_config);
        assert!(sandbox.is_ok(), "Custom config should create sandbox successfully");

        let mut sandbox = sandbox.unwrap();

        // 测试能够执行脚本
        let result = sandbox.execute("1 + 1");
        assert!(result.is_ok(), "Sandbox with custom config should execute scripts");
        assert_eq!(result.unwrap().value, "2");
    }

    /// 验证：ScriptError variants: timeout errors, memory limit errors
    #[test]
    fn test_script_error_variants() {
        // 测试各种 ScriptError 变体
        let errors = [
            ScriptError::CompileError("syntax error".to_string()),
            ScriptError::RuntimeError("reference error".to_string()),
            ScriptError::Timeout("5s".to_string()),
            ScriptError::NotInitialized,
            ScriptError::InvalidInput("empty".to_string()),
            ScriptError::EngineUnavailable("v8".to_string()),
        ];

        for error in errors {
            // 测试每个错误的 Display 实现
            let error_str = error.to_string();
            assert!(!error_str.is_empty(), "Error string should not be empty");
        }

        // 验证错误类型的区分
        let compile_error = ScriptError::CompileError("syntax".to_string());
        let runtime_error = ScriptError::RuntimeError("runtime".to_string());
        let timeout_error = ScriptError::Timeout("5s".to_string());

        assert_ne!(compile_error.to_string(), runtime_error.to_string());
        assert_ne!(compile_error.to_string(), timeout_error.to_string());
    }

    /// 验证：execute_json with arrays, nested objects, null
    #[test]
    fn test_execute_json_complex_values() {
        let mut sandbox = V8Sandbox::new().unwrap();

        // 测试数组
        let array_result = sandbox.execute_json("[1, 2, 3, 'hello', null]").unwrap();
        assert!(array_result.value.contains("["));
        assert!(array_result.value.contains("]"));
        assert!(array_result.value.contains("1"));
        assert!(array_result.value.contains("hello"));

        // 测试嵌套对象
        let nested_result = sandbox
            .execute_json("({person: {name: 'John', age: 30, hobbies: ['reading', 'coding']}})")
            .unwrap();
        assert!(nested_result.value.contains("John"));
        assert!(nested_result.value.contains("30"));
        assert!(nested_result.value.contains("reading"));

        // 测试 null
        let null_result = sandbox.execute_json("null").unwrap();
        assert_eq!(null_result.value, "null");
    }

    /// 验证：v8_version() returns non-empty string
    #[test]
    fn test_v8_version_non_empty() {
        let version = V8Sandbox::v8_version();
        assert!(!version.is_empty(), "V8 version should not be empty");
        assert!(version.contains('.'), "Version should contain dots");

        // 版本号应该符合语义化版本格式 (major.minor.patch)
        let parts: Vec<&str> = version.split('.').collect();
        assert!(parts.len() >= 2, "Version should have at least major and minor");

        // 主要版本号应该是数字
        if let Ok(major) = parts[0].parse::<u32>() {
            assert!(major > 0, "Major version should be positive");
        }
    }

    /// 验证：Empty script execution (should return InvalidInput)
    #[test]
    fn test_empty_script_execution() {
        let mut sandbox = V8Sandbox::new().unwrap();

        // 测试空字符串
        let result = sandbox.execute("");
        assert!(matches!(result, Err(ScriptError::InvalidInput(_))));

        // 测试只有空白字符
        let result = sandbox.execute("   ");
        assert!(matches!(result, Err(ScriptError::InvalidInput(_))));

        // 测试只有换行符
        let result = sandbox.execute("\n\t\r");
        assert!(matches!(result, Err(ScriptError::InvalidInput(_))));
    }

    /// 验证：Script returning undefined
    #[test]
    fn test_script_returning_undefined() {
        let mut sandbox = V8Sandbox::new().unwrap();

        // 测试直接返回 undefined
        let result = sandbox.execute("undefined");
        assert!(result.is_ok(), "Should execute undefined without error");
        assert_eq!(result.unwrap().value, "undefined");

        // 测试没有显式返回的函数
        let result = sandbox.execute("(function() {})()");
        assert!(result.is_ok(), "Should execute function without return");
        assert_eq!(result.unwrap().value, "undefined");

        // 测试变量声明但没有赋值
        let result = sandbox.execute("var x; x");
        assert!(result.is_ok(), "Should declare variable without value");
        assert_eq!(result.unwrap().value, "undefined");
    }

    /// 验证：Script with syntax error (verify error type)
    #[test]
    fn test_script_with_syntax_error() {
        let mut sandbox = V8Sandbox::new().unwrap();

        // 测试各种语法错误
        let syntax_error_scripts = [
            "function(",        // 函数定义不完整
            "var = 1",          // 缺少变量名
            "if {",             // if 语句不完整
            "for ()",           // for 循环缺少条件
            "42)",              // 多余的括号
            "'unclosed string", // 未闭合的字符串
        ];

        for script in syntax_error_scripts {
            let result = sandbox.execute(script);
            assert!(
                matches!(result, Err(ScriptError::CompileError(_))),
                "Script '{}' should return CompileError",
                script
            );
        }

        // 验证编译错误消息不为空
        let result = sandbox.execute("function(");
        if let Err(ScriptError::CompileError(msg)) = result {
            assert!(!msg.is_empty(), "Compile error message should not be empty");
            // V8 错误类型名首字母大写（如 "SyntaxError: Unexpected..."），按大小写不敏感匹配
            let lower = msg.to_lowercase();
            assert!(
                lower.contains("syntax") || lower.contains("error") || lower.contains("unexpected"),
                "Error message should indicate syntax issue, got: {msg}"
            );
        } else {
            panic!("Expected CompileError");
        }
    }

    // ── 持久化 Context（V8 快照优化）测试 ──

    #[test]
    /// 测试 persistent_context 模式下变量在多次 execute 间保持。
    fn test_persistent_context_state_persists() {
        let config = SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).unwrap();

        // 第一次执行：定义变量
        let r1 = sandbox.execute("var persistentVar = 42; persistentVar").unwrap();
        assert_eq!(r1.value, "42");

        // 第二次执行：变量应该仍然存在
        let r2 = sandbox.execute("persistentVar + 8").unwrap();
        assert_eq!(r2.value, "50");
    }

    #[test]
    /// 测试 persistent_context 模式下函数在多次 execute 间保持。
    fn test_persistent_context_function_persists() {
        let config = SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).unwrap();

        sandbox.execute("function add(a, b) { return a + b; }").unwrap();
        let r = sandbox.execute("add(3, 4)").unwrap();
        assert_eq!(r.value, "7");
    }

    #[test]
    /// 测试 persistent_context=false（默认）时状态隔离仍然有效。
    fn test_fresh_context_state_isolated() {
        let mut sandbox = V8Sandbox::new().unwrap();

        sandbox.execute("var x = 42; x").unwrap();
        let r = sandbox.execute("typeof x === 'undefined'");
        match r {
            Ok(result) => assert_eq!(result.value, "true", "默认模式下变量不应泄漏"),
            Err(ScriptError::RuntimeError(_)) => {} // ReferenceError 也合法
            Err(e) => panic!("Unexpected error: {e}"),
        }
    }

    #[test]
    /// 测试 reset_context 清除持久化上下文。
    fn test_reset_context_clears_state() {
        let config = SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).unwrap();

        sandbox.execute("var beforeReset = 99; beforeReset").unwrap();
        sandbox.reset_context();
        // reset 后应该在新 Context 中，变量不存在
        let r = sandbox.execute("typeof beforeReset === 'undefined'");
        match r {
            Ok(result) => assert_eq!(result.value, "true", "reset 后变量应消失"),
            Err(ScriptError::RuntimeError(_)) => {}
            Err(e) => panic!("Unexpected error: {e}"),
        }
    }

    #[test]
    /// 测试 persistent_context 模式下 execute_json 也复用上下文。
    fn test_persistent_context_execute_json() {
        let config = SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).unwrap();

        sandbox.execute("var data = {x: 1};").unwrap();
        let r = sandbox.execute_json("data").unwrap();
        assert!(r.value.contains("\"x\""), "execute_json 应看到之前定义的变量");
    }

    #[test]
    /// 测试 persistent_context 模式下多次执行不会累积内存问题。
    fn test_persistent_context_many_executions() {
        let config = SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).unwrap();

        for i in 0..50 {
            let code = format!("var v{i} = {i}; v{i}");
            let r = sandbox.execute(&code).unwrap();
            assert_eq!(r.value, i.to_string());
        }
    }

    #[test]
    /// SEC-13 超时看门狗（2026-08-10 持久化重构）：死循环脚本在 timeout_ms 后被
    /// terminate_execution 终止（Timeout 错误，非长期挂起）；且看门狗可恢复——
    /// 后续 execute 正常（旧 per-execute 线程实现下，脚本出错提前返回会残留
    /// 看门狗线程于 timeout 后误伤下一次执行；新 Arm/Disarm 协议经 guard Drop
    /// 统一撤除，无残留）。多次死循环→恢复循环验证 seq 协议无累积。
    fn test_execute_timeout_terminates_dead_loop_and_reuse() {
        let config = SandboxConfig {
            persistent_context: true,
            timeout_ms: 200,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).unwrap();

        let start = std::time::Instant::now();
        let r = sandbox.execute("while(true){}");
        assert!(
            matches!(r, Err(ScriptError::Timeout(_))),
            "死循环应在 timeout_ms 后被终止: {:?}",
            r.err()
        );
        assert!(
            start.elapsed().as_millis() < 5_000,
            "terminate 应在 ~timeout 内到达而非长期等待: {}ms",
            start.elapsed().as_millis()
        );
        // 看门狗撤除后 sandbox 可复用（无残留 terminate 误伤）。
        let r2 = sandbox.execute("1 + 1");
        assert!(r2.is_ok(), "terminate 后 sandbox 应可复用: {:?}", r2.err());
        assert_eq!(r2.unwrap().value, "2");
        // 连续多次死循环→恢复：seq 协议无累积残留。
        for _ in 0..3 {
            let r3 = sandbox.execute("while(true){}");
            assert!(matches!(r3, Err(ScriptError::Timeout(_))), "死循环应持续被终止: {r3:?}");
            assert_eq!(sandbox.execute("2 * 3").unwrap().value, "6");
        }
    }
}
