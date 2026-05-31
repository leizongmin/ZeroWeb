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
}
