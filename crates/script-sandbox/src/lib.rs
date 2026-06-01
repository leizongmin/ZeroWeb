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
pub use v8_runtime::*;

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
}

#[cfg(not(any(feature = "v8", feature = "quickjs")))]
compile_error!("至少需要启用一个JS引擎feature: `v8` 或 `quickjs`");

// ── 无feature gate时的占位实现（仅编译时检查） ──

#[cfg(all(
    test,
    not(any(feature = "v8", feature = "quickjs"))
))]
mod tests {
    #[test]
    fn placeholder() {
        // 无feature gate时编译失败，不会到达此测试
    }
}
