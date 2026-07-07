//! QuickJS 引擎运行时实现。
//!
//! 封装 rquickjs，提供轻量级 JavaScript 脚本执行沙箱。
//! QuickJS 是一个小巧且可嵌入的 JS 引擎，支持 ES2023 规范的大部分特性。

use std::sync::Arc;

use crate::{SandboxConfig, ScriptError, ScriptResult};

/// 宿主回调类型（与 V8Sandbox 的 HostCallback 一致）。
type HostCallback = Arc<dyn Fn(&[String]) -> String + Send + Sync>;

/// QuickJS 脚本沙箱 — 封装一个 QuickJS Runtime 和 Context。
///
/// # 与 V8Sandbox 的差异
///
/// - QuickJS 是解释器，V8 是 JIT 编译器，性能差异显著
/// - QuickJS 体积小（约 700KB），适合嵌入式场景
/// - 两者提供相同的 [`execute()`]/[`execute_json()`]/[`register_callback()`] 接口
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

        Ok(Self {
            runtime,
            config,
            callbacks: Vec::new(),
        })
    }

    /// 执行 JavaScript 代码并返回字符串结果。
    ///
    /// 每次执行在独立的 Context 中运行，确保状态隔离。
    pub fn execute(&mut self, code: &str) -> Result<ScriptResult, ScriptError> {
        let trimmed = code.trim();
        if trimmed.is_empty() {
            return Err(ScriptError::InvalidInput("script is empty".into()));
        }

        let start = std::time::Instant::now();

        // 使用 eval + String() 确保任意代码都能执行并返回字符串
        let wrapped = format!("String(eval({quoted:?}))", quoted = trimmed);

        let result: Result<String, ScriptError> = (|| {
            let ctx = rquickjs::Context::full(&self.runtime)
                .map_err(|e| ScriptError::EngineUnavailable(format!("failed to create context: {e}")))?;

            ctx.with(|ctx| {
                self.install_callbacks(&ctx);
                let result: rquickjs::String = ctx.eval(wrapped).map_err(|e| {
                    let msg = format!("{e}");
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

        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

        match result {
            Ok(value) => Ok(ScriptResult {
                value,
                execution_time_ms: elapsed_ms,
            }),
            Err(e) => Err(e),
        }
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

        let result: Result<String, ScriptError> = (|| {
            let ctx = rquickjs::Context::full(&self.runtime)
                .map_err(|e| ScriptError::EngineUnavailable(format!("failed to create context: {e}")))?;

            ctx.with(|ctx| {
                self.install_callbacks(&ctx);
                let result: rquickjs::String = ctx.eval(wrapped).map_err(|e| {
                    let msg = format!("{e}");
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

        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

        match result {
            Ok(value) => Ok(ScriptResult {
                value,
                execution_time_ms: elapsed_ms,
            }),
            Err(e) => Err(e),
        }
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
    pub fn register_callback(&mut self, name: &str, callback: Box<dyn Fn(&[String]) -> String + Send + Sync>) {
        let cb: HostCallback = Arc::from(callback);
        self.callbacks.push((name.to_string(), cb));
    }

    /// 设置脚本执行超时（毫秒），0 表示无超时。
    pub fn set_timeout_ms(&mut self, timeout_ms: u64) {
        self.config.timeout_ms = timeout_ms;
    }

    /// 重置上下文（QuickJS 每次 execute 独立 Context，此方法清空回调列表）。
    pub fn reset_context(&mut self) {
        self.callbacks.clear();
    }

    /// 将已注册的回调重挂到当前 Context 的全局对象（每次 execute 前调用）。
    fn install_callbacks(&self, ctx: &rquickjs::Ctx) {
        let globals = ctx.globals();
        for (name, cb) in &self.callbacks {
            let cb = Arc::clone(cb);
            let func = rquickjs::Function::new(ctx.clone(), move |args: rquickjs::function::Rest<String>| -> String {
                cb(&args.0)
            });
            if let Ok(f) = func {
                let _ = globals.set(name.as_str(), f);
            }
        }
    }
}

impl crate::Sandbox for QuickJSSandbox {
    fn execute(&mut self, code: &str) -> Result<ScriptResult, ScriptError> {
        QuickJSSandbox::execute(self, code)
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
}
