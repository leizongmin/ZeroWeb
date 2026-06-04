//! 额外的 transform 解析覆盖率测试
//!
//! 覆盖 parse_transform.rs 中的未被覆盖的路径：
//! - parse_transform_args 的错误路径
//! - parse_css_number 的边界情况
//! - 各种错误路径

use crate::values::{parse_box_shadow, parse_gradient, parse_grid_area, parse_text_shadow, parse_transform};

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

// 测试 perspective 零（调用 parse_css_number 并检查 <= 0.0）
#[test]
fn test_parse_transform_perspective_zero() {
    let result = parse_transform("perspective(0px)");
    assert!(result.is_none());
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
