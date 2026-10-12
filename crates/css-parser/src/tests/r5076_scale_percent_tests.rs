//! R5076（css-transforms-1 §funcdef-scale 系）：scale 族参数接受 `<percentage>`（%
//! 同义 number/100）——scale/scaleX/scaleY/scaleZ/scale3d 与独立 `scale` 属性；
//! rotate3d 分量与 matrix 系保持纯 `<number>`（% 仍须拒绝）。
//! driving: css/css-transforms transform-scale-percent-001（scale(50%, 75%) 旧解析
//! 失败致整个 transform 丢弃，渲原尺寸不缩放）。

use crate::values::{TransformFunction, TransformValue, parse_individual_scale, parse_transform};

#[test]
fn r5076_scale_accepts_percentages() {
    let result = parse_transform("scale(50%, 75%)");
    assert!(result.is_some(), "scale(50%, 75%) 应解析成功");
    if let Some(TransformValue::List(fns)) = result {
        assert!(
            matches!(fns[0], TransformFunction::Scale(0.5, Some(0.75))),
            "实际 {:?}",
            fns[0]
        );
    }
}

#[test]
fn r5076_scale_single_percent_arg() {
    let result = parse_transform("scale(200%)");
    assert!(result.is_some(), "scale(200%) 应解析成功（单值等比）");
    if let Some(TransformValue::List(fns)) = result {
        assert!(
            matches!(fns[0], TransformFunction::Scale(2.0, None)),
            "实际 {:?}",
            fns[0]
        );
    }
}

#[test]
fn r5076_scale_xy_percent_mixed_with_number() {
    let result = parse_transform("scaleX(150%)");
    assert!(result.is_some(), "scaleX(150%) 应解析成功");
    if let Some(TransformValue::List(fns)) = result {
        assert!(matches!(fns[0], TransformFunction::ScaleX(1.5)), "实际 {:?}", fns[0]);
    }
    let result = parse_transform("scaleY(25%)");
    assert!(result.is_some(), "scaleY(25%) 应解析成功");
    if let Some(TransformValue::List(fns)) = result {
        assert!(matches!(fns[0], TransformFunction::ScaleY(0.25)), "实际 {:?}", fns[0]);
    }
}

#[test]
fn r5076_scale3d_percent_args() {
    let result = parse_transform("scale3d(50%, 100%, 10%)");
    assert!(result.is_some(), "scale3d 百分比参数应解析成功");
    if let Some(TransformValue::List(fns)) = result {
        assert!(
            matches!(fns[0], TransformFunction::Scale3d(0.5, 1.0, 0.1)),
            "实际 {:?}",
            fns[0]
        );
    }
}

#[test]
fn r5076_individual_scale_property_percent() {
    // css-transforms-2：`scale: <number> | <percentage>`（R4201 独立属性语法）。
    let result = parse_individual_scale("50% 75%");
    assert!(result.is_some(), "独立 scale 属性百分比应解析成功");
    if let Some(f) = result {
        assert!(matches!(f, TransformFunction::Scale(0.5, Some(0.75))), "实际 {:?}", f);
    }
}

#[test]
fn r5076_rotate3d_matrix_still_reject_percent() {
    // 负守卫：rotate3d 分量与 matrix 系参数是纯 <number>，% 不得放行（语法收紧不越界）。
    assert!(
        parse_transform("rotate3d(50%, 0, 0, 45deg)").is_none(),
        "rotate3d 分量 % 应拒绝"
    );
    assert!(
        parse_transform("matrix(50%, 0, 0, 1, 0, 0)").is_none(),
        "matrix 分量 % 应拒绝"
    );
}

#[test]
fn r5076_invalid_percent_still_rejected() {
    // 非法数字仍拒绝（% 后缀非数字内容）。
    assert!(parse_transform("scale(ab%, 75%)").is_none(), "非法 % 内容应拒绝");
}
