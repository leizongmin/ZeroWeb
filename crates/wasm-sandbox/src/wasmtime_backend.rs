//! Wasmtime 后端实现
//!
//! 基于 Wasmtime JIT 编译器的高性能 WASM 运行时。
//! 适用于需要接近原生执行速度的页面级 WASM 场景。

use crate::{LinkerConfig, SandboxConfig, WasmError, WasmValue, WasmValueType};
use std::cell::RefCell;
use std::sync::Arc;

/// WASM 沙箱运行时（Wasmtime JIT 后端）
pub struct WasmSandbox {
    engine: wasmtime::Engine,
    config: SandboxConfig,
}

impl WasmSandbox {
    /// 创建新的 WASM 沙箱（默认配置）
    pub fn new() -> Self {
        Self::with_config(SandboxConfig::default())
    }

    /// 使用指定配置创建 WASM 沙箱
    pub fn with_config(config: SandboxConfig) -> Self {
        let mut wasmtime_config = wasmtime::Config::default();
        wasmtime_config.consume_fuel(config.is_consume_fuel());
        let engine = wasmtime::Engine::new(&wasmtime_config).expect("failed to create Wasmtime engine");
        Self { engine, config }
    }

    /// 编译 WASM 模块
    pub fn compile(&self, bytes: &[u8]) -> Result<WasmModule, WasmError> {
        let module =
            wasmtime::Module::from_binary(&self.engine, bytes).map_err(|e| WasmError::InvalidBinary(e.to_string()))?;
        Ok(WasmModule { module })
    }

