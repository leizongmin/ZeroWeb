// sign()/abs() 写面求值钉（css-values-4 §10.5 型别表 sign(X)→<number>、§10.9
// IEEE 符号位零）——signs-abs-computed 语料 INVALID 桶驱动：
// ①sign 单维度叶短路面（em/vh/% 任一 font/viewport 相对单位符号 = 标量符号，
//   无 ctx 可判——此前无基准通道 fail-closed 整拒 60 形）；
// ②保族负号重写（Time 叶 `0s - x` 镜像 Length/Resolution 先例，Number 叶
//   不毒化纯维度门）；
// ③sign 型别擦除（实参维度族不外泄树级 length 性，progress 同口径先例）。

use super::super::*;

#[test]
fn sign_single_dimension_leaf_short_circuit() {
    // 无 ctx（eval_calc None）：font/viewport 相对叶符号直判。% 叶不在短路
    // 面（css-values-4 §10.5：百分比无解析上下文不可决——sign(%) 仅在有 %
    // 上下文的槽于 used-value 期可决；percentage-without-context 语料钉面，
    // 排除臂见 sign_percentage_leaf_no_short_circuit）。
    for s in ["sign(1em)", "sign(1vh)", "sign(1vw)", "sign(3s)", "sign(3ms)"] {
        let expr = parse_math_function(s).or_else(|| parse_time_calc(s));
        assert_eq!(eval_calc(&expr.unwrap(), None), Some(1.0), "{s}");
    }
    for s in ["sign(-1em)", "sign(-1vh)", "sign(-3s)"] {
        let expr = parse_math_function(s).or_else(|| parse_time_calc(s));
        assert_eq!(eval_calc(&expr.unwrap(), None), Some(-1.0), "{s}");
    }
    // 负号重写形（parse_factor 保族臂产物 `0px - x`）同短路面。
    let expr = parse_time_calc("sign(-3s)").unwrap();
    assert!(matches!(expr, CalcExpr::UnaryOp(UnaryMathOp::Sign, _)));
    assert_eq!(eval_calc(&expr, None), Some(-1.0));
    // 保零嵌套：sign(sign(3vh)) → 1；sign(sign(-3vh)) → -1。
    let expr = parse_time_calc("sign(sign(3s))").unwrap();
    assert_eq!(eval_calc(&expr, None), Some(1.0));
}

#[test]
fn sign_negative_zero_ieee_semantics() {
    // sign(-0px) = -0（§10.9 符号位传零）——`1 / sign(sign(-0px))` = -∞，
    // clamp(-1, -∞, 1) = -1（signs-abs-computed 钳形三单位驱动）。
    for s in ["sign(sign(-0px))", "sign(sign(-0cm))", "sign(sign(-0s))"] {
        let expr = parse_math_function(s).or_else(|| parse_time_calc(s));
        let v = eval_calc(&expr.unwrap(), None).unwrap();
        assert_eq!(v, -0.0, "{s} 传入符号位零");
        assert!(v.is_sign_negative(), "{s} 须负符号位");
    }
    let expr = parse_math_function("clamp(-1, calc( 1 / sign(sign(-0px))), 1)").unwrap();
    assert_eq!(eval_calc_with_default_font(&expr), Some(-1.0));
    // 正零对照：sign(sign(0px)) = +0 → 1/+0 = +∞ → clamp = 1。
    let expr = parse_math_function("clamp(-1, calc( 1 / sign(sign(0px))), 1)").unwrap();
    assert_eq!(eval_calc_with_default_font(&expr), Some(1.0));
}

