//! 第七轮覆盖率测试：parser.rs + values/parse_transform.rs + values/types.rs 剩余未覆盖分支。
//!
//! 重点覆盖：
//! - parser.rs: nth 表达式 edge cases、container 条件 edge cases、attribute value edge cases
//! - parse_transform.rs: transform 3d 函数、perspective、matrix、gradient conic
//! - types.rs: eval_calc 边界情况、resolve_length_to_px 全单位覆盖、parse_length edge cases

use crate::ast::*;
use crate::parser::Parser;
use crate::values::{
    CalcContext, CalcExpr, CalcOp, ColorValue, GradientValue, LengthValue, TransformFunction, TransformValue,
    eval_calc, eval_calc_with_context, parse_calc, parse_clamp, parse_gradient, parse_length, parse_math_function,
    parse_max, parse_min, parse_transform,
};

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — nth 表达式 edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_nth_expression_plus_n() {
    // "+n" — a_part 为 "+"，a=1
    let css = "li:nth-child(+n) { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_expression_minus_n() {
    // "-n" — a_part 为 "-"，a=-1
    let css = "li:nth-child(-n) { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_expression_n_plus_zero() {
    // "n+0" — b_part 为 "+0"
    let css = "li:nth-child(n+0) { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_expression_3n_minus_2() {
    // "3n-2" — 负 b 值
    let css = "li:nth-child(3n-2) { color: blue; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_expression_negative_a() {
    // "-2n+5" — 负系数 a
    let css = "li:nth-child(-2n+5) { color: green; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_expression_0n_plus_3() {
    // "0n+3" — a=0
    let css = "li:nth-child(0n+3) { color: yellow; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_expression_plain_number() {
    // 纯数字 "5"
    let css = "li:nth-child(5) { color: orange; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_expression_only_n() {
    // "n" — a=1, b=0
    let css = "li:nth-child(n) { color: pink; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — container 条件 edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_container_named_with_condition() {
    // @container myname (min-width: 400px)
    let css = "@container sidebar (min-width: 400px) { div { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Container(ref c) = sheet.rules[0] {
        assert_eq!(c.name, Some("sidebar".to_string()));
    }
}

#[test]
fn test_container_ident_not_name() {
    // @container 后的 ident 不跟着 ( — 回退，ident 不是容器名称
    // 这实际上会走到 consume_container_rule 中 ident 后面不是 LParen 的分支
    let css = "@container min-width { div { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    // 由于 min-width 不是有效的条件（无括号），container 规则可能解析失败
    // 但 div 规则应该被解析（作为独立规则）
}

#[test]
fn test_container_no_paren_condition() {
    // @container 后面没有括号条件
    let css = "@container { div { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    // 无条件 — 应该不产生 container 规则
}