    /// 获取引擎引用
    pub fn engine(&self) -> &wasmtime::Engine {
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
    module: wasmtime::Module,
}

fn wasm_value_type_to_wasmtime(ty: WasmValueType) -> wasmtime::ValType {
    match ty {
        WasmValueType::I32 => wasmtime::ValType::I32,
        WasmValueType::I64 => wasmtime::ValType::I64,
        WasmValueType::F32 => wasmtime::ValType::F32,
        WasmValueType::F64 => wasmtime::ValType::F64,
    }
}

fn wasm_value_to_wasmtime(v: &WasmValue) -> wasmtime::Val {
    match v {
        WasmValue::I32(n) => wasmtime::Val::I32(*n),
        WasmValue::I64(n) => wasmtime::Val::I64(*n),
        WasmValue::F32(n) => wasmtime::Val::F32(n.to_bits()),
        WasmValue::F64(n) => wasmtime::Val::F64(n.to_bits()),
    }
}

fn wasmtime_val_to_wasm(v: &wasmtime::Val) -> WasmValue {
    match v {
        wasmtime::Val::I32(n) => WasmValue::I32(*n),
        wasmtime::Val::I64(n) => WasmValue::I64(*n),
        wasmtime::Val::F32(bits) => WasmValue::F32(f32::from_bits(*bits)),
        wasmtime::Val::F64(bits) => WasmValue::F64(f64::from_bits(*bits)),
        _ => WasmValue::I32(0),
    }
}

fn default_wasmtime_val_for_type(ty: &wasmtime::ValType) -> wasmtime::Val {
    match ty {
        wasmtime::ValType::I32 => wasmtime::Val::I32(0),
        wasmtime::ValType::I64 => wasmtime::Val::I64(0),
        wasmtime::ValType::F32 => wasmtime::Val::F32(0),
        wasmtime::ValType::F64 => wasmtime::Val::F64(0),
        _ => wasmtime::Val::I32(0),
    }
}

fn map_wasmtime_error(error: wasmtime::Error) -> WasmError {
    if matches!(error.downcast_ref::<wasmtime::Trap>(), Some(wasmtime::Trap::OutOfFuel)) {
        return WasmError::FuelExhausted;
    }

    let msg = error.to_string();
    for cause in error.chain() {
        let cause_msg = cause.to_string();
        if cause_msg.contains("host function error:") {
            return WasmError::CallError(cause_msg);
        }
    }

    if msg.contains("out of fuel") || msg.contains("all fuel consumed") {
        WasmError::FuelExhausted
    } else if error.downcast_ref::<wasmtime::Trap>().is_some() {
        WasmError::CallError(format!("trap: {msg}"))
    } else if msg.contains("wasm backtrace") {
        WasmError::CallError(format!("trap: {msg}"))
    } else {
        WasmError::CallError(msg)
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
        let mut store = wasmtime::Store::new(&sandbox.engine, ());
        let mut linker = wasmtime::Linker::new(&sandbox.engine);

        for host_func in linker_config.functions() {
            let params: Vec<wasmtime::ValType> = host_func
                .params
                .iter()
                .map(|&p| wasm_value_type_to_wasmtime(p))
                .collect();
            let results: Vec<wasmtime::ValType> = host_func
                .results
                .iter()
                .map(|&r| wasm_value_type_to_wasmtime(r))
                .collect();
            let default_results: Vec<wasmtime::Val> = results.iter().map(default_wasmtime_val_for_type).collect();
            let func_type = wasmtime::FuncType::new(&sandbox.engine, params, results);

            let arc_func: Arc<crate::HostFn> = host_func.func.clone();
            linker
                .func_new(
                    &host_func.module,
                    &host_func.name,
                    func_type,
                    move |_caller, params, results| {
                        let wasm_params: Vec<WasmValue> = params.iter().map(wasmtime_val_to_wasm).collect();
                        let mut wasm_results = Vec::new();
                        match arc_func(&wasm_params, &mut wasm_results) {
                            Ok(()) => {
                                for (i, val) in default_results.iter().enumerate() {
                                    if i < results.len() {
                                        results[i] = val.clone();
                                    }
                                }
                                for (i, val) in wasm_results.iter().enumerate() {
                                    if i < results.len() {
                                        results[i] = wasm_value_to_wasmtime(val);
                                    }
                                }
                                Ok(())
                            }
                            Err(e) => Err(wasmtime::Error::msg(format!("host function error: {e}"))),
                        }
                    },
                )
                .map_err(|e| WasmError::LinkError(e.to_string()))?;
        }

        let instance = linker
            .instantiate(&mut store, &self.module)
            .map_err(|e| WasmError::InstantiationError(map_wasmtime_error(e).to_string()))?;

        // 执行 start 函数（如果有）
        let start_func = instance.get_func(&mut store, "_start");
        if let Some(func) = start_func {
            func.call(&mut store, &[], &mut [])
                .map_err(|e| WasmError::InstantiationError(map_wasmtime_error(e).to_string()))?;
        }

        Ok(WasmInstance {
            store: RefCell::new(store),
            instance,
        })
    }

    /// 获取导出名称列表
    pub fn exports(&self) -> Vec<String> {
        self.module.exports().map(|e| e.name().to_string()).collect()
    }
}

/// WASM 实例
///
/// 使用 RefCell 包装 Store 以支持 wasmtime 的内部可变性需求。
/// Wasmtime 的 Instance 访问方法需要 AsContextMut（即 &mut Store），
/// 但公共 API 保持了与 wasmi 后端一致的 &self 方法签名。
pub struct WasmInstance {
    store: RefCell<wasmtime::Store<()>>,
    instance: wasmtime::Instance,
}

impl WasmInstance {
    /// 调用导出函数
    pub fn call(&self, name: &str, args: &[WasmValue]) -> Result<Vec<WasmValue>, WasmError> {
        let mut store = self.store.borrow_mut();
        let func = self
            .instance
            .get_func(&mut *store, name)
            .ok_or_else(|| WasmError::ExportNotFound { name: name.to_string() })?;

        let params: Vec<wasmtime::Val> = args.iter().map(wasm_value_to_wasmtime).collect();

        let func_type = func.ty(&*store);
        let result_types: Vec<wasmtime::ValType> = func_type.results().collect();
        let mut outputs: Vec<wasmtime::Val> = result_types.iter().map(default_wasmtime_val_for_type).collect();

        func.call(&mut *store, &params, &mut outputs)
            .map_err(map_wasmtime_error)?;

        Ok(outputs.iter().map(wasmtime_val_to_wasm).collect())
    }

