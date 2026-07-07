//! # zero-script-sandbox
//!
//! 扩展/用户脚本引擎（V8/QuickJS feature gate）。
//!
//! 提供JavaScript脚本执行沙箱，用于扩展脚本、用户脚本和自动化脚本。
//! 通过feature gate选择后端引擎：
//! - `v8`（默认推荐）— 使用rusty_v8绑定V8引擎
//! - `quickjs` — 使用rquickjs绑定QuickJS引擎

#![warn(missing_docs)]

#[cfg(feature = "v8")]
mod v8_runtime;

#[cfg(feature = "v8")]
mod worker;

#[cfg(feature = "quickjs")]
mod quickjs_worker;

#[cfg(any(feature = "v8", feature = "quickjs"))]
mod es_module;

#[cfg(feature = "v8")]
pub use v8_runtime::*;

#[cfg(feature = "v8")]
pub use worker::*;

#[cfg(feature = "quickjs")]
pub use quickjs_worker::*;

#[cfg(any(feature = "v8", feature = "quickjs"))]
pub use es_module::*;

#[cfg(feature = "quickjs")]
mod quickjs_runtime;

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

/// 沙箱配置。
#[derive(Debug, Clone, Default)]
pub struct SandboxConfig {
    /// 堆内存上限（字节），0表示无限制。
    pub heap_limit: usize,
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
    fn register_callback(&mut self, name: &str, callback: Box<dyn Fn(&[String]) -> String + Send + Sync>);
    /// 设置脚本执行超时（毫秒），0 表示无超时。
    fn set_timeout_ms(&mut self, timeout_ms: u64);
    /// 重置上下文（清空 JS 状态）。
    fn reset_context(&mut self);
    /// 返回沙箱配置的引用。
    fn config(&self) -> &SandboxConfig;
}

#[cfg(not(any(feature = "v8", feature = "quickjs")))]
compile_error!("至少需要启用一个JS引擎feature: `v8` 或 `quickjs`");
