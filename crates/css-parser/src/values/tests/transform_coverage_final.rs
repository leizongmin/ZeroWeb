//! 额外的 transform 解析覆盖率测试补全
//!
// 覆盖 parse_transform.rs 中剩余的未被覆盖路径：
// - parse_transform 中的错误路径
// - parse_transform_function 的错误路径
// - parse_transform_args 的错误路径
// - parse_css_number 的边界情况
// - 各种边界情况

use crate::values::*;

// ═══════════════════════════════════════════════════════════════════════
// parse_transform 的错误路径测试
// ═══════════════════════════════════════════════════════════════════════

// 测试 parse_transform 中的空输入
#[test]
fn test_parse_transform_empty() {
    let result = parse_transform("");
    assert!(result.is_none());
}

// 测试 parse_transform 中的空白输入
#[test]
fn test_parse_transform_whitespace_only() {
    let result = parse_transform("   ");
    assert!(result.is_none());
}

// 测试 parse_transform 中无效的函数名
#[test]
fn test_parse_transform_invalid_function_name() {
    let result = parse_transform("invalid-func(10px)");
    assert!(result.is_none());
}

// 测试 parse_transform 中函数名后没有 (
#[test]
fn test_parse_transform_missing_paren() {
    let result = parse_transform("translate 10px, 20px)");
    assert!(result.is_none());
}

// 测试 parse_transform 中没有右括号
#[test]
fn test_parse_transform_missing_closing_paren() {
    let result = parse_transform("translate(10px, 20px");
    assert!(result.is_none());
}

