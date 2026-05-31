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
}
