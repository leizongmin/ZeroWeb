//! R2280 + CSS Values 4 §10.5：individual transform 属性（rotate）`<angle>` 文法位
//! 接受顶层 L4 数学函数——含角度产出函数（asin/acos/atan/atan2）的式子求值结果为
//! 弧度，转度后入 RotateZ/RotateX/RotateY/Rotate3d；非法输入（|x|>1、长度叶、
//! 混型）维持拒绝。number 型别判定（[`calc_expr_is_number`]）拒角度产出函数。
//! driving: wptfyi-css DC-2 values trig 桶（WPT css-values acos-asin-atan-atan2-*
//! 以 `rotate: acos(1)` 形态驱动，wptfyi-90-plan DC-2）。

use crate::values::{
    BinaryMathOp, CalcExpr, TransformFunction, UnaryMathOp, calc_expr_has_angle_result, calc_expr_is_number,
    parse_individual_rotate, parse_individual_scale,
};

#[test]
fn r2280_rotate_bare_inverse_trig() {
    // acos(1) = 0rad = 0deg；asin(0) = atan(0) = 0；atan2(0,0) = 0。
    let cases = [
        ("acos(1)", 0.0_f64),
        ("asin(0)", 0.0),
        ("atan(0)", 0.0),
        ("atan2(0, 0)", 0.0),
    ];
    for (value, expected) in cases {
        let result = parse_individual_rotate(value);
        assert!(result.is_some(), "{value} 应解析成功");
        assert!(
            matches!(result, Some(TransformFunction::RotateZ(a)) if (a - expected).abs() < 1e-9),
            "{value} 实际 {:?}",
            result
        );
    }
}

#[test]
fn r2280_rotate_calc_wrapped_and_axis_form() {
    // calc() 包裹与轴形式同一求值路径。
    let result = parse_individual_rotate("calc(acos(1))");
    assert!(
        matches!(result, Some(TransformFunction::RotateZ(a)) if a.abs() < 1e-9),
        "calc(acos(1)) 实际 {:?}",
        result
    );
    let result = parse_individual_rotate("x acos(1)");
    assert!(
        matches!(result, Some(TransformFunction::RotateX(a)) if a.abs() < 1e-9),
        "x acos(1) 实际 {:?}",
        result
    );
    // 四元组（<number>{3} <angle>）角度位同接受。
    let result = parse_individual_rotate("1 0 0 acos(1)");
    assert!(
        matches!(result, Some(TransformFunction::Rotate3d(1.0, 0.0, 0.0, a)) if a.abs() < 1e-9),
        "rotate3d + acos(1) 实际 {:?}",
        result
    );
}

#[test]
fn r2280_rotate_pi_constant_trig_input() {
    // sin(pi/2) = 1 → asin(1) = π/2 rad = 90deg。
    let result = parse_individual_rotate("asin(sin(pi/2))");
    assert!(result.is_some(), "asin(sin(pi/2)) 应解析成功");
    if let Some(TransformFunction::RotateZ(a)) = result {
        assert!((a - 90.0).abs() < 1e-6, "asin(sin(pi/2)) 应为 90deg，实际 {a}");
    }
}

#[test]
fn r2280_rotate_invalid_inputs_rejected() {
    // 域外 |x|>1：r2320 起 NaN 结果合法（CSS Values 4 §10.10 非 invalid，
    // serialize `calc(NaN * 1deg)`——原 IACVT 口径随 §10.10 对齐作废）。
    assert!(parse_individual_rotate("acos(2)").is_some(), "acos(2) NaN 域外合法");
    // 长度叶作 acos 输入无效（acos 接受 <number>）。
    assert!(parse_individual_rotate("acos(1px)").is_none(), "acos(1px) 应拒绝");
    // 角度叶作 asin/acos/atan 输入无效（实参仅 <number>，字符串面门）。
    assert!(parse_individual_rotate("acos(45deg)").is_none(), "acos(45deg) 应拒绝");
    // 混型 atan2（number, <length>）无效。
    assert!(
        parse_individual_rotate("atan2(0, 1px)").is_none(),
        "atan2(0, 1px) 应拒绝"
    );
    // 单轴关键字裸值维持拒绝（既有口径）。
    assert!(parse_individual_rotate("x").is_none(), "裸 x 轴关键字应拒绝");
}

#[test]
fn r2280_scale_math_number_slot() {
    // <number> 文法位：sign/round/pow/hypot 求值。
    let cases = [
        ("sign(-2)", -1.0_f64),
        // round 需双参（WPT round-mod-rem-invalid：round(0px)/round(0deg) 单参形
        // 均 invalid，Blink 通过）——单参缺省步长不成立。
        ("round(1.4, 1)", 1.0),
        ("pow(2, 3)", 8.0),
        ("hypot(3, 4)", 5.0),
        ("calc(2)", 2.0),
    ];
    for (value, expected) in cases {
        let result = parse_individual_scale(value);
        assert!(result.is_some(), "scale: {value} 应解析成功");
        assert!(
            matches!(result, Some(TransformFunction::Scale(a, _)) if (a - expected).abs() < 1e-9),
            "scale: {value} 实际 {:?}",
            result
        );
    }
}

