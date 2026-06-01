//! 高级测试：浮点特殊值、比较运算、燃料单调递减、内存增长、多沙箱独立性

use crate::{HostFunction, LinkerConfig, SandboxConfig, WasmError, WasmSandbox, WasmValue, WasmValueType};

/// 辅助函数：编译 WAT 文本为 WASM 字节
fn wat_to_wasm(wat: &str) -> Vec<u8> {
    wat::parse_str(wat).expect("invalid WAT")
}

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

// =======================================================================
// 新增测试：更多边界条件
// =======================================================================

/// 实例化一个没有任何导出的 WASM 模块，验证实例化成功、导出列表为空，
/// 且所有导出查询方法（has_func、has_memory、has_table）均返回 false。
#[test]
fn test_instantiate_module_with_no_exports() {
    let sandbox = WasmSandbox::new();
    // 模块只包含内部函数和内存，没有任何导出
    let wasm = wat_to_wasm(
        r#"(module
            (func $internal nop)
            (memory 1)
            (start $internal)
        )"#,
    );
    let module = sandbox.compile(&wasm).expect("compile");
    assert!(module.exports().is_empty(), "无导出模块的 exports() 应为空");

    // 实例化应成功（start 函数执行成功）
    let instance = module.instantiate(&sandbox).expect("instantiate");
    assert!(!instance.has_func("anything"), "无导出函数时 has_func 应返回 false");
    assert!(!instance.has_memory("mem"), "未导出内存时 has_memory 应返回 false");
    assert!(!instance.has_table("tbl"), "无表导出时 has_table 应返回 false");
}

/// 调用函数时传入错误类型的参数（例如用 I64 调用期望 I32 参数的函数），
/// wasmi 应因类型不匹配而返回错误。
#[test]
fn test_call_with_wrong_param_type() {
    let sandbox = WasmSandbox::new();
    // 函数期望 i32 参数
    let wasm = wat_to_wasm(
        r#"(module
            (func (export "double") (param i32) (result i32)
                local.get 0 local.get 0 i32.add)
        )"#,
    );
    let module = sandbox.compile(&wasm).expect("compile");
    let mut instance = module.instantiate(&sandbox).expect("instantiate");

    // 传入 I64 参数调用期望 I32 的函数 → 类型不匹配错误
    let result = instance.call("double", &[WasmValue::I64(21)]);
    assert!(result.is_err(), "传入错误类型的参数应返回错误");

    // 传入 F32 参数调用期望 I32 的函数 → 类型不匹配错误
    let result = instance.call("double", &[WasmValue::F32(21.0)]);
    assert!(result.is_err(), "传入 F32 给期望 I32 的函数应返回错误");

    // 传入 F64 参数调用期望 I32 的函数 → 类型不匹配错误
    let result = instance.call("double", &[WasmValue::F64(21.0)]);
    assert!(result.is_err(), "传入 F64 给期望 I32 的函数应返回错误");

    // 正确类型（I32）调用应成功
    let ok = instance.call("double", &[WasmValue::I32(21)]).expect("correct type");
    assert_eq!(ok[0], WasmValue::I32(42), "正确类型调用应返回 42");
}

/// 在没有全局导出变量的模块上调用 get_global_export，应返回 None（对应 false 语义），
/// 验证不会 panic 或返回错误值。
#[test]
fn test_get_global_export_on_module_with_no_globals() {
    let sandbox = WasmSandbox::new();
    // 模块只有函数和内存导出，没有全局变量导出
    let wasm = wat_to_wasm(
        r#"(module
            (func (export "answer") (result i32) i32.const 42)
            (memory (export "mem") 1)
        )"#,
    );
    let module = sandbox.compile(&wasm).expect("compile");
    let instance = module.instantiate(&sandbox).expect("instantiate");

    // 查询不存在的全局导出应返回 None
    assert!(
        instance.get_global_export("counter").is_none(),
        "模块没有全局导出时 get_global_export 应返回 None"
    );
    assert!(
        instance.get_global_export("g").is_none(),
        "任意名称的全局导出查询应返回 None"
    );
    assert!(
        instance.get_global_export("").is_none(),
        "空字符串全局导出查询应返回 None"
    );

    // 确认函数和内存导出不受影响
    assert!(instance.has_func("answer"), "函数导出应仍可查询");
    assert!(instance.has_memory("mem"), "内存导出应仍可查询");
}

