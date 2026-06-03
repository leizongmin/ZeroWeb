//! 类型定义测试（不依赖 wasmi feature 的纯类型测试）。

use crate::*;

// ── WasmValue 测试 ─────────────────────────────────────────

#[test]
fn test_wasm_value_i32_equality() {
    assert_eq!(WasmValue::I32(42), WasmValue::I32(42));
    assert_ne!(WasmValue::I32(42), WasmValue::I32(0));
    assert_ne!(WasmValue::I32(42), WasmValue::I64(42));
}

#[test]
fn test_wasm_value_i64_equality() {
    assert_eq!(WasmValue::I64(123456789), WasmValue::I64(123456789));
    assert_ne!(WasmValue::I64(-1), WasmValue::I64(1));
}

#[test]
fn test_wasm_value_f32_equality() {
    assert_eq!(WasmValue::F32(3.14), WasmValue::F32(3.14));
    assert_ne!(WasmValue::F32(1.0), WasmValue::F64(1.0));
}

#[test]
fn test_wasm_value_f64_equality() {
    assert_eq!(WasmValue::F64(2.71828), WasmValue::F64(2.71828));
}

#[test]
fn test_wasm_value_clone() {
    let v = WasmValue::I32(99);
    let cloned = v.clone();
    assert_eq!(v, cloned);
}

#[test]
fn test_wasm_value_display_i32() {
    assert_eq!(format!("{}", WasmValue::I32(42)), "i32(42)");
    assert_eq!(format!("{}", WasmValue::I32(-1)), "i32(-1)");
    assert_eq!(format!("{}", WasmValue::I32(0)), "i32(0)");
}

#[test]
fn test_wasm_value_display_i64() {
    assert_eq!(format!("{}", WasmValue::I64(100)), "i64(100)");
    assert_eq!(format!("{}", WasmValue::I64(-999)), "i64(-999)");
}

#[test]
fn test_wasm_value_display_f32() {
    let s = format!("{}", WasmValue::F32(1.5));
    assert!(s.starts_with("f32("));
    assert!(s.ends_with(")"));
}

#[test]
fn test_wasm_value_display_f64() {
    let s = format!("{}", WasmValue::F64(3.14));
    assert!(s.starts_with("f64("));
    assert!(s.ends_with(")"));
}

#[test]
fn test_wasm_value_debug() {
    let v = WasmValue::I32(7);
    let debug = format!("{:?}", v);
    assert!(debug.contains("I32"));
}

// ── WasmValueType 测试 ────────────────────────────────────

#[test]
fn test_wasm_value_type_equality() {
    assert_eq!(WasmValueType::I32, WasmValueType::I32);
    assert_eq!(WasmValueType::I64, WasmValueType::I64);
    assert_eq!(WasmValueType::F32, WasmValueType::F32);
    assert_eq!(WasmValueType::F64, WasmValueType::F64);
    assert_ne!(WasmValueType::I32, WasmValueType::I64);
    assert_ne!(WasmValueType::F32, WasmValueType::F64);
}

#[test]
fn test_wasm_value_type_copy() {
    let a = WasmValueType::I32;
    let b = a; // Copy
    assert_eq!(a, b);
}

#[test]
fn test_wasm_value_type_debug() {
    assert!(format!("{:?}", WasmValueType::I32).contains("I32"));
    assert!(format!("{:?}", WasmValueType::I64).contains("I64"));
    assert!(format!("{:?}", WasmValueType::F32).contains("F32"));
    assert!(format!("{:?}", WasmValueType::F64).contains("F64"));
}

// ── WasmError 测试 ─────────────────────────────────────────

#[test]
fn test_wasm_error_display() {
    let e = WasmError::InvalidBinary("bad bytes".into());
    assert!(e.to_string().contains("bad bytes"));

    let e = WasmError::ExportNotFound { name: "foo".into() };
    assert!(e.to_string().contains("foo"));

    let e = WasmError::CallError("fail".into());
    assert!(e.to_string().contains("fail"));

    let e = WasmError::MemoryError("oom".into());
    assert!(e.to_string().contains("oom"));

    let e = WasmError::InstantiationError("nope".into());
    assert!(e.to_string().contains("nope"));

    let e = WasmError::LinkError("unlinkable".into());
    assert!(e.to_string().contains("unlinkable"));

    let e = WasmError::FuelExhausted;
    assert!(e.to_string().contains("fuel"));
}

#[test]
fn test_wasm_error_debug() {
    let e = WasmError::InvalidBinary("test".into());
    let debug = format!("{:?}", e);
    assert!(debug.contains("InvalidBinary"));
}

#[test]
fn test_wasm_error_variants() {
    // 确保 Debug + Display 不 panic
    let _ = format!("{:?}", WasmError::InvalidBinary(String::new()));
    let _ = format!("{:?}", WasmError::ExportNotFound { name: String::new() });
    let _ = format!("{:?}", WasmError::CallError(String::new()));
    let _ = format!("{:?}", WasmError::MemoryError(String::new()));
    let _ = format!("{:?}", WasmError::InstantiationError(String::new()));
    let _ = format!("{:?}", WasmError::LinkError(String::new()));
    let _ = format!("{:?}", WasmError::FuelExhausted);
}

