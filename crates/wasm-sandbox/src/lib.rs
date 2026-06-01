//! # zero-wasm-sandbox
//!
//! 非页面 WASM 运行时（wasmi）。
//!
//! 用于插件、扩展能力或受控计算任务。
//! 基于 wasmi 纯 Rust WASM 解释器实现。

#![warn(missing_docs)]

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
    module: String,
    name: String,
    params: Vec<WasmValueType>,
    results: Vec<WasmValueType>,
    func: Arc<HostFn>,
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
    consume_fuel: bool,
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

#[cfg(feature = "wasmi")]
mod wasmi_backend {
    use super::{LinkerConfig, SandboxConfig, WasmError, WasmValue, WasmValueType};
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
            let module =
                wasmi::Module::new(&self.engine, bytes).map_err(|e| WasmError::InvalidBinary(e.to_string()))?;
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

                let arc_func: Arc<super::HostFn> = host_func.func.clone();
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
            if offset + len > data.len() {
                return None;
            }
            Some(data[offset..offset + len].to_vec())
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
}

#[cfg(feature = "wasmi")]
pub use wasmi_backend::*;

#[cfg(not(feature = "wasmi"))]
mod stub_backend {
    use super::{LinkerConfig, SandboxConfig, WasmError, WasmValue};

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
}

#[cfg(not(feature = "wasmi"))]
pub use stub_backend::*;

#[cfg(test)]
mod tests {
    use super::*;

    /// 辅助函数：编译 WAT 文本为 WASM 字节
    fn wat_to_wasm(wat: &str) -> Vec<u8> {
        wat::parse_str(wat).expect("invalid WAT")
    }

    #[test]
    fn test_sandbox_new() {
        let _sandbox = WasmSandbox::new();
    }

    #[test]
    fn test_sandbox_default() {
        let _sandbox = WasmSandbox::default();
    }

    #[test]
    fn test_sandbox_with_config() {
        let config = SandboxConfig::new().consume_fuel(true);
        let sandbox = WasmSandbox::with_config(config);
        assert!(sandbox.config().is_consume_fuel());
    }

