#[cfg(test)]
use zero_wasm_sandbox::{WasmSandbox, WasmValue};

/// 编译并实例化一个简单的 WASM 模块
#[test]
fn test_wasm_compile_and_call_add() {
    let wat_text = r#"
        (module
            (func $add (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add
            )
            (export "add" (func $add))
        )
    "#;
    let wasm_bytes = wat::parse_str(wat_text).expect("parse WAT");

    let sandbox = WasmSandbox::new();
    let module = sandbox.compile(&wasm_bytes).expect("compile");
    let mut instance = module.instantiate(&sandbox).expect("instantiate");

    let result = instance
        .call("add", &[WasmValue::I32(3), WasmValue::I32(7)])
        .expect("call add");

    assert_eq!(result.len(), 1);
    assert_eq!(result[0], WasmValue::I32(10));
}

/// WASM 模块导出查询
#[test]
fn test_wasm_module_exports() {
    let wat_text = r#"
        (module
            (func $f1 (result i32) i32.const 42)
            (func $f2 (result i32) i32.const 99)
            (export "f1" (func $f1))
            (export "f2" (func $f2))
        )
    "#;
    let wasm_bytes = wat::parse_str(wat_text).expect("parse WAT");

    let sandbox = WasmSandbox::new();
    let module = sandbox.compile(&wasm_bytes).expect("compile");

    let exports = module.exports();
    assert!(exports.contains(&"f1".to_string()), "应导出 f1");
    assert!(exports.contains(&"f2".to_string()), "应导出 f2");
}

/// WASM 内存读写集成
#[test]
fn test_wasm_memory_read_write() {
    let wat_text = r#"
        (module
            (memory (export "mem") 1)
            (func $store (export "store") (param i32 i32)
                local.get 0
                local.get 1
                i32.store
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat_text).expect("parse WAT");

    let sandbox = WasmSandbox::new();
    let module = sandbox.compile(&wasm_bytes).expect("compile");
    let mut instance = module.instantiate(&sandbox).expect("instantiate");

    // 写入值
    instance
        .call("store", &[WasmValue::I32(0), WasmValue::I32(0x4243_4445)])
        .expect("call store");

    // 读取内存
    let data = instance.read_memory("mem", 0, 4).expect("read memory");
    assert_eq!(data.len(), 4);
    // 验证小端字节序
    assert_eq!(data[0], 0x45);
    assert_eq!(data[3], 0x42);
}

/// WASM 调用不存在的导出应返回错误
#[test]
fn test_wasm_call_nonexistent_export() {
    let wat_text = r#"
        (module
            (func $f (export "exists") (result i32) i32.const 1)
        )
    "#;
    let wasm_bytes = wat::parse_str(wat_text).expect("parse WAT");

    let sandbox = WasmSandbox::new();
    let module = sandbox.compile(&wasm_bytes).expect("compile");
    let mut instance = module.instantiate(&sandbox).expect("instantiate");

    let result = instance.call("nonexistent", &[]);
    assert!(result.is_err(), "调用不存在的导出应失败");
}

/// 无效 WASM 二进制应返回编译错误
#[test]
fn test_wasm_invalid_binary() {
    let sandbox = WasmSandbox::new();
    let result = sandbox.compile(&[0x00, 0x01, 0x02, 0x03]);
    assert!(result.is_err(), "无效 WASM 二进制应编译失败");
}
