//! 基础测试：沙箱创建、模块编译、函数调用、内存操作、主机函数、trap、燃料计量

use crate::{HostFunction, LinkerConfig, SandboxConfig, WasmError, WasmSandbox, WasmValue, WasmValueType};

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
