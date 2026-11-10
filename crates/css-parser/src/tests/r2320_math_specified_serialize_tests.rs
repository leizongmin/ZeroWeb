//! 数学函数 specified 值常量折叠序列化 + <angle> 槽位 math 接线（r2320）。
//!
//! `transform` 列表 `rotate(` 函数臂接 `parse_angle_or_math`（此前仅 individual
//! rotate 属性接线，`transform: rotate(acos(1))` 被拒——WPT acos serialize 页
//! "got disallowed value" 根因）；asin/acos/atan 实参 <number>-only 字符串面
//! 门（`atan(45deg)` invalid）；NaN 结果合法（CSS Values 4 §10.10）；折叠
//! 序列化 `fold_math_specified`（§10.11）供 inline style 读回面接线。
//!
//! driving: wptfyi-css DC-2 values serialize 桶（sin-cos-tan-serialize 230 /
//! acos-asin-atan-atan2-serialize 62，WPT css-values，wptfyi-90-plan DC-2）。

use crate::values::{MathFoldKind, TransformValue, fold_math_specified, parse_transform};

#[test]
fn transform_rotate_function_accepts_math() {
    // <angle> 文法位 math：合法域内值。
    let parse_rotate = |v: &str| match parse_transform(v) {
        Some(TransformValue::List(funcs)) => funcs.first().map(ToOwned::to_owned),
        _ => None,
    };
    assert!(parse_rotate("rotate(acos(1))").is_some(), "rotate(acos(1)) 合法");
    assert!(parse_rotate("rotate(calc(acos(1)))").is_some(), "calc 包裹形合法");
    assert!(parse_rotate("rotate(acos(2))").is_some(), "NaN 域外合法（§10.10）");
    // 注：`rotate(sin(30deg) * 1deg)` 类角度单位算术 CalcParser 不支持（FIXME
    // 挂账，parse_transform.rs parse_angle_or_math 注释），不在本批范围。
}

#[test]
fn transform_rotate_function_rejects_invalid_forms() {
    // invalid 面（acos-asin-atan-atan2-invalid / sin-cos-tan-invalid 语法形）。
    assert!(parse_transform("rotate(asin())").is_none(), "空参拒绝");
    assert!(parse_transform("rotate(asin(1dag))").is_none(), "坏单位拒绝");
    assert!(
        parse_transform("rotate(atan(45deg))").is_none(),
        "asin/acos/atan 角度叶拒绝"
    );
    assert!(parse_transform("rotate(asin(90px))").is_none(), "长度叶拒绝");
    assert!(parse_transform("rotate(acos(1deg, 2deg))").is_none(), "双参拒绝");
    // sin/cos/tan 与 atan2 角度/时值实参仍合法。
    assert!(parse_transform("rotate(sin(30deg))").is_some(), "sin 角度实参合法");
    assert!(parse_transform("rotate(atan2(1, 1))").is_some(), "atan2 合法");
}

#[test]
fn fold_number_kind() {
    use MathFoldKind::Number;
    assert_eq!(fold_math_specified("cos(0)", Number).as_deref(), Some("calc(1)"));
    assert_eq!(fold_math_specified("sin(30deg)", Number).as_deref(), Some("calc(0.5)"));
    assert_eq!(
        fold_math_specified("calc(sin(infinity))", Number).as_deref(),
        Some("calc(NaN)")
    );
    // 非数学函数/型别不符保留原文（None）。
    assert_eq!(fold_math_specified("45deg", Number), None);
    assert_eq!(fold_math_specified("acos(1)", Number), None);
}

#[test]
fn fold_angle_kind() {
    use MathFoldKind::Angle;
    assert_eq!(fold_math_specified("acos(1)", Angle).as_deref(), Some("calc(0deg)"));
    assert_eq!(fold_math_specified("acos(-1)", Angle).as_deref(), Some("calc(180deg)"));
    assert_eq!(fold_math_specified("asin(0.5)", Angle).as_deref(), Some("calc(30deg)"));
    assert_eq!(
        fold_math_specified("atan(infinity)", Angle).as_deref(),
        Some("calc(90deg)")
    );
    assert_eq!(
        fold_math_specified("acos(2)", Angle).as_deref(),
        Some("calc(NaN * 1deg)")
    );
    // 非数学函数拒折。
    assert_eq!(fold_math_specified("45deg", Angle), None);
}