#[test]
fn sign_time_argument_pure_time_family() {
    // Time 叶保族负号（parse_factor `0s - x` 重写）：abs(-1s) 树纯 Time 族
    //（calc_expr_is_pure_time 认两侧 Time 叶）——泛用臂 Number(0) 左叶毒化
    // 已修。字面 number−time 混型不经本臂、门仍拒。
    let expr = parse_time_calc("abs(-1s)").unwrap();
    assert!(calc_expr_is_pure_time(&expr), "abs(-1s) 树须纯 Time 族");
    let expr = parse_time_calc("calc(1s + abs(-2s))").unwrap();
    assert!(calc_expr_is_pure_time(&expr));
    assert_eq!(parse_time("abs(-1s)"), Some(1.0));
    assert_eq!(parse_time("abs(-1ms)"), Some(0.001));
    assert_eq!(parse_time("calc(1s + abs(-2s))"), Some(3.0));
    // 字面混型拒（裸数字臂被 strip_suffix 陷阱误吞的形态恢复门裁决）。
    assert_eq!(parse_time("0 - 1s"), None);
    // abs 型别保留：abs(1s) 是 <time> 非 <number>。
    let expr = parse_time_calc("abs(1s)").unwrap();
    assert!(!calc_expr_is_number(&expr));
}

#[test]
fn sign_type_erasure_in_angle_slot() {
    // sign 实参维度族不外泄（§10.5）：rotate 角度槽 sign(42px - 2em) 合法
    //（parse_angle_or_math 的 calc_expr_has_length 门豁免 Sign 子树）。
    // abs 不豁免（型随 X）。
    let expr = parse_math_function("calc(90deg + 30deg * sign(42px - 2em))").unwrap();
    assert!(!calc_expr_has_length(&expr), "sign 型别擦除后无树级 length 性");
    assert!(parse_individual_rotate("calc(90deg + 30deg * sign(42px - 2em))", None).is_some());
    let expr = parse_math_function("calc(90deg + abs(2px))").unwrap();
    assert!(calc_expr_has_length(&expr), "abs 保留维度族");
    assert!(parse_individual_rotate("calc(90deg + abs(2px))", None).is_none());
    // 混合和式依默认字体基准求值：42px - 2em(16px) = 10px → sign 1 → 120deg
    //（deg↔rad 往返 1ulp 浮点容差）。
    let f = parse_individual_rotate("calc(90deg + 30deg * sign(42px - 2em))", None).unwrap();
    assert!(
        matches!(f, TransformFunction::RotateZ(v) if (v - 120.0).abs() < 1e-9),
        "{f:?}"
    );
}

#[test]
fn sign_percentage_leaf_no_short_circuit() {
    // sign() 短路面不豁免 % 叶（css-values-4 §10.5：百分比无静态解析上下文
    // 不可决——percentage-without-context `calc(sign(50%) * 1s)` time/angle
    // 槽 should-not-set 语料钉面）。短路 None 回落全 eval，无上下文槽拒收。
    let expr = crate::values::parse_time_calc("calc(sign(50%) * 1s)").unwrap();
    assert!(
        crate::values::eval_calc_with_default_font(&expr).is_none(),
        "sign(50%) 无上下文不可决"
    );
    // 非 % 维度叶照常短路（对照臂：vh 正值可决）。
    let expr = crate::values::parse_time_calc("calc(sign(1vh) * 1s)").unwrap();
    assert_eq!(crate::values::eval_calc_with_default_font(&expr), Some(1.0));
}

#[test]
fn abs_pct_ratio_resolution_scale_slot() {
    // 同型 % 比值上下文无关消解（css-values-4 §10.5 型表 + 合成基准比值
    // 不变式）：progress(abs(5%), hypot(3%, 4%), 10%) 全 % 族 = (5−5)/(10−5)
    // = 0——abs/hypot 实参 % 叶照常参与比值消解（eval_number_ratio_pct_free
    // 钉面，scale number 槽受理路径）。对照：sign 内裸 % 无比值伙伴不可
    // 消解（calc_expr_has_pct_under_sign 否决面，scale: sign(50%)*N 槽族
    // 拒收钉面 style-system sign_abs_scale_slot_static_acceptance）。
    let expr = crate::values::parse_math_function("progress(abs(5%), hypot(3%, 4%), 10%)").unwrap();
    assert_eq!(
        crate::values::eval_calc_with_default_font(&expr),
        Some(0.0),
        "同型 % 比值消解 progress=0"
    );
    let expr = crate::values::parse_math_function("calc(sign(50%) * 180)").unwrap();
    assert_eq!(
        crate::values::eval_calc_with_default_font(&expr),
        None,
        "sign 内 % 不可借合成基准消解"
    );
}