// ── SandboxConfig 测试 ─────────────────────────────────────

#[test]
fn test_sandbox_config_default() {
    let config = SandboxConfig::default();
    assert!(!config.consume_fuel);
}

#[test]
fn test_sandbox_config_new() {
    let config = SandboxConfig::new();
    assert!(!config.consume_fuel);
}

#[test]
fn test_sandbox_config_consume_fuel_enable() {
    let config = SandboxConfig::new().consume_fuel(true);
    assert!(config.is_consume_fuel());
}

#[test]
fn test_sandbox_config_consume_fuel_disable() {
    let config = SandboxConfig::new().consume_fuel(false);
    assert!(!config.is_consume_fuel());
}

#[test]
fn test_sandbox_config_debug() {
    let config = SandboxConfig::default();
    let debug = format!("{:?}", config);
    assert!(debug.contains("consume_fuel"));
}

#[test]
fn test_sandbox_config_clone() {
    let config = SandboxConfig::new().consume_fuel(true);
    let cloned = config.clone();
    assert!(cloned.is_consume_fuel());
}

// ── LinkerConfig 测试 ──────────────────────────────────────

#[test]
fn test_linker_config_default() {
    let config = LinkerConfig::default();
    assert!(config.functions().is_empty());
}

#[test]
fn test_linker_config_new() {
    let config = LinkerConfig::new();
    assert!(config.functions().is_empty());
}

#[test]
fn test_linker_config_define() {
    let mut config = LinkerConfig::new();
    let func = HostFunction::new("env", "log", vec![WasmValueType::I32], vec![], |_params, _results| {
        Ok(())
    });
    config.define(func);
    assert_eq!(config.functions().len(), 1);
    assert_eq!(config.functions()[0].module, "env");
    assert_eq!(config.functions()[0].name, "log");
}

#[test]
fn test_linker_config_define_multiple() {
    let mut config = LinkerConfig::new();
    for i in 0..5 {
        let func = HostFunction::new(
            "env",
            format!("fn_{i}"),
            vec![],
            vec![WasmValueType::I32],
            |_params, _results| Ok(()),
        );
        config.define(func);
    }
    assert_eq!(config.functions().len(), 5);
}

#[test]
fn test_linker_config_clone() {
    let mut config = LinkerConfig::new();
    config.define(HostFunction::new("m", "f", vec![], vec![], |_, _| Ok(())));
    let cloned = config.clone();
    assert_eq!(cloned.functions().len(), 1);
}

#[test]
fn test_linker_config_debug() {
    let config = LinkerConfig::new();
    let debug = format!("{:?}", config);
    assert!(debug.contains("functions"));
}

// ── HostFunction 测试 ──────────────────────────────────────

#[test]
fn test_host_function_new() {
    let func = HostFunction::new(
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
    );
    assert_eq!(func.module, "env");
    assert_eq!(func.name, "add");
    assert_eq!(func.params.len(), 2);
    assert_eq!(func.results.len(), 1);
}

#[test]
fn test_host_function_debug() {
    let func = HostFunction::new("mod", "fn", vec![], vec![], |_, _| Ok(()));
    let debug = format!("{:?}", func);
    assert!(debug.contains("mod"));
    assert!(debug.contains("fn"));
}

#[test]
fn test_host_function_clone() {
    let func = HostFunction::new("env", "hello", vec![], vec![], |_, _| Ok(()));
    let cloned = func.clone();
    assert_eq!(cloned.module, "env");
    assert_eq!(cloned.name, "hello");
}

#[test]
fn test_host_function_invoke() {
    let func = HostFunction::new(
        "env",
        "double",
        vec![WasmValueType::I32],
        vec![WasmValueType::I32],
        |params, results| {
            if let WasmValue::I32(v) = params[0] {
                results.push(WasmValue::I32(v * 2));
            }
            Ok(())
        },
    );
    let mut results = vec![];
    (func.func)(&[WasmValue::I32(21)], &mut results).unwrap();
    assert_eq!(results, vec![WasmValue::I32(42)]);
}

#[test]
fn test_host_function_invoke_error() {
    let func = HostFunction::new("env", "fail", vec![], vec![], |_, _| {
        Err(WasmError::CallError("intentional".into()))
    });
    let result = (func.func)(&[], &mut vec![]);
    assert!(result.is_err());
}

#[test]
fn test_host_function_with_f64() {
    let func = HostFunction::new(
        "math",
        "sqrt_approx",
        vec![WasmValueType::F64],
        vec![WasmValueType::F64],
        |params, results| {
            if let WasmValue::F64(v) = params[0] {
                results.push(WasmValue::F64(v.sqrt()));
            }
            Ok(())
        },
    );
    let mut results = vec![];
    (func.func)(&[WasmValue::F64(4.0)], &mut results).unwrap();
    assert_eq!(results.len(), 1);
    if let WasmValue::F64(v) = results[0] {
        assert!((v - 2.0).abs() < f64::EPSILON);
    } else {
        panic!("expected F64 result");
    }
}
