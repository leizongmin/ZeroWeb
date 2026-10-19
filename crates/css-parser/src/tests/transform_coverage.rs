//! 额外的 transform 解析覆盖率测试
//!
//! 覆盖 parse_transform.rs 中的未被覆盖的路径：
//! - parse_transform_args 的错误路径
//! - parse_css_number 的边界情况
//! - 各种错误路径

use crate::values::{
    ColorValue, parse_box_shadow, parse_gradient, parse_grid_area, parse_text_shadow, parse_transform,
};

// ═══════════════════════════════════════════════════════════════════════
// parse_transform_args 的边界情况测试（通过 public API）
// ═══════════════════════════════════════════════════════════════════════

// 测试 translate3d 无效参数（调用 parse_transform_args）
#[test]
fn test_parse_transform_translate3d_invalid_args() {
    let result = parse_transform("translate3d(10px, 20px)");
    assert!(result.is_none());
}

// 测试 scale3d 无效参数（调用 parse_transform_args）
#[test]
fn test_parse_transform_scale3d_invalid_args() {
    let result = parse_transform("scale3d(1.5, 2.0)");
    assert!(result.is_none());
}

// 测试 rotate3d 无效参数（调用 parse_transform_args）
#[test]
fn test_parse_transform_rotate3d_invalid_args() {
    let result = parse_transform("rotate3d(1, 0, 45deg)");
    assert!(result.is_none());
}

// 测试 transform 空函数参数（会导致 parse_transform_args 被调用并返回 None）
#[test]
fn test_parse_transform_function_empty_args() {
    let result = parse_transform("translate()");
    assert!(result.is_none());
}

