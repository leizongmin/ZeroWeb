//! CSS Values 4 §10.5 型别表 + §10.9 IEEE 语义 + §10.8 hypot 变参 + §10.11
//! scale 族折叠测试（WPT css-values signs-abs / hypot-pow-sqrt / serialize 桶
//! 驱动的 specified 层契约）。

use crate::values::parse_extended::{calc_expr_is_length_percentage, calc_expr_is_number, calc_expr_math_type};
use crate::values::serialize_math::{
    MathFoldKind, fold_decl_specified, fold_math_specified, fold_transform_math_specified,
};
use crate::values::{CalcMathType, eval_calc, parse_math_function};

#[test]
fn sign_result_is_number_for_any_argument_type() {
    // §10.5 型别表：sign(X) 结果恒 <number>，实参任意型别
    //（signs-abs-computed `sign(1px)` → 1 等 30 案驱动——number 槽位 scale/
    // z-index 合法）。
    for src in [
        "sign(1)",
        "sign(1px)",
        "sign(-1cm)",
        "sign(10%)",
        "sign(1em - 16px)",
        "sign(abs(1px))",
        "sign(hypot(3px, 4px))",
    ] {
        let expr = parse_math_function(src).unwrap_or_else(|| panic!("{src} 应解析"));
        assert!(
            calc_expr_is_number(&expr),
            "{src} 结果应为 <number>（§10.5 sign 恒 number）"
        );
    }
}

#[test]
fn abs_round_hypot_result_follows_argument_family() {
    // §10.5：abs(A)/round/mod/rem/hypot 结果随同族实参——长度族实参在
    // <length-percentage> 槽合法（signs-abs-computed `abs(10%)`/`abs(10px + 10%)`
    // 案驱动）；number 族实参在 number 槽合法。
    for src in [
        "abs(10px)",
        "abs(-10%)",
        "hypot(3px, 4px)",
        "round(10px, 5px)",
        "mod(10px, 3%)",
    ] {
        let expr = parse_math_function(src).unwrap_or_else(|| panic!("{src} 应解析"));
        assert!(
            calc_expr_is_length_percentage(&expr),
            "{src} 结果应为 <length-percentage>"
        );
        assert!(!calc_expr_is_number(&expr), "{src} 非 <number>");
    }
    for src in ["abs(1)", "hypot(3, 4)", "round(10, 5)", "rem(10, 3)"] {
        let expr = parse_math_function(src).unwrap_or_else(|| panic!("{src} 应解析"));
        assert!(calc_expr_is_number(&expr), "{src} 结果应为 <number>");
    }
    // 混族实参整式无效（§8.1）。
    for src in ["hypot(2px, 3)", "round(10px, 5)", "abs(1px + 1)"] {
        let expr = parse_math_function(src).unwrap_or_else(|| panic!("{src} 应解析"));
        assert!(
            calc_expr_math_type(&expr) == CalcMathType::Invalid,
            "{src} 混族应为 Invalid"
        );
    }
}

#[test]
fn dimension_arithmetic_number_times_length_is_length() {
    // §8.1 维度算术：number·length=length、length/number=length
    //（signs-abs-computed `calc((1em + 1px) * (sign(...) + 1))`、
    // calc-infinity-nan `calc(1em / sign(...))` 案驱动）；同族相加守恒；
    // number+length 混加无效。
    let cases = [
        ("calc(2 * 10px)", CalcMathType::LengthPercentage),
        ("calc(10px * 2)", CalcMathType::LengthPercentage),
        ("calc(10px / 2)", CalcMathType::LengthPercentage),
        ("calc(10px + 10%)", CalcMathType::LengthPercentage),
        ("calc(10px + 1)", CalcMathType::Invalid),
        ("calc(1 / 2)", CalcMathType::Number),
        ("calc(asin(0.5) * 1)", CalcMathType::Angle),
        ("calc(asin(0.5) + 30deg)", CalcMathType::Invalid),
    ];
    for (src, want) in cases {
        let expr = parse_math_function(src).unwrap_or_else(|| panic!("{src} 应解析"));
        assert_eq!(calc_expr_math_type(&expr), want, "{src}");
    }
}

