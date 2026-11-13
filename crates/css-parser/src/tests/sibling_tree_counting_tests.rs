//! sibling-index()/sibling-count() 求值 + 写面 canonical 串化钉面（CSS Values 5
//! §tree-counting）。
//!
//! CalcContext 兄弟序注入求值（引擎桥消费契约：有 ctx 出值、无 ctx fail-closed
//! None）；`fold_tree_counting_calc` 串化（opaque 函数名归一/静态积规范单位/
//! 单叶 calc 解包）；`tree_counting_slot_valid` 槽族门（calc-sibling-function-
//! parsing 语料 13 invalid 形逐钉）。
//!
//! driving: wptfyi-css DC-2 g19（tree-counting calc-sibling-function-parsing）。

use crate::values::{
    CalcContext, CalcExpr, MathFoldKind, fold_decl_specified, fold_math_specified, parse_calc, parse_math_function,
    tree_counting_slot_valid,
};

fn ctx_sibling(ordinal: u64, count: u64) -> CalcContext {
    CalcContext {
        sibling_ordinal: Some(ordinal),
        sibling_count: Some(count),
        ..Default::default()
    }
}

#[test]
fn tree_counting_eval_with_context() {
    let expr = parse_calc("calc(sibling-index())").unwrap();
    assert_eq!(
        crate::values::eval_calc_with_context(&expr, &ctx_sibling(3, 5)),
        Some(3.0)
    );

    let expr = parse_calc("calc(sibling-count())").unwrap();
    assert_eq!(
        crate::values::eval_calc_with_context(&expr, &ctx_sibling(3, 5)),
        Some(5.0)
    );

    // 组合树：静态因子随兄弟序乘积。
    let expr = parse_calc("calc(2 * sibling-index() + sibling-count())").unwrap();
    assert_eq!(
        crate::values::eval_calc_with_context(&expr, &ctx_sibling(3, 5)),
        Some(11.0)
    );
}

#[test]
fn tree_counting_eval_without_context_fail_closed() {
    // 无 DOM 视图（纯 parse/序列化消费方）保持 None——既有 fail-closed 契约。
    let expr = parse_calc("calc(sibling-index())").unwrap();
    assert_eq!(
        crate::values::eval_calc_with_context(&expr, &CalcContext::default()),
        None
    );
    assert_eq!(crate::values::eval_calc(&expr, Some(100.0)), None);
}

#[test]
fn tree_counting_fold_angle_static_product() {
    // 角度静态积 opaque→1 掩码求值 + deg 规范（rotate 语料钉面）。
    assert_eq!(
        fold_math_specified("calc(10deg * sibling-index())", MathFoldKind::Angle).as_deref(),
        Some("calc(10deg * sibling-index())")
    );
    assert_eq!(
        fold_math_specified("calc(sibling-index() * 2rad * pi)", MathFoldKind::Angle).as_deref(),
        Some("calc(360deg * sibling-index())")
    );
    assert_eq!(
        fold_math_specified("calc(1turn * sibling-count())", MathFoldKind::Angle).as_deref(),
        Some("calc(360deg * sibling-count())")
    );
    assert_eq!(
        fold_math_specified("calc(sibling-count() * 1rad * pi)", MathFoldKind::Angle).as_deref(),
        Some("calc(180deg * sibling-count())")
    );
}

#[test]
fn tree_counting_fold_time_unit_canonical() {
    assert_eq!(
        fold_math_specified("calc(1s * sibling-index())", MathFoldKind::Time).as_deref(),
        Some("calc(1s * sibling-index())")
    );
    assert_eq!(
        fold_math_specified("calc(100ms * sibling-count())", MathFoldKind::Time).as_deref(),
        Some("calc(0.1s * sibling-count())")
    );
    assert_eq!(
        fold_math_specified("calc(250ms * sibling-index())", MathFoldKind::Time).as_deref(),
        Some("calc(0.25s * sibling-index())")
    );
}

#[test]
fn tree_counting_fold_length_atom_echo() {
    // Length 位静态因子保维度 canonical（em/%/vh 布局依赖不折叠）。
    for v in [
        "calc(1px * sibling-index())",
        "calc(1em * sibling-index())",
        "calc(1rem * sibling-count())",
        "calc(1vh * sibling-index())",
        "calc(1ch * sibling-count())",
        "calc(1% * sibling-index())",
        "calc(5% * sibling-count())",
    ] {
        assert_eq!(
            fold_math_specified(v, MathFoldKind::Length).as_deref(),
            Some(v),
            "{v} 应维度原子原文回显"
        );
    }
}

#[test]
fn tree_counting_fold_number_and_unpack() {
    assert_eq!(
        fold_math_specified("calc(0.1 * sibling-index())", MathFoldKind::Number).as_deref(),
        Some("calc(0.1 * sibling-index())")
    );
    assert_eq!(
        fold_math_specified("calc(2 * sibling-count())", MathFoldKind::Number).as_deref(),
        Some("calc(2 * sibling-count())")
    );
    // 单 opaque 叶 calc 包裹解包（z-index 语料钉面）。
    assert_eq!(
        fold_math_specified("calc(sibling-index())", MathFoldKind::Number).as_deref(),
        Some("sibling-index()")
    );
    assert_eq!(
        fold_math_specified("calc(sibling-count())", MathFoldKind::Number).as_deref(),
        Some("sibling-count()")
    );
}

