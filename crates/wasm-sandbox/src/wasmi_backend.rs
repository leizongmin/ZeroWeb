//! wasmi 后端实现
//!
//! 基于 wasmi 纯 Rust WASM 解释器的沙箱运行时实现。

use crate::{LinkerConfig, SandboxConfig, WasmError, WasmValue, WasmValueType};
use std::fmt;
use std::sync::Arc;

/// Wrapper for host function error messages that implements wasmi's HostError.
#[derive(Debug)]
struct HostStringError(String);

impl fmt::Display for HostStringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for HostStringError {}

impl wasmi::core::HostError for HostStringError {}

/// WASM 沙箱运行时
pub struct WasmSandbox {
    engine: wasmi::Engine,
    config: SandboxConfig,
}

impl WasmSandbox {
    /// 创建新的 WASM 沙箱（默认配置）
    pub fn new() -> Self {
        Self::with_config(SandboxConfig::default())
    }

    /// 使用指定配置创建 WASM 沙箱
    pub fn with_config(config: SandboxConfig) -> Self {
        let mut wasmi_config = wasmi::Config::default();
        wasmi_config.consume_fuel(config.is_consume_fuel());
        let engine = wasmi::Engine::new(&wasmi_config);
        Self { engine, config }
    }

    /// 编译 WASM 模块
    pub fn compile(&self, bytes: &[u8]) -> Result<WasmModule, WasmError> {
        let module = wasmi::Module::new(&self.engine, bytes).map_err(|e| WasmError::InvalidBinary(e.to_string()))?;
        Ok(WasmModule { module })
    }

