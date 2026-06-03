//! types.rs 第二轮覆盖率测试。
//!
//! 重点覆盖：
//! - calc() 减法、乘法、除法运算
//! - calc() 嵌套括号表达式
//! - min()/max()/clamp() 解析与求值
//! - eval_calc_with_context 各种单位（Vmin、Vmax、Ch）
//! - 负数因子解析
//! - parse_math_function 入口
//! - parse_atom 空白/空值路径

use super::*;
use crate::values::{
    CalcContext, CalcExpr, CalcOp, LengthValue, eval_calc, eval_calc_with_context, parse_calc, parse_clamp,
    parse_math_function, parse_max, parse_min,
};

// ═══════════════════════════════════════════════════════════════════════
// calc() 减法运算 (line 447)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_calc_subtraction() {
    let expr = parse_calc("calc(100px - 30px)").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, Some(70.0));
}

#[test]
fn test_calc_subtraction_negative_result() {
    let expr = parse_calc("calc(10px - 50px)").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, Some(-40.0));
}

// ═══════════════════════════════════════════════════════════════════════
// calc() 乘法运算 (line 466)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_calc_multiplication() {
    let expr = parse_calc("calc(10 * 5)").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, Some(50.0));
}

#[test]
fn test_calc_multiply_px_by_number() {
    let expr = parse_calc("calc(100px * 2)").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, Some(200.0));
}

// ═══════════════════════════════════════════════════════════════════════
// calc() 除法运算 (line 470)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_calc_division() {
    let expr = parse_calc("calc(100px / 4)").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, Some(25.0));
}

#[test]
fn test_calc_division_by_zero() {
    let expr = parse_calc("calc(100px / 0)").unwrap();
    let result = eval_calc(&expr, None);
    assert!(result.is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// calc() 括号表达式 (line 568, 571)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_calc_parenthesized_expression() {
    let expr = parse_calc("calc((100px + 50px) * 2)").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, Some(300.0));
}

#[test]
fn test_calc_nested_parens() {
    let expr = parse_calc("calc(((10 + 5)))").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, Some(15.0));
}

// ═══════════════════════════════════════════════════════════════════════
// min() 解析与求值 (line 519, 522, 699, 708, 711)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_min_basic() {
    let expr = parse_min("min(100px, 200px)").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, Some(100.0));
}

#[test]
fn test_parse_min_three_values() {
    let expr = parse_min("min(50px, 100px, 75px)").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, Some(50.0));
}

#[test]
fn test_parse_min_with_calc() {
    let expr = parse_min("min(calc(100px - 50px), 60px)").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, Some(50.0));
}

#[test]
fn test_parse_min_invalid_prefix() {
    assert!(parse_min("max(100px, 200px)").is_none());
}

#[test]
fn test_parse_min_empty() {
    assert!(parse_min("min()").is_none());
}

#[test]
fn test_parse_min_no_rparen() {
    assert!(parse_min("min(100px, 200px").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// max() 解析与求值 (line 532, 535, 724, 733, 736)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_max_basic() {
    let expr = parse_max("max(100px, 200px)").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, Some(200.0));
}

#[test]
fn test_parse_max_three_values() {
    let expr = parse_max("max(50px, 100px, 75px)").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, Some(100.0));
}

#[test]
fn test_parse_max_invalid_prefix() {
    assert!(parse_max("min(100px, 200px)").is_none());
}

#[test]
fn test_parse_max_empty() {
    assert!(parse_max("max()").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// clamp() 解析与求值 (line 545, 548, 550, 553, 555, 558, 749, 758, 763, 768)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_clamp_basic() {
    let expr = parse_clamp("clamp(50px, 100px, 200px)").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, Some(100.0));
}

#[test]
fn test_parse_clamp_min_bounded() {
    // val < min → should return min
    let expr = parse_clamp("clamp(100px, 50px, 200px)").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, Some(100.0));
}

#[test]
fn test_parse_clamp_max_bounded() {
    // val > max → should return max
    let expr = parse_clamp("clamp(50px, 300px, 200px)").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, Some(200.0));
}

#[test]
fn test_parse_clamp_invalid_prefix() {
    assert!(parse_clamp("min(50px, 100px)").is_none());
}

#[test]
fn test_parse_clamp_empty() {
    assert!(parse_clamp("clamp()").is_none());
}