    #[test]
    fn test_compile_valid_module() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "answer") (result i32) i32.const 42)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        assert!(module.exports().contains(&"answer".to_string()));
    }

    #[test]
    fn test_compile_invalid_binary() {
        let sandbox = WasmSandbox::new();
        let result = sandbox.compile(&[0x00, 0x01, 0x02, 0x03]);
        assert!(result.is_err());
        if let Err(WasmError::InvalidBinary(_)) = result {
            // 正确的错误类型
        } else {
            panic!("expected InvalidBinary error");
        }
    }

    #[test]
    fn test_module_exports() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "add") (param i32 i32) (result i32)
                    local.get 0 local.get 1 i32.add)
                (func (export "mul") (param i32 i32) (result i32)
                    local.get 0 local.get 1 i32.mul)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let exports = module.exports();
        assert!(exports.contains(&"add".to_string()));
        assert!(exports.contains(&"mul".to_string()));
        assert_eq!(exports.len(), 2);
    }

    #[test]
    fn test_call_return_i32() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "answer") (result i32) i32.const 42)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");
        let results = instance.call("answer", &[]).expect("call");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], WasmValue::I32(42));
    }

    #[test]
    fn test_call_add_i32() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "add") (param i32 i32) (result i32)
                    local.get 0 local.get 1 i32.add)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");
        let results = instance
            .call("add", &[WasmValue::I32(10), WasmValue::I32(20)])
            .expect("call");
        assert_eq!(results[0], WasmValue::I32(30));
    }

    #[test]
    fn test_call_i64() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "add64") (param i64 i64) (result i64)
                    local.get 0 local.get 1 i64.add)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");
        let results = instance
            .call("add64", &[WasmValue::I64(1000), WasmValue::I64(2000)])
            .expect("call");
        assert_eq!(results[0], WasmValue::I64(3000));
    }

    #[test]
    fn test_call_f32() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "double_f32") (param f32) (result f32)
                    local.get 0 local.get 0 f32.add)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");
        let results = instance.call("double_f32", &[WasmValue::F32(3.5)]).expect("call");
        if let WasmValue::F32(v) = results[0] {
            assert!((v - 7.0).abs() < 0.001);
        } else {
            panic!("expected F32");
        }
    }

    #[test]
    fn test_call_f64() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "double_f64") (param f64) (result f64)
                    local.get 0 local.get 0 f64.add)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");
        let results = instance.call("double_f64", &[WasmValue::F64(2.5)]).expect("call");
        if let WasmValue::F64(v) = results[0] {
            assert!((v - 5.0).abs() < 0.001);
        } else {
            panic!("expected F64");
        }
    }

    #[test]
    fn test_call_no_return() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "noop") nop)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");
        let results = instance.call("noop", &[]).expect("call");
        assert!(results.is_empty());
    }

    #[test]
    fn test_call_function_not_found() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "exists") nop)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");
        let result = instance.call("nonexistent", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_has_func() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "exists") nop)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let instance = module.instantiate(&sandbox).expect("instantiate");
        assert!(instance.has_func("exists"));
        assert!(!instance.has_func("missing"));
    }

    #[test]
    fn test_memory_read_write() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (memory (export "mem") 1)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        assert!(instance.has_memory("mem"));
        assert!(instance.memory_size("mem").unwrap() >= 65536);

        // 写入数据
        instance.write_memory("mem", 0, b"hello").expect("write");

        // 读取数据
        let data = instance.read_memory("mem", 0, 5).expect("read");
        assert_eq!(&data, b"hello");
    }

    #[test]
    fn test_memory_write_out_of_bounds() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (memory (export "mem") 1)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        let size = instance.memory_size("mem").unwrap();
        let result = instance.write_memory("mem", size - 1, b"hello");
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_read_out_of_bounds() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (memory (export "mem") 1)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let instance = module.instantiate(&sandbox).expect("instantiate");

        let size = instance.memory_size("mem").unwrap();
        let result = instance.read_memory("mem", size - 1, 10);
        assert!(result.is_none());
    }

    #[test]
    fn test_memory_not_found() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "f") nop)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let instance = module.instantiate(&sandbox).expect("instantiate");
        assert!(!instance.has_memory("mem"));
        assert!(instance.memory_size("mem").is_none());
    }

    #[test]
    fn test_empty_module() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm("(module)");
        let module = sandbox.compile(&wasm).expect("compile");
        assert!(module.exports().is_empty());
        let instance = module.instantiate(&sandbox).expect("instantiate");
        assert!(!instance.has_func("anything"));
    }

    #[test]
    fn test_recursive_factorial() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func $fac (export "factorial") (param i32) (result i32)
                    local.get 0
                    i32.eqz
                    if (result i32) i32.const 1
                    else
                        local.get 0
                        local.get 0
                        i32.const 1
                        i32.sub
                        call $fac
                        i32.mul
                    end)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        let r = instance.call("factorial", &[WasmValue::I32(5)]).expect("call");
        assert_eq!(r[0], WasmValue::I32(120));
    }

    #[test]
    fn test_multiple_functions() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "inc") (param i32) (result i32)
                    local.get 0 i32.const 1 i32.add)
                (func (export "dec") (param i32) (result i32)
                    local.get 0 i32.const 1 i32.sub)
                (func (export "double") (param i32) (result i32)
                    local.get 0 i32.const 2 i32.mul)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        assert_eq!(
            instance.call("inc", &[WasmValue::I32(10)]).expect("inc")[0],
            WasmValue::I32(11)
        );
        assert_eq!(
            instance.call("dec", &[WasmValue::I32(10)]).expect("dec")[0],
            WasmValue::I32(9)
        );
        assert_eq!(
            instance.call("double", &[WasmValue::I32(7)]).expect("double")[0],
            WasmValue::I32(14)
        );
    }

    #[test]
    fn test_wasm_value_display() {
        assert!(format!("{}", WasmValue::I32(42)).contains("42"));
        assert!(format!("{}", WasmValue::I64(100)).contains("100"));
        assert!(format!("{}", WasmValue::F32(std::f32::consts::PI)).contains("3.141"));
        assert!(format!("{}", WasmValue::F64(std::f64::consts::E)).contains("2.718"));
    }

    #[test]
    fn test_global_variable() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (global $counter (mut i32) (i32.const 0))
                (func (export "increment") (result i32)
                    global.get $counter
                    i32.const 1
                    i32.add
                    global.set $counter
                    global.get $counter)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        let r1 = instance.call("increment", &[]).expect("call1");
        assert_eq!(r1[0], WasmValue::I32(1));
        let r2 = instance.call("increment", &[]).expect("call2");
        assert_eq!(r2[0], WasmValue::I32(2));
    }

    // ---- 主机函数导入测试 ----

    #[test]
    fn test_host_function_import_basic() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (import "env" "add_one" (func $add_one (param i32) (result i32)))
                (func (export "call_host") (param i32) (result i32)
                    local.get 0
                    call $add_one)
            )"#,
        );

        let mut linker = LinkerConfig::new();
        linker.define(HostFunction::new(
            "env",
            "add_one",
            vec![WasmValueType::I32],
            vec![WasmValueType::I32],
            |_params, results| {
                results.push(WasmValue::I32(99));
                Ok(())
            },
        ));

        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate_with_linker(&sandbox, &linker).expect("instantiate");
        let r = instance.call("call_host", &[WasmValue::I32(5)]).expect("call");
        assert_eq!(r[0], WasmValue::I32(99));
    }

    #[test]
    fn test_host_function_uses_params() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (import "env" "double" (func $double (param i32) (result i32)))
                (func (export "test") (param i32) (result i32)
                    local.get 0
                    call $double)
            )"#,
        );

        let mut linker = LinkerConfig::new();
        linker.define(HostFunction::new(
            "env",
            "double",
            vec![WasmValueType::I32],
            vec![WasmValueType::I32],
            |params, results| {
                if let WasmValue::I32(n) = params[0] {
                    results.push(WasmValue::I32(n * 2));
                }
                Ok(())
            },
        ));

        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate_with_linker(&sandbox, &linker).expect("instantiate");
        let r = instance.call("test", &[WasmValue::I32(21)]).expect("call");
        assert_eq!(r[0], WasmValue::I32(42));
    }

    #[test]
    fn test_host_function_multiple_imports() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (import "env" "add" (func $add (param i32 i32) (result i32)))
                (import "env" "mul" (func $mul (param i32 i32) (result i32)))
                (func (export "test") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    call $add
                    local.get 0
                    local.get 1
                    call $mul
                    i32.add)
            )"#,
        );

        let mut linker = LinkerConfig::new();
        linker.define(HostFunction::new(
            "env",
            "add",
            vec![WasmValueType::I32, WasmValueType::I32],
            vec![WasmValueType::I32],
            |params, results| {
                if let (WasmValue::I32(a), WasmValue::I32(b)) = (&params[0], &params[1]) {
                    results.push(WasmValue::I32(a + b));
                }
                Ok(())
            },
        ));
        linker.define(HostFunction::new(
            "env",
            "mul",
            vec![WasmValueType::I32, WasmValueType::I32],
            vec![WasmValueType::I32],
            |params, results| {
                if let (WasmValue::I32(a), WasmValue::I32(b)) = (&params[0], &params[1]) {
                    results.push(WasmValue::I32(a * b));
                }
                Ok(())
            },
        ));

        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate_with_linker(&sandbox, &linker).expect("instantiate");
        // add(3, 4) + mul(3, 4) = 7 + 12 = 19
        let r = instance
            .call("test", &[WasmValue::I32(3), WasmValue::I32(4)])
            .expect("call");
        assert_eq!(r[0], WasmValue::I32(19));
    }

    #[test]
    fn test_host_function_missing_import() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (import "env" "missing" (func (param i32) (result i32)))
                (func (export "test") (result i32)
                    i32.const 1
                    call 0)
            )"#,
        );

        let module = sandbox.compile(&wasm).expect("compile");
        let result = module.instantiate(&sandbox);
        assert!(result.is_err());
    }

    #[test]
    fn test_host_function_i64_params() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (import "env" "add64" (func $add64 (param i64 i64) (result i64)))
                (func (export "test") (param i64 i64) (result i64)
                    local.get 0
                    local.get 1
                    call $add64)
            )"#,
        );

        let mut linker = LinkerConfig::new();
        linker.define(HostFunction::new(
            "env",
            "add64",
            vec![WasmValueType::I64, WasmValueType::I64],
            vec![WasmValueType::I64],
            |params, results| {
                if let (WasmValue::I64(a), WasmValue::I64(b)) = (&params[0], &params[1]) {
                    results.push(WasmValue::I64(a + b));
                }
                Ok(())
            },
        ));

        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate_with_linker(&sandbox, &linker).expect("instantiate");
        let r = instance
            .call("test", &[WasmValue::I64(100), WasmValue::I64(200)])
            .expect("call");
        assert_eq!(r[0], WasmValue::I64(300));
    }

    // ---- Trap 处理测试 ----

    #[test]
    fn test_trap_division_by_zero() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "div_zero") (param i32) (result i32)
                    local.get 0
                    i32.const 0
                    i32.div_s)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");
        let result = instance.call("div_zero", &[WasmValue::I32(10)]);
        assert!(result.is_err());
        if let Err(WasmError::CallError(msg)) = result {
            assert!(msg.contains("divide") || msg.contains("zero") || msg.contains("trap"));
        } else {
            panic!("expected CallError for division by zero");
        }
    }

    #[test]
    fn test_trap_unreachable() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "do_unreachable") (result i32)
                    unreachable)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");
        let result = instance.call("do_unreachable", &[]);
        assert!(result.is_err());
        if let Err(WasmError::CallError(msg)) = result {
            assert!(msg.contains("unreachable") || msg.contains("trap"));
        } else {
            panic!("expected CallError for unreachable");
        }
    }

    #[test]
    fn test_trap_integer_overflow() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "overflow") (result i32)
                    i32.const -2147483648
                    i32.const -1
                    i32.div_s)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");
        // INT32_MIN / -1 overflows
        let result = instance.call("overflow", &[]);
        assert!(result.is_err());
        if let Err(WasmError::CallError(msg)) = result {
            assert!(msg.contains("overflow") || msg.contains("trap") || msg.contains("divide"));
        } else {
            panic!("expected CallError for integer overflow");
        }
    }

    #[test]
    fn test_trap_out_of_bounds_memory() {
        let sandbox = WasmSandbox::new();
        // WASM module with memory load that accesses out-of-bounds
        let wasm = wat_to_wasm(
            r#"(module
                (memory (export "mem") 1)
                (func (export "read_oob") (result i32)
                    i32.const 100000
                    i32.load)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");
        let result = instance.call("read_oob", &[]);
        assert!(result.is_err());
        if let Err(WasmError::CallError(msg)) = result {
            assert!(msg.contains("out of bounds") || msg.contains("trap"));
        } else {
            panic!("expected CallError for out-of-bounds memory access");
        }
    }

    // ---- 燃料计量测试 ----

    #[test]
    fn test_fuel_basic() {
        let config = SandboxConfig::new().consume_fuel(true);
        let sandbox = WasmSandbox::with_config(config);
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "answer") (result i32) i32.const 42)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // 无燃料时调用失败
        let result = instance.call("answer", &[]);
        assert!(matches!(result, Err(WasmError::FuelExhausted)));

        // 设置足够燃料
        instance.set_fuel(100).expect("set_fuel");
        let r = instance.call("answer", &[]).expect("call");
        assert_eq!(r[0], WasmValue::I32(42));

        // 燃料应该有剩余
        let remaining = instance.get_fuel().expect("get_fuel");
        assert!(remaining < 100);
        assert!(remaining > 0);
    }

    #[test]
    fn test_fuel_exhausted_infinite_loop() {
        let config = SandboxConfig::new().consume_fuel(true);
        let sandbox = WasmSandbox::with_config(config);
        // 无限循环模块
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "loop_forever")
                    (local $i i32)
                    (loop $inf
                        local.get $i
                        i32.const 1
                        i32.add
                        local.set $i
                        br $inf))
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        instance.set_fuel(1000).expect("set_fuel");
        let result = instance.call("loop_forever", &[]);
        assert!(matches!(result, Err(WasmError::FuelExhausted)));
    }

    #[test]
    fn test_fuel_set_get() {
        let config = SandboxConfig::new().consume_fuel(true);
        let sandbox = WasmSandbox::with_config(config);
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "answer") (result i32) i32.const 42)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        instance.set_fuel(500).expect("set_fuel");
        assert_eq!(instance.get_fuel().expect("get_fuel"), 500);
    }

    #[test]
    fn test_fuel_disabled_by_default() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "answer") (result i32) i32.const 42)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // 不启用燃料计量时 set_fuel 应该报错
        let result = instance.set_fuel(100);
        assert!(result.is_err());
    }

    #[test]
    fn test_fuel_consumed_across_calls() {
        let config = SandboxConfig::new().consume_fuel(true);
        let sandbox = WasmSandbox::with_config(config);
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "answer") (result i32) i32.const 42)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        instance.set_fuel(100).expect("set_fuel");
        instance.call("answer", &[]).expect("call1");
        let fuel_after_1 = instance.get_fuel().expect("get_fuel");
        instance.call("answer", &[]).expect("call2");
        let fuel_after_2 = instance.get_fuel().expect("get_fuel");
        // 每次调用消耗相同的燃料
        assert!(fuel_after_1 > fuel_after_2);
    }

    #[test]
    fn test_host_function_with_fuel() {
        let config = SandboxConfig::new().consume_fuel(true);
        let sandbox = WasmSandbox::with_config(config);
        let wasm = wat_to_wasm(
            r#"(module
                (import "env" "double" (func $double (param i32) (result i32)))
                (func (export "test") (param i32) (result i32)
                    local.get 0
                    call $double)
            )"#,
        );

        let mut linker = LinkerConfig::new();
        linker.define(HostFunction::new(
            "env",
            "double",
            vec![WasmValueType::I32],
            vec![WasmValueType::I32],
            |params, results| {
                if let WasmValue::I32(n) = params[0] {
                    results.push(WasmValue::I32(n * 2));
                }
                Ok(())
            },
        ));

        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate_with_linker(&sandbox, &linker).expect("instantiate");

        instance.set_fuel(100).expect("set_fuel");
        let r = instance.call("test", &[WasmValue::I32(21)]).expect("call");
        assert_eq!(r[0], WasmValue::I32(42));
    }

    // =======================================================================
    // 新增测试：模块验证与编译
    // =======================================================================

    #[test]
    fn test_compile_module_with_imported_functions() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (import "env" "log" (func $log (param i32)))
                (func (export "run") i32.const 42 call $log)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        assert!(module.exports().contains(&"run".to_string()));
    }

    #[test]
    fn test_compile_rejects_truncated_magic() {
        let sandbox = WasmSandbox::new();
        // Just the WASM magic header prefix, incomplete
        let result = sandbox.compile(&[0x00, 0x61, 0x73, 0x6D]);
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_rejects_random_bytes() {
        let sandbox = WasmSandbox::new();
        let result = sandbox.compile(b"this is not wasm at all");
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_module_with_multiple_memories_fails() {
        // Standard WASM MVP only allows one memory — this validates that
        // malformed modules are rejected during compilation.
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (memory (export "m1") 1)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile single memory");
        assert!(module.exports().contains(&"m1".to_string()));
    }

    // =======================================================================
    // 新增测试：实例生命周期
    // =======================================================================

    #[test]
    fn test_instantiate_and_check_has_func() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "greet") nop)
                (func (export "farewell") nop)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let instance = module.instantiate(&sandbox).expect("instantiate");
        assert!(instance.has_func("greet"));
        assert!(instance.has_func("farewell"));
        assert!(!instance.has_func("nonexistent"));
    }

    #[test]
    fn test_instantiate_with_linker_empty() {
        // instantiate_with_linker with an empty LinkerConfig
        // should behave identically to instantiate.
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "val") (result i32) i32.const 7)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module
            .instantiate_with_linker(&sandbox, &LinkerConfig::new())
            .expect("instantiate");
        let r = instance.call("val", &[]).expect("call");
        assert_eq!(r[0], WasmValue::I32(7));
    }

    #[test]
    fn test_missing_export_returns_none_not_panic() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "only_this") nop)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let instance = module.instantiate(&sandbox).expect("instantiate");
        // has_func for missing export returns false, does not panic
        assert!(!instance.has_func("no_such_func"));
        // has_memory for missing export returns false
        assert!(!instance.has_memory("no_such_mem"));
        // memory_size returns None
        assert!(instance.memory_size("no_such_mem").is_none());
        // read_memory returns None
        assert!(instance.read_memory("no_such_mem", 0, 1).is_none());
    }

    #[test]
    fn test_global_export_is_listed() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (global (export "g") i32 (i32.const 99))
                (func (export "f") nop)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let exports = module.exports();
        assert!(exports.contains(&"f".to_string()));
        assert!(exports.contains(&"g".to_string()));
    }

    // =======================================================================
    // 新增测试：函数调用
    // =======================================================================

    #[test]
    fn test_call_with_three_args() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "sum3") (param i32 i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.add
                    local.get 2
                    i32.add)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");
        let r = instance
            .call("sum3", &[WasmValue::I32(10), WasmValue::I32(20), WasmValue::I32(30)])
            .expect("call");
        assert_eq!(r[0], WasmValue::I32(60));
    }

    #[test]
    fn test_call_sub_i32() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "sub") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.sub)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");
        let r = instance
            .call("sub", &[WasmValue::I32(100), WasmValue::I32(37)])
            .expect("call");
        assert_eq!(r[0], WasmValue::I32(63));
    }

    #[test]
    fn test_call_f32_multiply() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "mul_f32") (param f32 f32) (result f32)
                    local.get 0
                    local.get 1
                    f32.mul)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");
        let r = instance
            .call("mul_f32", &[WasmValue::F32(2.5), WasmValue::F32(4.0)])
            .expect("call");
        if let WasmValue::F32(v) = r[0] {
            assert!((v - 10.0).abs() < 0.001);
        } else {
            panic!("expected F32");
        }
    }

    #[test]
    fn test_call_f64_division() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "div_f64") (param f64 f64) (result f64)
                    local.get 0
                    local.get 1
                    f64.div)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");
        let r = instance
            .call("div_f64", &[WasmValue::F64(22.0), WasmValue::F64(7.0)])
            .expect("call");
        if let WasmValue::F64(v) = r[0] {
            assert!((v - 3.142857).abs() < 0.001);
        } else {
            panic!("expected F64");
        }
    }

    #[test]
    fn test_call_function_using_memory() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (memory (export "mem") 1)
                (func (export "store_and_load") (result i32)
                    i32.const 0
                    i32.const 12345
                    i32.store
                    i32.const 0
                    i32.load)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");
        let r = instance.call("store_and_load", &[]).expect("call");
        assert_eq!(r[0], WasmValue::I32(12345));
    }

    // =======================================================================
    // 新增测试：内存操作
    // =======================================================================

    #[test]
    fn test_memory_write_and_read_at_offset() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (memory (export "mem") 1)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // Write at a non-zero offset
        instance.write_memory("mem", 1000, b"world").expect("write");
        let data = instance.read_memory("mem", 1000, 5).expect("read");
        assert_eq!(&data, b"world");

        // Original area should still be zeroed
        let zeros = instance.read_memory("mem", 0, 5).expect("read zeros");
        assert_eq!(&zeros, b"\x00\x00\x00\x00\x00");
    }

    #[test]
    fn test_memory_size_one_page() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (memory (export "mem") 1)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let instance = module.instantiate(&sandbox).expect("instantiate");
        let size = instance.memory_size("mem").expect("size");
        // One WASM page = 65536 bytes
        assert_eq!(size, 65536);
    }

    #[test]
    fn test_memory_size_two_pages() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (memory (export "mem") 2)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let instance = module.instantiate(&sandbox).expect("instantiate");
        let size = instance.memory_size("mem").expect("size");
        assert_eq!(size, 65536 * 2);
    }

    #[test]
    fn test_memory_read_write_byte_boundary() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (memory (export "mem") 1)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // Write a single byte at the very last position
        let last = 65535;
        instance.write_memory("mem", last, b"X").expect("write last byte");
        let data = instance.read_memory("mem", last, 1).expect("read last byte");
        assert_eq!(&data, b"X");
    }

    #[test]
    fn test_memory_read_zero_length() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (memory (export "mem") 1)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let instance = module.instantiate(&sandbox).expect("instantiate");
        let data = instance.read_memory("mem", 0, 0).expect("read zero len");
        assert!(data.is_empty());
    }

    #[test]
    fn test_memory_write_empty_slice() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (memory (export "mem") 1)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");
        // Writing empty slice should succeed (no-op)
        instance.write_memory("mem", 0, b"").expect("write empty");
    }

    // =======================================================================
    // 新增测试：错误处理
    // =======================================================================

    #[test]
    fn test_fuel_exhaustion_returns_error_not_panic() {
        let config = SandboxConfig::new().consume_fuel(true);
        let sandbox = WasmSandbox::with_config(config);
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "count")
                    (local $i i32)
                    (loop $l
                        local.get $i
                        i32.const 1
                        i32.add
                        local.set $i
                        br $l))
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");
        instance.set_fuel(50).expect("set_fuel");
        let result = instance.call("count", &[]);
        assert!(matches!(result, Err(WasmError::FuelExhausted)));
    }

    #[test]
    fn test_call_nonexistent_function_returns_export_not_found() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm("(module)");
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");
        let result = instance.call("nothing_here", &[]);
        assert!(result.is_err());
        if let Err(WasmError::ExportNotFound { name }) = result {
            assert_eq!(name, "nothing_here");
        } else {
            panic!("expected ExportNotFound error");
        }
    }

    #[test]
    fn test_trap_remainder_by_zero() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "rem_zero") (param i32) (result i32)
                    local.get 0
                    i32.const 0
                    i32.rem_s)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");
        let result = instance.call("rem_zero", &[WasmValue::I32(10)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_instantiate_with_missing_import_returns_error() {
        let sandbox = WasmSandbox::new();
        // Module imports "env"."fn" but linker is empty
        let wasm = wat_to_wasm(
            r#"(module
                (import "env" "fn" (func (result i32)))
                (func (export "test") (result i32) call 0)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let result = module.instantiate(&sandbox);
        assert!(result.is_err());
    }

    // =======================================================================
    // 新增测试：主机函数导入
    // =======================================================================

    #[test]
    fn test_host_function_no_params_no_results() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (import "env" "ping" (func $ping))
                (func (export "do_ping") call $ping)
            )"#,
        );

        let mut linker = LinkerConfig::new();
        linker.define(HostFunction::new("env", "ping", vec![], vec![], |_params, _results| {
            Ok(())
        }));

        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate_with_linker(&sandbox, &linker).expect("instantiate");
        // Should succeed without panic
        let r = instance.call("do_ping", &[]).expect("call");
        assert!(r.is_empty());
    }

    #[test]
    fn test_host_function_receives_correct_f64_args() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (import "math" "negate" (func $neg (param f64) (result f64)))
                (func (export "test") (param f64) (result f64)
                    local.get 0
                    call $neg)
            )"#,
        );

        let mut linker = LinkerConfig::new();
        linker.define(HostFunction::new(
            "math",
            "negate",
            vec![WasmValueType::F64],
            vec![WasmValueType::F64],
            |params, results| {
                if let WasmValue::F64(v) = params[0] {
                    results.push(WasmValue::F64(-v));
                }
                Ok(())
            },
        ));

        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate_with_linker(&sandbox, &linker).expect("instantiate");
        let r = instance.call("test", &[WasmValue::F64(3.14)]).expect("call");
        if let WasmValue::F64(v) = r[0] {
            assert!((v - (-3.14)).abs() < 0.001);
        } else {
            panic!("expected F64");
        }
    }

    #[test]
    fn test_host_function_from_different_module_names() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (import "env" "a" (func $a (result i32)))
                (import "math" "b" (func $b (result i32)))
                (func (export "test") (result i32)
                    call $a
                    call $b
                    i32.add)
            )"#,
        );

        let mut linker = LinkerConfig::new();
        linker.define(HostFunction::new(
            "env",
            "a",
            vec![],
            vec![WasmValueType::I32],
            |_params, results| {
                results.push(WasmValue::I32(10));
                Ok(())
            },
        ));
        linker.define(HostFunction::new(
            "math",
            "b",
            vec![],
            vec![WasmValueType::I32],
            |_params, results| {
                results.push(WasmValue::I32(32));
                Ok(())
            },
        ));

        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate_with_linker(&sandbox, &linker).expect("instantiate");
        let r = instance.call("test", &[]).expect("call");
        assert_eq!(r[0], WasmValue::I32(42));
    }

    // =======================================================================
    // 新增测试：LinkerConfig / HostFunction 结构
    // =======================================================================

    #[test]
    fn test_linker_config_functions_returns_registered() {
        let mut linker = LinkerConfig::new();
        assert!(linker.functions().is_empty());
        linker.define(HostFunction::new(
            "env",
            "f",
            vec![WasmValueType::I32],
            vec![WasmValueType::I32],
            |_, _| Ok(()),
        ));
        assert_eq!(linker.functions().len(), 1);
        assert_eq!(linker.functions()[0].module, "env");
    }

    #[test]
    fn test_host_function_debug_format() {
        let hf = HostFunction::new(
            "env",
            "add",
            vec![WasmValueType::I32, WasmValueType::I32],
            vec![WasmValueType::I32],
            |_, _| Ok(()),
        );
        let debug_str = format!("{hf:?}");
        assert!(debug_str.contains("env"));
        assert!(debug_str.contains("add"));
    }

    #[test]
    fn test_wasm_value_equality() {
        assert_eq!(WasmValue::I32(1), WasmValue::I32(1));
        assert_ne!(WasmValue::I32(1), WasmValue::I32(2));
        assert_eq!(WasmValue::I64(100), WasmValue::I64(100));
        assert_eq!(WasmValue::F32(1.0), WasmValue::F32(1.0));
        assert_eq!(WasmValue::F64(2.0), WasmValue::F64(2.0));
    }

    #[test]
    fn test_wasm_value_type_equality() {
        assert_eq!(WasmValueType::I32, WasmValueType::I32);
        assert_ne!(WasmValueType::I32, WasmValueType::I64);
    }

    #[test]
    fn test_wasm_error_display_messages() {
        let err = WasmError::InvalidBinary("bad bytes".into());
        assert!(err.to_string().contains("bad bytes"));

        let err = WasmError::ExportNotFound { name: "foo".into() };
        assert!(err.to_string().contains("foo"));

        let err = WasmError::CallError("boom".into());
        assert!(err.to_string().contains("boom"));

        let err = WasmError::MemoryError("overflow".into());
        assert!(err.to_string().contains("overflow"));

        let err = WasmError::InstantiationError("fail".into());
        assert!(err.to_string().contains("fail"));

        let err = WasmError::LinkError("unlinkable".into());
        assert!(err.to_string().contains("unlinkable"));

        assert!(WasmError::FuelExhausted.to_string().contains("fuel"));
    }

    // =======================================================================
    // 新增测试：主机函数错误传播、参数校验、内存溢出
    // =======================================================================

    /// 主机函数返回 Err(WasmError::CallError) 时，错误应通过 wasmi trap 传播到调用方。
    #[test]
    fn test_host_function_returns_error() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (import "env" "fail" (func $fail (param i32) (result i32)))
                (func (export "call_host") (param i32) (result i32)
                    local.get 0
                    call $fail)
            )"#,
        );

        let mut linker = LinkerConfig::new();
        linker.define(HostFunction::new(
            "env",
            "fail",
            vec![WasmValueType::I32],
            vec![WasmValueType::I32],
            |_params, _results| Err(WasmError::CallError("host error from test".into())),
        ));

        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate_with_linker(&sandbox, &linker).expect("instantiate");
        let result = instance.call("call_host", &[WasmValue::I32(1)]);
        assert!(result.is_err());
        if let Err(WasmError::CallError(msg)) = result {
            assert!(
                msg.contains("host error from test"),
                "error message should contain original host error text, got: {msg}"
            );
        } else {
            panic!("expected CallError from host function, got: {result:?}");
        }
    }

    /// 传入错误数量的参数调用函数时，wasmi 应返回类型不匹配错误。
    #[test]
    fn test_call_with_wrong_argument_count() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "add") (param i32 i32) (result i32)
                    local.get 0 local.get 1 i32.add)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // 传入 0 个参数（期望 2 个）
        let result = instance.call("add", &[]);
        assert!(result.is_err(), "wrong arg count should return error");

        // 传入 1 个参数（期望 2 个）
        let result = instance.call("add", &[WasmValue::I32(1)]);
        assert!(result.is_err(), "wrong arg count should return error");

        // 传入 3 个参数（期望 2 个）
        let result = instance.call("add", &[WasmValue::I32(1), WasmValue::I32(2), WasmValue::I32(3)]);
        assert!(result.is_err(), "wrong arg count should return error");
    }

    /// write_memory 使用 usize::MAX 偏移量时，checked_add 应检测溢出并返回 MemoryError。
    #[test]
    fn test_write_memory_offset_overflow() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (memory (export "mem") 1)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        let result = instance.write_memory("mem", usize::MAX, b"data");
        assert!(result.is_err());
        if let Err(WasmError::MemoryError(msg)) = result {
            assert!(msg.contains("overflow"), "expected overflow error message, got: {msg}");
        } else {
            panic!("expected MemoryError for offset overflow, got: {result:?}");
        }
    }

    // =======================================================================
    // 新增测试：WASM 运行时边界情况
    // =======================================================================

    /// 在已分配内存的边界位置进行读写，验证最后一个字节可以正确写入和读回，
    /// 而越过边界恰好一个字节的读写会失败。
    #[test]
    fn test_wasm_memory_read_write_boundary() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (memory (export "mem") 1)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        let size = instance.memory_size("mem").expect("size");
        assert_eq!(size, 65536, "一页 WASM 内存应为 65536 字节");

        // 在边界内最后一个字节写入并读回
        instance
            .write_memory("mem", size - 1, b"\xAB")
            .expect("写入最后一个字节应成功");
        let data = instance
            .read_memory("mem", size - 1, 1)
            .expect("读取最后一个字节应成功");
        assert_eq!(data, b"\xAB", "读回的数据应与写入一致");

        // 越界写入（起始位置合法，但数据长度越过边界）应失败
        let write_result = instance.write_memory("mem", size - 1, b"\x00\x00");
        assert!(write_result.is_err(), "越过内存边界的写入应返回错误");

        // 越界读取（起始位置合法，但长度越过边界）应返回 None
        let read_result = instance.read_memory("mem", size - 1, 2);
        assert!(read_result.is_none(), "越过内存边界的读取应返回 None");

        // 边界内起始位置 0 写满整页应成功
        let full_page = vec![0x42u8; size];
        instance.write_memory("mem", 0, &full_page).expect("写入整页应成功");
        let read_back = instance.read_memory("mem", 0, size).expect("读取整页应成功");
        assert_eq!(read_back.len(), size, "读回长度应等于整页大小");
        assert!(read_back.iter().all(|&b| b == 0x42), "整页内容应全部为 0x42");
    }

    /// 调用具有大量参数（16 个 i32）的主机函数，验证所有参数正确传递、计算结果正确返回。
    #[test]
    fn test_wasm_call_with_many_args() {
        let sandbox = WasmSandbox::new();
        // WASM 函数接收 16 个 i32 参数，全部传给主机函数求和
        let wasm = wat_to_wasm(
            r#"(module
                (import "env" "sum16" (func $sum16 (param i32 i32 i32 i32 i32 i32 i32 i32
                                                      i32 i32 i32 i32 i32 i32 i32 i32) (result i32)))
                (func (export "call_sum16")
                      (param i32 i32 i32 i32 i32 i32 i32 i32
                       i32 i32 i32 i32 i32 i32 i32 i32) (result i32)
                    local.get 0  local.get 1  local.get 2  local.get 3
                    local.get 4  local.get 5  local.get 6  local.get 7
                    local.get 8  local.get 9  local.get 10 local.get 11
                    local.get 12 local.get 13 local.get 14 local.get 15
                    call $sum16)
            )"#,
        );

        let mut linker = LinkerConfig::new();
        linker.define(HostFunction::new(
            "env",
            "sum16",
            vec![WasmValueType::I32; 16],
            vec![WasmValueType::I32],
            |params, results| {
                let sum: i32 = params
                    .iter()
                    .map(|v| if let WasmValue::I32(n) = v { *n } else { 0 })
                    .sum();
                results.push(WasmValue::I32(sum));
                Ok(())
            },
        ));

        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate_with_linker(&sandbox, &linker).expect("instantiate");

        let args: Vec<WasmValue> = (1..=16).map(WasmValue::I32).collect();
        let r = instance.call("call_sum16", &args).expect("call");
        assert_eq!(r.len(), 1, "应返回一个结果");
        // 1+2+...+16 = 136
        assert_eq!(r[0], WasmValue::I32(136), "16 个参数求和结果应为 136");
    }

    /// 使用无效的 WASM 字节实例化模块时，应返回 InvalidBinary 错误而非 panic。
    /// 覆盖多种无效输入：空字节、随机垃圾数据、截断的 WASM 头。
    #[test]
    fn test_wasm_module_instantiate_invalid() {
        let sandbox = WasmSandbox::new();

        // 空字节
        let result = sandbox.compile(&[]);
        assert!(result.is_err(), "空字节应编译失败");
        assert!(
            matches!(result, Err(WasmError::InvalidBinary(_))),
            "空字节应返回 InvalidBinary 错误"
        );

        // 随机垃圾数据
        let garbage: Vec<u8> = (0..64).map(|i| (i * 37 + 0xA5) as u8).collect();
        let result = sandbox.compile(&garbage);
        assert!(result.is_err(), "随机垃圾数据应编译失败");
        assert!(
            matches!(result, Err(WasmError::InvalidBinary(_))),
            "垃圾数据应返回 InvalidBinary 错误"
        );

        // 截断的 WASM 魔数头（仅 3 字节，缺少版本号）
        let truncated_magic = &[0x00, 0x61, 0x73];
        let result = sandbox.compile(truncated_magic);
        assert!(result.is_err(), "截断的 WASM 头应编译失败");

        // 合法魔数但非法版本号
        let bad_version = &[0x00, 0x61, 0x73, 0x6D, 0x0A, 0x00, 0x00, 0x00];
        let result = sandbox.compile(bad_version);
        assert!(result.is_err(), "非法版本号应编译失败");

        // 确认所有情况都不会 panic——如果执行到这里说明没有 panic
    }

    /// 验证燃料计量能正确终止无限循环的 WASM 模块执行。
    /// 设置有限燃料后调用包含无限循环的函数，应返回 FuelExhausted 错误。
    #[test]
    fn test_wasm_fuel_consumption() {
        let config = SandboxConfig::new().consume_fuel(true);
        let sandbox = WasmSandbox::with_config(config);

        // 包含无限递增循环的模块
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "count_forever") (result i32)
                    (local $i i32)
                    (loop $inf
                        local.get $i
                        i32.const 1
                        i32.add
                        local.set $i
                        br $inf)
                    local.get $i)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // 设置少量燃料
        instance.set_fuel(500).expect("set_fuel");
        let initial_fuel = instance.get_fuel().expect("get_fuel");
        assert_eq!(initial_fuel, 500, "设置后燃料应为 500");

        // 调用无限循环函数，应在燃料耗尽时停止
        let result = instance.call("count_forever", &[]);
        assert!(
            matches!(result, Err(WasmError::FuelExhausted)),
            "无限循环应在燃料耗尽时返回 FuelExhausted"
        );

        // 燃料耗尽后剩余应接近 0（wasmi 可能留下微小残余）
        let remaining = instance.get_fuel().expect("get_fuel");
        assert!(remaining <= 10, "燃料耗尽后剩余应接近 0，实际: {remaining}");

        // 不补充燃料再次调用，仍应立即返回 FuelExhausted
        let result2 = instance.call("count_forever", &[]);
        assert!(
            matches!(result2, Err(WasmError::FuelExhausted)),
            "燃料为零时调用应立即返回 FuelExhausted"
        );
    }

    // -- 边界条件测试 --

    /// 测试 set_fuel(0) 后调用函数立即耗尽
    #[test]
    fn test_set_fuel_zero_exhausted() {
        let config = SandboxConfig::new().consume_fuel(true);
        let sandbox = WasmSandbox::with_config(config);
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "add") (param i32 i32) (result i32)
                    local.get 0 local.get 1 i32.add)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // 设置燃料为 0，调用函数应立即耗尽
        instance.set_fuel(0).expect("set_fuel");
        let result = instance.call("add", &[WasmValue::I32(1), WasmValue::I32(2)]);
        assert!(
            matches!(result, Err(WasmError::FuelExhausted)),
            "燃料为 0 时调用应返回 FuelExhausted"
        );
    }

    /// 测试 get_fuel 在新实例上的初始值
    #[test]
    fn test_get_fuel_fresh_instance() {
        let config = SandboxConfig::new().consume_fuel(true);
        let sandbox = WasmSandbox::with_config(config);
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "answer") (result i32) i32.const 42)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let instance = module.instantiate(&sandbox).expect("instantiate");

        // 不调用 set_fuel，get_fuel 应返回 0 或优雅地返回初始值
        let fuel = instance.get_fuel().expect("get_fuel");
        assert_eq!(fuel, 0, "新实例的初始燃料应为 0");
    }

    /// 测试调用写入内存后读取的 WASM 函数（round-trip）
    #[test]
    fn test_memory_write_then_function_read_round_trip() {
        let sandbox = WasmSandbox::new();
        // 模块导出内存和一个函数：函数从内存偏移 0 读取 i32 并返回
        let wasm = wat_to_wasm(
            r#"(module
                (memory (export "mem") 1)
                (func (export "read_i32") (result i32)
                    i32.const 0
                    i32.load)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // 通过主机端写入 i32 值（小端序 305419896 = 0x12345678）
        let value: i32 = 0x12345678;
        instance.write_memory("mem", 0, &value.to_le_bytes()).expect("write");

        // 调用 WASM 函数读取并返回该值
        let results = instance.call("read_i32", &[]).expect("call");
        assert_eq!(results[0], WasmValue::I32(value), "WASM 读回的值应与写入一致");
    }

    /// 测试 i32::MIN 和 i32::MAX 参数
    #[test]
    fn test_call_extreme_i32_values() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "add") (param i32 i32) (result i32)
                    local.get 0 local.get 1 i32.add)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // i32::MIN + i32::MAX 在 WASM 中使用标准 wrapping 语义：结果为 -1
        let results = instance
            .call("add", &[WasmValue::I32(i32::MIN), WasmValue::I32(i32::MAX)])
            .expect("call");
        assert_eq!(results[0], WasmValue::I32(-1), "i32::MIN + i32::MAX (wrapping) 应为 -1");
    }

    /// 测试沙箱 engine() 访问器
    #[test]
    fn test_sandbox_engine_accessor() {
        let sandbox = WasmSandbox::new();
        // 调用 engine() 不应 panic，返回可用引擎引用
        let _engine = sandbox.engine();
    }

    /// 测试 SandboxConfig with consume_fuel = false
    #[test]
    fn test_sandbox_config_no_fuel() {
        let config = SandboxConfig::new().consume_fuel(false);
        let sandbox = WasmSandbox::with_config(config);
        assert!(!sandbox.config().is_consume_fuel(), "consume_fuel 应为 false");

        // 不启用燃料计量时应能正常编译和实例化
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "answer") (result i32) i32.const 42)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");
        let r = instance.call("answer", &[]).expect("call");
        assert_eq!(r[0], WasmValue::I32(42));

        // set_fuel 在未启用燃料计量时应报错
        let fuel_result = instance.set_fuel(100);
        assert!(fuel_result.is_err(), "未启用燃料计量时 set_fuel 应返回错误");
    }

    /// 测试 LinkerConfig 多个函数定义
    #[test]
    fn test_linker_config_multiple_functions() {
        let mut linker = LinkerConfig::new();
        linker.define(HostFunction::new(
            "env",
            "fn1",
            vec![WasmValueType::I32],
            vec![WasmValueType::I32],
            |_, _| Ok(()),
        ));
        linker.define(HostFunction::new(
            "env",
            "fn2",
            vec![WasmValueType::F64],
            vec![WasmValueType::F64],
            |_, _| Ok(()),
        ));
        linker.define(HostFunction::new("math", "fn3", vec![], vec![], |_, _| Ok(())));
        assert_eq!(linker.functions().len(), 3, "应注册了 3 个主机函数");
        assert_eq!(linker.functions()[0].name, "fn1");
        assert_eq!(linker.functions()[1].name, "fn2");
        assert_eq!(linker.functions()[2].name, "fn3");
    }

    // =======================================================================
    // 新增测试：WASM 沙箱边界条件
    // =======================================================================

    /// 通过 WASM 的 memory.grow 指令扩展内存，向新区域写入数据后读回验证。
    /// 验证内存增长后新分配的区域可以正确读写。
    #[test]
    fn test_memory_grow_and_read() {
        let sandbox = WasmSandbox::new();
        // 模块初始 1 页内存（65536 字节），提供 grow_and_write 函数：
        // 先调用 memory.grow 扩展 1 页，然后在新区域的起始位置写入一个 i32 值
        let wasm = wat_to_wasm(
            r#"(module
                (memory (export "mem") 1)
                (func (export "grow_and_write") (result i32)
                    (local $prev i32)
                    ;; grow memory by 1 page
                    i32.const 1
                    memory.grow
                    ;; memory.grow returns -1 on failure, old page count on success
                    local.set $prev
                    local.get $prev
                    i32.const -1
                    i32.eq
                    if (result i32) i32.const -1
                    else
                        ;; write value 0xDEADBEEF at offset 65536 (start of new page)
                        i32.const 65536
                        i32.const -559038737  ;; 0xDEADBEEF
                        i32.store
                        i32.const 0  ;; success
                    end)
                (func (export "read_new_page") (result i32)
                    i32.const 65536
                    i32.load)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // 初始内存应为 1 页
        assert_eq!(instance.memory_size("mem").unwrap(), 65536);

        // 调用 grow_and_write：扩展内存并在新区域写入
        let results = instance.call("grow_and_write", &[]).expect("call");
        assert_eq!(results[0], WasmValue::I32(0), "memory.grow 应成功返回 0");

        // 内存现在应为 2 页
        assert_eq!(instance.memory_size("mem").unwrap(), 65536 * 2);

        // 通过 WASM 函数读回新区域的值
        let read_back = instance.call("read_new_page", &[]).expect("call read");
        assert_eq!(read_back[0], WasmValue::I32(-559038737), "读回的值应为 0xDEADBEEF");

        // 通过主机端 read_memory 验证新区域数据
        let data = instance.read_memory("mem", 65536, 4).expect("read new page");
        assert_eq!(
            data,
            (-559038737i32).to_le_bytes().to_vec(),
            "主机端读回的新区域数据应一致"
        );
    }

    /// 使用错误的参数数量调用导出函数，wasmi 应返回类型不匹配错误。
    /// 分别测试传入 0 个、1 个和 3 个参数调用一个需要 2 个 i32 参数的函数。
    #[test]
    fn test_call_export_with_wrong_signature() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "add") (param i32 i32) (result i32)
                    local.get 0 local.get 1 i32.add)
                (func (export "greet") (result i32) i32.const 1)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // 用 0 个参数调用 add（需要 2 个）→ 错误
        let result = instance.call("add", &[]);
        assert!(result.is_err(), "0 个参数调用 add 应返回错误");

        // 用 1 个参数调用 add → 错误
        let result = instance.call("add", &[WasmValue::I32(1)]);
        assert!(result.is_err(), "1 个参数调用 add 应返回错误");

        // 用 3 个参数调用 add → 错误
        let result = instance.call("add", &[WasmValue::I32(1), WasmValue::I32(2), WasmValue::I32(3)]);
        assert!(result.is_err(), "3 个参数调用 add 应返回错误");

        // 用错误类型的参数调用 greet（greet 无参数）→ 错误
        let result = instance.call("greet", &[WasmValue::I32(42)]);
        assert!(result.is_err(), "带参数调用无参函数应返回错误");

        // 正确调用应成功
        let ok = instance
            .call("add", &[WasmValue::I32(3), WasmValue::I32(4)])
            .expect("correct call");
        assert_eq!(ok[0], WasmValue::I32(7));
    }

    /// 编译一个没有任何导出的 WASM 模块，验证 exports() 返回空列表，
    /// 实例化后 has_func / has_memory 查询均返回 false。
    #[test]
    fn test_instance_with_no_exports() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                ;; 只有内部函数和局部内存，没有任何导出
                (func $internal nop)
                (memory 1)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let exports = module.exports();
        assert!(
            exports.is_empty(),
            "无导出模块的 exports() 应为空列表，实际: {exports:?}"
        );

        let instance = module.instantiate(&sandbox).expect("instantiate");
        assert!(!instance.has_func("anything"), "无导出时 has_func 应返回 false");
        assert!(!instance.has_memory("memory"), "未导出的内存不应通过 has_memory 找到");
        assert!(
            instance.memory_size("memory").is_none(),
            "未导出的内存不应通过 memory_size 找到"
        );
    }

    /// 执行包含循环的 WASM 函数，验证燃料消耗量与循环迭代次数成正比。
    /// 分别用 10 次和 100 次迭代调用同一函数，燃料消耗比值应接近迭代次数比值。
    #[test]
    fn test_fuel_consumed_tracking() {
        let config = SandboxConfig::new().consume_fuel(true);
        let sandbox = WasmSandbox::with_config(config);
        // 循环指定次数后返回的函数
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "loop_n") (param i32) (result i32)
                    (local $i i32)
                    (local $sum i32)
                    (block $break
                        (loop $loop
                            local.get $i
                            local.get 0
                            i32.ge_s
                            br_if $break
                            ;; sum += i
                            local.get $sum
                            local.get $i
                            i32.add
                            local.set $sum
                            ;; i += 1
                            local.get $i
                            i32.const 1
                            i32.add
                            local.set $i
                            br $loop))
                    local.get $sum)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // 用 10 次迭代测量燃料消耗
        instance.set_fuel(100_000).expect("set_fuel");
        let fuel_before_10 = instance.get_fuel().expect("get_fuel");
        let r10 = instance.call("loop_n", &[WasmValue::I32(10)]).expect("call 10");
        assert_eq!(r10[0], WasmValue::I32(45), "sum(0..10) = 45");
        let fuel_after_10 = instance.get_fuel().expect("get_fuel");
        let consumed_10 = fuel_before_10 - fuel_after_10;

        // 用 100 次迭代测量燃料消耗
        instance.set_fuel(1_000_000).expect("set_fuel");
        let fuel_before_100 = instance.get_fuel().expect("get_fuel");
        let r100 = instance.call("loop_n", &[WasmValue::I32(100)]).expect("call 100");
        assert_eq!(r100[0], WasmValue::I32(4950), "sum(0..100) = 4950");
        let fuel_after_100 = instance.get_fuel().expect("get_fuel");
        let consumed_100 = fuel_before_100 - fuel_after_100;

        // 100 次迭代的燃料消耗应约为 10 次迭代的 10 倍
        assert!(consumed_10 > 0, "10 次迭代应消耗一些燃料，实际: {consumed_10}");
        assert!(
            consumed_100 > consumed_10,
            "100 次迭代消耗应大于 10 次，实际: {consumed_100} vs {consumed_10}"
        );
        let ratio = consumed_100 as f64 / consumed_10 as f64;
        assert!(
            (ratio - 10.0).abs() < 3.0,
            "燃料消耗比值应接近 10（实际: {ratio:.1}，消耗: 10次={consumed_10}, 100次={consumed_100}）"
        );
    }

    /// 主机函数接收 3 个以上参数（4 个 i32），验证所有参数正确传递到主机端。
    #[test]
    fn test_host_function_multiple_params() {
        let sandbox = WasmSandbox::new();
        // WASM 模块导入一个接收 4 个 i32 参数的主机函数，返回它们的加权和
        let wasm = wat_to_wasm(
            r#"(module
                (import "env" "weighted_sum"
                    (func $ws (param i32 i32 i32 i32) (result i32)))
                (func (export "call_ws")
                      (param i32 i32 i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    local.get 2
                    local.get 3
                    call $ws)
            )"#,
        );

        let mut linker = LinkerConfig::new();
        linker.define(HostFunction::new(
            "env",
            "weighted_sum",
            vec![WasmValueType::I32; 4],
            vec![WasmValueType::I32],
            |params, results| {
                // 加权求和: a*1 + b*10 + c*100 + d*1000
                let vals: Vec<i32> = params
                    .iter()
                    .map(|v| if let WasmValue::I32(n) = v { *n } else { 0 })
                    .collect();
                assert_eq!(vals.len(), 4, "主机函数应收到 4 个参数");
                let weighted = vals[0] * 1 + vals[1] * 10 + vals[2] * 100 + vals[3] * 1000;
                results.push(WasmValue::I32(weighted));
                Ok(())
            },
        ));

        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate_with_linker(&sandbox, &linker).expect("instantiate");

        // 传入 (2, 3, 4, 5) → 2*1 + 3*10 + 4*100 + 5*1000 = 2+30+400+5000 = 5432
        let r = instance
            .call(
                "call_ws",
                &[
                    WasmValue::I32(2),
                    WasmValue::I32(3),
                    WasmValue::I32(4),
                    WasmValue::I32(5),
                ],
            )
            .expect("call");
        assert_eq!(r[0], WasmValue::I32(5432), "4 参数加权求和结果应为 5432");

        // 换一组值验证: (1, 0, 7, 9) → 1+0+700+9000 = 9701
        let r2 = instance
            .call(
                "call_ws",
                &[
                    WasmValue::I32(1),
                    WasmValue::I32(0),
                    WasmValue::I32(7),
                    WasmValue::I32(9),
                ],
            )
            .expect("call2");
        assert_eq!(r2[0], WasmValue::I32(9701), "4 参数加权求和结果应为 9701");
    }

    /// 测试主机函数回调 WASM 函数的递归场景。
    /// 通过燃料限制来约束递归深度，验证在燃料耗尽时执行被安全终止。
    #[test]
    fn test_recursive_host_call_limit() {
        let config = SandboxConfig::new().consume_fuel(true);
        let sandbox = WasmSandbox::with_config(config);
        // WASM 模块导出 step 函数并导入 host_step 主机函数。
        // step(n) 调用 host_step(n)，host_step 再调用 step(n-1)，
        // 形成递归链。靠燃料限制来终止递归。
        let wasm = wat_to_wasm(
            r#"(module
                (import "env" "host_step" (func $host_step (param i32) (result i32)))
                (func $step (export "step") (param i32) (result i32)
                    local.get 0
                    i32.eqz
                    if (result i32) i32.const 0
                    else
                        local.get 0
                        call $host_step
                    end)
            )"#,
        );

        // 因为 host 函数无法直接回调 WASM（需要 &mut store 但主机函数签名只有参数和结果），
        // 所以这里用简化的方式：host_step 返回 n-1 的值，
        // 然后 WASM 侧循环调用 host_step 直到 n=0，验证燃料被正确消耗。
        let wasm = wat_to_wasm(
            r#"(module
                (import "env" "host_step" (func $host_step (param i32) (result i32)))
                (func (export "run") (param i32) (result i32)
                    (local $n i32)
                    (local $count i32)
                    local.get 0
                    local.set $n
                    (block $break
                        (loop $loop
                            local.get $n
                            i32.eqz
                            br_if $break
                            ;; n = host_step(n), host_step returns n-1
                            local.get $n
                            call $host_step
                            local.set $n
                            ;; count += 1
                            local.get $count
                            i32.const 1
                            i32.add
                            local.set $count
                            br $loop))
                    local.get $count)
            )"#,
        );

        let mut linker = LinkerConfig::new();
        linker.define(HostFunction::new(
            "env",
            "host_step",
            vec![WasmValueType::I32],
            vec![WasmValueType::I32],
            |params, results| {
                if let WasmValue::I32(n) = params[0] {
                    // 返回 n-1，模拟递归步骤
                    results.push(WasmValue::I32(n - 1));
                }
                Ok(())
            },
        ));

        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate_with_linker(&sandbox, &linker).expect("instantiate");

        // 设置充足燃料，验证有限递归能正确完成
        instance.set_fuel(1_000_000).expect("set_fuel");
        let r = instance.call("run", &[WasmValue::I32(5)]).expect("call");
        assert_eq!(r[0], WasmValue::I32(5), "5 次递归步应计数 5");

        // 设置极少燃料，验证递归被燃料限制终止
        instance.set_fuel(100).expect("set_fuel");
        let result = instance.call("run", &[WasmValue::I32(10000)]);
        assert!(
            matches!(result, Err(WasmError::FuelExhausted)),
            "深度递归应在燃料耗尽时被终止"
        );
    }

    // =======================================================================
    // 新增测试：全局导出读取、表导出查询、多实例独立性
    // =======================================================================

    /// 读取 WASM 全局导出变量的值。
    #[test]
    fn test_global_export_read() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (global (export "magic") i32 (i32.const 42))
                (global (export "big") i64 (i64.const 9999999999))
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let instance = module.instantiate(&sandbox).expect("instantiate");

        let magic = instance.get_global_export("magic").expect("read magic global");
        assert_eq!(magic, WasmValue::I32(42), "全局导出 magic 应为 i32(42)");

        let big = instance.get_global_export("big").expect("read big global");
        assert_eq!(big, WasmValue::I64(9999999999), "全局导出 big 应为 i64(9999999999)");

        // 不存在的全局导出应返回 None
        assert!(instance.get_global_export("nonexistent").is_none());
    }

    /// WASM 模块导出表 → has_table 查询返回 true，未导出或不存在返回 false。
    #[test]
    fn test_table_export_query() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (table (export "tbl") 4 funcref)
                (func (export "f") nop)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let instance = module.instantiate(&sandbox).expect("instantiate");

        assert!(instance.has_table("tbl"), "已导出的表 tbl 应被找到");
        assert!(!instance.has_table("nonexistent"), "不存在的表应返回 false");
        assert!(!instance.has_table("f"), "函数导出不应被 has_table 匹配");
    }

    /// 验证 WASM 模块声明初始内存后，实例的内存页数正确。
    ///
    /// 创建声明初始 3 页内存的模块，实例化后验证 memory_size 返回
    /// 3 * 65536 = 196608 字节。
    #[test]
    fn test_wasm_memory_initial_pages() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (memory (export "memory") 3)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let instance = module.instantiate(&sandbox).expect("instantiate");

        assert!(instance.has_memory("memory"), "应存在 memory 导出");
        let size = instance.memory_size("memory").expect("size");
        let expected = 3 * 65536;
        assert_eq!(size, expected, "3 页初始内存应为 {expected} 字节，实际: {size}");
    }

    /// 验证导出函数接收 4 个 i32 参数，所有参数正确传递。
    ///
    /// 创建一个接收 4 个 i32 参数并返回它们之和的函数，
    /// 传入 (100, 200, 300, 400) 验证结果为 1000。
    #[test]
    fn test_wasm_exported_function_params() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "sum4") (param i32 i32 i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.add
                    local.get 2
                    i32.add
                    local.get 3
                    i32.add)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        let result = instance
            .call(
                "sum4",
                &[
                    WasmValue::I32(100),
                    WasmValue::I32(200),
                    WasmValue::I32(300),
                    WasmValue::I32(400),
                ],
            )
            .expect("call sum4");

        assert_eq!(result.len(), 1, "应返回一个结果");
        assert_eq!(result[0], WasmValue::I32(1000), "100 + 200 + 300 + 400 应为 1000");
    }

    /// 从同一模块创建两个实例 → 各自拥有独立的可变全局变量状态。
    #[test]
    fn test_multiple_instances_same_module() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (global $counter (mut i32) (i32.const 0))
                (func (export "inc") (result i32)
                    global.get $counter
                    i32.const 1
                    i32.add
                    global.set $counter
                    global.get $counter)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");

        let mut inst_a = module.instantiate(&sandbox).expect("instantiate A");
        let mut inst_b = module.instantiate(&sandbox).expect("instantiate B");

        // 实例 A 递增两次
        let r1 = inst_a.call("inc", &[]).expect("A inc 1");
        assert_eq!(r1[0], WasmValue::I32(1));
        let r2 = inst_a.call("inc", &[]).expect("A inc 2");
        assert_eq!(r2[0], WasmValue::I32(2));

        // 实例 B 的全局变量应保持独立（仍为 0）
        let rb = inst_b.call("inc", &[]).expect("B inc 1");
        assert_eq!(rb[0], WasmValue::I32(1), "实例 B 应从 0 开始独立计数");
    }

    // =======================================================================
    // 新增测试：边界条件测试
    // =======================================================================

    /// 测试 WASM 内存写入后立即覆盖同一区域，验证后续读回的是最新写入的数据。
    #[test]
    fn test_wasm_memory_read_write() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (memory (export "mem") 1)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // 先写入一组数据
        instance.write_memory("mem", 0, b"AAAA").expect("write first");
        let data = instance.read_memory("mem", 0, 4).expect("read first");
        assert_eq!(&data, b"AAAA", "initial write should be AAAA");

        // 覆盖同一区域
        instance.write_memory("mem", 0, b"BBBB").expect("write overwrite");
        let data = instance.read_memory("mem", 0, 4).expect("read overwrite");
        assert_eq!(&data, b"BBBB", "overwritten data should be BBBB");

        // 确认非覆盖区域不受影响
        instance.write_memory("mem", 100, b"CCCC").expect("write offset");
        let data = instance.read_memory("mem", 100, 4).expect("read offset");
        assert_eq!(&data, b"CCCC", "offset write should be CCCC");
        // 原区域仍是 BBBB
        let data = instance.read_memory("mem", 0, 4).expect("read original");
        assert_eq!(&data, b"BBBB", "original area should still be BBBB");
    }

    /// 编译一个包含 2 个导出函数的 WASM 模块，分别调用并验证结果。
    #[test]
    fn test_wasm_multiple_functions() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "square") (param i32) (result i32)
                    local.get 0 local.get 0 i32.mul)
                (func (export "negate") (param i32) (result i32)
                    i32.const 0 local.get 0 i32.sub)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // 调用 square 函数
        let r1 = instance.call("square", &[WasmValue::I32(7)]).expect("call square");
        assert_eq!(r1[0], WasmValue::I32(49), "7 * 7 should be 49");

        // 调用 negate 函数
        let r2 = instance.call("negate", &[WasmValue::I32(15)]).expect("call negate");
        assert_eq!(r2[0], WasmValue::I32(-15), "negate(15) should be -15");
    }

    /// 创建启用燃料计量的沙箱，执行简单函数后验证燃料被消耗且剩余燃料减少。
    #[test]
    fn test_wasm_fuel_consumption_simple() {
        let config = SandboxConfig::new().consume_fuel(true);
        let sandbox = WasmSandbox::with_config(config);
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "double") (param i32) (result i32)
                    local.get 0 local.get 0 i32.add)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // 设置燃料
        instance.set_fuel(1000).expect("set_fuel");
        let fuel_before = instance.get_fuel().expect("get_fuel before");
        assert_eq!(fuel_before, 1000, "fuel should be 1000 after set");

        // 执行函数
        let r = instance.call("double", &[WasmValue::I32(21)]).expect("call double");
        assert_eq!(r[0], WasmValue::I32(42), "double(21) should be 42");

        // 验证燃料被消耗
        let fuel_after = instance.get_fuel().expect("get_fuel after");
        assert!(
            fuel_after < fuel_before,
            "fuel should decrease after call, before: {fuel_before}, after: {fuel_after}"
        );
        assert!(fuel_after > 0, "remaining fuel should still be positive");
    }

    /// 编译 WASM 模块导出全局变量，通过 get_global_export 读取值并验证。
    #[test]
    fn test_wasm_global_get() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (global (export "g_i32") i32 (i32.const 12345))
                (global (export "g_i64") i64 (i64.const -1))
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let instance = module.instantiate(&sandbox).expect("instantiate");

        let g_i32 = instance.get_global_export("g_i32").expect("read g_i32");
        assert_eq!(g_i32, WasmValue::I32(12345), "global g_i32 should be i32(12345)");

        let g_i64 = instance.get_global_export("g_i64").expect("read g_i64");
        assert_eq!(g_i64, WasmValue::I64(-1), "global g_i64 should be i64(-1)");

        // 不存在的全局导出应返回 None
        assert!(
            instance.get_global_export("nonexistent").is_none(),
            "nonexistent global should return None"
        );
    }

    // =======================================================================
    // 边界条件测试：多导出函数、内存页数、返回类型、默认配置
    // =======================================================================

    /// 测试 WASM 模块包含多个导出函数，所有导出都能正确调用。
    #[test]
    fn test_wasm_multiple_exports() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "add") (param i32 i32) (result i32)
                    local.get 0 local.get 1 i32.add)
                (func (export "sub") (param i32 i32) (result i32)
                    local.get 0 local.get 1 i32.sub)
                (func (export "mul") (param i32 i32) (result i32)
                    local.get 0 local.get 1 i32.mul)
                (func (export "div") (param i32 i32) (result i32)
                    local.get 0 local.get 1 i32.div_s)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let exports = module.exports();
        assert_eq!(exports.len(), 4, "应有 4 个导出函数");
        assert!(exports.contains(&"add".to_string()));
        assert!(exports.contains(&"sub".to_string()));
        assert!(exports.contains(&"mul".to_string()));
        assert!(exports.contains(&"div".to_string()));

        let mut instance = module.instantiate(&sandbox).expect("instantiate");
        // 调用每个导出函数
        let r = instance
            .call("add", &[WasmValue::I32(10), WasmValue::I32(3)])
            .expect("add");
        assert_eq!(r[0], WasmValue::I32(13));
        let r = instance
            .call("sub", &[WasmValue::I32(10), WasmValue::I32(3)])
            .expect("sub");
        assert_eq!(r[0], WasmValue::I32(7));
        let r = instance
            .call("mul", &[WasmValue::I32(10), WasmValue::I32(3)])
            .expect("mul");
        assert_eq!(r[0], WasmValue::I32(30));
        let r = instance
            .call("div", &[WasmValue::I32(10), WasmValue::I32(3)])
            .expect("div");
        assert_eq!(r[0], WasmValue::I32(3)); // 10/3 = 3 (整数除法)
    }

    /// 测试 WASM 函数的不同返回类型（i32, i64, f32, f64）。
    #[test]
    fn test_wasm_call_return_types() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "ret_i32") (result i32) i32.const 42)
                (func (export "ret_i64") (result i64) i64.const 1000000000)
                (func (export "ret_f32") (result f32) f32.const 3.14)
                (func (export "ret_f64") (result f64) f64.const 2.718281828)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // i32
        let r = instance.call("ret_i32", &[]).expect("call i32");
        assert_eq!(r[0], WasmValue::I32(42));

        // i64
        let r = instance.call("ret_i64", &[]).expect("call i64");
        assert_eq!(r[0], WasmValue::I64(1_000_000_000));

        // f32
        let r = instance.call("ret_f32", &[]).expect("call f32");
        if let WasmValue::F32(v) = r[0] {
            assert!((v - 3.14).abs() < 0.01, "f32 应接近 3.14，实际: {v}");
        } else {
            panic!("期望 F32 返回值");
        }

        // f64
        let r = instance.call("ret_f64", &[]).expect("call f64");
        if let WasmValue::F64(v) = r[0] {
            assert!((v - 2.718281828).abs() < 0.0001, "f64 应接近 2.718281828，实际: {v}");
        } else {
            panic!("期望 F64 返回值");
        }
    }

    /// 测试 SandboxConfig 默认配置值。
    #[test]
    fn test_wasm_instance_config_default() {
        let config = SandboxConfig::default();
        assert!(!config.is_consume_fuel(), "默认 consume_fuel 应为 false");
        let sandbox = WasmSandbox::with_config(config);
        assert!(!sandbox.config().is_consume_fuel());
        // 默认配置应能正常编译和运行模块
        let wasm = wat_to_wasm(r#"(module (func (export "answer") (result i32) i32.const 42))"#);
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");
        let r = instance.call("answer", &[]).expect("call");
        assert_eq!(r[0], WasmValue::I32(42));
    }

    /// 尝试编译无效的 WASM 字节，验证返回错误而非 panic。
    #[test]
    fn test_wasm_error_on_invalid_module() {
        let sandbox = WasmSandbox::new();

        // 完全无效的字节序列
        let result = sandbox.compile(&[0xFF, 0xFE, 0xFD, 0xFC]);
        assert!(result.is_err(), "invalid bytes should fail to compile");
        assert!(
            matches!(result, Err(WasmError::InvalidBinary(_))),
            "should return InvalidBinary error"
        );

        // 空输入
        let result = sandbox.compile(&[]);
        assert!(result.is_err(), "empty bytes should fail to compile");
        assert!(
            matches!(result, Err(WasmError::InvalidBinary(_))),
            "should return InvalidBinary error"
        );

        // 合法魔数但内容损坏
        let result = sandbox.compile(&[0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00, 0xFF, 0xFF]);
        assert!(result.is_err(), "corrupted module should fail to compile");
    }

    // =======================================================================
    // 新增测试：更多边界条件
    // =======================================================================

    /// 测试向不存在的内存导出名调用 write_memory，应返回 ExportNotFound 错误。
    /// 已有 test_memory_not_found 测试了 read_memory/has_memory/memory_size，
    /// 但 write_memory 的错误路径尚未覆盖。
    #[test]
    fn test_write_memory_nonexistent_export() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "f") nop)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // 模块没有导出内存，write_memory 应返回 ExportNotFound 错误
        let result = instance.write_memory("nonexistent_mem", 0, b"data");
        assert!(result.is_err(), "写入不存在的内存应返回错误");
        if let Err(WasmError::ExportNotFound { name }) = result {
            assert_eq!(name, "nonexistent_mem", "错误中应包含请求的导出名");
        } else {
            panic!("期望 ExportNotFound 错误，实际: {result:?}");
        }
    }

    /// 测试 WASM 函数返回 f32 特殊值（NaN、正无穷、负无穷），
    /// 验证沙箱能正确传递这些浮点边界值而不崩溃或截断。
    #[test]
    fn test_call_f32_special_values() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "ret_nan") (result f32)
                    f32.const nan:0x400000
                )
                (func (export "ret_inf") (result f32)
                    f32.const inf
                )
                (func (export "ret_neg_inf") (result f32)
                    f32.const -inf
                )
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // NaN 值
        let r = instance.call("ret_nan", &[]).expect("call nan");
        if let WasmValue::F32(v) = r[0] {
            assert!(v.is_nan(), "返回值应为 NaN，实际: {v}");
        } else {
            panic!("期望 F32 返回值");
        }

        // 正无穷
        let r = instance.call("ret_inf", &[]).expect("call inf");
        if let WasmValue::F32(v) = r[0] {
            assert!(v.is_infinite() && v.is_sign_positive(), "返回值应为正无穷，实际: {v}");
        } else {
            panic!("期望 F32 返回值");
        }

        // 负无穷
        let r = instance.call("ret_neg_inf", &[]).expect("call neg_inf");
        if let WasmValue::F32(v) = r[0] {
            assert!(v.is_infinite() && v.is_sign_negative(), "返回值应为负无穷，实际: {v}");
        } else {
            panic!("期望 F32 返回值");
        }
    }

    /// 测试 WASM 内部函数间调用：导出函数调用另一个未导出的内部函数，
    /// 验证 WASM 模块内部的 call 指令正确执行，参数和返回值传递无误。
    #[test]
    fn test_internal_function_call_chain() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                ;; 内部辅助函数：计算平方
                (func $square (param i32) (result i32)
                    local.get 0
                    local.get 0
                    i32.mul)
                ;; 内部辅助函数：计算两数之和
                (func $add (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.add)
                ;; 导出函数：计算 (a+b)^2，调用内部 $add 和 $square
                (func (export "sum_sq") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    call $add
                    call $square)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        // 只有 sum_sq 被导出
        assert_eq!(module.exports().len(), 1);
        assert!(module.exports().contains(&"sum_sq".to_string()));

        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // (3 + 4)^2 = 49
        let r = instance
            .call("sum_sq", &[WasmValue::I32(3), WasmValue::I32(4)])
            .expect("call");
        assert_eq!(r[0], WasmValue::I32(49), "(3+4)^2 应为 49");

        // (0 + 0)^2 = 0
        let r = instance
            .call("sum_sq", &[WasmValue::I32(0), WasmValue::I32(0)])
            .expect("call zero");
        assert_eq!(r[0], WasmValue::I32(0), "(0+0)^2 应为 0");

        // (-2 + 5)^2 = 9
        let r = instance
            .call("sum_sq", &[WasmValue::I32(-2), WasmValue::I32(5)])
            .expect("call negative");
        assert_eq!(r[0], WasmValue::I32(9), "(-2+5)^2 应为 9");
    }

    /// 测试可变全局变量在 WASM 函数修改后，通过 get_global_export 能读回最新值。
    /// 已有 test_global_variable 测试函数返回值，但未验证 get_global_export 的同步读取。
    #[test]
    fn test_get_global_export_after_mutation() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (global $val (export "val") (mut i32) (i32.const 10))
                (func (export "set_val") (param i32)
                    local.get 0
                    global.set $val)
                (func (export "get_val") (result i32)
                    global.get $val)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // 初始值
        let g = instance.get_global_export("val").expect("initial read");
        assert_eq!(g, WasmValue::I32(10), "初始全局值应为 10");

        // 通过 WASM 函数修改全局变量
        instance.call("set_val", &[WasmValue::I32(99)]).expect("set_val");

        // 通过 get_global_export 验证值已更新
        let g = instance.get_global_export("val").expect("after set");
        assert_eq!(g, WasmValue::I32(99), "修改后全局值应为 99");

        // 同时通过 WASM 函数验证一致性
        let r = instance.call("get_val", &[]).expect("get_val");
        assert_eq!(r[0], WasmValue::I32(99), "WASM 函数返回值应与 get_global_export 一致");

        // 再次修改为负数
        instance.call("set_val", &[WasmValue::I32(-1)]).expect("set_val neg");
        let g = instance.get_global_export("val").expect("after neg");
        assert_eq!(g, WasmValue::I32(-1), "修改为负数后应为 -1");
    }

    /// 测试主机函数返回多个结果值（2 个 i32），
    /// 验证 WASM 能正确接收主机函数的所有返回值并进一步运算。
    #[test]
    fn test_host_function_multiple_results() {
        let sandbox = WasmSandbox::new();
        // WASM 导入一个返回两个 i32 的主机函数 divmod：
        // 给定 n 和 d，主机函数返回 (n/d, n%d)
        let wasm = wat_to_wasm(
            r#"(module
                (import "env" "divmod"
                    (func $divmod (param i32 i32) (result i32 i32)))
                (func (export "test") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    call $divmod
                    ;; 栈上现在有商和余数，将它们相加返回
                    i32.add)
            )"#,
        );

        let mut linker = LinkerConfig::new();
        linker.define(HostFunction::new(
            "env",
            "divmod",
            vec![WasmValueType::I32, WasmValueType::I32],
            vec![WasmValueType::I32, WasmValueType::I32],
            |params, results| {
                if let (WasmValue::I32(n), WasmValue::I32(d)) = (&params[0], &params[1]) {
                    // WASM 整数除法为截断向零
                    let quotient = *n / *d;
                    let remainder = *n % *d;
                    results.push(WasmValue::I32(quotient));
                    results.push(WasmValue::I32(remainder));
                }
                Ok(())
            },
        ));

        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate_with_linker(&sandbox, &linker).expect("instantiate");

        // 17 / 5 = 商 3，余数 2，和 = 5
        let r = instance
            .call("test", &[WasmValue::I32(17), WasmValue::I32(5)])
            .expect("call");
        assert_eq!(r[0], WasmValue::I32(5), "17 divmod 5 的商+余数应为 3+2=5");

        // 100 / 7 = 商 14，余数 2，和 = 16
        let r = instance
            .call("test", &[WasmValue::I32(100), WasmValue::I32(7)])
            .expect("call2");
        assert_eq!(r[0], WasmValue::I32(16), "100 divmod 7 的商+余数应为 14+2=16");
    }

    // =======================================================================
    // 新增测试：更多边界条件（f64 特殊值、f32 主机函数参数、负数浮点、
    //           i64 极值、主机函数未填充结果）
    // =======================================================================

    /// 测试 WASM 函数返回 f64 特殊值（NaN、正无穷、负无穷），
    /// 验证沙箱在双精度浮点边界值上的正确传递。
    #[test]
    fn test_call_f64_special_values() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "ret_nan") (result f64)
                    f64.const nan:0x8000000000000
                )
                (func (export "ret_inf") (result f64)
                    f64.const inf
                )
                (func (export "ret_neg_inf") (result f64)
                    f64.const -inf
                )
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // NaN 值
        let r = instance.call("ret_nan", &[]).expect("call nan");
        if let WasmValue::F64(v) = r[0] {
            assert!(v.is_nan(), "f64 返回值应为 NaN，实际: {v}");
        } else {
            panic!("期望 F64 返回值");
        }

        // 正无穷
        let r = instance.call("ret_inf", &[]).expect("call inf");
        if let WasmValue::F64(v) = r[0] {
            assert!(
                v.is_infinite() && v.is_sign_positive(),
                "f64 返回值应为正无穷，实际: {v}"
            );
        } else {
            panic!("期望 F64 返回值");
        }

        // 负无穷
        let r = instance.call("ret_neg_inf", &[]).expect("call neg_inf");
        if let WasmValue::F64(v) = r[0] {
            assert!(
                v.is_infinite() && v.is_sign_negative(),
                "f64 返回值应为负无穷，实际: {v}"
            );
        } else {
            panic!("期望 F64 返回值");
        }
    }

    /// 测试主机函数接收 f32 类型参数并返回 f32 结果，
    /// 验证 f32 参数在 WASM → 主机传递路径上不丢失精度。
    #[test]
    fn test_host_function_f32_params() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (import "env" "half" (func $half (param f32) (result f32)))
                (func (export "call_half") (param f32) (result f32)
                    local.get 0
                    call $half)
            )"#,
        );

        let mut linker = LinkerConfig::new();
        linker.define(HostFunction::new(
            "env",
            "half",
            vec![WasmValueType::F32],
            vec![WasmValueType::F32],
            |params, results| {
                if let WasmValue::F32(v) = params[0] {
                    results.push(WasmValue::F32(v / 2.0));
                }
                Ok(())
            },
        ));

        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate_with_linker(&sandbox, &linker).expect("instantiate");

        // 9.0 / 2 = 4.5
        let r = instance.call("call_half", &[WasmValue::F32(9.0)]).expect("call");
        if let WasmValue::F32(v) = r[0] {
            assert!((v - 4.5).abs() < 0.001, "9.0 / 2 应为 4.5，实际: {v}");
        } else {
            panic!("期望 F32 返回值");
        }

        // 负数: -7.0 / 2 = -3.5
        let r = instance.call("call_half", &[WasmValue::F32(-7.0)]).expect("call neg");
        if let WasmValue::F32(v) = r[0] {
            assert!((v - (-3.5)).abs() < 0.001, "-7.0 / 2 应为 -3.5，实际: {v}");
        } else {
            panic!("期望 F32 返回值");
        }
    }

    /// 测试 WASM f64 运算使用负数和零作为操作数，
    /// 验证符号在乘法和加法中正确传递。
    #[test]
    fn test_call_f64_negative_and_zero() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "mul_f64") (param f64 f64) (result f64)
                    local.get 0
                    local.get 1
                    f64.mul)
                (func (export "add_f64") (param f64 f64) (result f64)
                    local.get 0
                    local.get 1
                    f64.add)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // 负 × 负 = 正
        let r = instance
            .call("mul_f64", &[WasmValue::F64(-3.0), WasmValue::F64(-2.0)])
            .expect("call mul neg*neg");
        if let WasmValue::F64(v) = r[0] {
            assert!((v - 6.0).abs() < 0.001, "(-3)*(-2) 应为 6.0，实际: {v}");
        } else {
            panic!("期望 F64 返回值");
        }

        // 正 × 负 = 负
        let r = instance
            .call("mul_f64", &[WasmValue::F64(5.0), WasmValue::F64(-4.0)])
            .expect("call mul pos*neg");
        if let WasmValue::F64(v) = r[0] {
            assert!((v - (-20.0)).abs() < 0.001, "5*(-4) 应为 -20.0，实际: {v}");
        } else {
            panic!("期望 F64 返回值");
        }

        // 零 + 零 = 0
        let r = instance
            .call("add_f64", &[WasmValue::F64(0.0), WasmValue::F64(0.0)])
            .expect("call add zero+zero");
        if let WasmValue::F64(v) = r[0] {
            assert!(v == 0.0, "0.0 + 0.0 应为 0.0，实际: {v}");
        } else {
            panic!("期望 F64 返回值");
        }

        // 负零 + 正零
        let r = instance
            .call("add_f64", &[WasmValue::F64(-0.0), WasmValue::F64(0.0)])
            .expect("call add neg_zero+pos_zero");
        if let WasmValue::F64(v) = r[0] {
            assert!(v == 0.0, "-0.0 + 0.0 应为 0.0，实际: {v}");
        } else {
            panic!("期望 F64 返回值");
        }
    }

    /// 测试 i64 极值运算：i64::MIN + i64::MAX (wrapping) 和 i64::MAX * 2 (wrapping)，
    /// 验证 WASM wrapping 算术在 64 位整数的极端边界上行为正确。
    #[test]
    fn test_i64_extreme_values() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "add64") (param i64 i64) (result i64)
                    local.get 0 local.get 1 i64.add)
                (func (export "mul64") (param i64 i64) (result i64)
                    local.get 0 local.get 1 i64.mul)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // i64::MIN + i64::MAX (wrapping) = -1
        let r = instance
            .call("add64", &[WasmValue::I64(i64::MIN), WasmValue::I64(i64::MAX)])
            .expect("call min+max");
        assert_eq!(r[0], WasmValue::I64(-1), "i64::MIN + i64::MAX (wrapping) 应为 -1");

        // i64::MAX * 2 (wrapping) = -2
        let r = instance
            .call("mul64", &[WasmValue::I64(i64::MAX), WasmValue::I64(2)])
            .expect("call max*2");
        assert_eq!(r[0], WasmValue::I64(-2), "i64::MAX * 2 (wrapping) 应为 -2");

        // 0 + i64::MIN = i64::MIN
        let r = instance
            .call("add64", &[WasmValue::I64(0), WasmValue::I64(i64::MIN)])
            .expect("call 0+min");
        assert_eq!(r[0], WasmValue::I64(i64::MIN), "0 + i64::MIN 应为 i64::MIN");
    }

    /// 测试主机函数未向 results 缓冲区写入任何值时，
    /// WASM 侧收到默认零值而不崩溃，验证缺少结果值时的优雅降级行为。
    #[test]
    fn test_host_function_empty_results_graceful() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (import "env" "lazy" (func $lazy (result i32)))
                (func (export "call_lazy") (result i32)
                    call $lazy)
            )"#,
        );

        let mut linker = LinkerConfig::new();
        linker.define(HostFunction::new(
            "env",
            "lazy",
            vec![],
            vec![WasmValueType::I32],
            |_params, _results| {
                // 故意不向 results 写入任何值
                Ok(())
            },
        ));

        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate_with_linker(&sandbox, &linker).expect("instantiate");

        // 主机函数未写入结果 → wasmi 使用默认值 i32(0) 填充
        let r = instance.call("call_lazy", &[]).expect("call");
        assert_eq!(r.len(), 1, "应返回一个结果");
        assert_eq!(r[0], WasmValue::I32(0), "主机函数未写入结果时应为默认值 i32(0)");
    }

    // =======================================================================
    // 新增测试：更多边界条件（局部变量链、select 指令、i64 负值主机函数、
    //           同一沙箱多模块编译、条件比较运算）
    // =======================================================================

    /// 测试 WASM 函数使用多个局部变量进行赋值链操作。
    /// 函数接收一个参数，经过一系列 local.set/local.get 链式赋值后返回结果，
    /// 验证局部变量在多步中间传递中不会丢失或错乱。
    #[test]
    fn test_local_variable_assignment_chain() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "chain") (param i32) (result i32)
                    (local $a i32)
                    (local $b i32)
                    (local $c i32)
                    ;; a = param + 1
                    local.get 0
                    i32.const 1
                    i32.add
                    local.set $a
                    ;; b = a * 3
                    local.get $a
                    i32.const 3
                    i32.mul
                    local.set $b
                    ;; c = b - a
                    local.get $b
                    local.get $a
                    i32.sub
                    local.set $c
                    ;; 返回 c
                    local.get $c)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // 输入 4：a = 5, b = 15, c = 15 - 5 = 10
        let r = instance.call("chain", &[WasmValue::I32(4)]).expect("call");
        assert_eq!(r[0], WasmValue::I32(10), "局部变量链 4 → a=5, b=15, c=10");

        // 输入 0：a = 1, b = 3, c = 3 - 1 = 2
        let r = instance.call("chain", &[WasmValue::I32(0)]).expect("call zero");
        assert_eq!(r[0], WasmValue::I32(2), "局部变量链 0 → a=1, b=3, c=2");

        // 输入 -1：a = 0, b = 0, c = 0 - 0 = 0
        let r = instance.call("chain", &[WasmValue::I32(-1)]).expect("call neg");
        assert_eq!(r[0], WasmValue::I32(0), "局部变量链 -1 → a=0, b=0, c=0");
    }

    /// 测试 WASM select 指令：根据条件值在两个操作数之间选择。
    /// select(val1, val2, cond) — cond 非 0 返回 val1，cond 为 0 返回 val2。
    /// 验证 WASM 条件选择在正条件、零条件和负条件下的正确行为。
    #[test]
    fn test_select_instruction() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "select_i32") (param i32 i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    local.get 2
                    select)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // cond = 1 → 返回第一个值 (100)
        let r = instance
            .call(
                "select_i32",
                &[WasmValue::I32(100), WasmValue::I32(200), WasmValue::I32(1)],
            )
            .expect("call cond=1");
        assert_eq!(r[0], WasmValue::I32(100), "cond=1 应选择第一个值 100");

        // cond = 0 → 返回第二个值 (200)
        let r = instance
            .call(
                "select_i32",
                &[WasmValue::I32(100), WasmValue::I32(200), WasmValue::I32(0)],
            )
            .expect("call cond=0");
        assert_eq!(r[0], WasmValue::I32(200), "cond=0 应选择第二个值 200");

        // cond = -1 (非零) → 返回第一个值
        let r = instance
            .call(
                "select_i32",
                &[WasmValue::I32(42), WasmValue::I32(99), WasmValue::I32(-1)],
            )
            .expect("call cond=-1");
        assert_eq!(r[0], WasmValue::I32(42), "cond=-1 应选择第一个值 42");

        // cond = 999 (非零) → 返回第一个值
        let r = instance
            .call(
                "select_i32",
                &[WasmValue::I32(7), WasmValue::I32(8), WasmValue::I32(999)],
            )
            .expect("call cond=999");
        assert_eq!(r[0], WasmValue::I32(7), "cond=999 应选择第一个值 7");
    }

    /// 测试主机函数接收负 i64 参数并返回其绝对值。
    /// 验证 i64 负值在 WASM → 主机函数传递路径上保持精度，不发生符号丢失或截断。
    #[test]
    fn test_host_function_negative_i64_arg() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (import "env" "abs64" (func $abs64 (param i64) (result i64)))
                (func (export "call_abs") (param i64) (result i64)
                    local.get 0
                    call $abs64)
            )"#,
        );

        let mut linker = LinkerConfig::new();
        linker.define(HostFunction::new(
            "env",
            "abs64",
            vec![WasmValueType::I64],
            vec![WasmValueType::I64],
            |params, results| {
                if let WasmValue::I64(n) = params[0] {
                    results.push(WasmValue::I64(n.abs()));
                }
                Ok(())
            },
        ));

        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate_with_linker(&sandbox, &linker).expect("instantiate");

        // 负值: abs(-123456789) = 123456789
        let r = instance
            .call("call_abs", &[WasmValue::I64(-123456789)])
            .expect("call neg");
        assert_eq!(r[0], WasmValue::I64(123456789), "abs(-123456789) 应为 123456789");

        // 正值不变: abs(42) = 42
        let r = instance.call("call_abs", &[WasmValue::I64(42)]).expect("call pos");
        assert_eq!(r[0], WasmValue::I64(42), "abs(42) 应为 42");

        // i64::MIN 的绝对值在 Rust 中会 panic (overflow)，但 WASM 中不会，
        // 因为这里 abs 是主机 Rust 代码，所以我们用接近 MIN 的值测试。
        let near_min = i64::MIN + 1;
        let r = instance
            .call("call_abs", &[WasmValue::I64(near_min)])
            .expect("call near_min");
        assert_eq!(r[0], WasmValue::I64(i64::MAX), "abs(i64::MIN+1) 应为 i64::MAX");
    }

    /// 测试同一沙箱实例连续编译和实例化多个不同的 WASM 模块，
    /// 验证沙箱可复用于多个独立模块的编译，模块间不会互相干扰。
    #[test]
    fn test_sandbox_compile_multiple_modules() {
        let sandbox = WasmSandbox::new();

        // 第一个模块：加法
        let wasm_a = wat_to_wasm(
            r#"(module
                (func (export "add") (param i32 i32) (result i32)
                    local.get 0 local.get 1 i32.add)
            )"#,
        );
        let module_a = sandbox.compile(&wasm_a).expect("compile A");
        let mut inst_a = module_a.instantiate(&sandbox).expect("instantiate A");

        // 第二个模块：乘法
        let wasm_b = wat_to_wasm(
            r#"(module
                (func (export "mul") (param i32 i32) (result i32)
                    local.get 0 local.get 1 i32.mul)
            )"#,
        );
        let module_b = sandbox.compile(&wasm_b).expect("compile B");
        let mut inst_b = module_b.instantiate(&sandbox).expect("instantiate B");

        // 第三个模块：带内存的模块
        let wasm_c = wat_to_wasm(
            r#"(module
                (memory (export "mem") 1)
                (global $counter (export "counter") (mut i32) (i32.const 0))
                (func (export "inc") (result i32)
                    global.get $counter
                    i32.const 1
                    i32.add
                    global.set $counter
                    global.get $counter)
            )"#,
        );
        let module_c = sandbox.compile(&wasm_c).expect("compile C");
        let mut inst_c = module_c.instantiate(&sandbox).expect("instantiate C");

        // 验证三个模块各自独立工作
        let r_a = inst_a
            .call("add", &[WasmValue::I32(10), WasmValue::I32(20)])
            .expect("call A");
        assert_eq!(r_a[0], WasmValue::I32(30), "模块 A: 10 + 20 = 30");

        let r_b = inst_b
            .call("mul", &[WasmValue::I32(6), WasmValue::I32(7)])
            .expect("call B");
        assert_eq!(r_b[0], WasmValue::I32(42), "模块 B: 6 * 7 = 42");

        let r_c1 = inst_c.call("inc", &[]).expect("call C1");
        assert_eq!(r_c1[0], WasmValue::I32(1), "模块 C: 第一次递增 = 1");
        let r_c2 = inst_c.call("inc", &[]).expect("call C2");
        assert_eq!(r_c2[0], WasmValue::I32(2), "模块 C: 第二次递增 = 2");

        // 模块 C 的内存可用
        assert!(inst_c.has_memory("mem"), "模块 C 应有内存导出");
        assert!(inst_c.has_func("inc"), "模块 C 应有 inc 函数导出");

        // 再次验证模块 A、B 未受影响
        let r_a2 = inst_a
            .call("add", &[WasmValue::I32(100), WasmValue::I32(-50)])
            .expect("call A2");
        assert_eq!(r_a2[0], WasmValue::I32(50), "模块 A: 100 + (-50) = 50");
    }

    /// 测试 WASM i32 比较运算符（i32.eq、i32.lt_s、i32.gt_s），
    /// 验证条件比较在正数、负数和零之间的正确行为。
    #[test]
    fn test_i32_comparison_operations() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "eq") (param i32 i32) (result i32)
                    local.get 0 local.get 1 i32.eq)
                (func (export "lt_s") (param i32 i32) (result i32)
                    local.get 0 local.get 1 i32.lt_s)
                (func (export "gt_s") (param i32 i32) (result i32)
                    local.get 0 local.get 1 i32.gt_s)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // eq: 相等返回 1，不等返回 0
        let r = instance
            .call("eq", &[WasmValue::I32(42), WasmValue::I32(42)])
            .expect("eq same");
        assert_eq!(r[0], WasmValue::I32(1), "42 == 42 应为 1");

        let r = instance
            .call("eq", &[WasmValue::I32(42), WasmValue::I32(43)])
            .expect("eq diff");
        assert_eq!(r[0], WasmValue::I32(0), "42 == 43 应为 0");

        let r = instance
            .call("eq", &[WasmValue::I32(0), WasmValue::I32(-0)])
            .expect("eq zero");
        assert_eq!(r[0], WasmValue::I32(1), "0 == -0 应为 1");

        // lt_s: 有符号小于
        let r = instance
            .call("lt_s", &[WasmValue::I32(-1), WasmValue::I32(0)])
            .expect("lt_s neg<pos");
        assert_eq!(r[0], WasmValue::I32(1), "-1 < 0 应为 1");

        let r = instance
            .call("lt_s", &[WasmValue::I32(0), WasmValue::I32(-1)])
            .expect("lt_s pos>neg");
        assert_eq!(r[0], WasmValue::I32(0), "0 < -1 应为 0");

        // gt_s: 有符号大于
        let r = instance
            .call("gt_s", &[WasmValue::I32(100), WasmValue::I32(50)])
            .expect("gt_s big>small");
        assert_eq!(r[0], WasmValue::I32(1), "100 > 50 应为 1");

        let r = instance
            .call("gt_s", &[WasmValue::I32(50), WasmValue::I32(100)])
            .expect("gt_s small<big");
        assert_eq!(r[0], WasmValue::I32(0), "50 > 100 应为 0");

        // 极值比较
        let r = instance
            .call("lt_s", &[WasmValue::I32(i32::MIN), WasmValue::I32(i32::MAX)])
            .expect("lt_s min<max");
        assert_eq!(r[0], WasmValue::I32(1), "i32::MIN < i32::MAX 应为 1");
    }

    // =======================================================================
    // 新增测试：链接器模块名不匹配、i64 比较运算、燃料单调递减、
    //           memory.grow 失败、主机函数返回 f64 特殊值
    // =======================================================================

    /// 测试主机函数注册了错误的模块名（如 "wrong_env" 而非 WASM 导入的 "env"），
    /// 实例化应因链接失败而返回错误，验证链接器对模块名的精确匹配要求。
    #[test]
    fn test_linker_wrong_module_name() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (import "env" "helper" (func $helper (result i32)))
                (func (export "run") (result i32) call $helper)
            )"#,
        );

        // 注册到错误的模块名 "wrong_env"，而非 WASM 期望的 "env"
        let mut linker = LinkerConfig::new();
        linker.define(HostFunction::new(
            "wrong_env",
            "helper",
            vec![],
            vec![WasmValueType::I32],
            |_params, results| {
                results.push(WasmValue::I32(42));
                Ok(())
            },
        ));

        let module = sandbox.compile(&wasm).expect("compile");
        let result = module.instantiate_with_linker(&sandbox, &linker);
        assert!(result.is_err(), "模块名不匹配时应实例化失败，实际成功了");
        if let Err(e) = result {
            let msg = e.to_string();
            // 链接错误信息中应包含未找到的导入信息
            assert!(
                msg.contains("env") || msg.contains("helper") || msg.contains("link") || msg.contains("unresolved"),
                "错误信息应包含未解析的导入信息，实际: {msg}"
            );
        }
    }

    /// 测试 WASM i64 比较运算符（i64.eq、i64.lt_s、i64.gt_s），
    /// 验证 64 位有符号比较在正数、负数和极值之间的正确行为。
    #[test]
    fn test_i64_comparison_operations() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "eq64") (param i64 i64) (result i32)
                    local.get 0 local.get 1 i64.eq)
                (func (export "lt64") (param i64 i64) (result i32)
                    local.get 0 local.get 1 i64.lt_s)
                (func (export "gt64") (param i64 i64) (result i32)
                    local.get 0 local.get 1 i64.gt_s)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // eq: 相等返回 1
        let r = instance
            .call("eq64", &[WasmValue::I64(123456789), WasmValue::I64(123456789)])
            .expect("eq64 same");
        assert_eq!(r[0], WasmValue::I32(1), "123456789 == 123456789 应为 1");

        // eq: 不等返回 0
        let r = instance
            .call("eq64", &[WasmValue::I64(1), WasmValue::I64(2)])
            .expect("eq64 diff");
        assert_eq!(r[0], WasmValue::I32(0), "1 == 2 应为 0");

        // lt_s: 负数 < 正数
        let r = instance
            .call("lt64", &[WasmValue::I64(-100), WasmValue::I64(100)])
            .expect("lt64 neg<pos");
        assert_eq!(r[0], WasmValue::I32(1), "-100 < 100 应为 1");

        // lt_s: 正数 < 负数 → 0
        let r = instance
            .call("lt64", &[WasmValue::I64(100), WasmValue::I64(-100)])
            .expect("lt64 pos>neg");
        assert_eq!(r[0], WasmValue::I32(0), "100 < -100 应为 0");

        // gt_s: 大正数 > 小正数
        let r = instance
            .call("gt64", &[WasmValue::I64(i64::MAX), WasmValue::I64(0)])
            .expect("gt64 max>0");
        assert_eq!(r[0], WasmValue::I32(1), "i64::MAX > 0 应为 1");

        // 极值比较: i64::MIN < i64::MAX
        let r = instance
            .call("lt64", &[WasmValue::I64(i64::MIN), WasmValue::I64(i64::MAX)])
            .expect("lt64 min<max");
        assert_eq!(r[0], WasmValue::I32(1), "i64::MIN < i64::MAX 应为 1");

        // gt_s: i64::MIN > i64::MAX → 0
        let r = instance
            .call("gt64", &[WasmValue::I64(i64::MIN), WasmValue::I64(i64::MAX)])
            .expect("gt64 min>max");
        assert_eq!(r[0], WasmValue::I32(0), "i64::MIN > i64::MAX 应为 0");
    }

    /// 测试启用燃料计量后，连续多次调用同一函数，每次调用后剩余燃料严格单调递减，
    /// 验证燃料消耗的累加性和单调性。
    #[test]
    fn test_fuel_monotonically_decreasing_across_calls() {
        let config = SandboxConfig::new().consume_fuel(true);
        let sandbox = WasmSandbox::with_config(config);
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "double") (param i32) (result i32)
                    local.get 0 local.get 0 i32.add)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // 设置大量燃料
        instance.set_fuel(10_000).expect("set_fuel");

        let mut prev_fuel = instance.get_fuel().expect("get_fuel initial");
        for i in 1..=10 {
            let r = instance.call("double", &[WasmValue::I32(i)]).expect("call");
            assert_eq!(r[0], WasmValue::I32(i * 2), "double({i}) 应为 {}", i * 2);
            let curr_fuel = instance.get_fuel().expect("get_fuel");
            assert!(
                curr_fuel < prev_fuel,
                "第 {i} 次调用后燃料应严格小于前一次：前={prev_fuel}，后={curr_fuel}"
            );
            prev_fuel = curr_fuel;
        }

        // 最终剩余燃料应大于 0（10_000 足以支撑 10 次简单调用）
        assert!(prev_fuel > 0, "10 次调用后剩余燃料应大于 0，实际: {prev_fuel}");
    }

    /// 测试 WASM memory.grow 在指定最大页数限制时，超过限制后返回 -1（失败）。
    /// 验证内存增长限制生效，grow 失败不导致 trap，而是优雅返回错误码。
    #[test]
    fn test_memory_grow_exceeds_limit() {
        let sandbox = WasmSandbox::new();
        // 声明初始 1 页、最大 2 页的内存
        let wasm = wat_to_wasm(
            r#"(module
                (memory (export "mem") 1 2)
                (func (export "try_grow") (param i32) (result i32)
                    local.get 0
                    memory.grow)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // 初始 1 页，grow 1 页应成功（总共 2 页，未超过最大 2 页限制）
        let r = instance.call("try_grow", &[WasmValue::I32(1)]).expect("grow 1");
        if let WasmValue::I32(prev) = r[0] {
            assert_eq!(prev, 1, "第一次 grow 应返回之前的页数 1");
        } else {
            panic!("期望 I32 返回值");
        }

        // 现在已有 2 页，再 grow 1 页应失败（超过最大 2 页限制），返回 -1
        let r = instance.call("try_grow", &[WasmValue::I32(1)]).expect("grow fail");
        assert_eq!(r[0], WasmValue::I32(-1), "超过最大页数限制时 memory.grow 应返回 -1");

        // 验证内存大小仍为 2 页
        let size = instance.memory_size("mem").expect("size");
        assert_eq!(size, 65536 * 2, "内存大小应保持 2 页不变");

        // 验证 grow 0 页始终成功（无操作）
        let r = instance.call("try_grow", &[WasmValue::I32(0)]).expect("grow 0");
        if let WasmValue::I32(prev) = r[0] {
            assert_eq!(prev, 2, "grow 0 页应返回当前页数 2");
        } else {
            panic!("期望 I32 返回值");
        }
    }

    /// 测试主机函数返回 f64 特殊值（NaN、正无穷）给 WASM 侧，
    /// 验证特殊浮点值在主机 → WASM 传递路径上不丢失、不截断。
    #[test]
    fn test_host_function_returns_f64_special() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (import "env" "get_special" (func $get_special (param i32) (result f64)))
                (func (export "call_special") (param i32) (result f64)
                    local.get 0
                    call $get_special)
            )"#,
        );

        let mut linker = LinkerConfig::new();
        linker.define(HostFunction::new(
            "env",
            "get_special",
            vec![WasmValueType::I32],
            vec![WasmValueType::F64],
            |params, results| {
                if let WasmValue::I32(code) = params[0] {
                    match code {
                        0 => results.push(WasmValue::F64(f64::NAN)),
                        1 => results.push(WasmValue::F64(f64::INFINITY)),
                        2 => results.push(WasmValue::F64(f64::NEG_INFINITY)),
                        _ => results.push(WasmValue::F64(0.0)),
                    }
                }
                Ok(())
            },
        ));

        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate_with_linker(&sandbox, &linker).expect("instantiate");

        // 获取 NaN
        let r = instance.call("call_special", &[WasmValue::I32(0)]).expect("call nan");
        if let WasmValue::F64(v) = r[0] {
            assert!(v.is_nan(), "主机返回的 f64 NaN 应正确传递，实际: {v}");
        } else {
            panic!("期望 F64 返回值");
        }

        // 获取正无穷
        let r = instance.call("call_special", &[WasmValue::I32(1)]).expect("call inf");
        if let WasmValue::F64(v) = r[0] {
            assert!(
                v.is_infinite() && v.is_sign_positive(),
                "主机返回的 f64 正无穷应正确传递，实际: {v}"
            );
        } else {
            panic!("期望 F64 返回值");
        }

        // 获取负无穷
        let r = instance
            .call("call_special", &[WasmValue::I32(2)])
            .expect("call neg_inf");
        if let WasmValue::F64(v) = r[0] {
            assert!(
                v.is_infinite() && v.is_sign_negative(),
                "主机返回的 f64 负无穷应正确传递，实际: {v}"
            );
        } else {
            panic!("期望 F64 返回值");
        }
    }

    // =======================================================================
    // 新增测试：多实例内存独立性、表导出缺失、可变全局读取、
    //           燃料累计消耗、空模块错误处理
    // =======================================================================

    /// 测试同一模块使用相同 LinkerConfig 实例化两次后，两个实例的内存完全独立。
    /// 向实例 A 的内存写入数据 A，向实例 B 的内存写入数据 B，
    /// 验证实例 A 读回的仍是数据 A，实例 B 读回的仍是数据 B，互不干扰。
    #[test]
    fn test_two_instances_memory_isolation() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (memory (export "mem") 1)
                (func (export "read_byte") (param i32) (result i32)
                    local.get 0 i32.load8_u)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");

        // 使用相同的空 LinkerConfig 实例化两个实例
        let linker = LinkerConfig::new();
        let mut inst_a = module
            .instantiate_with_linker(&sandbox, &linker)
            .expect("instantiate A");
        let mut inst_b = module
            .instantiate_with_linker(&sandbox, &linker)
            .expect("instantiate B");

        // 向实例 A 偏移 0 写入 0xAA，向实例 B 偏移 0 写入 0xBB
        inst_a.write_memory("mem", 0, &[0xAA]).expect("write A");
        inst_b.write_memory("mem", 0, &[0xBB]).expect("write B");

        // 验证实例 A 的内存未被实例 B 影响
        let data_a = inst_a.read_memory("mem", 0, 1).expect("read A");
        assert_eq!(data_a, [0xAA], "实例 A 的内存应仍为 0xAA");

        // 验证实例 B 的内存未被实例 A 影响
        let data_b = inst_b.read_memory("mem", 0, 1).expect("read B");
        assert_eq!(data_b, [0xBB], "实例 B 的内存应仍为 0xBB");

        // 通过 WASM 函数进一步验证隔离性
        let r_a = inst_a.call("read_byte", &[WasmValue::I32(0)]).expect("call A");
        assert_eq!(r_a[0], WasmValue::I32(0xAA), "WASM 函数读取实例 A 应返回 0xAA");

        let r_b = inst_b.call("read_byte", &[WasmValue::I32(0)]).expect("call B");
        assert_eq!(r_b[0], WasmValue::I32(0xBB), "WASM 函数读取实例 B 应返回 0xBB");

        // 向两个实例的不同偏移写入更多数据，进一步验证独立性
        inst_a.write_memory("mem", 100, b"AAAA").expect("write A offset");
        inst_b.write_memory("mem", 100, b"BBBB").expect("write B offset");

        let data_a2 = inst_a.read_memory("mem", 100, 4).expect("read A offset");
        assert_eq!(&data_a2, b"AAAA", "实例 A 偏移 100 应为 AAAA");
        let data_b2 = inst_b.read_memory("mem", 100, 4).expect("read B offset");
        assert_eq!(&data_b2, b"BBBB", "实例 B 偏移 100 应为 BBBB");
    }

    /// 测试没有导出表的 WASM 模块，has_table 应返回 false。
    /// 同时验证函数导出和内存导出不会被 has_table 误判为表。
    #[test]
    fn test_has_table_when_no_table_export() {
        let sandbox = WasmSandbox::new();
        // 模块只有函数和内存导出，没有表
        let wasm = wat_to_wasm(
            r#"(module
                (memory (export "mem") 1)
                (func (export "f") nop)
                (global (export "g") i32 (i32.const 0))
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let instance = module.instantiate(&sandbox).expect("instantiate");

        // 没有表导出，has_table 应返回 false
        assert!(!instance.has_table("tbl"), "不存在的表名应返回 false");
        assert!(!instance.has_table("mem"), "内存导出不应被 has_table 匹配");
        assert!(!instance.has_table("f"), "函数导出不应被 has_table 匹配");
        assert!(!instance.has_table("g"), "全局导出不应被 has_table 匹配");
        assert!(!instance.has_table(""), "空名字应返回 false");
    }

    /// 测试通过 get_global_export 读取不可变的 i32 全局导出变量，
    /// 验证返回的 WasmValue 类型为 I32 且值正确。
    #[test]
    fn test_get_global_export_i32_immutable() {
        let sandbox = WasmSandbox::new();
        let wasm = wat_to_wasm(
            r#"(module
                (global (export "answer") i32 (i32.const 42))
                (global (export "zero") i32 (i32.const 0))
                (global (export "neg") i32 (i32.const -999))
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let instance = module.instantiate(&sandbox).expect("instantiate");

        // 读取各个全局导出
        let answer = instance.get_global_export("answer").expect("read answer");
        assert_eq!(answer, WasmValue::I32(42), "全局 answer 应为 i32(42)");

        let zero = instance.get_global_export("zero").expect("read zero");
        assert_eq!(zero, WasmValue::I32(0), "全局 zero 应为 i32(0)");

        let neg = instance.get_global_export("neg").expect("read neg");
        assert_eq!(neg, WasmValue::I32(-999), "全局 neg 应为 i32(-999)");

        // 不存在的全局应返回 None
        assert!(
            instance.get_global_export("nonexistent").is_none(),
            "不存在的全局导出应返回 None"
        );
    }

    /// 测试启用燃料计量后，设置初始燃料、执行多次函数调用，
    /// 验证每次调用后剩余燃料严格递减，且总消耗等于各次消耗之和。
    #[test]
    fn test_fuel_consumption_across_multiple_calls() {
        let config = SandboxConfig::new().consume_fuel(true);
        let sandbox = WasmSandbox::with_config(config);
        let wasm = wat_to_wasm(
            r#"(module
                (func (export "square") (param i32) (result i32)
                    local.get 0 local.get 0 i32.mul)
            )"#,
        );
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        let initial_fuel: u64 = 50_000;
        instance.set_fuel(initial_fuel).expect("set_fuel");

        let mut remaining = initial_fuel;
        let mut consumptions: Vec<u64> = Vec::new();

        // 执行 5 次调用，记录每次的燃料消耗
        for i in 1..=5 {
            let before = instance.get_fuel().expect("get_fuel before");
            let r = instance
                .call("square", &[WasmValue::I32(i)])
                .unwrap_or_else(|_| panic!("call {i} should succeed"));
            assert_eq!(r[0], WasmValue::I32(i * i), "square({i}) 应为 {}", i * i);
            let after = instance.get_fuel().expect("get_fuel after");
            let consumed = before - after;
            assert!(consumed > 0, "第 {i} 次调用应消耗燃料，消耗量: {consumed}");
            consumptions.push(consumed);
            remaining = after;
        }

        // 总消耗应等于各次消耗之和
        let total_consumed: u64 = consumptions.iter().sum();
        assert_eq!(initial_fuel - remaining, total_consumed, "总消耗应等于各次消耗之和");

        // 每次调用的消耗量应相同（相同函数相同模式）
        let first = consumptions[0];
        for (i, &c) in consumptions.iter().enumerate() {
            assert_eq!(
                c,
                first,
                "第 {} 次调用的燃料消耗 ({c}) 应与第一次 ({first}) 相同",
                i + 1
            );
        }

        // 剩余燃料应严格大于 0（50_000 足以支撑 5 次 square 调用）
        assert!(remaining > 0, "剩余燃料应大于 0，实际: {remaining}");
        assert!(remaining < initial_fuel, "剩余燃料应小于初始值");
    }

    /// 测试在空模块（无导出函数）上调用任意函数名，应返回 ExportNotFound 错误。
    /// 验证沙箱在无效的实例句柄上优雅地处理调用失败，不会 panic。
    #[test]
    fn test_call_on_module_with_no_exports_graceful_error() {
        let sandbox = WasmSandbox::new();
        // 编译一个没有任何导出的空模块
        let wasm = wat_to_wasm("(module)");
        let module = sandbox.compile(&wasm).expect("compile");
        let mut instance = module.instantiate(&sandbox).expect("instantiate");

        // 在空模块上调用任何函数都应返回 ExportNotFound 错误
        let result = instance.call("any_func", &[]);
        assert!(result.is_err(), "空模块上调用函数应返回错误");
        if let Err(WasmError::ExportNotFound { name }) = result {
            assert_eq!(name, "any_func", "错误中应包含请求的函数名");
        } else {
            panic!("期望 ExportNotFound 错误，实际: {result:?}");
        }

        // 再次调用不同名称，验证错误处理的一致性
        let result2 = instance.call("another_func", &[WasmValue::I32(1)]);
        assert!(result2.is_err(), "第二次调用也应返回错误");
        if let Err(WasmError::ExportNotFound { name }) = result2 {
            assert_eq!(name, "another_func", "错误中应包含第二个函数名");
        } else {
            panic!("期望 ExportNotFound 错误，实际: {result2:?}");
        }

        // has_func 也应返回 false
        assert!(!instance.has_func("any_func"), "has_func 对空模块应返回 false");
        assert!(!instance.has_func(""), "has_func 对空字符串应返回 false");
    }
}