    /// 获取引擎引用
    pub fn engine(&self) -> &wasmi::Engine {
        &self.engine
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

/// 编译后的 WASM 模块
pub struct WasmModule {
    module: wasmi::Module,
}

fn wasm_value_type_to_wasmi(ty: WasmValueType) -> wasmi::core::ValType {
    match ty {
        WasmValueType::I32 => wasmi::core::ValType::I32,
        WasmValueType::I64 => wasmi::core::ValType::I64,
        WasmValueType::F32 => wasmi::core::ValType::F32,
        WasmValueType::F64 => wasmi::core::ValType::F64,
    }
}

fn wasm_value_to_wasmi(v: &WasmValue) -> wasmi::Val {
    match v {
        WasmValue::I32(n) => wasmi::Val::I32(*n),
        WasmValue::I64(n) => wasmi::Val::I64(*n),
        WasmValue::F32(n) => wasmi::Val::F32((*n).into()),
        WasmValue::F64(n) => wasmi::Val::F64((*n).into()),
    }
}

fn wasmi_val_to_wasm(v: &wasmi::Val) -> WasmValue {
    match v {
        wasmi::Val::I32(n) => WasmValue::I32(*n),
        wasmi::Val::I64(n) => WasmValue::I64(*n),
        wasmi::Val::F32(n) => WasmValue::F32(f32::from(*n)),
        wasmi::Val::F64(n) => WasmValue::F64(f64::from(*n)),
        _ => WasmValue::I32(0),
    }
}

impl WasmModule {
    /// 实例化模块（无主机函数）
    pub fn instantiate(&self, sandbox: &WasmSandbox) -> Result<WasmInstance, WasmError> {
        self.instantiate_with_linker(sandbox, &LinkerConfig::new())
    }

    /// 使用主机函数链接器实例化模块
    pub fn instantiate_with_linker(
        &self,
        sandbox: &WasmSandbox,
        linker_config: &LinkerConfig,
    ) -> Result<WasmInstance, WasmError> {
        let mut store = wasmi::Store::new(sandbox.engine(), ());
        let mut linker = wasmi::Linker::new(sandbox.engine());

        for host_func in linker_config.functions() {
            let params: Vec<wasmi::core::ValType> =
                host_func.params.iter().map(|&p| wasm_value_type_to_wasmi(p)).collect();
            let results: Vec<wasmi::core::ValType> =
                host_func.results.iter().map(|&r| wasm_value_type_to_wasmi(r)).collect();
            let func_type = wasmi::FuncType::new(params, results);

            let arc_func: Arc<crate::HostFn> = host_func.func.clone();
            linker
                .func_new(
                    &host_func.module,
                    &host_func.name,
                    func_type,
                    move |_caller, params, results| {
                        let wasm_params: Vec<WasmValue> = params.iter().map(wasmi_val_to_wasm).collect();
                        let mut wasm_results = Vec::new();
                        match arc_func(&wasm_params, &mut wasm_results) {
                            Ok(()) => {
                                for (i, val) in wasm_results.iter().enumerate() {
                                    if i < results.len() {
                                        results[i] = wasm_value_to_wasmi(val);
                                    }
                                }
                                Ok(())
                            }
                            Err(e) => Err(wasmi::Error::host(HostStringError(e.to_string()))),
                        }
                    },
                )
                .map_err(|e| WasmError::LinkError(e.to_string()))?;
        }

        let instance = linker
            .instantiate(&mut store, &self.module)
            .map_err(|e| WasmError::InstantiationError(e.to_string()))?
            .start(&mut store)
            .map_err(|e| WasmError::InstantiationError(e.to_string()))?;

        Ok(WasmInstance { store, instance })
    }

    /// 获取导出名称列表
    pub fn exports(&self) -> Vec<String> {
        self.module.exports().map(|e| e.name().to_string()).collect()
    }
}

/// WASM 实例
pub struct WasmInstance {
    store: wasmi::Store<()>,
    instance: wasmi::Instance,
}

impl WasmInstance {
    /// 调用导出函数
    pub fn call(&mut self, name: &str, args: &[WasmValue]) -> Result<Vec<WasmValue>, WasmError> {
        let func = self
            .instance
            .get_func(&self.store, name)
            .ok_or_else(|| WasmError::ExportNotFound { name: name.to_string() })?;

        let params: Vec<wasmi::Val> = args.iter().map(wasm_value_to_wasmi).collect();

        // 获取返回值类型以分配输出缓冲区
        let func_type = func.ty(&self.store);
        let result_types: Vec<_> = func_type.results().to_vec();
        let mut outputs: Vec<wasmi::Val> = result_types.iter().map(|t| wasmi::Val::default(*t)).collect();

        func.call(&mut self.store, &params, &mut outputs).map_err(|e| {
            // 检查是否是燃料耗尽的 trap
            if e.as_trap_code() == Some(wasmi::core::TrapCode::OutOfFuel) {
                WasmError::FuelExhausted
            } else {
                WasmError::CallError(e.to_string())
            }
        })?;

        Ok(outputs.iter().map(wasmi_val_to_wasm).collect())
    }

    /// 读取线性内存
    pub fn read_memory(&self, name: &str, offset: usize, len: usize) -> Option<Vec<u8>> {
        let memory = self.instance.get_memory(&self.store, name)?;
        let data = memory.data(&self.store);
        // R3347 deep-review：用 checked_add 防 offset+len 溢出——裸 `offset + len` 在
        // offset=usize::MAX, len>=2 时溢出回绕为小值，`> data.len()` 误判通过 →
        // data[offset..offset+len] OOB 切片 panic。与 write_memory（已用 checked_add）一致。
        let end = offset.checked_add(len)?;
        if end > data.len() {
            return None;
        }
        Some(data[offset..end].to_vec())
    }

    /// 写入线性内存
    pub fn write_memory(&mut self, name: &str, offset: usize, data: &[u8]) -> Result<(), WasmError> {
        let memory = self
            .instance
            .get_memory(&self.store, name)
            .ok_or_else(|| WasmError::ExportNotFound { name: name.to_string() })?;
        let mem_data = memory.data_mut(&mut self.store);
        let end = offset
            .checked_add(data.len())
            .ok_or_else(|| WasmError::MemoryError("offset overflow".into()))?;
        if end > mem_data.len() {
            return Err(WasmError::MemoryError("write out of bounds".into()));
        }
        mem_data[offset..end].copy_from_slice(data);
        Ok(())
    }

    /// 检查导出函数是否存在
    pub fn has_func(&self, name: &str) -> bool {
        self.instance.get_func(&self.store, name).is_some()
    }

    /// 检查导出内存是否存在
    pub fn has_memory(&self, name: &str) -> bool {
        self.instance.get_memory(&self.store, name).is_some()
    }

    /// 获取内存大小（字节数）
    pub fn memory_size(&self, name: &str) -> Option<usize> {
        let memory = self.instance.get_memory(&self.store, name)?;
        Some(memory.data(&self.store).len())
    }

    /// 读取全局导出变量的值。
    ///
    /// `name` 为全局变量导出名称。
    /// 返回 `None` 表示导出不存在或类型不支持。
    pub fn get_global_export(&self, name: &str) -> Option<WasmValue> {
        let global = self.instance.get_global(&self.store, name)?;
        let val = global.get(&self.store);
        Some(wasmi_val_to_wasm(&val))
    }

    /// 检查导出表是否存在。
    pub fn has_table(&self, name: &str) -> bool {
        self.instance.get_table(&self.store, name).is_some()
    }

    /// 设置剩余燃料（需要启用燃料计量）
    ///
    /// # 错误
    ///
    /// 如果燃料计量未启用，返回错误。
    pub fn set_fuel(&mut self, fuel: u64) -> Result<(), WasmError> {
        self.store
            .set_fuel(fuel)
            .map_err(|e| WasmError::CallError(e.to_string()))
    }

    /// 获取剩余燃料（需要启用燃料计量）
    ///
    /// # 错误
    ///
    /// 如果燃料计量未启用，返回错误。
    pub fn get_fuel(&self) -> Result<u64, WasmError> {
        self.store.get_fuel().map_err(|e| WasmError::CallError(e.to_string()))
    }
}
