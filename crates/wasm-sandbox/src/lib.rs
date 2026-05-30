//! # zero-wasm-sandbox
//!
//! 非页面 WASM 运行时（wasmi）。
//!
//! 用于插件、扩展能力或受控计算任务。
//! 基于 wasmi 纯 Rust WASM 解释器实现。

#![warn(missing_docs)]

use std::fmt;

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

#[cfg(feature = "wasmi")]
mod wasmi_backend {
    use super::{WasmError, WasmValue};

    /// WASM 沙箱运行时
    pub struct WasmSandbox {
        engine: wasmi::Engine,
    }

    impl WasmSandbox {
        /// 创建新的 WASM 沙箱
        pub fn new() -> Self {
            Self {
                engine: wasmi::Engine::default(),
            }
        }

        /// 编译 WASM 模块
        pub fn compile(&self, bytes: &[u8]) -> Result<WasmModule, WasmError> {
            let module = wasmi::Module::new(&self.engine, bytes)
                .map_err(|e| WasmError::InvalidBinary(e.to_string()))?;
            Ok(WasmModule { module })
        }

        /// 获取引擎引用
        pub fn engine(&self) -> &wasmi::Engine {
            &self.engine
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

    impl WasmModule {
        /// 实例化模块
        pub fn instantiate(&self, sandbox: &WasmSandbox) -> Result<WasmInstance, WasmError> {
            let mut store = wasmi::Store::new(sandbox.engine(), ());
            let linker = wasmi::Linker::new(sandbox.engine());

            let instance = linker
                .instantiate(&mut store, &self.module)
                .map_err(|e| WasmError::InstantiationError(e.to_string()))?
                .start(&mut store)
                .map_err(|e| WasmError::InstantiationError(e.to_string()))?;

            Ok(WasmInstance { store, instance })
        }

        /// 获取导出名称列表
        pub fn exports(&self) -> Vec<String> {
            self.module
                .exports()
                .map(|e| e.name().to_string())
                .collect()
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
            let func = self.instance.get_func(&self.store, name).ok_or_else(|| {
                WasmError::ExportNotFound {
                    name: name.to_string(),
                }
            })?;

            let params: Vec<wasmi::Val> = args.iter().map(|v| match v {
                WasmValue::I32(n) => wasmi::Val::I32(*n),
                WasmValue::I64(n) => wasmi::Val::I64(*n),
                WasmValue::F32(n) => wasmi::Val::F32((*n).into()),
                WasmValue::F64(n) => wasmi::Val::F64((*n).into()),
            }).collect();

            // 获取返回值类型以分配输出缓冲区
            let func_type = func.ty(&self.store);
            let result_types: Vec<_> = func_type.results().to_vec();
            let mut outputs: Vec<wasmi::Val> = result_types.iter().map(|t| wasmi::Val::default(*t)).collect();

            func.call(&mut self.store, &params, &mut outputs)
                .map_err(|e| WasmError::CallError(e.to_string()))?;

            Ok(outputs.iter().map(|v| match v {
                wasmi::Val::I32(n) => WasmValue::I32(*n),
                wasmi::Val::I64(n) => WasmValue::I64(*n),
                wasmi::Val::F32(n) => WasmValue::F32(f32::from(*n)),
                wasmi::Val::F64(n) => WasmValue::F64(f64::from(*n)),
                _ => WasmValue::I32(0),
            }).collect())
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
            let memory = self.instance.get_memory(&self.store, name)
                .ok_or_else(|| WasmError::ExportNotFound { name: name.to_string() })?;
            let mem_data = memory.data_mut(&mut self.store);
            let end = offset.checked_add(data.len())
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
    }
}

#[cfg(feature = "wasmi")]
pub use wasmi_backend::*;

#[cfg(not(feature = "wasmi"))]
mod stub_backend {
    use super::{WasmError, WasmValue};

    /// WASM 沙箱运行时（占位实现）
    pub struct WasmSandbox;

    impl WasmSandbox {
        /// 创建新的 WASM 沙箱
        pub fn new() -> Self {
            Self
        }

        /// 编译 WASM 模块
        pub fn compile(&self, _bytes: &[u8]) -> Result<WasmModule, WasmError> {
            Err(WasmError::InvalidBinary(
                "no WASM backend enabled (enable 'wasmi' feature)".into(),
            ))
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
        let results = instance.call("add", &[WasmValue::I32(10), WasmValue::I32(20)])
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
        let results = instance.call("add64", &[WasmValue::I64(1000), WasmValue::I64(2000)])
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
        let results = instance.call("double_f32", &[WasmValue::F32(3.5)])
            .expect("call");
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
        let results = instance.call("double_f64", &[WasmValue::F64(2.5)])
            .expect("call");
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
        instance
            .write_memory("mem", 0, b"hello")
            .expect("write");

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
        assert!(format!("{}", WasmValue::F32(3.14)).contains("3.14"));
        assert!(format!("{}", WasmValue::F64(2.718)).contains("2.718"));
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
}
