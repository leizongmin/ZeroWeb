//! QuickJS 引擎运行时实现。
//!
//! 封装 rquickjs，提供轻量级 JavaScript 脚本执行沙箱。
//! QuickJS 是一个小巧且可嵌入的 JS 引擎，支持 ES2023 规范的大部分特性。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::{SandboxConfig, ScriptError, ScriptResult};

/// 宿主回调类型（与 V8Sandbox 的 HostCallback 一致）。
type HostCallback = Arc<dyn Fn(&[String]) -> String + Send + Sync>;

/// 执行超时装载守卫：`arm` 设 `timeout_deadline = now + timeout_ms`（timeout_ms > 0），
/// Drop 时清 deadline，防 interrupt handler 误伤后续 execute。
///
/// R3400：QuickJS interrupt handler 在 runtime 级注册（跨 execute 持续），故每次
/// execute 须显式 arm/disarm 截止时刻——guard Drop 保证即便 eval 出错提前返回也清。
struct TimeoutGuard {
    deadline: Arc<Mutex<Option<Instant>>>,
}

impl TimeoutGuard {
    /// 装载超时：timeout_ms > 0 时设 deadline = now + timeout_ms，返回 guard。
    fn arm(deadline: &Arc<Mutex<Option<Instant>>>, timeout_ms: u64) -> Self {
        if timeout_ms > 0
            && let Ok(mut g) = deadline.lock()
        {
            *g = Some(Instant::now() + std::time::Duration::from_millis(timeout_ms));
        }
        TimeoutGuard {
            deadline: Arc::clone(deadline),
        }
    }
}

impl Drop for TimeoutGuard {
    fn drop(&mut self) {
        if let Ok(mut g) = self.deadline.lock() {
            *g = None;
        }
    }
}

/// QuickJS 脚本沙箱 — 封装一个 QuickJS Runtime 和 Context。
///
/// # 与 V8Sandbox 的差异
///
/// - QuickJS 是解释器，V8 是 JIT 编译器，性能差异显著
/// - QuickJS 体积小（约 700KB），适合嵌入式场景
/// - 两者提供相同的 [`execute()`]/[`execute_json()`]/[`register_callback()`] 接口
/// - 两者都经 `timeout_ms` 强制执行超时：V8 用 SEC-13 看门狗 `terminate_execution`，
///   QuickJS 用 `set_interrupt_handler`（R3400 对齐——旧实现 `set_timeout_ms` 是静默
///   no-op，`while(true){}` 永久挂死，无执行中断）。
///
/// # 线程安全
///
/// QuickJS Runtime 不是线程安全的。每个线程应创建独立的沙箱实例。
///
/// # 回调注册
///
/// `register_callback` 存储回调列表，每次 `execute` 新建 Context 时重挂到全局对象
/// （QuickJS 每次 execute 独立 Context，与 V8 持久化 Context 语义不同）。
pub struct QuickJSSandbox {
    runtime: rquickjs::Runtime,
    config: SandboxConfig,
    callbacks: Vec<(String, HostCallback)>,
    /// 持久上下文（`persistent_context: true` 时跨 execute 复用，2026-08-08——
    /// 对齐 V8 持久化语义；false 时每次 execute 新建 = 旧行为状态隔离）。
    context: Option<rquickjs::Context>,
    /// R3400：执行超时截止时刻。`Some(d)` 时，interrupt handler 在 `now >= d` 返 true
    /// 中断执行。execute 前 set（= now + timeout_ms），后 clear。与 runtime 级
    /// `interrupt_handler` 共享（handler 构造时 clone 一份）。
    timeout_deadline: Arc<Mutex<Option<Instant>>>,
    /// R3400：标记本次 execute 是否因超时被 interrupt handler 中断（execute 后据此
    /// 把 QuickJS 抛出的 uncatchable 异常映射为 `ScriptError::Timeout`，区别于普通
    /// RuntimeError）。每次 execute 前 reset。
    timeout_fired: Arc<AtomicBool>,
}

impl QuickJSSandbox {
    /// 创建新的 QuickJS 脚本沙箱。
    ///
    /// 使用默认配置。
    pub fn new() -> Result<Self, ScriptError> {
        Self::with_config(SandboxConfig::default())
    }