#[test]
fn r2280_scale_invalid_math_rejected() {
    // 角度产出函数对 number 文法位无效（atan2 → <angle>）。
    assert!(
        parse_individual_scale("atan2(1, 2)").is_none(),
        "scale: atan2 角度结果应拒绝"
    );
    // 长度叶混入 number 槽无效。
    assert!(parse_individual_scale("calc(10px)").is_none(), "scale: 10px 应拒绝");
    assert!(
        parse_individual_scale("hypot(1px, 2px)").is_none(),
        "scale: hypot 长度叶应拒绝"
    );
    // 单参 round 上游 invalid（WPT round-mod-rem-invalid：round(0px)/round(0s) 等）。
    assert!(
        parse_individual_scale("round(1.4)").is_none(),
        "scale: 单参 round 应拒绝"
    );
}

#[test]
fn r2280_calc_expr_number_type_check() {
    // number 型：纯 number 叶 + number 产出函数（sin/cos/tan 输入弧度归一）。
    assert!(calc_expr_is_number(&CalcExpr::Number(1.0)));
    assert!(
        calc_expr_is_number(&CalcExpr::UnaryOp(UnaryMathOp::Sin, Box::new(CalcExpr::Number(0.0)))),
        "sin(...) 为 number 型"
    );
    assert!(
        calc_expr_is_number(&CalcExpr::BinaryMathOp(
            BinaryMathOp::Pow,
            Box::new(CalcExpr::Number(2.0)),
            Box::new(CalcExpr::Number(3.0))
        )),
        "pow(...) 为 number 型"
    );
    // 角度产出：asin/acos/atan/atan2 非 number 型。
    for op in [UnaryMathOp::Asin, UnaryMathOp::Acos, UnaryMathOp::Atan] {
        assert!(
            !calc_expr_is_number(&CalcExpr::UnaryOp(op, Box::new(CalcExpr::Number(0.0)))),
            "{op:?} 产出 <angle>，非 number 型"
        );
    }
    assert!(
        !calc_expr_is_number(&CalcExpr::BinaryMathOp(
            BinaryMathOp::Atan2,
            Box::new(CalcExpr::Number(0.0)),
            Box::new(CalcExpr::Number(0.0))
        )),
        "atan2 产出 <angle>，非 number 型"
    );
    assert!(!calc_expr_is_number(&CalcExpr::Length(crate::values::LengthValue::Px(
        1.0
    ))));
}

#[test]
fn r2280_calc_expr_angle_result_detection() {
    // 嵌套式中的角度产出函数仍可检出（acos(cos(pi)) → asin 输入弧度）。
    let expr = crate::values::parse_math_function("asin(cos(pi))").expect("应解析");
    assert!(calc_expr_has_angle_result(&expr), "asin 外层应为角度产出");
    let expr = crate::values::parse_math_function("calc(sin(pi/2) * 2)").expect("应解析");
    assert!(!calc_expr_has_angle_result(&expr), "纯 number 式无角度产出");
}

#[test]
fn r2280_round_strategy_keywords() {
    // CSS Values 4 §11.4 rounding-strategy（WPT round-mod-rem-computed 34-64 行驱动：
    // round(up, 101, 10) = 110、round(down, 106, 10) = 100、round(to-zero, ±105, 10)
    // = ±100、整倍数四策略恒等）。大小写不敏感；nearest 即默认 Round。
    let cases = [
        ("round(up, 101, 10)", 110.0_f64),
        ("round(down, 106, 10)", 100.0),
        ("round(to-zero, 105, 10)", 100.0),
        ("round(to-zero, -105, 10)", -100.0),
        ("round(up, -103, 10)", -100.0),
        ("round(NEAREST, 105, 10)", 110.0),
        ("round(up, 105, 5)", 105.0),
        ("round(to-zero, 0, 5)", 0.0),
    ];
    for (value, expected) in cases {
        let expr = crate::values::parse_math_function(value).unwrap_or_else(|| panic!("{value} 应解析成功"));
        let got = crate::values::eval_calc(&expr, None).unwrap_or_else(|| panic!("{value} 应求值"));
        assert_eq!(got, expected, "{value}");
    }
    // 非法策略位（WPT round-mod-rem-invalid：策略重复/错位）整体拒绝。
    assert!(crate::values::parse_math_function("round(nearest, 1, nearest)").is_none());
    assert!(crate::values::parse_math_function("round(1, nearest, 12)").is_none());
    // 更长 ident 不误吞策略前缀（round(upper, 1, 1) 非法）。
    assert!(crate::values::parse_math_function("round(upper, 1, 1)").is_none());
}

#[test]
fn r2280_round_strategy_single_arg_step_default() {
    // CSS Values 4 §11.4：round(<strategy>, A) step 缺省 1（WPT round-function.html
    // round(down, -1.5) = -2、round(up, 1.5) = 2、round(to-zero, -1.5) = -1）。
    let cases = [
        ("round(down, -1.5)", -2.0_f64),
        ("round(down, 1.5)", 1.0),
        ("round(up, 1.5)", 2.0),
        ("round(up, -1.5)", -1.0),
        ("round(to-zero, -1.5)", -1.0),
        ("round(to-zero, 1.5)", 1.0),
        ("round(nearest, 1.5)", 2.0),
    ];
    for (value, expected) in cases {
        let expr = crate::values::parse_math_function(value).unwrap_or_else(|| panic!("{value} 应解析成功"));
        let got = crate::values::eval_calc(&expr, None).unwrap_or_else(|| panic!("{value} 应求值"));
        assert_eq!(got, expected, "{value}");
    }
    // 裸单参（无策略）维持 invalid。
    assert!(crate::values::parse_math_function("round(-1.5)").is_none());
}
