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
    Some((config.initial_heap_size, max))
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
    /// 设置脚本执行超时（毫秒），0 表示无超时。
    fn set_timeout_ms(&mut self, timeout_ms: u64);
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
}

#[cfg(not(any(feature = "v8", feature = "quickjs")))]
compile_error!("至少需要启用一个JS引擎feature: `v8` 或 `quickjs`");