    /// 读取线性内存
    pub fn read_memory(&self, name: &str, offset: usize, len: usize) -> Option<Vec<u8>> {
        let mut store = self.store.borrow_mut();
        let memory = self.instance.get_memory(&mut *store, name)?;
        let data = memory.data(&*store);
        // R3347 deep-review：checked_add 防 offset+len 溢出致 OOB 切片 panic（见 wasmi_backend 注释）。
        let end = offset.checked_add(len)?;
        if end > data.len() {
            return None;
        }
        Some(data[offset..end].to_vec())
    }

    /// 写入线性内存
    pub fn write_memory(&self, name: &str, offset: usize, data: &[u8]) -> Result<(), WasmError> {
        let mut store = self.store.borrow_mut();
        let memory = self
            .instance
            .get_memory(&mut *store, name)
            .ok_or_else(|| WasmError::ExportNotFound { name: name.to_string() })?;
        let mem_data = memory.data_mut(&mut *store);
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
        let mut store = self.store.borrow_mut();
        self.instance.get_func(&mut *store, name).is_some()
    }

    /// 检查导出内存是否存在
    pub fn has_memory(&self, name: &str) -> bool {
        let mut store = self.store.borrow_mut();
        self.instance.get_memory(&mut *store, name).is_some()
    }

    /// 获取内存大小（字节数）
    pub fn memory_size(&self, name: &str) -> Option<usize> {
        let mut store = self.store.borrow_mut();
        let memory = self.instance.get_memory(&mut *store, name)?;
        Some(memory.data_size(&*store))
    }

    /// 读取全局导出变量的值
    pub fn get_global_export(&self, name: &str) -> Option<WasmValue> {
        let mut store = self.store.borrow_mut();
        let global = self.instance.get_global(&mut *store, name)?;
        let val = global.get(&mut *store);
        Some(wasmtime_val_to_wasm(&val))
    }

    /// 检查导出表是否存在
    pub fn has_table(&self, name: &str) -> bool {
        let mut store = self.store.borrow_mut();
        self.instance.get_table(&mut *store, name).is_some()
    }

    /// 设置剩余燃料（需要启用燃料计量）
    pub fn set_fuel(&self, fuel: u64) -> Result<(), WasmError> {
        let mut store = self.store.borrow_mut();
        store.set_fuel(fuel).map_err(|e| WasmError::CallError(e.to_string()))
    }

    /// 获取剩余燃料（需要启用燃料计量）
    pub fn get_fuel(&self) -> Result<u64, WasmError> {
        let store = self.store.borrow_mut();
        store.get_fuel().map_err(|e| WasmError::CallError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HostFunction;

    /// 辅助函数：编译一个简单的 WASM 模块（返回 42 的函数）
    fn simple_module_bytes() -> Vec<u8> {
        wat::parse_str(
            r#"(module
                (func (export "answer") (result i32) i32.const 42)
                (memory (export "memory") 1)
            )"#,
        )
        .unwrap()
    }

    #[test]
    fn test_sandbox_creation() {
        let sandbox = WasmSandbox::new();
        assert!(!sandbox.config().is_consume_fuel());
    }

    #[test]
    fn test_sandbox_with_config() {
        let config = SandboxConfig::new().consume_fuel(true);
        let sandbox = WasmSandbox::with_config(config);
        assert!(sandbox.config().is_consume_fuel());
    }

    #[test]
    fn test_sandbox_default() {
        let sandbox = WasmSandbox::default();
        assert!(!sandbox.config().is_consume_fuel());
    }

    #[test]
    fn test_compile_valid_module() {
        let sandbox = WasmSandbox::new();
        let result = sandbox.compile(&simple_module_bytes());
        assert!(result.is_ok());
        let module = result.unwrap();
        let exports = module.exports();
        assert!(exports.contains(&"answer".to_string()));
        assert!(exports.contains(&"memory".to_string()));
    }

    #[test]
    fn test_compile_invalid_binary() {
        let sandbox = WasmSandbox::new();
        let result = sandbox.compile(&[0x00, 0x01, 0x02, 0x03]);
        assert!(result.is_err());
        if let Err(WasmError::InvalidBinary(_)) = result {
            // 预期错误类型
        } else {
            panic!("expected InvalidBinary error");
        }
    }

    #[test]
    fn test_instantiate_and_call() {
        let sandbox = WasmSandbox::new();
        let module = sandbox.compile(&simple_module_bytes()).unwrap();
        let instance = module.instantiate(&sandbox).unwrap();
        drop(instance);
    }

    #[test]
    fn test_call_function() {
        let sandbox = WasmSandbox::new();
        let module = sandbox.compile(&simple_module_bytes()).unwrap();
        let instance = module.instantiate(&sandbox).unwrap();
        let result = instance.call("answer", &[]).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], WasmValue::I32(42));
    }

