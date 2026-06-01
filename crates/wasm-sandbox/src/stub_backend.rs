//! 占位后端实现
//!
//! 当未启用 wasmi feature 时提供的空壳实现，所有操作返回错误。

use crate::{LinkerConfig, SandboxConfig, WasmError, WasmValue};

/// WASM 沙箱运行时（占位实现）
pub struct WasmSandbox {
    config: SandboxConfig,
}

impl WasmSandbox {
    /// 创建新的 WASM 沙箱
    pub fn new() -> Self {
        Self::with_config(SandboxConfig::default())
    }

    /// 使用指定配置创建 WASM 沙箱
    pub fn with_config(config: SandboxConfig) -> Self {
        Self { config }
    }

    /// 编译 WASM 模块
    pub fn compile(&self, _bytes: &[u8]) -> Result<WasmModule, WasmError> {
        Err(WasmError::InvalidBinary(
            "no WASM backend enabled (enable 'wasmi' feature)".into(),
        ))
    }

    /// 返回沙箱配置的引用
    pub fn config(&self) -> &SandboxConfig {
        &self.config
    }
}

impl Default for WasmSandbox {
    fn default() -> Self {
        Self::new()
    }
}

/// 编译后的 WASM 模块（占位）
pub struct WasmModule;

impl WasmModule {
    /// 实例化模块
    pub fn instantiate(&self, _sandbox: &WasmSandbox) -> Result<WasmInstance, WasmError> {
        Err(WasmError::InstantiationError("no backend".into()))
    }

    /// 使用主机函数链接器实例化模块
    pub fn instantiate_with_linker(
        &self,
        _sandbox: &WasmSandbox,
        _linker_config: &LinkerConfig,
    ) -> Result<WasmInstance, WasmError> {
        Err(WasmError::InstantiationError("no backend".into()))
    }

    /// 获取导出名称列表
    pub fn exports(&self) -> Vec<String> {
        vec![]
    }
}

/// WASM 实例（占位）
pub struct WasmInstance;

impl WasmInstance {
    /// 调用导出函数
    pub fn call(&mut self, _name: &str, _args: &[WasmValue]) -> Result<Vec<WasmValue>, WasmError> {
        Err(WasmError::CallError("no backend".into()))
    }

    /// 读取线性内存
    pub fn read_memory(&self, _name: &str, _offset: usize, _len: usize) -> Option<Vec<u8>> {
        None
    }

    /// 写入线性内存
    pub fn write_memory(&mut self, _name: &str, _offset: usize, _data: &[u8]) -> Result<(), WasmError> {
        Err(WasmError::MemoryError("no backend".into()))
    }

    /// 检查导出函数是否存在
    pub fn has_func(&self, _name: &str) -> bool {
        false
    }

    /// 检查导出内存是否存在
    pub fn has_memory(&self, _name: &str) -> bool {
        false
    }

    /// 获取内存大小
    pub fn memory_size(&self, _name: &str) -> Option<usize> {
        None
    }

    /// 读取全局导出变量的值（占位）
    pub fn get_global_export(&self, _name: &str) -> Option<WasmValue> {
        None
    }

    /// 检查导出表是否存在（占位）
    pub fn has_table(&self, _name: &str) -> bool {
        false
    }

    /// 设置剩余燃料
    pub fn set_fuel(&mut self, _fuel: u64) -> Result<(), WasmError> {
        Err(WasmError::CallError("no backend".into()))
    }

    /// 获取剩余燃料
    pub fn get_fuel(&self) -> Result<u64, WasmError> {
        Err(WasmError::CallError("no backend".into()))
    }
}