/// 设置燃料为 0 后调用任何函数，应立即返回 FuelExhausted 错误，
/// 验证零燃料状态下沙箱完全阻止执行。
#[test]
fn test_set_fuel_zero_then_call_returns_error() {
    let config = SandboxConfig::new().consume_fuel(true);
    let sandbox = WasmSandbox::with_config(config);
    let wasm = wat_to_wasm(
        r#"(module
            (func (export "simple") (result i32) i32.const 1)
        )"#,
    );
    let module = sandbox.compile(&wasm).expect("compile");
    let mut instance = module.instantiate(&sandbox).expect("instantiate");

    // 设置燃料为 0
    instance.set_fuel(0).expect("set_fuel to 0");
    // 确认 get_fuel 返回 0
    assert_eq!(instance.get_fuel().expect("get_fuel"), 0, "燃料应为 0");

    // 调用最简单的函数也应立即返回 FuelExhausted
    let result = instance.call("simple", &[]);
    assert!(
        matches!(result, Err(WasmError::FuelExhausted)),
        "燃料为 0 时调用应返回 FuelExhausted"
    );

    // 补充燃料后调用应成功
    instance.set_fuel(100).expect("refuel");
    let r = instance.call("simple", &[]).expect("call after refuel");
    assert_eq!(r[0], WasmValue::I32(1), "补充燃料后调用应成功返回 1");
}

/// 对同一内存导出多次调用 read_memory，验证每次返回的数据完全一致，
/// 确认多次读取操作返回的是同一底层内存的快照（逻辑上同一引用）。
#[test]
fn test_multiple_read_memory_calls_return_same_data() {
    let sandbox = WasmSandbox::new();
    let wasm = wat_to_wasm(
        r#"(module
            (memory (export "mem") 1)
        )"#,
    );
    let module = sandbox.compile(&wasm).expect("compile");
    let mut instance = module.instantiate(&sandbox).expect("instantiate");

    // 写入测试数据
    instance.write_memory("mem", 0, b"HELLO").expect("write");

    // 连续多次读取同一区域
    let read1 = instance.read_memory("mem", 0, 5).expect("read 1");
    let read2 = instance.read_memory("mem", 0, 5).expect("read 2");
    let read3 = instance.read_memory("mem", 0, 5).expect("read 3");

    assert_eq!(read1, read2, "第一次和第二次读取应返回相同数据");
    assert_eq!(read2, read3, "第二次和第三次读取应返回相同数据");
    assert_eq!(&read1, b"HELLO", "读取数据应与写入一致");

    // memory_size 多次调用也应返回相同值
    let size1 = instance.memory_size("mem").expect("size 1");
    let size2 = instance.memory_size("mem").expect("size 2");
    assert_eq!(size1, size2, "多次 memory_size 应返回相同大小");
    assert_eq!(size1, 65536, "一页内存应为 65536 字节");
}

// ── 新增边界测试 ──