    #[test]
    fn test_call_not_found() {
        let sandbox = WasmSandbox::new();
        let module = sandbox.compile(&simple_module_bytes()).unwrap();
        let instance = module.instantiate(&sandbox).unwrap();
        let result = instance.call("nonexistent", &[]);
        assert!(result.is_err());
        if let Err(WasmError::ExportNotFound { name }) = result {
            assert_eq!(name, "nonexistent");
        } else {
            panic!("expected ExportNotFound");
        }
    }

    #[test]
    fn test_has_func() {
        let sandbox = WasmSandbox::new();
        let module = sandbox.compile(&simple_module_bytes()).unwrap();
        let instance = module.instantiate(&sandbox).unwrap();
        assert!(instance.has_func("answer"));
        assert!(!instance.has_func("nonexistent"));
    }

    #[test]
    fn test_memory_operations() {
        let bytes = wat::parse_str(
            r#"(module
                (memory (export "memory") 1)
                (func (export "store") (param i32 i32)
                    local.get 0
                    local.get 1
                    i32.store
                )
            )"#,
        )
        .unwrap();

        let sandbox = WasmSandbox::new();
        let module = sandbox.compile(&bytes).unwrap();
        let instance = module.instantiate(&sandbox).unwrap();

        // 写入内存
        instance.write_memory("memory", 0, &[0x42, 0x00, 0x00, 0x00]).unwrap();

        // 通过 WASM 函数验证
        let result = instance
            .call("store", &[WasmValue::I32(4), WasmValue::I32(99)])
            .unwrap();
        assert!(result.is_empty());

