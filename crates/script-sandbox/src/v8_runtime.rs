//! V8引擎运行时实现。
//!
//! 封装rusty_v8，提供安全的JavaScript脚本执行沙箱。

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

// 宿主回调注册表（线程局部）。rusty_v8 0.32 的 FunctionTemplate 回调须为 `Copy`
//（MapFnTo<FunctionCallback>），无法捕获 Arc 状态；故回调闭包存于此注册表，
// FunctionTemplate 经 builder().data(idx) 携带索引，fn 回调按 idx 查表调用。
thread_local! {
    static HOST_CALLBACKS: RefCell<Vec<HostCallback>> = RefCell::new(Vec::new());
}

/// rusty_v8 FunctionTemplate 回调：按 args.data() 的索引查 HOST_CALLBACKS 调用。
fn host_callback_invoke(
    scope: &mut rusty_v8::HandleScope,
    args: rusty_v8::FunctionCallbackArguments,
    mut rv: rusty_v8::ReturnValue,
) {
    let idx = args.data().and_then(|d| d.integer_value(scope)).unwrap_or(-1);
    if idx < 0 {
        return;
    }
    let n = args.length();
    let strs: Vec<String> = (0..n)
        .filter_map(|i| args.get(i).to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
        .collect();
    let result = HOST_CALLBACKS.with(|cbs| cbs.borrow().get(idx as usize).map(|cb| cb(&strs)).unwrap_or_default());
    if let Some(s) = rusty_v8::String::new(scope, &result) {
        rv.set(s.into());
    }
}

/// V8平台初始化守卫（全局只初始化一次）。
static V8_INIT: Once = Once::new();

/// 确保V8平台已初始化。
fn ensure_v8_initialized() {
    V8_INIT.call_once(|| {
        let platform = rusty_v8::new_default_platform(0, false).make_shared();
        // SAFETY: V8平台初始化在进程生命周期内只调用一次，
        // 且在所有Isolate创建之前完成。
        #[allow(unused_unsafe)]
        unsafe {
            rusty_v8::V8::initialize_platform(platform);
        }
        rusty_v8::V8::initialize();
    });
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
    /// V8 Isolate（拥有所有权）。
    isolate: Option<rusty_v8::OwnedIsolate>,
    /// 沙箱配置。
    config: SandboxConfig,
    /// 缓存的 V8 Context（当 persistent_context 启用时复用）。
    cached_context: Option<rusty_v8::Global<rusty_v8::Context>>,
    /// 宿主注入的回调名 + 线程局部注册表索引（register_callback 注册），execute 时挂到全局对象。
    callbacks: Vec<(String, usize)>,
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

        let mut create_params = rusty_v8::Isolate::create_params();
        if config.heap_limit > 0 {
            create_params = create_params.heap_limits(0, config.heap_limit);
        }

        let isolate = rusty_v8::Isolate::new(create_params);

        Ok(Self {
            isolate: Some(isolate),
            config,
            cached_context: None,
            callbacks: Vec::new(),
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

        let start = std::time::Instant::now();

        // SEC-13: 强制执行 timeout
        let _timeout_guard: Option<(std::thread::JoinHandle<()>, std::sync::mpsc::Sender<()>)> =
            if self.config.timeout_ms > 0 {
                let handle = isolate.thread_safe_handle();
                let timeout_ms = self.config.timeout_ms;
                let (tx, rx) = std::sync::mpsc::channel::<()>();
                let thread = std::thread::spawn(move || {
                    // 等待超时或取消信号
                    if rx.recv_timeout(std::time::Duration::from_millis(timeout_ms)).is_err() {
                        // 超时：终止执行
                        handle.terminate_execution();
                    }
                });
                Some((thread, tx))
            } else {
                None
            };

        let mut hs = rusty_v8::HandleScope::new(isolate);
        // SAFETY: cached_ptr 指向 self.cached_context，与 self.isolate 不重叠。
        // HandleScope 的借用仅涉及 isolate，不会修改 cached_context。
        let context = unsafe { resolve_context(persistent, cached_ptr, &mut hs) };
        let mut ctx_scope = rusty_v8::ContextScope::new(&mut hs, context);
        let try_catch = &mut rusty_v8::TryCatch::new(&mut ctx_scope);

        // 把宿主回调（register_callback 注册）挂到全局对象。无注册时为 no-op（零回归）。
        if !self.callbacks.is_empty() {
            let global = context.global(try_catch);
            for (name, idx) in &self.callbacks {
                let data = rusty_v8::Integer::new(try_catch, *idx as i32);
                let tmpl = rusty_v8::FunctionTemplate::builder(host_callback_invoke)
                    .data(data.into())
                    .build(try_catch);
                let Some(function) = tmpl.get_function(try_catch) else {
                    continue;
                };
                if let Some(key) = rusty_v8::String::new(try_catch, name) {
                    let _ = global.set(try_catch, key.into(), function.into());
                }
            }
        }

        // 编译脚本
        let code_str = rusty_v8::String::new(try_catch, code)
            .ok_or_else(|| ScriptError::InvalidInput("failed to create V8 string".into()))?;

        let script = rusty_v8::Script::compile(try_catch, code_str, None);
        if try_catch.has_caught() || script.is_none() {
            let msg = v8_try_catch_message!(try_catch);
            return Err(ScriptError::CompileError(msg));
        }
        let script = script.unwrap();

        // 执行脚本
        let result = script.run(try_catch);
        if try_catch.has_caught() || result.is_none() {
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

        // 清理 timeout guard（通知定时器线程取消，等待其结束）
        if let Some((thread, tx)) = _timeout_guard {
            let _ = tx.send(()); // 取消定时器
            let _ = thread.join();
        }

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

        let start = std::time::Instant::now();

        let mut hs = rusty_v8::HandleScope::new(isolate);
        let context = unsafe { resolve_context(persistent, cached_ptr, &mut hs) };
        let mut ctx_scope = rusty_v8::ContextScope::new(&mut hs, context);
        let try_catch = &mut rusty_v8::TryCatch::new(&mut ctx_scope);

        let code_str = rusty_v8::String::new(try_catch, code)
            .ok_or_else(|| ScriptError::InvalidInput("failed to create V8 string".into()))?;

        let script = rusty_v8::Script::compile(try_catch, code_str, None);
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
    cached_ptr: *mut Option<rusty_v8::Global<rusty_v8::Context>>,
    scope: &mut rusty_v8::HandleScope<'s, ()>,
) -> rusty_v8::Local<'s, rusty_v8::Context> {
    let cached = unsafe { &mut *cached_ptr };
    if !persistent {
        return rusty_v8::Context::new(scope);
    }

    // 尝试复用缓存的 Context
    if let Some(ref cached_ctx) = *cached {
        return rusty_v8::Local::new(scope, cached_ctx);
    }

    // 首次执行：创建并缓存 Context
    let context = rusty_v8::Context::new(scope);
    *cached = Some(rusty_v8::Global::new(scope, context));
    context
}

impl V8Sandbox {
    /// 重置缓存的 V8 Context。
    ///
    /// 下次 execute 时会创建新的 Context。仅在 `persistent_context` 模式下有意义。
    pub fn reset_context(&mut self) {
        self.cached_context = None;
    }

    /// 获取V8引擎版本号。
    pub fn v8_version() -> &'static str {
        ensure_v8_initialized();
        rusty_v8::V8::get_version()
    }

    /// 将V8值转换为JSON字符串。
    fn value_to_json_string(scope: &mut rusty_v8::HandleScope, value: rusty_v8::Local<rusty_v8::Value>) -> String {
        let context = scope.get_current_context();
        let global = context.global(scope);

        let json_key = rusty_v8::String::new(scope, "JSON").unwrap();
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

        let stringify_key = rusty_v8::String::new(scope, "stringify").unwrap();
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

        let Ok(stringify_fn) = rusty_v8::Local::<rusty_v8::Function>::try_from(stringify_val) else {
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
    fn set_timeout_ms(&mut self, timeout_ms: u64) {
        V8Sandbox::set_timeout_ms(self, timeout_ms)
    }
    fn reset_context(&mut self) {
        V8Sandbox::reset_context(self)
    }
    fn config(&self) -> &SandboxConfig {
        &self.config
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
        sandbox.register_callback("__zw_test", |args| format!("echo:{}:{}", args.len(), args.join("|")));
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
        sandbox.register_callback("__zw_greet", |args| format!("hi {}", args[0]));
        let r1 = sandbox.execute("__zw_greet('world')").unwrap();
        assert_eq!(r1.value, "hi world");
        // 第二次 execute 复用缓存 Context，回调仍可用。
        let r2 = sandbox.execute("__zw_greet('again')").unwrap();
        assert_eq!(r2.value, "hi again");
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
}