    /// 使用指定配置创建 QuickJS 脚本沙箱。
    pub fn with_config(config: SandboxConfig) -> Result<Self, ScriptError> {
        let runtime = rquickjs::Runtime::new()
            .map_err(|e| ScriptError::EngineUnavailable(format!("failed to create QuickJS runtime: {e}")))?;

        // 配置内存限制
        if config.heap_limit > 0 {
            runtime.set_memory_limit(config.heap_limit);
        }

        // R3400：注册执行超时 interrupt handler（镜像 V8 SEC-13 看门狗）。
        // QuickJS 解释器周期性调此 handler：若当前有 deadline 且 now >= deadline，
        // 返 true 抛 uncatchable 异常中断执行（`while(true){}` 等）。timeout_ms=0 时
        // deadline 恒为 None，handler 永不中断（无超时语义不变）。
        let timeout_deadline: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));
        let timeout_fired = Arc::new(AtomicBool::new(false));
        {
            let dl = Arc::clone(&timeout_deadline);
            let fired = Arc::clone(&timeout_fired);
            runtime.set_interrupt_handler(Some(Box::new(move || {
                let exceed = dl
                    .lock()
                    .ok()
                    .and_then(|g| *g)
                    .is_some_and(|deadline| Instant::now() >= deadline);
                if exceed {
                    fired.store(true, Ordering::Release);
                    true // 中断执行
                } else {
                    false
                }
            })));
        }

        Ok(Self {
            runtime,
            config,
            callbacks: Vec::new(),
            context: None,
            timeout_deadline,
            timeout_fired,
        })
    }

    /// 执行 JavaScript 代码并返回字符串结果。
    ///
    /// `persistent_context: true` 时复用持久 Context（跨 execute 保留全局状态——
    /// 对齐 V8；shim 注入 + 后续脚本引用的模式依赖此语义）；false 时每次新建
    /// Context（状态隔离，旧行为）。
    pub fn execute(&mut self, code: &str) -> Result<ScriptResult, ScriptError> {
        let trimmed = code.trim();
        if trimmed.is_empty() {
            return Err(ScriptError::InvalidInput("script is empty".into()));
        }

        let start = std::time::Instant::now();

        // 使用 eval + String() 确保任意代码都能执行并返回字符串
        let wrapped = format!("String(eval({quoted:?}))", quoted = trimmed);

        let result = self.run_eval(&wrapped, self.config.timeout_ms);

        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

        match result {
            Ok(value) => Ok(ScriptResult {
                value,
                execution_time_ms: elapsed_ms,
            }),
            Err(e) => Err(e),
        }
    }

    /// 在（持久或新建的）Context 中执行包装脚本（带 timeout 装载 + 超时映射）。
    ///
    /// R3400：`timeout_ms > 0` 时 arm 截止时刻 = now + timeout_ms；interrupt handler 到期
    /// 返 true 抛 uncatchable 异常 → eval 返 `Error::Exception`。`run_eval` 据超时标志把
    /// 该异常映射为 `ScriptError::Timeout`（区别普通 RuntimeError），与 V8 SEC-13 对齐。
    /// `timeout_ms = 0` 时不 arm（无超时，旧行为不变）。
    fn run_eval(&mut self, wrapped: &str, timeout_ms: u64) -> Result<String, ScriptError> {
        // 重置超时标志（上一轮 execute 若超时已置位）。
        self.timeout_fired.store(false, Ordering::Release);
        // arm 超时（guard Drop 时清 deadline，防 interrupt handler 误伤后续 execute）。
        let _guard = TimeoutGuard::arm(&self.timeout_deadline, timeout_ms);

        let result = self.eval_wrapped(wrapped);

        // 超时优先于 RuntimeError 报告（与 V8 execute 一致：terminate_execution 后
        // TryCatch 表现为异常，须先于 RuntimeError 判 Timeout）。
        if self.timeout_fired.load(Ordering::Acquire) {
            return Err(ScriptError::Timeout(format!("{}ms", self.config.timeout_ms)));
        }
        result
    }

    /// 在（持久或新建的）Context 中执行包装脚本并提取 JS 异常消息。
    fn eval_wrapped(&mut self, wrapped: &str) -> Result<String, ScriptError> {
        let result: Result<String, ScriptError> = (|| {
            if self.config.persistent_context {
                // 持久上下文：首次创建后复用（V8 persistent_context 对齐，
                // 2026-08-08：修复 QuickJS 下 shim 注入状态跨 execute 丢失）
                if self.context.is_none() {
                    self.context = Some(
                        rquickjs::Context::full(&self.runtime)
                            .map_err(|e| ScriptError::EngineUnavailable(format!("failed to create context: {e}")))?,
                    );
                }
                let persistent = self.context.as_ref().expect("context created above");
                persistent.with(|ctx| Self::eval_in_ctx(self, ctx, wrapped))
            } else {
                let ctx = rquickjs::Context::full(&self.runtime)
                    .map_err(|e| ScriptError::EngineUnavailable(format!("failed to create context: {e}")))?;
                ctx.with(|ctx| Self::eval_in_ctx(self, ctx, wrapped))
            }
        })();
        result
    }

    /// 在给定 Ctx 中执行脚本 + 回调安装 + 错误消息提取。
    fn eval_in_ctx(sandbox: &QuickJSSandbox, ctx: rquickjs::Ctx, wrapped: &str) -> Result<String, ScriptError> {
        sandbox.install_callbacks(&ctx);
        let result: rquickjs::String = ctx.eval(wrapped).map_err(|e| {
            // 提取 JS 异常具体消息：rquickjs Error::Exception 的 Display 只有
            // "Exception generated by QuickJS"（不含 message），无法定位问题——
            // 用 ctx.catch() 取真实 error.message（2026-08-08 修复）。
            let msg = match &e {
                rquickjs::Error::Exception => {
                    let caught = ctx.catch();
                    let exc_msg = if caught.is_object() {
                        caught
                            .as_object()
                            .and_then(|o| o.get::<_, Option<String>>("message").ok().flatten())
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };
                    if exc_msg.is_empty() {
                        format!("{e}")
                    } else {
                        format!("{e}: {exc_msg}")
                    }
                }
                _ => format!("{e}"),
            };
            if msg.contains("SyntaxError") || msg.contains("syntax error") {
                ScriptError::CompileError(msg)
            } else {
                ScriptError::RuntimeError(msg)
            }
        })?;

        // QuickJS microtask/job queue：Promise .then 等回调在 job queue，须手动 drain
        //（V8 eval 后自动执行；QuickJS 不自动——2026-08-08 修复 async 测试）。
        // 上限防无限 job 链（恶意/失控脚本）；正常 Promise 链远低于此。
        let mut jobs = 0;
        while jobs < 10_000 && ctx.execute_pending_job() {
            jobs += 1;
        }

        let value: String = result
            .get()
            .map_err(|e| ScriptError::RuntimeError(format!("failed to convert result: {e}")))?;

        Ok(value)
    }

    /// 执行 JavaScript 代码并返回 JSON 字符串结果。
    ///
    /// 使用 `JSON.stringify()` 将结果序列化为 JSON 字符串。
    /// 如果结果无法序列化，返回 `undefined`。
    pub fn execute_json(&mut self, code: &str) -> Result<ScriptResult, ScriptError> {
        let trimmed = code.trim();
        if trimmed.is_empty() {
            return Err(ScriptError::InvalidInput("script is empty".into()));
        }

        let start = std::time::Instant::now();

        // 使用 JSON.stringify 包装脚本 — 直接在顶层计算
        let wrapped = format!(
            "(function() {{ try {{ var __r = eval({quoted:?}); return JSON.stringify(__r); }} catch(e) {{ return String(e); }} }})()",
            quoted = trimmed
        );

        // R3400：经 run_eval 装载超时 + 映射 Timeout（与 execute 同路径）。
        let result = self.run_eval_json(&wrapped, self.config.timeout_ms);

        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

        match result {
            Ok(value) => Ok(ScriptResult {
                value,
                execution_time_ms: elapsed_ms,
            }),
            Err(e) => Err(e),
        }
    }

    /// `execute_json` 的超时装载 + JSON-wrap eval（镜像 [`Self::run_eval`]）。
    fn run_eval_json(&mut self, wrapped: &str, timeout_ms: u64) -> Result<String, ScriptError> {
        self.timeout_fired.store(false, Ordering::Release);
        let _guard = TimeoutGuard::arm(&self.timeout_deadline, timeout_ms);

        let result: Result<String, ScriptError> = (|| {
            let ctx = rquickjs::Context::full(&self.runtime)
                .map_err(|e| ScriptError::EngineUnavailable(format!("failed to create context: {e}")))?;

            ctx.with(|ctx| {
                self.install_callbacks(&ctx);
                let result: rquickjs::String = ctx.eval(wrapped).map_err(|e| {
                    // 提取 JS 异常具体消息：rquickjs Error::Exception 的 Display 只有
                    // "Exception generated by QuickJS"（不含 message），无法定位问题——
                    // 用 ctx.catch() 取真实 error.message（2026-08-08 修复）。
                    let msg = match &e {
                        rquickjs::Error::Exception => {
                            let caught = ctx.catch();
                            let exc_msg = if caught.is_object() {
                                caught
                                    .as_object()
                                    .and_then(|o| o.get::<_, Option<String>>("message").ok().flatten())
                                    .unwrap_or_default()
                            } else {
                                String::new()
                            };
                            if exc_msg.is_empty() {
                                format!("{e}")
                            } else {
                                format!("{e}: {exc_msg}")
                            }
                        }
                        _ => format!("{e}"),
                    };
                    if msg.contains("SyntaxError") || msg.contains("syntax error") {
                        ScriptError::CompileError(msg)
                    } else {
                        ScriptError::RuntimeError(msg)
                    }
                })?;

                let value: String = result
                    .get()
                    .map_err(|e| ScriptError::RuntimeError(format!("failed to convert result: {e}")))?;

                Ok(value)
            })
        })();

        if self.timeout_fired.load(Ordering::Acquire) {
            return Err(ScriptError::Timeout(format!("{}ms", self.config.timeout_ms)));
        }
        result
    }

    /// 返回 QuickJS 引擎版本号。
    pub fn quickjs_version() -> &'static str {
        "QuickJS (via rquickjs 0.7)"
    }

    /// 返回沙箱配置的引用。
    pub fn config(&self) -> &SandboxConfig {
        &self.config
    }

    /// 注册宿主回调，挂为全局函数 `name`（与 V8Sandbox::register_callback 语义一致）。
    ///
    /// 回调存入列表，每次 `execute` 新建 Context 时重挂到全局对象。
    #[allow(clippy::type_complexity)]
    pub fn register_callback(&mut self, name: &str, callback: Box<dyn Fn(&[String]) -> String + Send + Sync>) {
        let cb: HostCallback = Arc::from(callback);
        self.callbacks.push((name.to_string(), cb));
    }

    /// 设置脚本执行超时（毫秒），0 表示无超时。
    pub fn set_timeout_ms(&mut self, timeout_ms: u64) {
        self.config.timeout_ms = timeout_ms;
    }

    /// 重置持久上下文，保留宿主回调以便在下一个 document 中重新安装。
    pub fn reset_context(&mut self) {
        self.context = None;
    }

    /// 将已注册的回调重挂到当前 Context 的全局对象（每次 execute 前调用）。
    fn install_callbacks(&self, ctx: &rquickjs::Ctx) {
        let globals = ctx.globals();
        for (name, cb) in &self.callbacks {
            let cb = Arc::clone(cb);
            // 参数用 Coerced<String>（JS ToString 语义）——旧 Rest<String> 对数字参数
            // 报 "Error converting from js 'int' into type 'string'"（shim 调 __zw_setTimeout
            // 传 `delay|0` 数字），2026-08-08 修复（V8 侧自动 String 化无此问题）。
            let func = rquickjs::Function::new(
                ctx.clone(),
                move |args: rquickjs::function::Rest<rquickjs::Coerced<String>>| -> String {
                    let strs: Vec<String> = args.0.iter().map(|c| c.0.clone()).collect();
                    cb(&strs)
                },
            );
            if let Ok(f) = func {
                let _ = globals.set(name.as_str(), f);
            }
        }
    }
}