#[test]
fn sqrt_exp_log_pow_args_must_be_number() {
    // §10.5：sqrt/exp/log/pow 实参仅 <number>（hypot-pow-sqrt-invalid
    // `pow(2px, 3px)` 案驱动）；sin/cos/tan 角度实参 parse 期弧度归一后同门。
    for src in ["sqrt(100px)", "exp(1px)", "log(1%)", "pow(2px, 3px)", "pow(1, 2px)"] {
        let expr = parse_math_function(src).unwrap_or_else(|| panic!("{src} 应解析"));
        assert_eq!(
            calc_expr_math_type(&expr),
            CalcMathType::Invalid,
            "{src} 非 number 实参无效"
        );
    }
}

#[test]
fn eval_sign_ieee_signed_zero_and_nan() {
    // §10.9：sign(NaN)=NaN；sign(-0)=-0（IEEE 符号位传零，非 -1）。
    let nan = eval_calc(&parse_math_function("sign(1 * NaN)").unwrap(), None).unwrap();
    assert!(nan.is_nan(), "sign(NaN) 应为 NaN");
    let neg_zero = eval_calc(&parse_math_function("sign(-0)").unwrap(), None).unwrap();
    assert_eq!(neg_zero, 0.0);
    assert!(neg_zero.is_sign_negative(), "sign(-0) 应为 -0");
    assert_eq!(eval_calc(&parse_math_function("sign(0)").unwrap(), None), Some(0.0));
    assert!(
        eval_calc(&parse_math_function("sign(0)").unwrap(), None)
            .unwrap()
            .is_sign_positive()
    );
    assert_eq!(eval_calc(&parse_math_function("sign(5)").unwrap(), None), Some(1.0));
    assert_eq!(eval_calc(&parse_math_function("sign(-5)").unwrap(), None), Some(-1.0));
    // 嵌套：sign(sign(-0)) = -0；1/sign(sign(-0)) = -∞；clamp → -1
    //（signs-abs-computed `clamp(-1, calc(1 / sign(sign(-0))), 1)` 驱动）。
    let div = eval_calc(&parse_math_function("calc(1 / sign(sign(-0)))").unwrap(), None).unwrap();
    assert!(
        div.is_infinite() && div.is_sign_negative(),
        "1 / -0 应为 -∞，实际 {div}"
    );
    let clamp = eval_calc(
        &parse_math_function("clamp(-1, calc(1 / sign(sign(-0))), 1)").unwrap(),
        None,
    );
    assert_eq!(clamp, Some(-1.0));
    let clamp_pos = eval_calc(
        &parse_math_function("clamp(-1, calc(1 / sign(sign(0))), 1)").unwrap(),
        None,
    );
    assert_eq!(clamp_pos, Some(1.0));
}

#[test]
fn eval_divide_by_zero_is_ieee_infinity() {
    // §10.6：除零 ±∞ 非无效值。
    let pos = eval_calc(&parse_math_function("calc(1 / 0)").unwrap(), None).unwrap();
    assert!(pos.is_infinite() && pos.is_sign_positive());
    let neg = eval_calc(&parse_math_function("calc(1 / -0)").unwrap(), None).unwrap();
    assert!(neg.is_infinite() && neg.is_sign_negative());
    assert!(
        eval_calc(&parse_math_function("calc(0 / 0)").unwrap(), None)
            .unwrap()
            .is_nan()
    );
    // 常规除法不受影响。
    assert_eq!(
        eval_calc(&parse_math_function("calc(10px / 4)").unwrap(), None),
        Some(2.5)
    );
}