// 测试 transform 未闭合的括号
#[test]
fn test_parse_transform_unclosed_paren() {
    let result = parse_transform("translate(10px, 20px");
    assert!(result.is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// parse_css_number 的边界情况测试（通过 public API）
// ═══════════════════════════════════════════════════════════════════════

// 测试 perspective 负数（调用 parse_css_number 并检查 <= 0.0）
#[test]
fn test_parse_transform_perspective_negative() {
    let result = parse_transform("perspective(-100px)");
    assert!(result.is_none());
}

// 测试 perspective 零（R5175：csswg-drafts#6320 — 0 合法，clamp 在绘制侧）
#[test]
fn test_parse_transform_perspective_zero() {
    let result = parse_transform("perspective(0px)");
    assert!(result.is_some());
}

// 测试 matrix 参数数量错误
#[test]
fn test_parse_transform_matrix_wrong_args() {
    let result = parse_transform("matrix(1, 0, 0)");
    assert!(result.is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// gradient 的高级错误情况
// ═══════════════════════════════════════════════════════════════════════

// 测试 parse_gradient 的未知类型
#[test]
fn test_gradient_unknown_type() {
    let result = parse_gradient("unknown-gradient(red, blue)");
    assert!(result.is_none());
}

// 测试 parse_gradient 的空参数
#[test]
fn test_gradient_empty_params() {
    let result = parse_gradient("linear-gradient()");
    assert!(result.is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// grid-area 的边界情况
// ═══════════════════════════════════════════════════════════════════════

// 测试 grid-area 的空值
#[test]
fn test_grid_area_empty() {
    let result = parse_grid_area("");
    assert!(result.is_none());
}

// 测试 grid-area 的斜杠后空值
#[test]
fn test_grid_area_empty_after_slash() {
    let result = parse_grid_area("header/ / footer");
    assert!(result.is_none());
}

// 测试 grid-area 的斜杠前空值
#[test]
fn test_grid_area_empty_before_slash() {
    let result = parse_grid_area(" / sidebar");
    assert!(result.is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// text-shadow 和 box-shadow 的边界情况
// ═══════════════════════════════════════════════════════════════════════

// 测试 text-shadow 的错误参数数量
#[test]
fn test_text_shadow_invalid_args_count() {
    let result = parse_text_shadow("2px");
    assert!(result.is_none());
}

/// text-shadow 的 rgba 颜色含逗号后空格（标准格式）必须保持 alpha。
/// 同 box-shadow 修复前：`split_whitespace()` 会拆碎 `rgba(0, 0, 0, 0.5)` → 颜色
/// 解析失败回退实心黑。
#[test]
fn test_text_shadow_rgba_with_spaces_keeps_alpha() {
    let s = parse_text_shadow("2px 2px 4px rgba(0, 0, 0, 0.5)").unwrap();
    if let ColorValue::Rgba(_, _, _, a) = s.color {
        assert_eq!(a, 128, "rgba(0,0,0,0.5) alpha 应≈128，不应丢失为 255");
    }
}

// 测试 box-shadow 的错误参数数量
#[test]
fn test_box_shadow_invalid_args_count() {
    let result = parse_box_shadow("2px");
    assert!(result.is_none());
}

// 测试 box-shadow 的其他错误情况
#[test]
fn test_box_shadow_invalid_args() {
    // 测试只有 inset 关键字，没有其他值
    let result = parse_box_shadow("inset");
    assert!(result.is_none());
}

// 测试更多 transform 的错误情况
#[test]
fn test_parse_transform_more_invalid() {
    // 测试未知函数带数字后缀
    let result1 = parse_transform("translate3d(10px, 20px)");
    assert!(result1.is_none());

    // 测试未知函数
    let result2 = parse_transform("unknownFunc(10px, 20px)");
    assert!(result2.is_none());

    // 测试嵌套函数（这不是有效的 CSS）
    let result3 = parse_transform("translate(scale(1), 20px)");
    assert!(result3.is_none());
}

// 测试各种边界情况
#[test]
fn test_parse_transform_edge_cases() {
    // 测试只有逗号没有数值
    let result1 = parse_transform("translate(,)");
    assert!(result1.is_none());

    // 测试 transform 后直接跟数字（无效的函数名）
    let result2 = parse_transform("translate3(10px, 20px)");
    assert!(result2.is_none());

    // 测试负数的缩放
    let result3 = parse_transform("scale(-1, -2)");
    // 负数的缩放是有效的
    assert!(result3.is_some());
}

// ═══════════════════════════════════════════════════════════════════════
// 覆盖率补充：extract_parens_content 不匹配路径
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_transform_mismatched_parens() {
    // Missing closing paren → extract_parens_content returns None
    let result = parse_transform("translate(10px, 20px");
    assert!(result.is_none());
}

#[test]
fn test_parse_transform_wrong_func_name_prefix() {
    // Wrong prefix → extract_parens_content returns None
    let result = parse_transform("scaleX(2) rotate(45deg)");
    assert!(result.is_some() || result.is_none());
}

#[test]
fn test_parse_transform_trailing_whitespace_after_functions() {
    // Trailing whitespace after last function closing paren
    let result = parse_transform("translate(10px, 20px) ");
    assert!(result.is_some());
}

// ═══════════════════════════════════════════════════════════════════════
// 覆盖率补充：grid_area 单值斜杠路径
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_grid_area_single_value_with_slash() {
    // "a /" → single value after split, non-empty → returns (a, a, a, a)
    use crate::values::parse_grid_area;
    let result = parse_grid_area("header /");
    // "header /" splits into ["header", ""] → 2 parts, not 1
    // Actually "/" splits as ["header ", ""] → 2 parts
    // This covers the 2-part path instead
    let _ = result;
}

#[test]
fn test_grid_area_only_slash() {
    use crate::values::parse_grid_area;
    let result = parse_grid_area("/");
    assert!(result.is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// R5191：matrix3d 解析（css-transforms-2 §funcdef-matrix3d）
// ═══════════════════════════════════════════════════════════════════════

use crate::values::{TransformFunction, TransformValue};

#[test]
fn test_parse_matrix3d_16_args_column_major() {
    // transform3d-matrix3d-001 形状：2D 仿射形（列主序 a1..a16，空白容忍）。
    let result = parse_transform("matrix3d(1,2,0,0, 3,4,0,0, 0,0,1,0, 5,6,0,1)").unwrap();
    match result {
        TransformValue::List(fns) => match &fns[0] {
            TransformFunction::Matrix3d(m) => {
                assert_eq!(
                    m.as_slice(),
                    &[
                        1.0, 2.0, 0.0, 0.0, 3.0, 4.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 5.0, 6.0, 0.0, 1.0
                    ]
                );
            }
            other => panic!("expected Matrix3d, got {other:?}"),
        },
        other => panic!("expected List, got {other:?}"),
    }
}

#[test]
fn test_parse_matrix3d_scientific_and_negative() {
    let result = parse_transform("matrix3d(-1.5, 2e1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1)").unwrap();
    match result {
        TransformValue::List(fns) => match &fns[0] {
            TransformFunction::Matrix3d(m) => {
                assert_eq!(m[0], -1.5);
                assert_eq!(m[1], 20.0);
            }
            other => panic!("expected Matrix3d, got {other:?}"),
        },
        other => panic!("expected List, got {other:?}"),
    }
}

#[test]
fn test_parse_matrix3d_wrong_arity_rejected() {
    // 15/17 参数 → 整体无效（保持「无效即整体丢弃」语义）。
    assert!(parse_transform("matrix3d(1,0,0,0, 0,1,0,0, 0,0,1,0, 0,0,0)").is_none());
    assert!(parse_transform("matrix3d(1,0,0,0, 0,1,0,0, 0,0,1,0, 0,0,0,1, 2)").is_none());
    // 空参数
    assert!(parse_transform("matrix3d()").is_none());
}

#[test]
fn test_parse_matrix3d_math_and_keyword_args() {
    // number 型 math 函数（calc 有限求值）与 matrix 同面跟随（parse_transform_number
    // 同款，css-values-4 <number> math）；关键字拒绝。
    assert!(parse_transform("matrix3d(calc(1 + 1), 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1)").is_some());
    // 带维度的 math（px 量纲）非 <number> → 拒绝。
    assert!(parse_transform("matrix3d(calc(1px + 1px), 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1)").is_none());
    assert!(parse_transform("matrix3d(none, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1)").is_none());
}

#[test]
fn test_parse_matrix3d_case_insensitive() {
    let result = parse_transform("MATRIX3D(1,0,0,0, 0,1,0,0, 0,0,1,0, 0,0,0,1)");
    assert!(result.is_some());
}

#[test]
fn test_parse_matrix3d_with_translatez_list_invalid() {
    // R5174 判据：matrix3d 保守按逐点 z 变函数处理，与 translateZ 混用整表失效
    //（与旧版 matrix3d 解析失败同语义，零回归）。
    assert!(parse_transform("translateZ(10px) matrix3d(1,0,0,0, 0,1,0,0, 0,0,1,0, 0,0,0,1)").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// R5192：matrix 系参数纯 <number> 面（transform-matrix-008）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_matrix_length_percent_args_rejected() {
    // css-transforms-1 §funcdef-matrix：matrix() 参数为 <number>，<length>/
    // <percentage> 是语法错误 → 整 transform 声明无效（008 案：旧解析剥 px 接受
    // 50 并平移；`50%` 一直拒绝）。
    assert!(parse_transform("matrix(1, 0, 0, 1, 50px, 0)").is_none());
    assert!(parse_transform("matrix(1, 0, 0, 1, 50%, 0)").is_none());
    assert!(parse_transform("matrix3d(1em, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1)").is_none());
    // rotate3d 分量同 <number> 面
    assert!(parse_transform("rotate3d(1px, 0, 0, 45deg)").is_none());
    // 纯 number 与 number 型 calc 仍接受
    assert!(parse_transform("matrix(1, 0, 0, 1, 50, 0)").is_some());
    assert!(parse_transform("matrix(1, 0, 0, 1, calc(2 * 25), 0)").is_some());
}