        // 读取内存
        let data = instance.read_memory("memory", 0, 8).unwrap();
        assert_eq!(data[0], 0x42);
        assert_eq!(data[4], 99);
        assert_eq!(data[5], 0);
    }

    #[test]
    fn test_has_memory() {
        let sandbox = WasmSandbox::new();
        let module = sandbox.compile(&simple_module_bytes()).unwrap();
        let instance = module.instantiate(&sandbox).unwrap();
        assert!(instance.has_memory("memory"));
        assert!(!instance.has_memory("nonexistent"));
    }

    #[test]
    fn test_memory_size() {
        let sandbox = WasmSandbox::new();
        let module = sandbox.compile(&simple_module_bytes()).unwrap();
        let instance = module.instantiate(&sandbox).unwrap();
        let size = instance.memory_size("memory").unwrap();
        assert_eq!(size, 65536);
    }

    #[test]
    fn test_read_memory_out_of_bounds() {
        let sandbox = WasmSandbox::new();
        let module = sandbox.compile(&simple_module_bytes()).unwrap();
        let instance = module.instantiate(&sandbox).unwrap();
        let result = instance.read_memory("memory", 65530, 10);
        assert!(result.is_none());
    }

    #[test]
    fn test_write_memory_out_of_bounds() {
        let sandbox = WasmSandbox::new();
        let module = sandbox.compile(&simple_module_bytes()).unwrap();
        let instance = module.instantiate(&sandbox).unwrap();
        let result = instance.write_memory("memory", 65530, &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert!(result.is_err());
    }

    #[test]
    fn test_host_function() {
        let bytes = wat::parse_str(
            r#"(module
                (import "env" "add" (func $add (param i32 i32) (result i32)))
                (func (export "double_add") (param i32) (result i32)
                    local.get 0
                    local.get 0
                    call $add
                )
            )"#,
        )
        .unwrap();

        let sandbox = WasmSandbox::new();
        let module = sandbox.compile(&bytes).unwrap();

        let mut linker = LinkerConfig::new();
        linker.define(HostFunction::new(
            "env",
            "add",
            vec![WasmValueType::I32, WasmValueType::I32],
            vec![WasmValueType::I32],
            |params, results| {
                let a = match params[0] {
                    WasmValue::I32(v) => v,
                    _ => 0,
                };
                let b = match params[1] {
                    WasmValue::I32(v) => v,
                    _ => 0,
                };
                results.push(WasmValue::I32(a + b));
                Ok(())
            },
        ));

        let instance = module.instantiate_with_linker(&sandbox, &linker).unwrap();
        let result = instance.call("double_add", &[WasmValue::I32(21)]).unwrap();
        assert_eq!(result[0], WasmValue::I32(42));
    }

    #[test]
    fn test_global_export() {
        let bytes = wat::parse_str(
            r#"(module
                (global $g (export "counter") (mut i32) (i32.const 100))
                (func (export "inc")
                    global.get $g
                    i32.const 1
                    i32.add
                    global.set $g
                )
            )"#,
        )
        .unwrap();

        let sandbox = WasmSandbox::new();
        let module = sandbox.compile(&bytes).unwrap();
        let instance = module.instantiate(&sandbox).unwrap();

        let val = instance.get_global_export("counter").unwrap();
        assert_eq!(val, WasmValue::I32(100));

        instance.call("inc", &[]).unwrap();

        let val = instance.get_global_export("counter").unwrap();
        assert_eq!(val, WasmValue::I32(101));
    }

    #[test]
    fn test_has_table() {
        let bytes = wat::parse_str(
            r#"(module
                (table (export "tbl") 1 funcref)
            )"#,
        )
        .unwrap();

        let sandbox = WasmSandbox::new();
        let module = sandbox.compile(&bytes).unwrap();
        let instance = module.instantiate(&sandbox).unwrap();
        assert!(instance.has_table("tbl"));
        assert!(!instance.has_table("nonexistent"));
    }

    #[test]
    fn test_fuel_consumption() {
        let config = SandboxConfig::new().consume_fuel(true);
        let sandbox = WasmSandbox::with_config(config);
        let module = sandbox.compile(&simple_module_bytes()).unwrap();
        let instance = module.instantiate_with_linker(&sandbox, &LinkerConfig::new()).unwrap();

        instance.set_fuel(10000).unwrap();
        let initial = instance.get_fuel().unwrap();
        assert!(initial <= 10000);

        instance.call("answer", &[]).unwrap();
        let after = instance.get_fuel().unwrap();
        assert!(after < initial);
    }

    #[test]
    fn test_fuel_exhausted() {
        let bytes = wat::parse_str(
            r#"(module
                (func (export "loop_forever")
                    (loop $l br $l)
                )
            )"#,
        )
        .unwrap();

        let config = SandboxConfig::new().consume_fuel(true);
        let sandbox = WasmSandbox::with_config(config);
        let module = sandbox.compile(&bytes).unwrap();
        let instance = module.instantiate_with_linker(&sandbox, &LinkerConfig::new()).unwrap();

        instance.set_fuel(100).unwrap();
        let result = instance.call("loop_forever", &[]);
        assert!(result.is_err());
        // Wasmtime 在燃料耗尽时返回 CallError，包含 "out of fuel" 或类似信息
        match result {
            Err(WasmError::FuelExhausted) | Err(WasmError::CallError(_)) => {
                // 两种结果都可接受：取决于 Wasmtime 版本的具体错误报告方式
            }
            _ => panic!("expected fuel-related error, got: {:?}", result),
        }
    }

    #[test]
    fn test_get_fuel_without_fuel_feature() {
        let sandbox = WasmSandbox::new();
        let module = sandbox.compile(&simple_module_bytes()).unwrap();
        let instance = module.instantiate_with_linker(&sandbox, &LinkerConfig::new()).unwrap();
        let result = instance.get_fuel();
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_functions() {
        let bytes = wat::parse_str(
            r#"(module
                (func (export "add") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.add
                )
                (func (export "mul") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.mul
                )
            )"#,
        )
        .unwrap();

        let sandbox = WasmSandbox::new();
        let module = sandbox.compile(&bytes).unwrap();
        let instance = module.instantiate(&sandbox).unwrap();

        let r = instance.call("add", &[WasmValue::I32(3), WasmValue::I32(4)]).unwrap();
        assert_eq!(r[0], WasmValue::I32(7));

        let r = instance.call("mul", &[WasmValue::I32(3), WasmValue::I32(4)]).unwrap();
        assert_eq!(r[0], WasmValue::I32(12));
    }

    #[test]
    fn test_f64_values() {
        let bytes = wat::parse_str(
            r#"(module
                (func (export "area") (param f64) (result f64)
                    local.get 0
                    local.get 0
                    f64.mul
                    f64.const 3.14159265358979
                    f64.mul
                )
            )"#,
        )
        .unwrap();

        let sandbox = WasmSandbox::new();
        let module = sandbox.compile(&bytes).unwrap();
        let instance = module.instantiate(&sandbox).unwrap();

        let r = instance.call("area", &[WasmValue::F64(5.0)]).unwrap();
        match r[0] {
            WasmValue::F64(v) => {
                let expected = std::f64::consts::PI * 25.0;
                assert!((v - expected).abs() < 0.001);
            }
            _ => panic!("expected F64"),
        }
    }

    #[test]
    fn test_i64_values() {
        let bytes = wat::parse_str(
            r#"(module
                (func (export "big") (result i64) i64.const 9223372036854775807)
            )"#,
        )
        .unwrap();

        let sandbox = WasmSandbox::new();
        let module = sandbox.compile(&bytes).unwrap();
        let instance = module.instantiate(&sandbox).unwrap();

        let r = instance.call("big", &[]).unwrap();
        assert_eq!(r[0], WasmValue::I64(i64::MAX));
    }

    #[test]
    fn test_host_function_error_propagation() {
        let bytes = wat::parse_str(
            r#"(module
                (import "env" "fail" (func (result i32)))
                (func (export "call_fail") (result i32) call 0)
            )"#,
        )
        .unwrap();

        let sandbox = WasmSandbox::new();
        let module = sandbox.compile(&bytes).unwrap();

        let mut linker = LinkerConfig::new();
        linker.define(HostFunction::new(
            "env",
            "fail",
            vec![],
            vec![WasmValueType::I32],
            |_params, _results| Err(WasmError::CallError("host error".into())),
        ));

        let instance = module.instantiate_with_linker(&sandbox, &linker).unwrap();
        let result = instance.call("call_fail", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_module() {
        let bytes = wat::parse_str("(module)").unwrap();
        let sandbox = WasmSandbox::new();
        let module = sandbox.compile(&bytes).unwrap();
        assert!(module.exports().is_empty());
        let instance = module.instantiate(&sandbox);
        assert!(instance.is_ok());
    }

    #[test]
    fn test_memory_roundtrip() {
        let bytes = wat::parse_str(
            r#"(module
                (memory (export "mem") 1)
            )"#,
        )
        .unwrap();

        let sandbox = WasmSandbox::new();
        let module = sandbox.compile(&bytes).unwrap();
        let instance = module.instantiate(&sandbox).unwrap();

        let data: Vec<u8> = (0..256).map(|i| i as u8).collect();
        instance.write_memory("mem", 0, &data).unwrap();
        let read_back = instance.read_memory("mem", 0, 256).unwrap();
        assert_eq!(data, read_back);
    }

    #[test]
    fn test_start_function() {
        let bytes = wat::parse_str(
            r#"(module
                (memory (export "memory") 1)
                (func $start
                    i32.const 0
                    i32.const 42
                    i32.store
                )
                (start $start)
            )"#,
        )
        .unwrap();

        let sandbox = WasmSandbox::new();
        let module = sandbox.compile(&bytes).unwrap();
        let instance = module.instantiate(&sandbox).unwrap();

        let data = instance.read_memory("memory", 0, 4).unwrap();
        assert_eq!(data[0], 42);
        assert_eq!(data[1], 0);
        assert_eq!(data[2], 0);
        assert_eq!(data[3], 0);
    }
}
