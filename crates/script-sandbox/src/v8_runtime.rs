//! V8引擎运行时实现。
//!
//! 封装rusty_v8，提供安全的JavaScript脚本执行沙箱。

use std::sync::Once;

use crate::{SandboxConfig, ScriptError, ScriptResult};

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
/// # 线程安全
///
/// V8 Isolate不是线程安全的。每个线程应创建独立的沙箱实例。
pub struct V8Sandbox {
    /// V8 Isolate（拥有所有权）。
    isolate: Option<rusty_v8::OwnedIsolate>,
    /// 沙箱配置。
    config: SandboxConfig,
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
        })
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

        let isolate = self
            .isolate
            .as_mut()
            .ok_or(ScriptError::NotInitialized)?;

        let start = std::time::Instant::now();

        let scope = &mut rusty_v8::HandleScope::new(isolate);
        let context = rusty_v8::Context::new(scope);
        let scope = &mut rusty_v8::ContextScope::new(scope, context);

        // 编译脚本
        let code_str = rusty_v8::String::new(scope, code)
            .ok_or_else(|| ScriptError::InvalidInput("failed to create V8 string".into()))?;

        let script = rusty_v8::Script::compile(scope, code_str, None).ok_or_else(|| {
            let msg = Self::extract_exception(scope);
            ScriptError::CompileError(msg)
        })?;

        // 执行脚本
        let result = script.run(scope).ok_or_else(|| {
            let msg = Self::extract_exception(scope);
            ScriptError::RuntimeError(msg)
        })?;

        // 转换结果为字符串
        let result_str = result
            .to_string(scope)
            .map(|s| s.to_rust_string_lossy(scope))
            .unwrap_or_default();

        let execution_time_ms = start.elapsed().as_secs_f64() * 1000.0;

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

        let isolate = self
            .isolate
            .as_mut()
            .ok_or(ScriptError::NotInitialized)?;

        let start = std::time::Instant::now();

        let scope = &mut rusty_v8::HandleScope::new(isolate);
        let context = rusty_v8::Context::new(scope);
        let scope = &mut rusty_v8::ContextScope::new(scope, context);

        let code_str = rusty_v8::String::new(scope, code)
            .ok_or_else(|| ScriptError::InvalidInput("failed to create V8 string".into()))?;

        let script = rusty_v8::Script::compile(scope, code_str, None).ok_or_else(|| {
            let msg = Self::extract_exception(scope);
            ScriptError::CompileError(msg)
        })?;

        let result = script.run(scope).ok_or_else(|| {
            let msg = Self::extract_exception(scope);
            ScriptError::RuntimeError(msg)
        })?;

        // 尝试JSON.stringify
        let json_str = Self::value_to_json_string(scope, result);

        let execution_time_ms = start.elapsed().as_secs_f64() * 1000.0;

        Ok(ScriptResult {
            value: json_str,
            execution_time_ms,
        })
    }

    /// 获取V8引擎版本号。
    pub fn v8_version() -> &'static str {
        ensure_v8_initialized();
        rusty_v8::V8::get_version()
    }

    /// 从当前scope中提取异常消息。
    fn extract_exception(scope: &mut rusty_v8::HandleScope) -> String {
        let try_catch = &mut rusty_v8::TryCatch::new(scope);
        if let Some(exception) = try_catch.exception() {
            // TryCatch 实现了 AsMut<HandleScope>，可以直接用 scope
            if let Some(msg) = exception.to_string(try_catch) {
                return msg.to_rust_string_lossy(try_catch);
            }
        }
        "unknown error".to_string()
    }

    /// 将V8值转换为JSON字符串。
    fn value_to_json_string(
        scope: &mut rusty_v8::HandleScope,
        value: rusty_v8::Local<rusty_v8::Value>,
    ) -> String {
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

        let Ok(stringify_fn) =
            rusty_v8::Local::<rusty_v8::Function>::try_from(stringify_val)
        else {
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
        };
        let sandbox = V8Sandbox::with_config(config);
        assert!(sandbox.is_ok());
    }

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert_eq!(config.heap_limit, 0);
        assert_eq!(config.timeout_ms, 0);
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

    #[test]
    fn test_execute_function_call() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox
            .execute("(function() { return 'success'; })()")
            .unwrap();
        assert_eq!(result.value, "success");
    }

    #[test]
    fn test_execute_object_creation() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox
            .execute("({ name: 'test', value: 123 })")
            .unwrap();
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
        let result = sandbox
            .execute("JSON.parse('{\"a\":1}').a")
            .unwrap();
        assert_eq!(result.value, "1");
    }

    #[test]
    fn test_execute_json_stringify() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox
            .execute("JSON.stringify({x: 1})")
            .unwrap();
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
        let result = sandbox
            .execute("const {a, b} = {a: 1, b: 2}; a + b")
            .unwrap();
        assert_eq!(result.value, "3");
    }

    #[test]
    fn test_execute_spread_operator() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox
            .execute("const arr = [1, 2, 3]; [...arr, 4].length")
            .unwrap();
        assert_eq!(result.value, "4");
    }

    #[test]
    fn test_execute_promise() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox
            .execute("Promise.resolve(42)")
            .unwrap();
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
        let result = sandbox
            .execute("let a = 10; const b = 20; a + b")
            .unwrap();
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
        let result = sandbox
            .execute("[1, 2, 3].map(x => x * 2).join(',')")
            .unwrap();
        assert_eq!(result.value, "2,4,6");
    }

    #[test]
    fn test_execute_string_methods() {
        let mut sandbox = V8Sandbox::new().unwrap();
        let result = sandbox
            .execute("'hello world'.split(' ').join('-')")
            .unwrap();
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
        };
        let debug = format!("{config:?}");
        assert!(debug.contains("2048"));
        assert!(debug.contains("200"));
    }
}