/// 测试 write_memory 后 read_memory 验证覆盖写入正确。
/// 先写入 ABC，再在同一偏移写入 XY，验证只有前两字节被覆盖。
#[test]
fn test_write_memory_partial_overwrite() {
    let sandbox = WasmSandbox::new();
    let wasm = wat_to_wasm(r#"(module (memory (export "mem") 1))"#);
    let module = sandbox.compile(&wasm).expect("compile");
    let mut instance = module.instantiate(&sandbox).expect("instantiate");

    instance.write_memory("mem", 0, b"ABC").expect("write ABC");
    instance.write_memory("mem", 0, b"XY").expect("write XY");
    let data = instance.read_memory("mem", 0, 3).expect("read");
    assert_eq!(&data, b"XYC", "部分覆盖后应为 XYC");
}

/// 测试 WasmValue I32 极端值 Display 格式化不 panic。
#[test]
fn test_wasm_value_i32_extreme_display() {
    let v1 = WasmValue::I32(i32::MIN);
    let v2 = WasmValue::I32(i32::MAX);
    let v3 = WasmValue::I32(0);
    assert!(!format!("{v1}").is_empty(), "I32::MIN Display 不应为空");
    assert!(!format!("{v2}").is_empty(), "I32::MAX Display 不应为空");
    assert!(!format!("{v3}").is_empty(), "I32(0) Display 不应为空");
}

/// 测试 WasmValue PartialEq 跨类型比较返回 false。
#[test]
fn test_wasm_value_cross_type_equality() {
    assert_ne!(WasmValue::I32(0), WasmValue::I64(0), "I32(0) != I64(0)");
    assert_ne!(WasmValue::F32(1.0), WasmValue::F64(1.0), "F32(1.0) != F64(1.0)");
    assert_eq!(WasmValue::I32(42), WasmValue::I32(42), "同值应相等");
}

/// 测试 LinkerConfig 多次 define 同名函数会追加而非覆盖。
#[test]
fn test_linker_config_duplicate_function_appends() {
    let mut linker = LinkerConfig::new();
    linker.define(HostFunction::new(
        "env",
        "add",
        vec![],
        vec![],
        |_: &[WasmValue], out: &mut Vec<WasmValue>| {
            out.push(WasmValue::I32(1));
            Ok(())
        },
    ));
    linker.define(HostFunction::new(
        "env",
        "add",
        vec![],
        vec![],
        |_: &[WasmValue], out: &mut Vec<WasmValue>| {
            out.push(WasmValue::I32(2));
            Ok(())
        },
    ));
    let fns = linker.functions();
    // 同名函数会追加（LinkerConfig 不做去重）
    let count = fns.iter().filter(|f| f.module == "env" && f.name == "add").count();
    assert_eq!(count, 2, "同名函数应追加为 2 个");
}

/// 测试 SandboxConfig 默认 consume_fuel 为 false。
#[test]
fn test_sandbox_config_default_no_fuel() {
    let config = SandboxConfig::default();
    assert!(!config.consume_fuel, "默认 consume_fuel 应为 false");
}

// ── 新增边界测试（第二轮） ──

/// 测试 has_memory 显式 true/false 判断。
#[test]
fn test_has_memory_explicit() {
    let sandbox = WasmSandbox::new();
    let wasm = wat_to_wasm(r#"(module (memory (export "mem") 1))"#);
    let module = sandbox.compile(&wasm).expect("compile");
    let instance = module.instantiate(&sandbox).expect("instantiate");

    assert!(instance.has_memory("mem"), "已导出内存 'mem' 应返回 true");
    assert!(!instance.has_memory("nonexistent"), "未导出内存应返回 false");
}

/// 测试 get_global_export 对 i64 全局变量。
#[test]
fn test_get_global_export_i64() {
    let sandbox = WasmSandbox::new();
    let wasm = wat_to_wasm(r#"(module (global (export "g") i64 (i64.const 123456789012)))"#);
    let module = sandbox.compile(&wasm).expect("compile");
    let instance = module.instantiate(&sandbox).expect("instantiate");

    let val = instance.get_global_export("g").expect("global");
    assert_eq!(val, WasmValue::I64(123456789012_i64), "i64 全局变量值应正确");
}

/// 测试 get_global_export 对 f32 全局变量。
#[test]
fn test_get_global_export_f32() {
    let sandbox = WasmSandbox::new();
    let wasm = wat_to_wasm(r#"(module (global (export "g") f32 (f32.const 3.14)))"#);
    let module = sandbox.compile(&wasm).expect("compile");
    let instance = module.instantiate(&sandbox).expect("instantiate");

    let val = instance.get_global_export("g").expect("global");
    match val {
        WasmValue::F32(_) => {} // f32 精度可接受
        other => panic!("期望 F32，得到 {other:?}"),
    }
}

/// 测试 get_global_export 对 f64 全局变量。
#[test]
fn test_get_global_export_f64() {
    let sandbox = WasmSandbox::new();
    let wasm = wat_to_wasm(r#"(module (global (export "g") f64 (f64.const 2.718281828)))"#);
    let module = sandbox.compile(&wasm).expect("compile");
    let instance = module.instantiate(&sandbox).expect("instantiate");

    let val = instance.get_global_export("g").expect("global");
    match val {
        WasmValue::F64(v) => {
            assert!((v - 2.718281828).abs() < 1e-6, "f64 全局变量值应接近 2.718");
        }
        other => panic!("期望 F64，得到 {other:?}"),
    }
}

/// 测试 WasmValue Display 特殊浮点值不 panic。
#[test]
fn test_wasm_value_float_special_display() {
    let nan = format!("{}", WasmValue::F32(f32::NAN));
    let inf = format!("{}", WasmValue::F32(f32::INFINITY));
    let neg_inf = format!("{}", WasmValue::F64(f64::NEG_INFINITY));
    assert!(!nan.is_empty(), "F32(NaN) Display 不应为空");
    assert!(!inf.is_empty(), "F32(Inf) Display 不应为空");
    assert!(!neg_inf.is_empty(), "F64(-Inf) Display 不应为空");
}

/// 测试 exports 列出混合类型导出（函数+内存+全局+表）。
#[test]
fn test_exports_mixed_types() {
    let sandbox = WasmSandbox::new();
    let wasm = wat_to_wasm(
        r#"(module
            (func (export "fn") (result i32) i32.const 1)
            (memory (export "mem") 1)
            (global (export "g") i32 (i32.const 42))
            (table (export "tbl") 1 funcref)
        )"#,
    );
    let module = sandbox.compile(&wasm).expect("compile");
    let exports = module.exports();
    assert!(exports.contains(&"fn".to_string()), "应包含函数导出");
    assert!(exports.contains(&"mem".to_string()), "应包含内存导出");
    assert!(exports.contains(&"g".to_string()), "应包含全局导出");
    assert!(exports.contains(&"tbl".to_string()), "应包含表导出");
}

/// 测试 memory.grow 后 memory_size 反映新大小。
#[test]
fn test_memory_size_after_grow() {
    let sandbox = WasmSandbox::new();
    // 使用分开的函数：grow 做增长，size 查询页数
    let wasm = wat_to_wasm(
        r#"(module
            (memory (export "mem") 1)
            (func (export "grow")
                i32.const 1
                memory.grow
                drop
            )
            (func (export "size") (result i32)
                memory.size
            )
        )"#,
    );
    let module = sandbox.compile(&wasm).expect("compile");
    let mut instance = module.instantiate(&sandbox).expect("instantiate");

    // 先确认初始大小为 1 页
    let before = instance.memory_size("mem").expect("size before");
    assert_eq!(before, 65536, "初始应为 1 页");

    // 调用 grow 函数
    instance.call("grow", &[]).expect("call grow");

    // 从 host 侧验证增长后大小
    let after = instance.memory_size("mem").expect("size after");
    assert_eq!(after, 2 * 65536, "host 侧 memory_size 应反映增长后大小");
}

/// 测试 WasmError::LinkError 场景（链接器注册失败）。
#[test]
fn test_link_error_variant() {
    // 构建一个需要导入的模块，但链接器注册了一个错误的签名
    let sandbox = WasmSandbox::new();
    let wasm = wat_to_wasm(
        r#"(module
            (import "env" "add" (func $add (param i32 i32) (result i32)))
            (func (export "call_add") (result i32)
                i32.const 1
                i32.const 2
                call $add
            )
        )"#,
    );
    let module = sandbox.compile(&wasm).expect("compile");

    // 定义一个签名不匹配的 host 函数（0 参数 vs 需要 2 参数）
    let mut linker = LinkerConfig::new();
    linker.define(HostFunction::new(
        "env",
        "add",
        vec![],
        vec![WasmValueType::I32],
        |_: &[WasmValue], _: &mut Vec<WasmValue>| Ok(()),
    ));

    let result = module.instantiate_with_linker(&sandbox, &linker);
    assert!(result.is_err(), "签名不匹配应导致实例化失败");
}

/// 测试 has_table 显式 true/false 判断。
#[test]
fn test_has_table_explicit() {
    let sandbox = WasmSandbox::new();
    let wasm_with = wat_to_wasm(r#"(module (table (export "t") 1 funcref))"#);
    let wasm_without = wat_to_wasm(r#"(module)"#);

    let module_with = sandbox.compile(&wasm_with).expect("compile");
    let instance_with = module_with.instantiate(&sandbox).expect("instantiate");
    assert!(instance_with.has_table("t"), "已导出表应返回 true");
    assert!(!instance_with.has_table("nonexistent"), "未导出表应返回 false");

    let module_without = sandbox.compile(&wasm_without).expect("compile");
    let instance_without = module_without.instantiate(&sandbox).expect("instantiate");
    assert!(!instance_without.has_table("any"), "无表模块 has_table 应返回 false");
}

/// 测试多次调用同一函数结果一致性。
#[test]
fn test_repeated_call_consistency() {
    let sandbox = WasmSandbox::new();
    let wasm = wat_to_wasm(r#"(module (func (export "id") (param i32) (result i32) local.get 0))"#);
    let module = sandbox.compile(&wasm).expect("compile");
    let mut instance = module.instantiate(&sandbox).expect("instantiate");

    for i in 0..10 {
        let result = instance.call("id", &[WasmValue::I32(i)]).expect("call");
        assert_eq!(result[0], WasmValue::I32(i), "重复调用结果应一致");
    }
}

// =======================================================================
// 新增边界测试：燃料禁用时 get_fuel、u64::MAX 燃料、内存边界、
//               I64 Display、SandboxConfig 链式调用、空字符串函数名、
//               多沙箱独立性、write/read 往返、start 函数 trap
// =======================================================================

/// 测试 get_fuel 在未启用燃料计量时返回 Err。
/// 已有 test_fuel_disabled_by_default 测试了 set_fuel 报错，
/// 此测试专门验证 get_fuel 在燃料禁用时的错误行为。
#[test]
fn test_get_fuel_when_fuel_disabled() {
    let sandbox = WasmSandbox::new();
    let wasm = wat_to_wasm(
        r#"(module
            (func (export "answer") (result i32) i32.const 42)
        )"#,
    );
    let module = sandbox.compile(&wasm).expect("compile");
    let instance = module.instantiate(&sandbox).expect("instantiate");

    // 燃料计量未启用时 get_fuel 应返回错误
    let result = instance.get_fuel();
    assert!(result.is_err(), "未启用燃料计量时 get_fuel 应返回 Err");
}

/// 测试 set_fuel(u64::MAX) 后 get_fuel() 返回相同值。
/// 验证最大燃料值不会溢出或被截断。
#[test]
fn test_set_get_fuel_max() {
    let config = SandboxConfig::new().consume_fuel(true);
    let sandbox = WasmSandbox::with_config(config);
    let wasm = wat_to_wasm(
        r#"(module
            (func (export "answer") (result i32) i32.const 42)
        )"#,
    );
    let module = sandbox.compile(&wasm).expect("compile");
    let mut instance = module.instantiate(&sandbox).expect("instantiate");

    instance.set_fuel(u64::MAX).expect("set_fuel u64::MAX");
    let fuel = instance.get_fuel().expect("get_fuel");
    assert_eq!(fuel, u64::MAX, "get_fuel 应返回 u64::MAX");
}

/// 测试 read_memory 在内存边界（offset = memory_size, len = 0），
/// 应返回空切片（Some(vec![])），而非 None。
#[test]
fn test_read_memory_at_exact_boundary() {
    let sandbox = WasmSandbox::new();
    let wasm = wat_to_wasm(
        r#"(module
            (memory (export "mem") 1)
        )"#,
    );
    let module = sandbox.compile(&wasm).expect("compile");
    let instance = module.instantiate(&sandbox).expect("instantiate");

    let size = instance.memory_size("mem").expect("memory_size");
    // offset == memory_size, len == 0 → 边界内，应返回 Some(空)
    let result = instance.read_memory("mem", size, 0);
    assert!(result.is_some(), "边界处读取 0 字节应返回 Some");
    assert!(result.unwrap().is_empty(), "边界处读取 0 字节应返回空 vec");
}

/// 测试 write_memory 向最后一个有效字节写入数据，
/// 验证偏移量为 memory_size - 1 时写入 1 字节成功。
#[test]
fn test_write_memory_last_valid_byte() {
    let sandbox = WasmSandbox::new();
    let wasm = wat_to_wasm(
        r#"(module
            (memory (export "mem") 1)
        )"#,
    );
    let module = sandbox.compile(&wasm).expect("compile");
    let mut instance = module.instantiate(&sandbox).expect("instantiate");

    let size = instance.memory_size("mem").expect("memory_size");
    // 写入到最后一个字节
    let result = instance.write_memory("mem", size - 1, &[0xFF]);
    assert!(result.is_ok(), "写入最后一个有效字节应成功");

    // 验证写入的数据正确
    let data = instance.read_memory("mem", size - 1, 1).expect("read back");
    assert_eq!(data, vec![0xFF], "读回的数据应为 0xFF");
}

/// 测试 WasmValue::I64 的 Display 格式化，
/// 验证负值、零值和 i64::MAX 的格式化结果完全匹配 "i64(...)" 模式。
#[test]
fn test_wasm_value_i64_display_exact() {
    assert_eq!(WasmValue::I64(-1).to_string(), "i64(-1)");
    assert_eq!(WasmValue::I64(0).to_string(), "i64(0)");
    assert_eq!(WasmValue::I64(i64::MAX).to_string(), format!("i64({})", i64::MAX));
}

/// 测试 SandboxConfig::consume_fuel 链式调用，
/// 验证 .consume_fuel(true).consume_fuel(false) 最终结果为禁用燃料。
#[test]
fn test_sandbox_config_consume_fuel_chaining() {
    let config = SandboxConfig::new().consume_fuel(true).consume_fuel(false);
    assert!(!config.is_consume_fuel(), "链式调用后 consume_fuel 应为 false");

    // 反向链式：false → true
    let config2 = SandboxConfig::new().consume_fuel(false).consume_fuel(true);
    assert!(config2.is_consume_fuel(), "反向链式调用后 consume_fuel 应为 true");
}

/// 测试 has_memory 使用函数导出名查询，应返回 false（不会把函数误认为内存）。
#[test]
fn test_has_memory_with_function_export_name() {
    let sandbox = WasmSandbox::new();
    // 模块有一个函数导出但无内存导出
    let wasm = wat_to_wasm(
        r#"(module
            (func (export "my_func") nop)
        )"#,
    );
    let module = sandbox.compile(&wasm).expect("compile");
    let instance = module.instantiate(&sandbox).expect("instantiate");

    assert!(
        !instance.has_memory("my_func"),
        "函数导出名不应被 has_memory 匹配为内存"
    );
    assert!(instance.has_func("my_func"), "函数导出应被 has_func 找到");
}