#[test]
fn hypot_variadic_single_arg_folds_to_abs() {
    // §10.8：hypot(A) 单参形合法（≡ |A|）；≥3 参嵌套表示求值等价
    //（hypot-pow-sqrt-serialize `hypot(1)`、hypot-pow-sqrt-invalid
    // `hypot()` 拒绝驱动）。
    assert_eq!(eval_calc(&parse_math_function("hypot(3)").unwrap(), None), Some(3.0));
    assert_eq!(eval_calc(&parse_math_function("hypot(-3)").unwrap(), None), Some(3.0));
    // hypot(3, 4, 12) = 13（5-12-13 三角嵌套求值）。
    assert_eq!(
        eval_calc(&parse_math_function("hypot(3, 4, 12)").unwrap(), None),
        Some(13.0)
    );
    // 0 参仍拒绝；非 hypot 函数裸单参仍拒绝。
    assert!(parse_math_function("hypot()").is_none());
    assert!(parse_math_function("pow(1)").is_none());
    assert!(parse_math_function("round(1)").is_none());
    // 型别表对嵌套 hypot 同口径。
    let expr = parse_math_function("hypot(1px, 2px, 3px)").unwrap();
    assert!(calc_expr_is_length_percentage(&expr));
}

#[test]
fn scale_family_fold_in_transform_wrapper() {
    // §10.11：transform scale 族 number 实参常量折叠（hypot-pow-sqrt-serialize
    // `scale(pow(1,1))` → `scale(calc(1))`、signs-abs-serialize `scale(abs(1))`
    // → `scale(calc(1))` 案驱动）。
    let cases = [
        ("scale(abs(1))", "scale(calc(1))"),
        ("scale(sign(.1))", "scale(calc(1))"),
        ("scale(pow(1,1))", "scale(calc(1))"),
        ("scale(hypot(1))", "scale(calc(1))"),
        ("scale(sqrt(1))", "scale(calc(1))"),
        ("scalex(abs(1))", "scalex(calc(1))"),
        ("scale(calc(sign(.1) + sign(.2)))", "scale(calc(2))"),
        ("scale(calc(sign(-1 * NaN)))", "scale(calc(NaN))"),
        ("scale(calc(abs(infinity)))", "scale(calc(infinity))"),
        ("scale(calc(sign(infinity)))", "scale(calc(1))"),
        ("scale3d(pow(1,1), 2, sqrt(4))", "scale3d(calc(1), 2, calc(2))"),
        // 非 math 实参原样（无折叠 → None 保留原文）。
    ];
    for (src, want) in cases {
        assert_eq!(
            fold_transform_math_specified(src).as_deref(),
            Some(want),
            "{src} 应折叠为 {want}"
        );
    }
    assert_eq!(fold_transform_math_specified("scale(2) translateX(10px)"), None);
    // 与 fold_decl_dispatch 合流：transform 声明值入口。
    assert_eq!(
        fold_decl_specified("transform", "scale(abs(1))").as_deref(),
        Some("scale(calc(1))")
    );
}

#[test]
fn fold_number_kind_sign_abs_length_free() {
    // fold_math_specified Number 位：sign/abs 纯 number 式折叠（§10.5 表放行
    // 后 `scale(abs(1))` 链路入口直折叠）。
    use MathFoldKind::Number;
    assert_eq!(fold_math_specified("sign(.1)", Number).as_deref(), Some("calc(1)"));
    assert_eq!(
        fold_math_specified("abs(1 + 2 + 3)", Number).as_deref(),
        Some("calc(6)")
    );
    assert_eq!(
        fold_math_specified("sign(1 * NaN)", Number).as_deref(),
        Some("calc(NaN)")
    );
    // 含长度叶仍拒折（<number> 位不容 length）。
    assert_eq!(fold_math_specified("abs(1px)", Number), None);
    assert_eq!(fold_math_specified("sign(1px)", Number), None);
}
