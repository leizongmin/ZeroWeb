//! 公共类型定义
//!
//! WASM 沙箱的所有公共类型：WasmError、WasmValue、HostFunction、SandboxConfig、LinkerConfig。

use std::fmt;
use std::sync::Arc;

/// WASM 运行时错误
#[derive(Debug, thiserror::Error)]
pub enum WasmError {
    /// 无效的 WASM 二进制
    #[error("invalid WASM binary: {0}")]
    InvalidBinary(String),
    /// 导出未找到
    #[error("export not found: {name}")]
    ExportNotFound {
        /// 导出名称
        name: String,
    },
    /// 函数调用错误
    #[error("call error: {0}")]
    CallError(String),
    /// 内存访问错误
    #[error("memory error: {0}")]
    MemoryError(String),
    /// 实例化错误
    #[error("instantiation error: {0}")]
    InstantiationError(String),
    /// 链接错误
    #[error("link error: {0}")]
    LinkError(String),
    /// 燃料耗尽
    #[error("all fuel consumed")]
    FuelExhausted,
}

/// WASM 值类型
#[derive(Debug, Clone, PartialEq)]
pub enum WasmValue {
    /// 32 位整数
    I32(i32),
    /// 64 位整数
    I64(i64),
    /// 32 位浮点数
    F32(f32),
    /// 64 位浮点数
    F64(f64),
}

impl fmt::Display for WasmValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WasmValue::I32(v) => write!(f, "i32({v})"),
            WasmValue::I64(v) => write!(f, "i64({v})"),
            WasmValue::F32(v) => write!(f, "f32({v})"),
            WasmValue::F64(v) => write!(f, "f64({v})"),
        }
    }
}

/// 主机函数签名
///
/// WASM 调用主机函数时传入的参数和返回值的缓冲区。
/// 主机函数从 `params` 读取参数，将结果写入 `results`。
pub type HostFn = dyn Fn(&[WasmValue], &mut Vec<WasmValue>) -> Result<(), WasmError> + Send + Sync;

/// 主机函数定义
///
/// 用于在 WASM 实例化时注册可供 WASM 模块导入的主机函数。
#[derive(Clone)]
pub struct HostFunction {
    /// WASM 导入模块名
    pub module: String,
    /// WASM 导入函数名
    pub name: String,
    /// 参数类型列表
    pub params: Vec<WasmValueType>,
    /// 返回值类型列表
    pub results: Vec<WasmValueType>,
    /// 主机函数实现
    pub func: Arc<HostFn>,
}

impl HostFunction {
    /// 创建新的主机函数定义
    ///
    /// # 参数
    /// - `module`: WASM 导入模块名（如 `"env"`）
    /// - `name`: WASM 导入函数名（如 `"log"`）
    /// - `params`: 参数类型列表
    /// - `results`: 返回值类型列表
    /// - `func`: 主机函数实现
    pub fn new(
        module: impl Into<String>,
        name: impl Into<String>,
        params: Vec<WasmValueType>,
        results: Vec<WasmValueType>,
        func: impl Fn(&[WasmValue], &mut Vec<WasmValue>) -> Result<(), WasmError> + Send + Sync + 'static,
    ) -> Self {
        Self {
            module: module.into(),
            name: name.into(),
            params,
            results,
            func: Arc::new(func),
        }
    }
}

impl fmt::Debug for HostFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HostFunction")
            .field("module", &self.module)
            .field("name", &self.name)
            .field("params", &self.params)
            .field("results", &self.results)
            .finish()
    }
}

/// WASM 值类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmValueType {
    /// 32 位整数
    I32,
    /// 64 位整数
    I64,
    /// 32 位浮点数
    F32,
    /// 64 位浮点数
    F64,
}

/// 沙箱配置
///
/// 用于创建 `WasmSandbox` 时指定可选功能。
#[derive(Debug, Default, Clone)]
pub struct SandboxConfig {
    /// 是否启用燃料计量（限制 WASM 执行指令数）
    pub consume_fuel: bool,
}

impl SandboxConfig {
    /// 创建默认配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置是否启用燃料计量
    ///
    /// 启用后可通过 [`WasmInstance::set_fuel`] 限制 WASM 执行指令数量。
    pub fn consume_fuel(mut self, enable: bool) -> Self {
        self.consume_fuel = enable;
        self
    }

    /// 返回是否启用了燃料计量
    pub fn is_consume_fuel(&self) -> bool {
        self.consume_fuel
    }
}

/// 主机函数链接器
///
/// 收集所有主机函数定义，供模块实例化时使用。
#[derive(Debug, Default, Clone)]
pub struct LinkerConfig {
    functions: Vec<HostFunction>,
}

impl LinkerConfig {
    /// 创建空的链接器配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册主机函数
    pub fn define(&mut self, func: HostFunction) {
        self.functions.push(func);
    }

    /// 返回已注册的主机函数列表
    pub fn functions(&self) -> &[HostFunction] {
        &self.functions
    }
}