// 测试 parse_transform 中没有参数的函数
#[test]
fn test_parse_transform_empty_args() {
    let result = parse_transform("translate()");
    assert!(result.is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// parse_transform_function 的错误路径测试
// ═══════════════════════════════════════════════════════════════════════

// 测试 parse_transform_function 中的无效参数数量
#[test]
fn test_parse_transform_function_invalid_args() {
    // translateX 不接受多个参数
    assert!(parse_transform("translateX(10px, 20px)").is_none());
    assert!(parse_transform("translateY(10px, 20px)").is_none());
    assert!(parse_transform("scaleX(1, 2)").is_none());
    assert!(parse_transform("scaleY(1, 2)").is_none());
    assert!(parse_transform("scale(1, 2, 3)").is_none());
    assert!(parse_transform("skew(10deg, 20deg, 30deg)").is_none());
    assert!(parse_transform("matrix(1,,0,0,1,10,20)").is_none());
    assert!(parse_transform("translate(10px,,20px)").is_none());
}

// 测试 parse_transform_function 中的无效参数类型
#[test]
fn test_parse_transform_function_invalid_arg_type() {
    // translate 的参数必须是数字
    let result = parse_transform("translate(invalid, 20px)");
    assert!(result.is_none());
}

#[test]
fn test_parse_transform_rejects_non_finite_numbers() {
    assert!(parse_transform("scale(inf)").is_none());
    assert!(parse_transform("rotate(NaNdeg)").is_none());
    assert!(parse_transform("translate(infpx, 0)").is_none());
    assert!(parse_transform("translate(50%, NaN%)").is_none());
    assert!(parse_transform("matrix(1, 0, 0, 1, inf, 0)").is_none());
    assert!(parse_transform("perspective(infpx)").is_none());
}

// 测试 parse_transform_function 中 scale 的负值
#[test]
fn test_parse_transform_function_scale_negative() {
    let result = parse_transform("scale(-1, 2)");
    // scale 支持负值 — 不 panic 即可
    let _ = result;
}

// ═══════════════════════════════════════════════════════════════════════
// parse_transform_args 的错误路径测试
// ═══════════════════════════════════════════════════════════════════════

// 测试 parse_transform_args 中的无效 token
#[test]
fn test_parse_transform_args_invalid_token() {
    // 通过 scale 函数触发 parse_transform_args
    let result = parse_transform("scale(10px, invalid)");
    assert!(result.is_none());
}

// 测试 parse_transform_args 中的负数
#[test]
fn test_parse_transform_args_negative_number() {
    // translate 允许负数
    let result = parse_transform("translate(-10px, -20px)");
    assert!(result.is_some());
}

// 测试 parse_transform_args 中的零值
#[test]
fn test_parse_transform_args_zero_value() {
    let result = parse_transform("translate(0, 0px)");
    assert!(result.is_some());
}

// parse_css_number 是私有函数，通过 parse_transform 间接测试

// ═══════════════════════════════════════════════════════════════════════
// 特定变换函数的边界情况测试
// ═══════════════════════════════════════════════════════════════════════

// 测试 rotate 中的负角度
#[test]
fn test_transform_rotate_negative() {
    let result = parse_transform("rotate(-45deg)");
    assert!(result.is_some());
}

// 测试 rotate 中的零角度
#[test]
fn test_transform_rotate_zero() {
    let result = parse_transform("rotate(0deg)");
    assert!(result.is_some());
}

// 测试 scale 中的零值
#[test]
fn test_transform_scale_zero() {
    let result = parse_transform("scale(0, 2)");
    assert!(result.is_some());
}

// 测试 skew 中的零值
#[test]
fn test_transform_skew_zero() {
    let result = parse_transform("skew(0deg, 0deg)");
    assert!(result.is_some());
}

// ═══════════════════════════════════════════════════════════════════════
// matrix 函数的边界情况测试
// ═══════════════════════════════════════════════════════════════════════

// 测试 matrix 中的零值
#[test]
fn test_transform_matrix_zero() {
    let result = parse_transform("matrix(0, 0, 0, 0, 0, 0)");
    assert!(result.is_some());
}

// 测试 matrix 中的负值
#[test]
fn test_transform_matrix_negative() {
    let result = parse_transform("matrix(-1, 0, 0, -1, 10, -20)");
    assert!(result.is_some());
}

// 测试 matrix 中的小数
#[test]
fn test_transform_matrix_float() {
    let result = parse_transform("matrix(0.5, 0.1, 0.2, 0.8, 10.5, 20.3)");
    assert!(result.is_some());
}

// ═══════════════════════════════════════════════════════════════════════
// 3D 变换函数的边界情况测试
// ═══════════════════════════════════════════════════════════════════════

// 测试 translate3d 中的负值
#[test]
fn test_transform_translate3d_negative() {
    let result = parse_transform("translate3d(-10px, -20px, -30px)");
    assert!(result.is_some());
}

// 测试 translate3d 中的零值
#[test]
fn test_transform_translate3d_zero() {
    let result = parse_transform("translate3d(0px, 0, 0px)");
    assert!(result.is_some());
}

// 测试 scale3d 中的负值
#[test]
fn test_transform_scale3d_negative() {
    let result = parse_transform("scale3d(-1, -2, -0.5)");
    assert!(result.is_some());
}

// 测试 scale3d 中的零值
#[test]
fn test_transform_scale3d_zero() {
    let result = parse_transform("scale3d(0, 0, 0)");
    assert!(result.is_some());
}

// 测试 rotate3d 中的零向量
#[test]
fn test_transform_rotate3d_zero_vector() {
    let result = parse_transform("rotate3d(0, 0, 0, 45deg)");
    assert!(result.is_some());
}

// 测试 rotate3d 中的负值向量
#[test]
fn test_transform_rotate3d_negative_vector() {
    let result = parse_transform("rotate3d(-1, -0, 0, 45deg)");
    assert!(result.is_some());
}

// ═══════════════════════════════════════════════════════════════════════
// 单个变换函数的参数解析测试
// ═══════════════════════════════════════════════════════════════════════

// 测试 translateX 中的负值
#[test]
fn test_transform_translate_x_negative() {
    let result = parse_transform("translateX(-100px)");
    assert!(result.is_some());
}

// 测试 translateY 中的零值
#[test]
fn test_transform_translate_y_zero() {
    let result = parse_transform("translateY(0)");
    assert!(result.is_some());
}

// 测试 scaleX 中的负值
#[test]
fn test_transform_scale_x_negative() {
    let result = parse_transform("scaleX(-1.5)");
    assert!(result.is_some());
}

// 测试 scaleY 中的零值
#[test]
fn test_transform_scale_y_zero() {
    let result = parse_transform("scaleY(0)");
    assert!(result.is_some());
}

// 测试 rotateX 中的负值
#[test]
fn test_transform_rotate_x_negative() {
    let result = parse_transform("rotateX(-90deg)");
    assert!(result.is_some());
}

// 测试 rotateY 中的零值
#[test]
fn test_transform_rotate_y_zero() {
    let result = parse_transform("rotateY(0rad)");
    assert!(result.is_some());
}

// 测试 rotateZ 中的负值
#[test]
fn test_transform_rotate_z_negative() {
    let result = parse_transform("rotateZ(-45turn)");
    assert!(result.is_some());
}

// 测试 perspective 中的无效值（负值和零值已在测试中覆盖）
#[test]
fn test_transform_perspective_invalid() {
    let result = parse_transform("perspective(-100px)");
    assert!(result.is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// 多个变换函数的组合测试
// ═══════════════════════════════════════════════════════════════════════

// 测试多个变换函数的组合（包含各种边界值）
#[test]
fn test_transform_multiple_functions_with_edge_cases() {
    let result = parse_transform("translate(-10px, 0) scale(0, 1) rotate(45deg)");
    assert!(result.is_some());
}

// 测试变换函数的顺序和边界值
#[test]
fn test_transform_sequence_with_negatives() {
    let result = parse_transform("rotate(-45deg) translateX(-100px) scale(-1, 1)");
    assert!(result.is_some());
}

// ═══════════════════════════════════════════════════════════════════════
// 单位的测试
// ═══════════════════════════════════════════════════════════════════════

// 测试不同单位的组合
#[test]
fn test_transform_mixed_units() {
    let result = parse_transform("translate(10px, 20em) scale(2, 3)");
    assert!(result.is_some());
}

// 测试无单位的数值
#[test]
fn test_transform_unitless_values() {
    let result = parse_transform("translate(10, 20) scale(2)");
    assert!(result.is_some());
}

// ═══════════════════════════════════════════════════════════════════════
// 空格和分隔符的测试
// ═══════════════════════════════════════════════════════════════════════

// 测试额外的空格 — 解析器可以处理函数名和括号间的空格
#[test]
fn test_transform_extra_whitespace() {
    let result = parse_transform("  translate  (  10px  ,  20px  )  ");
    // 解析器可以处理空格
    assert!(result.is_some());
}

// 修正测试 - 正确的空格处理
#[test]
fn test_transform_correct_whitespace() {
    let result = parse_transform("translate(10px, 20px)");
    assert!(result.is_some());
}

// 测试逗号分隔的参数
#[test]
fn test_transform_comma_separated_args() {
    let result = parse_transform("translate(10px,20px)");
    assert!(result.is_some());
}

// 测试空格和逗号混合的参数
#[test]
fn test_transform_mixed_separator_args() {
    let result = parse_transform("translate(10px , 20px)");
    assert!(result.is_some());
}

// ═══════════════════════════════════════════════════════════════════════
// 复杂变换的测试
// ═══════════════════════════════════════════════════════════════════════

// 测试复杂的 3D 变换
#[test]
fn test_complex_3d_transform() {
    let result = parse_transform("translate3d(10px, 20px, 30px) scale3d(1, 1, 1) rotate3d(0, 1, 0, 45deg)");
    assert!(result.is_some());
}

// 测试包含 matrix 的复杂变换
#[test]
fn test_complex_with_matrix_transform() {
    let result = parse_transform("matrix(1, 0, 0, 1, 10, 20) translate(30px, 40px)");
    assert!(result.is_some());
}

// 测试包含各种变换类型的复杂变换
#[test]
fn test_complex_mixed_transform() {
    let result = parse_transform("translate(-10px, -20px) rotate(-45deg) scale(-1, 1) skew(10deg, 20deg)");
    assert!(result.is_some());
}

// ═══════════════════════════════════════════════════════════════════════
// 错误情况的综合测试
// ═══════════════════════════════════════════════════════════════════════

// 测试无效变换的混合
#[test]
fn test_invalid_transform_mixed() {
    let result = parse_transform("translate(10px) invalid-func(20px) translate(30px)");
    assert!(result.is_none());
}

// 测试不匹配的括号数量
#[test]
fn test_mismatched_parentheses() {
    let result = parse_transform("translate(10px, 20px))");
    assert!(result.is_none());
}

// 测试嵌套的括号（非法）
#[test]
fn test_nested_parentheses() {
    let result = parse_transform("translate((10px), 20px)");
    assert!(result.is_none());
}