/// 测试空字符串作为函数名传入 call 和 has_func，
/// 验证不会 panic，而是返回正常的错误/false。
#[test]
fn test_empty_string_function_name() {
    let sandbox = WasmSandbox::new();
    let wasm = wat_to_wasm(
        r#"(module
            (func (export "exists") nop)
        )"#,
    );
    let module = sandbox.compile(&wasm).expect("compile");
    let mut instance = module.instantiate(&sandbox).expect("instantiate");

    // has_func("") 应返回 false（不会 panic）
    assert!(!instance.has_func(""), "空字符串函数名 has_func 应返回 false");

    // call("") 应返回 ExportNotFound 错误（不会 panic）
    let result = instance.call("", &[]);
    assert!(result.is_err(), "空字符串 call 应返回错误");
    if let Err(WasmError::ExportNotFound { name }) = result {
        assert_eq!(name, "", "错误名应为空字符串");
    } else {
        panic!("期望 ExportNotFound 错误，实际: {result:?}");
    }
}

/// 测试两个独立的沙箱实例（不同 WasmSandbox 对象），各自编译并实例化相同模块，
/// 验证两个实例完全独立，互不影响。
#[test]
fn test_multiple_independent_sandbox_instances() {
    let sandbox_a = WasmSandbox::new();
    let sandbox_b = WasmSandbox::new();

    let wasm = wat_to_wasm(
        r#"(module
            (global $counter (export "counter") (mut i32) (i32.const 0))
            (func (export "inc") (result i32)
                global.get $counter
                i32.const 1
                i32.add
                global.set $counter
                global.get $counter)
        )"#,
    );

    let module_a = sandbox_a.compile(&wasm).expect("compile A");
    let module_b = sandbox_b.compile(&wasm).expect("compile B");

    let mut inst_a = module_a.instantiate(&sandbox_a).expect("instantiate A");
    let mut inst_b = module_b.instantiate(&sandbox_b).expect("instantiate B");

    // 实例 A 递增 3 次
    for expected in [1, 2, 3] {
        let r = inst_a.call("inc", &[]).expect("A inc");
        assert_eq!(r[0], WasmValue::I32(expected), "实例 A 递增应为 {expected}");
    }

    // 实例 B 独立递增 1 次
    let r = inst_b.call("inc", &[]).expect("B inc");
    assert_eq!(r[0], WasmValue::I32(1), "实例 B 应从 0 开始独立计数");
}