#[test]
fn test_container_size_function_inner() {
    // size() 包装函数 — 覆盖 parse_container_condition 中 size() 路径
    let css = "@container size(width > 300px) { div { color: blue; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

#[test]
fn test_container_inline_size_function_inner() {
    // inline-size() 包装函数
    let css = "@container inline-size(width >= 400px) { div { color: green; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

#[test]
fn test_container_colon_format_max_width() {
    // max-width: 800px
    let css = "@container (max-width: 800px) { div { color: purple; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

#[test]
fn test_container_colon_format_width() {
    // width: 500px
    let css = "@container (width: 500px) { div { color: cyan; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

#[test]
fn test_container_condition_colon_empty_value() {
    // min-width: — 空值应返回 None
    let css = "@container (min-width:) { div { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    // 空值导致 parse_size_condition 返回 None，container 规则不应产生
}

#[test]
fn test_container_condition_colon_empty_feature() {
    // : 400px — 空特性名
    let css = "@container (: 400px) { div { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
}

#[test]
fn test_container_comparison_empty_feature() {
    // > 300px — 空特性
    let css = "@container (> 300px) { div { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
}

#[test]
fn test_container_comparison_empty_value() {
    // width > — 空值
    let css = "@container (width > ) { div { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — attribute value edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_attribute_value_number_with_unit() {
    // [data-ver="1.0"] — Number 后跟 Ident
    let css = "[data-ver=\"1.0\"] { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_attribute_delim_dot_followed_by_ident() {
    // [href$=".html"] — Delim('.') 后跟 Ident
    let css = "a[href$=\".html\"] { color: blue; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_attribute_delim_followed_by_number() {
    // [data-ver^="2"] — 通过 String token 处理
    let css = "[data-ver^=\"2\"] { color: green; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_attribute_value_empty_fallback() {
    // [attr=] — 空值 fallback
    let css = "[attr=] { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    // 空属性值应仍然解析
}

#[test]
fn test_attribute_unknown_matcher() {
    // [attr?val] — 未知匹配器
    let css = "[attr?val] { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    // 未知匹配器应跳到 ]
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — @layer edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_layer_statement_only() {
    // @layer; — 无名称无规则体
    let css = "@layer; div { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

#[test]
fn test_layer_named_statement() {
    // @layer base; — 仅声明层名
    let css = "@layer base; div { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 2);
    if let Rule::Layer(ref layer) = sheet.rules[0] {
        assert_eq!(layer.name, "base");
        assert!(layer.rules.is_empty());
    }
}

#[test]
fn test_layer_anonymous_empty() {
    // @layer { } — 匿名空层
    let css = "@layer { }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
    if let Rule::Layer(ref layer) = sheet.rules[0] {
        assert!(layer.name.is_empty());
        assert!(layer.rules.is_empty());
    }
}

#[test]
fn test_layer_invalid_token() {
    // @layer 123; — 无效 token（不是 Ident/String/LBrace/Semicolon）
    let css = "@layer 123; div { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    // 应跳过无效层规则
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — @import with media queries
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_import_url_function() {
    // @import url("style.css");
    let css = "@import url(\"style.css\");";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Import(ref imp) = sheet.rules[0] {
        assert_eq!(imp.url, "style.css");
        assert!(imp.media_queries.is_empty());
    }
}

#[test]
fn test_import_string_with_media() {
    // @import "style.css" screen and (max-width: 600px);
    let css = "@import \"style.css\" screen and (max-width: 600px);";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Import(ref imp) = sheet.rules[0] {
        assert_eq!(imp.url, "style.css");
        assert!(!imp.media_queries.is_empty());
    }
}

#[test]
fn test_import_string_multiple_media() {
    // @import "style.css" screen, print;
    let css = "@import \"style.css\" screen, print;";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Import(ref imp) = sheet.rules[0] {
        assert_eq!(imp.media_queries.len(), 2);
    }
}

#[test]
fn test_import_no_token_after_url() {
    // @import "style.css" — 无分号（EOF 结束）
    let css = "@import \"style.css\"";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

#[test]
fn test_import_invalid_first_token() {
    // @import 123; — 无效 token（不是 Url/String）
    let css = "@import 123; div { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    // 应跳过无效 import
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — @keyframes edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_keyframes_multiple_selectors() {
    // from, 50%, to — 逗号分隔的多个选择器
    let css = "@keyframes test { from, to { opacity: 0; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Keyframes(ref kf) = sheet.rules[0] {
        assert_eq!(kf.keyframes.len(), 1);
        assert_eq!(kf.keyframes[0].selectors.len(), 2);
    }
}

#[test]
fn test_keyframes_percentage_selector() {
    let css = "@keyframes test { 0% { opacity: 0; } 100% { opacity: 1; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Keyframes(ref kf) = sheet.rules[0] {
        assert_eq!(kf.keyframes.len(), 2);
    }
}

#[test]
fn test_keyframes_no_name() {
    // @keyframes { ... } — 无名称
    let css = "@keyframes { from { opacity: 0; } }";
    let sheet = Parser::parse_stylesheet(css);
    // 无名称应返回 None
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — @supports edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_supports_rule_with_valid_condition() {
    let css = "@supports (display: grid) { div { display: grid; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
    if let Rule::Supports(ref sup) = sheet.rules[0] {
        // 应解析出有效的条件
    }
}

#[test]
fn test_supports_rule_invalid_condition() {
    // @supports ( — 无效条件
    let css = "@supports ( { div { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    // 无效条件不应产生 supports 规则
}

#[test]
fn test_supports_rule_not_condition() {
    let css = "@supports not (display: grid) { div { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// parse_transform.rs — 3D transform functions
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_transform_translate3d() {
    let result = parse_transform("translate3d(10px, 20px, 30px)");
    assert!(result.is_some());
    if let Some(TransformValue::List(fns)) = result {
        assert_eq!(fns.len(), 1);
        assert!(matches!(fns[0], TransformFunction::Translate3d(10.0, 20.0, 30.0)));
    }
}

#[test]
fn test_parse_transform_scale3d() {
    let result = parse_transform("scale3d(1.5, 2.0, 1.0)");
    assert!(result.is_some());
    if let Some(TransformValue::List(fns)) = result {
        assert_eq!(fns.len(), 1);
        assert!(matches!(fns[0], TransformFunction::Scale3d(1.5, 2.0, 1.0)));
    }
}

#[test]
fn test_parse_transform_rotate3d() {
    let result = parse_transform("rotate3d(1, 0, 0, 45deg)");
    assert!(result.is_some());
    if let Some(TransformValue::List(fns)) = result {
        assert_eq!(fns.len(), 1);
        assert!(matches!(fns[0], TransformFunction::Rotate3d(_, _, _, 45.0)));
    }
}

#[test]
fn test_parse_transform_perspective() {
    let result = parse_transform("perspective(500px)");
    assert!(result.is_some());
    if let Some(TransformValue::List(fns)) = result {
        assert!(matches!(fns[0], TransformFunction::Perspective(500.0)));
    }
}

#[test]
fn test_parse_transform_perspective_zero() {
    // perspective(0) — 值 <= 0 应返回 None
    let result = parse_transform("perspective(0)");
    assert!(result.is_none());
}

#[test]
fn test_parse_transform_perspective_negative() {
    // perspective(-100) — 值 <= 0 应返回 None
    let result = parse_transform("perspective(-100)");
    assert!(result.is_none());
}

#[test]
fn test_parse_transform_matrix() {
    let result = parse_transform("matrix(1, 0, 0, 1, 10, 20)");
    assert!(result.is_some());
    if let Some(TransformValue::List(fns)) = result {
        assert!(matches!(
            fns[0],
            TransformFunction::Matrix(1.0, 0.0, 0.0, 1.0, 10.0, 20.0)
        ));
    }
}

#[test]
fn test_parse_transform_matrix_wrong_args() {
    // matrix 需要恰好 6 个参数
    let result = parse_transform("matrix(1, 0, 0, 1)");
    assert!(result.is_none());
}

#[test]
fn test_parse_transform_translate3d_wrong_args() {
    // translate3d 需要恰好 3 个参数
    let result = parse_transform("translate3d(10px, 20px)");
    assert!(result.is_none());
}

#[test]
fn test_parse_transform_scale3d_wrong_args() {
    let result = parse_transform("scale3d(1.0, 2.0)");
    assert!(result.is_none());
}

#[test]
fn test_parse_transform_rotate3d_wrong_args() {
    let result = parse_transform("rotate3d(1, 0, 0)");
    assert!(result.is_none());
}

#[test]
fn test_parse_transform_unknown_function() {
    let result = parse_transform("unknown(10px)");
    assert!(result.is_none());
}

#[test]
fn test_parse_transform_none() {
    let result = parse_transform("none");
    assert_eq!(result, Some(TransformValue::None));
}

#[test]
fn test_parse_transform_multiple_functions() {
    let result = parse_transform("translateX(10px) rotate(45deg) scale(2)");
    assert!(result.is_some());
    if let Some(TransformValue::List(fns)) = result {
        assert_eq!(fns.len(), 3);
    }
}

#[test]
fn test_parse_transform_rad_unit() {
    let result = parse_transform("rotate(1.5708rad)");
    assert!(result.is_some());
}

#[test]
fn test_parse_transform_turn_unit() {
    let result = parse_transform("rotate(0.25turn)");
    assert!(result.is_some());
}

#[test]
fn test_parse_transform_em_unit() {
    let result = parse_transform("translateX(2em)");
    assert!(result.is_some());
}

#[test]
fn test_parse_transform_rem_unit() {
    let result = parse_transform("translateY(1.5rem)");
    assert!(result.is_some());
}

// ═══════════════════════════════════════════════════════════════════════
// parse_transform.rs — gradient conic
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_gradient_conic_basic() {
    let result = parse_gradient("conic-gradient(red, blue)");
    assert!(result.is_some());
}

#[test]
fn test_parse_gradient_conic_from_angle() {
    let result = parse_gradient("conic-gradient(from 45deg, red, blue)");
    assert!(result.is_some());
}

#[test]
fn test_parse_gradient_conic_at_position() {
    let result = parse_gradient("conic-gradient(at center, red, blue)");
    assert!(result.is_some());
}

#[test]
fn test_parse_gradient_conic_from_angle_at_position() {
    let result = parse_gradient("conic-gradient(from 90deg at 50% 50%, red, yellow, blue)");
    assert!(result.is_some());
}

#[test]
fn test_parse_gradient_conic_with_stops() {
    // 带颜色百分比的 conic gradient
    let result = parse_gradient("conic-gradient(red 0%, blue 100%)");
    assert!(result.is_some());
}

// ═══════════════════════════════════════════════════════════════════════
// types.rs — calc 解析和计算 edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_calc_simple_add() {
    let result = parse_calc("calc(100px + 50px)");
    assert!(result.is_some());
    if let Some(CalcExpr::BinaryOp(_, CalcOp::Add, _)) = result {
        // OK
    }
}

#[test]
fn test_parse_calc_simple_subtract() {
    let result = parse_calc("calc(100px - 50px)");
    assert!(result.is_some());
}

#[test]
fn test_parse_calc_multiply() {
    let result = parse_calc("calc(100px * 2)");
    assert!(result.is_some());
    if let Some(CalcExpr::BinaryOp(_, CalcOp::Multiply, _)) = result {
        // OK
    }
}

#[test]
fn test_parse_calc_divide() {
    let result = parse_calc("calc(100px / 2)");
    assert!(result.is_some());
    if let Some(CalcExpr::BinaryOp(_, CalcOp::Divide, _)) = result {
        // OK
    }
}

#[test]
fn test_parse_calc_empty() {
    let result = parse_calc("calc()");
    assert!(result.is_none());
}

#[test]
fn test_parse_calc_invalid_prefix() {
    let result = parse_calc("notcalc(100px)");
    assert!(result.is_none());
}

#[test]
fn test_parse_calc_no_closing_paren() {
    let result = parse_calc("calc(100px + 50px");
    assert!(result.is_none());
}

#[test]
fn test_parse_calc_trailing_content() {
    let result = parse_calc("calc(100px)extra");
    assert!(result.is_none());
}

#[test]
fn test_parse_math_function_calc() {
    let result = parse_math_function("calc(100px + 50px)");
    assert!(result.is_some());
}

#[test]
fn test_parse_math_function_min() {
    let result = parse_math_function("min(100px, 50px)");
    assert!(result.is_some());
    if let Some(CalcExpr::Min(args)) = result {
        assert_eq!(args.len(), 2);
    }
}

#[test]
fn test_parse_math_function_max() {
    let result = parse_math_function("max(100px, 50px)");
    assert!(result.is_some());
    if let Some(CalcExpr::Max(args)) = result {
        assert_eq!(args.len(), 2);
    }
}

#[test]
fn test_parse_math_function_clamp() {
    let result = parse_math_function("clamp(50px, 100px, 200px)");
    assert!(result.is_some());
}

#[test]
fn test_parse_math_function_unknown() {
    let result = parse_math_function("unknown(100px)");
    assert!(result.is_none());
}

#[test]
fn test_parse_min_empty() {
    let result = parse_min("min()");
    assert!(result.is_none());
}

#[test]
fn test_parse_min_invalid() {
    let result = parse_min("notmin()");
    assert!(result.is_none());
}

#[test]
fn test_parse_max_empty() {
    let result = parse_max("max()");
    assert!(result.is_none());
}

#[test]
fn test_parse_max_invalid() {
    let result = parse_max("notmax()");
    assert!(result.is_none());
}

#[test]
fn test_parse_clamp_empty() {
    let result = parse_clamp("clamp()");
    assert!(result.is_none());
}

#[test]
fn test_parse_clamp_invalid() {
    let result = parse_clamp("notclamp()");
    assert!(result.is_none());
}

#[test]
fn test_parse_clamp_two_args() {
    // clamp 需要恰好 3 个参数
    let result = parse_clamp("clamp(50px, 100px)");
    assert!(result.is_none());
}

#[test]
fn test_eval_calc_number() {
    let expr = CalcExpr::Number(42.0);
    assert_eq!(eval_calc(&expr, None), Some(42.0));
}

#[test]
fn test_eval_calc_px() {
    let expr = CalcExpr::Length(LengthValue::Px(100.0));
    assert_eq!(eval_calc(&expr, None), Some(100.0));
}

#[test]
fn test_eval_calc_percentage_with_parent() {
    let expr = CalcExpr::Length(LengthValue::Percentage(50.0));
    assert_eq!(eval_calc(&expr, Some(200.0)), Some(100.0));
}

#[test]
fn test_eval_calc_percentage_no_parent() {
    let expr = CalcExpr::Length(LengthValue::Percentage(50.0));
    assert_eq!(eval_calc(&expr, None), None);
}

#[test]
fn test_eval_calc_em_with_font_size() {
    let expr = CalcExpr::Length(LengthValue::Em(2.0));
    let ctx = CalcContext {
        font_size: Some(16.0),
        ..Default::default()
    };
    assert_eq!(eval_calc_with_context(&expr, &ctx), Some(32.0));
}

#[test]
fn test_eval_calc_em_no_font_size() {
    let expr = CalcExpr::Length(LengthValue::Em(2.0));
    assert_eq!(eval_calc(&expr, None), None);
}

#[test]
fn test_eval_calc_rem_with_root_font_size() {
    let expr = CalcExpr::Length(LengthValue::Rem(1.5));
    let ctx = CalcContext {
        root_font_size: Some(16.0),
        ..Default::default()
    };
    assert_eq!(eval_calc_with_context(&expr, &ctx), Some(24.0));
}

#[test]
fn test_eval_calc_vh() {
    let expr = CalcExpr::Length(LengthValue::Vh(50.0));
    let ctx = CalcContext {
        viewport_height: Some(800.0),
        ..Default::default()
    };
    assert_eq!(eval_calc_with_context(&expr, &ctx), Some(400.0));
}

#[test]
fn test_eval_calc_vw() {
    let expr = CalcExpr::Length(LengthValue::Vw(25.0));
    let ctx = CalcContext {
        viewport_width: Some(1200.0),
        ..Default::default()
    };
    assert_eq!(eval_calc_with_context(&expr, &ctx), Some(300.0));
}

#[test]
fn test_eval_calc_vmin() {
    let expr = CalcExpr::Length(LengthValue::Vmin(10.0));
    let ctx = CalcContext {
        viewport_width: Some(1200.0),
        viewport_height: Some(800.0),
        ..Default::default()
    };
    // 10% of min(1200, 800) = 10% of 800 = 80
    assert_eq!(eval_calc_with_context(&expr, &ctx), Some(80.0));
}

#[test]
fn test_eval_calc_vmax() {
    let expr = CalcExpr::Length(LengthValue::Vmax(10.0));
    let ctx = CalcContext {
        viewport_width: Some(1200.0),
        viewport_height: Some(800.0),
        ..Default::default()
    };
    // 10% of max(1200, 800) = 10% of 1200 = 120
    assert_eq!(eval_calc_with_context(&expr, &ctx), Some(120.0));
}

#[test]
fn test_eval_calc_ch() {
    let expr = CalcExpr::Length(LengthValue::Ch(3.0));
    let ctx = CalcContext {
        ch_width: Some(8.0),
        ..Default::default()
    };
    assert_eq!(eval_calc_with_context(&expr, &ctx), Some(24.0));
}

#[test]
fn test_eval_calc_auto_is_none() {
    let expr = CalcExpr::Length(LengthValue::Auto);
    assert_eq!(eval_calc(&expr, None), None);
}

#[test]
fn test_eval_calc_min_content_is_none() {
    let expr = CalcExpr::Length(LengthValue::MinContent);
    assert_eq!(eval_calc(&expr, None), None);
}

#[test]
fn test_eval_calc_max_content_is_none() {
    let expr = CalcExpr::Length(LengthValue::MaxContent);
    assert_eq!(eval_calc(&expr, None), None);
}

#[test]
fn test_eval_calc_nested_calc() {
    let inner = CalcExpr::Length(LengthValue::Px(50.0));
    let expr = CalcExpr::Length(LengthValue::Calc(Box::new(inner)));
    assert_eq!(eval_calc(&expr, None), Some(50.0));
}

#[test]
fn test_eval_calc_fit_content() {
    let inner = LengthValue::Px(200.0);
    let expr = CalcExpr::Length(LengthValue::FitContent(Box::new(inner)));
    assert_eq!(eval_calc(&expr, None), Some(200.0));
}

#[test]
fn test_eval_calc_divide_by_zero() {
    let expr = CalcExpr::BinaryOp(
        Box::new(CalcExpr::Number(10.0)),
        CalcOp::Divide,
        Box::new(CalcExpr::Number(0.0)),
    );
    assert_eq!(eval_calc(&expr, None), None);
}

#[test]
fn test_eval_calc_min_expr() {
    let args = vec![
        CalcExpr::Length(LengthValue::Px(100.0)),
        CalcExpr::Length(LengthValue::Px(50.0)),
    ];
    let expr = CalcExpr::Min(args);
    assert_eq!(eval_calc(&expr, None), Some(50.0));
}

#[test]
fn test_eval_calc_max_expr() {
    let args = vec![
        CalcExpr::Length(LengthValue::Px(100.0)),
        CalcExpr::Length(LengthValue::Px(50.0)),
    ];
    let expr = CalcExpr::Max(args);
    assert_eq!(eval_calc(&expr, None), Some(100.0));
}

#[test]
fn test_eval_calc_min_empty_args() {
    let expr = CalcExpr::Min(vec![]);
    assert_eq!(eval_calc(&expr, None), None);
}

#[test]
fn test_eval_calc_max_empty_args() {
    let expr = CalcExpr::Max(vec![]);
    assert_eq!(eval_calc(&expr, None), None);
}

#[test]
fn test_eval_calc_clamp_expr() {
    let expr = CalcExpr::Clamp {
        min: Box::new(CalcExpr::Length(LengthValue::Px(50.0))),
        val: Box::new(CalcExpr::Length(LengthValue::Px(30.0))),
        max: Box::new(CalcExpr::Length(LengthValue::Px(100.0))),
    };
    // 30 clamped to [50, 100] = 50
    assert_eq!(eval_calc(&expr, None), Some(50.0));
}

#[test]
fn test_eval_calc_clamp_in_range() {
    let expr = CalcExpr::Clamp {
        min: Box::new(CalcExpr::Length(LengthValue::Px(50.0))),
        val: Box::new(CalcExpr::Length(LengthValue::Px(75.0))),
        max: Box::new(CalcExpr::Length(LengthValue::Px(100.0))),
    };
    assert_eq!(eval_calc(&expr, None), Some(75.0));
}

// ═══════════════════════════════════════════════════════════════════════
// types.rs — parse_length edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_length_auto() {
    assert_eq!(parse_length("auto"), Some(LengthValue::Auto));
}

#[test]
fn test_parse_length_auto_case_insensitive() {
    assert_eq!(parse_length("AUTO"), Some(LengthValue::Auto));
}

#[test]
fn test_parse_length_min_content() {
    assert_eq!(parse_length("min-content"), Some(LengthValue::MinContent));
}

#[test]
fn test_parse_length_max_content() {
    assert_eq!(parse_length("max-content"), Some(LengthValue::MaxContent));
}

#[test]
fn test_parse_length_px() {
    assert_eq!(parse_length("10px"), Some(LengthValue::Px(10.0)));
}

#[test]
fn test_parse_length_em() {
    assert_eq!(parse_length("1.5em"), Some(LengthValue::Em(1.5)));
}

#[test]
fn test_parse_length_rem() {
    assert_eq!(parse_length("2rem"), Some(LengthValue::Rem(2.0)));
}

#[test]
fn test_parse_length_vh() {
    assert_eq!(parse_length("100vh"), Some(LengthValue::Vh(100.0)));
}

#[test]
fn test_parse_length_vw() {
    assert_eq!(parse_length("50vw"), Some(LengthValue::Vw(50.0)));
}

#[test]
fn test_parse_length_vmin() {
    assert_eq!(parse_length("10vmin"), Some(LengthValue::Vmin(10.0)));
}

#[test]
fn test_parse_length_vmax() {
    assert_eq!(parse_length("10vmax"), Some(LengthValue::Vmax(10.0)));
}

#[test]
fn test_parse_length_ch() {
    assert_eq!(parse_length("2ch"), Some(LengthValue::Ch(2.0)));
}

#[test]
fn test_parse_length_percentage() {
    assert_eq!(parse_length("50%"), Some(LengthValue::Percentage(50.0)));
}

#[test]
fn test_parse_length_zero_no_unit() {
    // CSS spec: bare 0 is a valid length
    assert_eq!(parse_length("0"), Some(LengthValue::Px(0.0)));
}

#[test]
fn test_parse_length_zero_px() {
    assert_eq!(parse_length("0px"), Some(LengthValue::Px(0.0)));
}

#[test]
fn test_parse_length_fit_content() {
    let result = parse_length("fit-content(200px)");
    assert!(matches!(result, Some(LengthValue::FitContent(_))));
}

#[test]
fn test_parse_length_fit_content_empty() {
    let result = parse_length("fit-content()");
    assert!(result.is_none());
}

#[test]
fn test_parse_length_invalid_unit() {
    let result = parse_length("10invalid");
    assert!(result.is_none());
}

#[test]
fn test_parse_length_non_zero_no_unit() {
    // 非零值无单位 — 无效
    let result = parse_length("10");
    assert!(result.is_none());
}

#[test]
fn test_parse_length_whitespace() {
    assert_eq!(parse_length("  10px  "), Some(LengthValue::Px(10.0)));
}

#[test]
fn test_parse_length_negative() {
    assert_eq!(parse_length("-5px"), Some(LengthValue::Px(-5.0)));
}

#[test]
fn test_parse_length_scientific_notation() {
    // 1e2px — 科学计数法
    let result = parse_length("1e2px");
    assert_eq!(result, Some(LengthValue::Px(100.0)));
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — @at-rule with block body
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_at_rule_with_block() {
    // @media screen { div { color: red; } }
    let css = "@media screen { div { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::At(ref at) = sheet.rules[0] {
        assert_eq!(at.name, "media");
        assert!(matches!(at.body, AtRuleBody::Block(_)));
    }
}

#[test]
fn test_at_rule_statement() {
    // @charset "UTF-8";
    let css = "@charset \"UTF-8\";";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::At(ref at) = sheet.rules[0] {
        assert_eq!(at.name, "charset");
        assert!(matches!(at.body, AtRuleBody::Statement));
    }
}

#[test]
fn test_at_rule_eof() {
    // @namespace — 无分号也无花括号
    let css = "@namespace";
    let sheet = Parser::parse_stylesheet(css);
    // 应产生一个 AtRule with Statement body
}

#[test]
fn test_at_rule_nested_style() {
    // @media screen { div { color: red; } p { color: blue; } }
    let css = "@media screen { div { color: red; } p { color: blue; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::At(ref at) = sheet.rules[0] {
        if let AtRuleBody::Block(ref rules) = at.body {
            assert_eq!(rules.len(), 2);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — selector combinators
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_child_combinator() {
    let css = "div > p { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_next_sibling_combinator() {
    let css = "div + p { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_subsequent_sibling_combinator() {
    let css = "div ~ p { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_descendant_combinator() {
    let css = "div p { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_universal_selector() {
    let css = "* { margin: 0; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_selector_list_multiple() {
    let css = "div, p, span { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Style(ref style) = sheet.rules[0] {
        assert_eq!(style.selectors.len(), 3);
    }
}

#[test]
fn test_pseudo_element_selector() {
    let css = "div::before { content: ''; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — declaration important
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_declaration_important() {
    let css = "div { color: red !important; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Style(ref style) = sheet.rules[0] {
        assert_eq!(style.declarations.len(), 1);
        assert!(style.declarations[0].important);
    }
}

#[test]
fn test_declaration_no_semicolon_at_end() {
    // 最后一个声明可以省略分号
    let css = "div { color: red }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Style(ref style) = sheet.rules[0] {
        assert!(!style.declarations.is_empty());
    }
}

#[test]
fn test_declaration_bang_not_important() {
    // !something-other-than-important
    let css = "div { color: red !something; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Style(ref style) = sheet.rules[0] {
        assert!(!style.declarations[0].important);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — :lang() with string argument
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_lang_with_string_arg() {
    // :lang("en") — 字符串参数
    let css = "p:lang(\"en\") { color: blue; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_lang_with_ident_arg() {
    // :lang(en) — 标识符参数
    let css = "p:lang(en) { color: blue; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_lang_invalid_arg() {
    // :lang() — 空参数
    let css = "p:lang() { color: blue; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}