/// 将 Rust 字符串转为 JS 字符串字面量（含两端双引号），转义特殊字符防注入。
/// 供 [`QuickJSSandbox::resolve_async_callback`] 把 `id`/`result` 安全嵌入执行脚本。
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

impl crate::Sandbox for QuickJSSandbox {
    fn execute(&mut self, code: &str) -> Result<ScriptResult, ScriptError> {
        QuickJSSandbox::execute(self, code)
    }
    fn resolve_async_callback(&mut self, id: &str, result: &str) {
        // 2026-08-08 修复：旧实现为 trait 默认 no-op（QuickJS 降级）——resolver
        // resolve 后 Promise 永不 resolve，fetch/timer/observer 异步回调全失效。
        // 镜像 V8：执行 shim 的 __zwResolveCallback resolve pending Promise。
        let id_lit = js_string_literal(id);
        let result_lit = js_string_literal(result);
        let js = format!("if(globalThis.__zwResolveCallback){{__zwResolveCallback({id_lit},{result_lit});}}");
        let _ = self.execute(&js);
    }
    fn execute_json(&mut self, code: &str) -> Result<ScriptResult, ScriptError> {
        QuickJSSandbox::execute_json(self, code)
    }
    fn register_callback(&mut self, name: &str, callback: Box<dyn Fn(&[String]) -> String + Send + Sync>) {
        QuickJSSandbox::register_callback(self, name, callback)
    }
    fn set_timeout_ms(&mut self, timeout_ms: u64) {
        QuickJSSandbox::set_timeout_ms(self, timeout_ms)
    }
    fn reset_context(&mut self) {
        QuickJSSandbox::reset_context(self)
    }
    fn config(&self) -> &SandboxConfig {
        &self.config
    }
}