/// 测试 write_memory 后 read_memory 往返一致性。
/// 写入一组已知字节序列到非零偏移，读回后验证完全匹配。
#[test]
fn test_write_memory_read_memory_round_trip() {
    let sandbox = WasmSandbox::new();
    let wasm = wat_to_wasm(
        r#"(module
            (memory (export "mem") 1)
        )"#,
    );
    let module = sandbox.compile(&wasm).expect("compile");
    let mut instance = module.instantiate(&sandbox).expect("instantiate");

    // 写入一组已知字节到偏移 256
    let original: Vec<u8> = (0..16).map(|i| (i * 17 + 0xAB) as u8).collect();
    instance.write_memory("mem", 256, &original).expect("write");

    // 读回并验证
    let read_back = instance.read_memory("mem", 256, 16).expect("read");
    assert_eq!(read_back, original, "读回的数据应与写入完全一致");

    // 写入前后的区域应不受影响
    let before = instance.read_memory("mem", 255, 1).expect("read before");
    assert_eq!(before, [0x00], "写入区域前一个字节应为 0");
    let after = instance.read_memory("mem", 272, 1).expect("read after");
    assert_eq!(after, [0x00], "写入区域后一个字节应为 0");
}

/// 测试带 start 函数的 WASM 模块在 start 函数 trap 时的行为。
/// 编译应成功（语法正确），但实例化时应返回 InstantiationError
/// （因为 start 函数执行了 unreachable 导致 trap）。
#[test]
fn test_module_with_start_function_that_traps() {
    let sandbox = WasmSandbox::new();
    let wasm = wat_to_wasm(
        r#"(module
            (func $start unreachable)
            (start $start)
        )"#,
    );
    // 编译应成功（语法正确）
    let module = sandbox.compile(&wasm).expect("compile");

    // 实例化应失败（start 函数执行 trap）
    let result = module.instantiate(&sandbox);
    assert!(result.is_err(), "start 函数 trap 时实例化应返回错误");
    if let Err(WasmError::InstantiationError(msg)) = result {
        assert!(
            msg.contains("unreachable") || msg.contains("trap"),
            "错误信息应包含 trap 相关描述，实际: {msg}"
        );
    } else {
        panic!("期望 InstantiationError，实际: Ok");
    }
}
