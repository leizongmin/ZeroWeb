//! 模块与实例测试：内存操作、燃料计量、主机函数参数传递、全局导出、多实例

use crate::{HostFunction, LinkerConfig, SandboxConfig, WasmError, WasmSandbox, WasmValue, WasmValueType};

/// 辅助函数：编译 WAT 文本为 WASM 字节
fn wat_to_wasm(wat: &str) -> Vec<u8> {
    wat::parse_str(wat).expect("invalid WAT")
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
    // step(n) 调用 host_step(n)，host_step 返回 n-1，
    // WASM 侧循环调用 host_step 直到 n=0，验证燃料被正确消耗。
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