impl Default for QuickJSSandbox {
    fn default() -> Self {
        Self::new().expect("failed to create QuickJS sandbox")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_creation() {
        let sandbox = QuickJSSandbox::new().unwrap();
        assert_eq!(sandbox.config().heap_limit, 0);
        assert_eq!(sandbox.config().timeout_ms, 0);
    }

    #[test]
    fn test_sandbox_with_config() {
        let config = SandboxConfig {
            heap_limit: 1024 * 1024,
            timeout_ms: 5000,
            persistent_context: false,
            ..Default::default()
        };
        let sandbox = QuickJSSandbox::with_config(config).unwrap();
        assert_eq!(sandbox.config().heap_limit, 1024 * 1024);
    }

    #[test]
    fn test_sandbox_default() {
        let sandbox = QuickJSSandbox::default();
        assert_eq!(sandbox.config().heap_limit, 0);
    }

    #[test]
    fn test_reset_context_clears_persistent_page_state_but_keeps_callbacks() {
        let config = SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let mut sandbox = QuickJSSandbox::with_config(config).unwrap();
        sandbox.register_callback("host_value", Box::new(|_| "ok".to_string()));
        sandbox.execute("globalThis.old_page = 42; host_value()").unwrap();

        sandbox.reset_context();

        assert_eq!(
            sandbox.execute("typeof globalThis.old_page").unwrap().value,
            "undefined"
        );
        assert_eq!(sandbox.execute("host_value()").unwrap().value, "ok");
    }

    #[test]
    fn test_execute_simple() {
        let mut sandbox = QuickJSSandbox::new().unwrap();
        let result = sandbox.execute("1 + 1").unwrap();
        assert_eq!(result.value, "2");
        assert!(result.execution_time_ms >= 0.0);
    }

    #[test]
    fn test_execute_string() {
        let mut sandbox = QuickJSSandbox::new().unwrap();
        let result = sandbox.execute("'hello' + ' ' + 'world'").unwrap();
        assert_eq!(result.value, "hello world");
    }

    #[test]
    fn test_execute_empty_script() {
        let mut sandbox = QuickJSSandbox::new().unwrap();
        let result = sandbox.execute("");
        assert!(result.is_err());
        if let Err(ScriptError::InvalidInput(_)) = result {
            // 预期
        } else {
            panic!("expected InvalidInput, got: {:?}", result);
        }
    }

    #[test]
    fn test_execute_whitespace_only() {
        let mut sandbox = QuickJSSandbox::new().unwrap();
        let result = sandbox.execute("   \n\t  ");
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_error() {
        let mut sandbox = QuickJSSandbox::new().unwrap();
        let result = sandbox.execute("function(");
        assert!(result.is_err());
        // QuickJS 可能返回 CompileError 或 RuntimeError，取决于版本
        match result {
            Err(ScriptError::CompileError(_)) | Err(ScriptError::RuntimeError(_)) => {
                // 两种都可接受
            }
            _ => panic!("expected compile or runtime error, got: {:?}", result),
        }
    }

    #[test]
    fn test_runtime_error() {
        let mut sandbox = QuickJSSandbox::new().unwrap();
        let result = sandbox.execute("undefined_function()");
        assert!(result.is_err());
        if let Err(ScriptError::RuntimeError(_)) = result {
            // 预期
        } else {
            panic!("expected RuntimeError, got: {:?}", result);
        }
    }

    #[test]
    fn test_state_isolation() {
        let mut sandbox = QuickJSSandbox::new().unwrap();

        let r1 = sandbox.execute("var x = 42; x").unwrap();
        assert_eq!(r1.value, "42");

        // 变量 x 不应在第二次执行中存在（新 Context）
        let r2 = sandbox.execute("typeof x").unwrap();
        assert_eq!(r2.value, "undefined");
    }

    #[test]
    fn test_execute_json_object() {
        let mut sandbox = QuickJSSandbox::new().unwrap();
        let result = sandbox.execute_json("({a: 1, b: 'hello'})").unwrap();
        assert!(result.value.contains("\"a\""));
        assert!(result.value.contains("1"));
    }

    #[test]
    fn test_execute_json_array() {
        let mut sandbox = QuickJSSandbox::new().unwrap();
        let result = sandbox.execute_json("[1, 2, 3]").unwrap();
        assert_eq!(result.value, "[1,2,3]");
    }

    #[test]
    fn test_execute_json_null() {
        let mut sandbox = QuickJSSandbox::new().unwrap();
        let result = sandbox.execute_json("null").unwrap();
        assert_eq!(result.value, "null");
    }

    #[test]
    fn test_execute_json_string() {
        let mut sandbox = QuickJSSandbox::new().unwrap();
        let result = sandbox.execute_json("'test'").unwrap();
        assert_eq!(result.value, "\"test\"");
    }

    #[test]
    fn test_es6_arrow_functions() {
        let mut sandbox = QuickJSSandbox::new().unwrap();
        let result = sandbox.execute("[1,2,3].map(x => x * 2).join(',')").unwrap();
        assert_eq!(result.value, "2,4,6");
    }

    #[test]
    fn test_es6_template_strings() {
        let mut sandbox = QuickJSSandbox::new().unwrap();
        let result = sandbox.execute("`hello ${'world'}`").unwrap();
        assert_eq!(result.value, "hello world");
    }

    #[test]
    fn test_es6_destructuring() {
        let mut sandbox = QuickJSSandbox::new().unwrap();
        let result = sandbox.execute("var [a, b] = [1, 2]; a + b").unwrap();
        assert_eq!(result.value, "3");
    }

    #[test]
    fn test_es6_spread() {
        let mut sandbox = QuickJSSandbox::new().unwrap();
        let result = sandbox.execute("var a = [1,2]; var b = [...a, 3]; b.length").unwrap();
        assert_eq!(result.value, "3");
    }

    #[test]
    fn test_json_builtin() {
        let mut sandbox = QuickJSSandbox::new().unwrap();
        let result = sandbox.execute("JSON.stringify({key: 'value'})").unwrap();
        assert!(result.value.contains("key"));
    }

    #[test]
    fn test_math_builtin() {
        let mut sandbox = QuickJSSandbox::new().unwrap();
        let result = sandbox.execute("Math.floor(3.7)").unwrap();
        assert_eq!(result.value, "3");
    }

    #[test]
    fn test_array_methods() {
        let mut sandbox = QuickJSSandbox::new().unwrap();
        let result = sandbox.execute("[5,3,8,1].sort().join('-')").unwrap();
        assert_eq!(result.value, "1-3-5-8");
    }

    #[test]
    fn test_class_syntax() {
        let mut sandbox = QuickJSSandbox::new().unwrap();
        let result = sandbox
            .execute("class Foo { bar() { return 99; } }; new Foo().bar()")
            .unwrap();
        assert_eq!(result.value, "99");
    }

    #[test]
    fn test_map_set() {
        let mut sandbox = QuickJSSandbox::new().unwrap();
        let result = sandbox.execute("var m = new Map(); m.set('a', 1); m.get('a')").unwrap();
        assert_eq!(result.value, "1");
    }

    #[test]
    fn test_symbol() {
        let mut sandbox = QuickJSSandbox::new().unwrap();
        let result = sandbox.execute("typeof Symbol('test')").unwrap();
        assert_eq!(result.value, "symbol");
    }

    #[test]
    fn test_for_of() {
        let mut sandbox = QuickJSSandbox::new().unwrap();
        let result = sandbox.execute("var s = 0; for (var x of [1,2,3]) s += x; s").unwrap();
        assert_eq!(result.value, "6");
    }

    #[test]
    fn test_quickjs_version() {
        let version = QuickJSSandbox::quickjs_version();
        assert!(version.contains("QuickJS"));
    }

    #[test]
    fn test_execution_time_measured() {
        let mut sandbox = QuickJSSandbox::new().unwrap();
        let result = sandbox
            .execute("var s = 0; for (var i = 0; i < 10000; i++) s += i; s")
            .unwrap();
        assert!(result.execution_time_ms >= 0.0);
        assert!(
            result.execution_time_ms < 5000.0,
            "execution took too long: {}ms",
            result.execution_time_ms
        );
    }

    #[test]
    fn test_large_string_concatenation() {
        let mut sandbox = QuickJSSandbox::new().unwrap();
        let result = sandbox
            .execute("var s = ''; for (var i = 0; i < 1000; i++) s += 'x'; s.length")
            .unwrap();
        assert_eq!(result.value, "1000");
    }

    #[test]
    fn test_nested_objects() {
        let mut sandbox = QuickJSSandbox::new().unwrap();
        let result = sandbox.execute_json("({a: {b: {c: 42}}})").unwrap();
        assert!(result.value.contains("42"));
    }

    #[test]
    fn test_default_parameters() {
        let mut sandbox = QuickJSSandbox::new().unwrap();
        let result = sandbox.execute("function f(x = 10) { return x; }; f()").unwrap();
        assert_eq!(result.value, "10");
    }

    #[test]
    fn test_rest_parameters() {
        let mut sandbox = QuickJSSandbox::new().unwrap();
        let result = sandbox
            .execute("function f(...args) { return args.length; }; f(1,2,3)")
            .unwrap();
        assert_eq!(result.value, "3");
    }

    #[test]
    fn test_static_methods() {
        let mut sandbox = QuickJSSandbox::new().unwrap();
        let result = sandbox.execute("Object.keys({a: 1, b: 2}).length").unwrap();
        assert_eq!(result.value, "2");
    }

    #[test]
    fn test_multiple_sandboxes_isolated() {
        let mut s1 = QuickJSSandbox::new().unwrap();
        let mut s2 = QuickJSSandbox::new().unwrap();

        let r1 = s1.execute("42").unwrap();
        let r2 = s2.execute("'hello'").unwrap();

        assert_eq!(r1.value, "42");
        assert_eq!(r2.value, "hello");
    }

    #[test]
    fn test_register_callback_invokable() {
        // 2026-08-08 调试：回调安装后能否在 JS 中调用（shim setTimeout 的
        // __zw_setTimeout 走 host 回调，调用失败会被 shim try-catch 吞掉回退同步）。
        let mut sandbox = QuickJSSandbox::with_config(SandboxConfig {
            persistent_context: true,
            ..Default::default()
        })
        .unwrap();
        sandbox.register_callback("__zw_probe_cb", Box::new(|args| format!("cb:{}", args.join(","))));
        // 1. 全局函数存在？
        let t = sandbox.execute("typeof __zw_probe_cb").unwrap();
        assert_eq!(t.value, "function", "callback should be installed on globals");
        // 2. 调用成功？
        let r = sandbox.execute("__zw_probe_cb('a','b')").unwrap();
        assert_eq!(r.value, "cb:a,b", "callback should be invokable");
        // 3. 持久上下文下跨 execute 仍可调用
        let r2 = sandbox.execute("__zw_probe_cb('x')").unwrap();
        assert_eq!(r2.value, "cb:x");
    }

    // ── R3400：QuickJS set_timeout_ms 须被 execute/execute_json 强制执行（SEC-13 对称）──
    // 修复前：set_timeout_ms 是静默 no-op，`while(true){}` 永久挂死（无 interrupt handler）。
    // 修复后：interrupt handler 到期抛 uncatchable 异常 → ScriptError::Timeout。

    #[test]
    fn test_timeout_interrupts_dead_loop_r3400() {
        let mut sandbox = QuickJSSandbox::with_config(SandboxConfig {
            timeout_ms: 500,
            ..Default::default()
        })
        .unwrap();
        let start = std::time::Instant::now();
        let result = sandbox.execute("while(true){}");
        let elapsed = start.elapsed();
        assert!(
            matches!(result, Err(crate::ScriptError::Timeout(_))),
            "死循环应被超时中断为 Timeout，实际: {result:?}"
        );
        // 应在 ~timeout_ms 附近返回，远低于 5s（修复前永久挂死）。
        assert!(
            elapsed < std::time::Duration::from_secs(3),
            "超时应及时生效（耗时 {elapsed:?}），R3400 回归"
        );
    }

    #[test]
    fn test_timeout_json_interrupts_dead_loop_r3400() {
        let mut sandbox = QuickJSSandbox::with_config(SandboxConfig {
            timeout_ms: 500,
            ..Default::default()
        })
        .unwrap();
        let start = std::time::Instant::now();
        let result = sandbox.execute_json("while(true){}");
        let elapsed = start.elapsed();
        assert!(
            matches!(result, Err(crate::ScriptError::Timeout(_))),
            "execute_json 死循环应被超时中断为 Timeout，实际: {result:?}"
        );
        assert!(
            elapsed < std::time::Duration::from_secs(3),
            "execute_json 超时应及时生效（耗时 {elapsed:?}），R3400 回归"
        );
    }

    #[test]
    fn test_set_timeout_ms_dynamic_then_interrupt_r3400() {
        // 默认无超时（timeout_ms=0）→ 正常快速返回；set_timeout_ms 后 → 超时中断。
        let mut sandbox = QuickJSSandbox::new().unwrap();
        sandbox.set_timeout_ms(400);
        let start = std::time::Instant::now();
        let result = sandbox.execute("while(true){}");
        assert!(
            matches!(result, Err(crate::ScriptError::Timeout(_))),
            "set_timeout_ms 后死循环应被中断，实际: {result:?}"
        );
        assert!(start.elapsed() < std::time::Duration::from_secs(3));
    }

    #[test]
    fn test_no_timeout_zero_ms_does_not_interrupt_r3400() {
        // timeout_ms=0 = 无超时，正常快速脚本不受影响（回归保护：handler 不误伤）。
        let mut sandbox = QuickJSSandbox::with_config(SandboxConfig {
            timeout_ms: 0,
            ..Default::default()
        })
        .unwrap();
        let r = sandbox.execute("var s=0; for(var i=0;i<1000;i++) s+=i; s").unwrap();
        assert_eq!(r.value, "499500");
    }

    #[test]
    fn test_timeout_recovers_for_next_execute_r3400() {
        // 超时中断后 deadline 被 guard Drop 清除——下一次正常脚本可执行
        //（防 interrupt handler 残留 deadline 误伤后续 execute）。
        let mut sandbox = QuickJSSandbox::with_config(SandboxConfig {
            timeout_ms: 300,
            ..Default::default()
        })
        .unwrap();
        // 触发超时。
        let _ = sandbox.execute("while(true){}");
        // 清除超时，正常脚本应成功执行。
        sandbox.set_timeout_ms(0);
        let r = sandbox.execute("1 + 1").unwrap();
        assert_eq!(r.value, "2", "超时后 deadline 须清除，后续 execute 不受误伤");
    }
}