#[test]
fn test_parse_clamp_no_rparen() {
    assert!(parse_clamp("clamp(50px, 100px, 200px").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// parse_math_function 入口 (line 675)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_math_function_calc() {
    let result = parse_math_function("calc(100px + 50px)");
    assert!(result.is_some());
}

#[test]
fn test_parse_math_function_min() {
    let result = parse_math_function("min(100px, 200px)");
    assert!(result.is_some());
}

#[test]
fn test_parse_math_function_max() {
    let result = parse_math_function("max(100px, 200px)");
    assert!(result.is_some());
}

#[test]
fn test_parse_math_function_clamp() {
    let result = parse_math_function("clamp(50px, 100px, 200px)");
    assert!(result.is_some());
}

#[test]
fn test_parse_math_function_unknown() {
    assert!(parse_math_function("unknown(100px)").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// eval_calc_with_context — BinaryOp (line 802)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_eval_calc_with_context_addition() {
    let expr = parse_calc("calc(50px + 30px)").unwrap();
    let ctx = CalcContext::default();
    let result = eval_calc_with_context(&expr, &ctx);
    assert_eq!(result, Some(80.0));
}

#[test]
fn test_eval_calc_with_context_subtraction() {
    let expr = parse_calc("calc(100px - 30px)").unwrap();
    let ctx = CalcContext::default();
    let result = eval_calc_with_context(&expr, &ctx);
    assert_eq!(result, Some(70.0));
}

#[test]
fn test_eval_calc_with_context_multiplication() {
    let expr = parse_calc("calc(10 * 5)").unwrap();
    let ctx = CalcContext::default();
    let result = eval_calc_with_context(&expr, &ctx);
    assert_eq!(result, Some(50.0));
}

#[test]
fn test_eval_calc_with_context_division() {
    let expr = parse_calc("calc(100px / 4)").unwrap();
    let ctx = CalcContext::default();
    let result = eval_calc_with_context(&expr, &ctx);
    assert_eq!(result, Some(25.0));
}

// ═══════════════════════════════════════════════════════════════════════
// eval_calc_with_context — Clamp (line 833-835)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_eval_clamp_with_context() {
    let expr = parse_clamp("clamp(50px, 100px, 200px)").unwrap();
    let ctx = CalcContext::default();
    let result = eval_calc_with_context(&expr, &ctx);
    assert_eq!(result, Some(100.0));
}

// ═══════════════════════════════════════════════════════════════════════
// eval_calc_with_context — Vmin/Vmax/Ch (line 854, 858)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_resolve_vmin_with_context() {
    let expr = CalcExpr::Length(LengthValue::Vmin(50.0));
    let ctx = CalcContext {
        viewport_width: Some(1000.0),
        viewport_height: Some(600.0),
        ..Default::default()
    };
    // vmin(50) = 50 * min(1000, 600) / 100 = 50 * 600 / 100 = 300
    let result = eval_calc_with_context(&expr, &ctx);
    assert_eq!(result, Some(300.0));
}

#[test]
fn test_resolve_vmax_with_context() {
    let expr = CalcExpr::Length(LengthValue::Vmax(50.0));
    let ctx = CalcContext {
        viewport_width: Some(1000.0),
        viewport_height: Some(600.0),
        ..Default::default()
    };
    // vmax(50) = 50 * max(1000, 600) / 100 = 50 * 1000 / 100 = 500
    let result = eval_calc_with_context(&expr, &ctx);
    assert_eq!(result, Some(500.0));
}

#[test]
fn test_resolve_ch_with_context() {
    let expr = CalcExpr::Length(LengthValue::Ch(10.0));
    let ctx = CalcContext {
        ch_width: Some(8.0),
        ..Default::default()
    };
    // 10ch * 8 = 80
    let result = eval_calc_with_context(&expr, &ctx);
    assert_eq!(result, Some(80.0));
}

#[test]
fn test_resolve_vmin_no_viewport() {
    let expr = CalcExpr::Length(LengthValue::Vmin(50.0));
    let ctx = CalcContext::default();
    // 没有 viewport 尺寸
    assert!(eval_calc_with_context(&expr, &ctx).is_none());
}

#[test]
fn test_resolve_vmax_no_viewport() {
    let expr = CalcExpr::Length(LengthValue::Vmax(50.0));
    let ctx = CalcContext::default();
    assert!(eval_calc_with_context(&expr, &ctx).is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// 负数因子解析 (line 589, 595)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_calc_negative_in_expr() {
    let expr = parse_calc("calc(100px + -30px)").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, Some(70.0));
}

#[test]
fn test_calc_negative_multiplier() {
    let expr = parse_calc("calc(10 * -3)").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, Some(-30.0));
}

// ═══════════════════════════════════════════════════════════════════════
// parse_calc — 混合运算优先级
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_calc_mixed_operations() {
    // 100 + 20 * 3 = 160
    let expr = parse_calc("calc(100px + 20 * 3)").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, Some(160.0));
}

#[test]
fn test_calc_mixed_div_and_add() {
    // 100 + 200 / 4 = 150
    let expr = parse_calc("calc(100px + 200px / 4)").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, Some(150.0));
}

// ═══════════════════════════════════════════════════════════════════════
// parse_atom 空值路径 (line 617)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_calc_whitespace_only_atom() {
    // calc 表达式中连续空白应该无法解析
    assert!(parse_calc("calc( )").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// parse_calc — 百分比求值
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_calc_percentage_with_parent() {
    let expr = parse_calc("calc(50% + 10px)").unwrap();
    let result = eval_calc(&expr, Some(200.0));
    // 50% of 200 = 100, + 10 = 110
    assert_eq!(result, Some(110.0));
}

// ═══════════════════════════════════════════════════════════════════════
// parse_calc — em/rem 求值
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_calc_em_with_context() {
    let expr = parse_calc("calc(2em + 10px)").unwrap();
    let ctx = CalcContext {
        font_size: Some(16.0),
        ..Default::default()
    };
    let result = eval_calc_with_context(&expr, &ctx);
    // 2em = 32, + 10 = 42
    assert_eq!(result, Some(42.0));
}

#[test]
fn test_calc_rem_with_context() {
    let expr = parse_calc("calc(1.5rem + 5px)").unwrap();
    let ctx = CalcContext {
        root_font_size: Some(20.0),
        ..Default::default()
    };
    let result = eval_calc_with_context(&expr, &ctx);
    // 1.5rem = 30, + 5 = 35
    assert_eq!(result, Some(35.0));
}