#[test]
fn tree_counting_fold_whitespace_normalize() {
    // 函数名内空白归一（parse 吞空白 → opaque re-format）。
    assert_eq!(
        fold_math_specified("calc(1px * sibling-index( ))", MathFoldKind::Length).as_deref(),
        Some("calc(1px * sibling-index())")
    );
    assert_eq!(
        fold_math_specified("calc(1s * sibling-count( ))", MathFoldKind::Time).as_deref(),
        Some("calc(1s * sibling-count())")
    );
}

#[test]
fn tree_counting_fold_boundary_keeps_src() {
    // 加减链/min-max/多 opaque：串化边界外保留原文（FIXME 无钉面）。
    assert_eq!(
        fold_math_specified("calc(sibling-index() + 1px)", MathFoldKind::Length),
        None
    );
    assert_eq!(
        fold_math_specified("calc(sibling-index() * sibling-count())", MathFoldKind::Number),
        None
    );
    assert_eq!(
        fold_math_specified("calc(min(sibling-index(), 3px))", MathFoldKind::Length),
        None
    );
}

#[test]
fn tree_counting_slot_valid_matrix() {
    let parse = |v: &str| parse_math_function(v).unwrap_or_else(|| panic!("parse failed: {v:?}"));
    // Time 叶仅 allow_time 入口可达（共享 calc 入口拒 time 原子——types.rs 注）。
    let parse_time =
        |v: &str| crate::values::parse_time_calc(v).unwrap_or_else(|| panic!("parse_time_calc failed: {v:?}"));
    // valid 面（槽族一致）。
    assert!(tree_counting_slot_valid("left", &parse("calc(1px * sibling-index())")));
    assert!(tree_counting_slot_valid("left", &parse("calc(1% * sibling-count())")));
    assert!(tree_counting_slot_valid(
        "rotate",
        &parse("calc(10deg * sibling-index())")
    ));
    assert!(tree_counting_slot_valid(
        "rotate",
        &parse("calc(sibling-index() * 2rad * pi)")
    ));
    assert!(tree_counting_slot_valid(
        "opacity",
        &parse("calc(0.1 * sibling-index())")
    ));
    assert!(tree_counting_slot_valid("scale", &parse("calc(2 * sibling-count())")));
    // text-size-adjust <percentage> 槽（css-size-adjust-1）：% 叶经
    // LengthPercentage 归 MTLength（text-size-adjust-valid 接受腿驱动）。
    assert!(tree_counting_slot_valid(
        "text-size-adjust",
        &parse("calc(10% * sibling-index())")
    ));
    assert!(tree_counting_slot_valid("z-index", &parse("calc(sibling-index())")));
    assert!(tree_counting_slot_valid(
        "animation-duration",
        &parse_time("calc(1s * sibling-index())")
    ));
    // invalid 面（槽族错配，语料逐钉）。
    assert!(!tree_counting_slot_valid("left", &parse("calc(10 * sibling-index())")));
    assert!(!tree_counting_slot_valid(
        "rotate",
        &parse("calc(1px * sibling-index())")
    ));
    assert!(!tree_counting_slot_valid(
        "rotate",
        &parse_time("calc(1s * sibling-index())")
    ));
    assert!(!tree_counting_slot_valid(
        "opacity",
        &parse_time("calc(1ms * sibling-index())")
    ));
    assert!(!tree_counting_slot_valid(
        "opacity",
        &parse("calc(1px * sibling-count())")
    ));
    assert!(!tree_counting_slot_valid(
        "animation-duration",
        &parse("calc(1px * sibling-index())")
    ));
    assert!(!tree_counting_slot_valid(
        "animation-duration",
        &parse("calc(1deg * sibling-index())")
    ));
    // 裸叶（z-index <integer> 槽）按 number 型别归族。
    assert!(tree_counting_slot_valid(
        "z-index",
        &CalcExpr::TreeCounting(crate::values::TreeCountingFn::SiblingIndex)
    ));
    assert!(!tree_counting_slot_valid(
        "left",
        &CalcExpr::TreeCounting(crate::values::TreeCountingFn::SiblingCount)
    ));
}

#[test]
fn tree_counting_fold_decl_dispatch() {
    // left/z-index 走 fold_decl_specified 分派（写面 canonicalize 链）。
    assert_eq!(
        fold_decl_specified("left", "calc(1px * sibling-index( ))").as_deref(),
        Some("calc(1px * sibling-index())")
    );
    // 非 sibling calc 形不折叠（既有 store 原文行为零漂移）。
    assert_eq!(fold_decl_specified("left", "calc(20% + 10px)"), None);
    // z-index 裸形归一 + calc 解包。
    assert_eq!(
        fold_decl_specified("z-index", "sibling-index( )").as_deref(),
        Some("sibling-index()")
    );
    assert_eq!(
        fold_decl_specified("z-index", "calc(sibling-index())").as_deref(),
        Some("sibling-index()")
    );
    assert_eq!(
        fold_decl_specified("z-index", "calc(2 * sibling-index())").as_deref(),
        Some("calc(2 * sibling-index())")
    );
    // 非 sibling z-index 形不折叠。
    assert_eq!(fold_decl_specified("z-index", "calc(2 + 3)"), None);
    assert_eq!(fold_decl_specified("z-index", "auto"), None);
}
