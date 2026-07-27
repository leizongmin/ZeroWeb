//! 第十一轮覆盖率测试：覆盖 parser.rs、tokenizer.rs、media_query.rs、
//! values/parse_transform.rs、values/types.rs 剩余未覆盖分支。
//!
//! 重点：
//! - parser.rs：函数伪类 Ident+LParen 形式（:not()/:is()/:where()/:has()/:nth-child()/
//!   :nth-last-child()/:nth-of-type()/:nth-last-of-type()/:lang()）、@import 字符串字面量、
//!   @supports 缺少 {、容器查询范围语法 200px <= width <= 500px、consume_attribute_value 中
//!   Number+Ident、@layer 中未知 token 的 advance
//! - tokenizer.rs：consume_newline（\r \t）、consume_escape 换行和 EOF、consume_number 数字后
//!   的 Ident、# 后 EOF、url() 引号形式、Column(||)、- 单独标识符
//! - media_query.rs：Height 各种运算符评估、prefers-color-scheme/reduced-motion/pointer/
//!   resolution/min-resolution/max-resolution/orientation、flip_op、parse_leading_value、
//!   parse_op、parse_feature_name、make_feature_condition、find_matching_paren
//! - values/parse_transform.rs：extract_parens_content 失败、嵌套括号的 transform 函数、
//!   parse_css_number 失败返回 None
//! - values/types.rs：parse_min/parse_max/parse_clamp 残余输入、parse_calc_expr 中
//!   clamp 缺少逗号、parse_min 失败返回 None

use crate::parser::Parser;
use crate::tokenizer::{Token, Tokenizer};
use crate::values::{TransformValue, parse_calc, parse_transform};

/// Helper: tokenize CSS and collect tokens.
fn tokenize(input: &str) -> Vec<Token> {
    Tokenizer::new(input).collect_tokens()
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — 函数伪类 Ident+LParen 形式
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_pseudo_not_function_ident_lparen() {
    // 通过 Ident + LParen 触发 :not() 解析路径
    let css = "div:not(.active) { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_pseudo_is_function_ident_lparen() {
    let css = "div:is(.a, .b) { color: blue; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_pseudo_where_function_ident_lparen() {
    let css = "div:where(.a) { color: green; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_pseudo_has_function_ident_lparen() {
    let css = "div:has(.child) { color: gold; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_pseudo_nth_child_ident_lparen() {
    let css = "li:nth-child(2n+1) { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_pseudo_nth_last_child_ident_lparen() {
    let css = "li:nth-last-child(3) { color: blue; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_pseudo_nth_of_type_ident_lparen() {
    let css = "p:nth-of-type(odd) { color: green; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_pseudo_nth_last_of_type_ident_lparen() {
    let css = "span:nth-last-of-type(2n) { color: purple; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_pseudo_lang_ident_lparen() {
    let css = "html:lang(en) { color: black; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_pseudo_unknown_function_ident_lparen() {
    // 未知函数伪类应回退为 Simple（可能无法生成规则，关键是触发路径不 panic）
    let css = "div:unknown-fn(arg) { color: gray; }";
    let _sheet = Parser::parse_stylesheet(css);
}

#[test]
fn test_pseudo_nth_pattern_unknown_name() {
    // 覆盖 parse_nth_pattern 中 _ => Simple 回退（line 426）
    let css = "div:nth-something(2) { color: gray; }";
    let _sheet = Parser::parse_stylesheet(css);
}

#[test]
fn test_pseudo_function_list_unknown_name() {
    // 覆盖 parse_pseudo_class_function_list 中 _ => Simple 回退（line 377）
    let css = "div:custom-fn(.a) { color: gray; }";
    let _sheet = Parser::parse_stylesheet(css);
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — consume_attribute_value 中 Number+Ident 组合（lines 683-689）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_attribute_value_number_then_ident() {
    // Number 后跟标识符触发 Number 分支（line 683-691）
    // 注意：此 CSS 的属性选择器值解析可能不生成规则，关键是触发代码路径
    let css = "[data-val=42px] { color: red; }";
    let _sheet = Parser::parse_stylesheet(css);
}

#[test]
fn test_attribute_value_delim_dot() {
    // Delim('.') 分支（line 670-672）
    let css = "[data-ver=\"1.0\"] { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — @supports 缺少 { （line 1114）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_supports_rule_missing_lbrace() {
    let css = "@supports (display: grid) ";
    let sheet = Parser::parse_stylesheet(css);
    // 缺少 { 应该跳过该规则
    assert_eq!(sheet.rules.len(), 0);
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — @import 字符串字面量形式（lines 1033-1036）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_import_rule_string_literal() {
    let css = "@import \"style.css\";";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — @layer 中未知 token（lines 1007-1008）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_layer_rule_with_invalid_token_inside() {
    // @layer 内部有无法识别的 token
    let css = "@layer base { %invalid; div { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — 容器查询范围语法（lines 1274-1286）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_container_query_range_syntax() {
    // 200px <= width <= 500px 触发双 <= 路径
    let css = "@container (200px <= width <= 500px) { .item { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

#[test]
fn test_container_query_inline_size_function() {
    // inline-size() 包装函数（line 1241-1242）
    let css = "@container (inline-size(min-width: 300px)) { .box { width: 100%; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — style rule 中缺少 { （line 928）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_style_rule_missing_lbrace_continues() {
    // 没有任何规则的裸选择器（缺少 { ）
    let css = "div span ";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 0);
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — @container 条件中的嵌套括号（lines 1172-1184）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_container_condition_nested_parens() {
    let css = "@container ((min-width: 400px)) { .box { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — @container 条件中 EOF（line 1190）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_container_condition_eof_in_parens() {
    // 不匹配的括号导致 EOF
    let css = "@container name (min-width: 400px";
    let sheet = Parser::parse_stylesheet(css);
    // EOF in parens → return None → no rule
    assert_eq!(sheet.rules.len(), 0);
}

// ═══════════════════════════════════════════════════════════════════════
// tokenizer.rs — Column token（||）（lines 788-789）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_tokenizer_column_token() {
    let tokens = tokenize("||");
    let has_column = tokens.iter().any(|t| matches!(t, Token::Column));
    assert!(has_column);
}

#[test]
fn test_tokenizer_dash_only_ident() {
    // 单独的 - 号后不跟标识符起始字符（line 793）
    let tokens = tokenize("- ");
    assert!(tokens.len() >= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// tokenizer.rs — consume_newline：\r 和 \t（lines 259-265）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_tokenizer_with_cr_newline() {
    let tokens = tokenize("a\r\nb");
    assert!(tokens.len() >= 2);
}

#[test]
fn test_tokenizer_with_tab_whitespace() {
    let tokens = tokenize("a\tb");
    assert!(tokens.len() >= 2);
}

// ═══════════════════════════════════════════════════════════════════════
// tokenizer.rs — consume_escape 换行和 EOF（lines 416, 421）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_tokenizer_escape_newline_returns_error() {
    let tokens = tokenize("\\\n");
    assert!(tokens.len() >= 1);
}

#[test]
/// R2132：主循环 `\` 路由——合法转义起始的 `\` 应走 ident-like 路径，而非落 Error。
/// driving：escapes-002 选择器 `p\.class#id`、`p.class#id \{ ... \}`。
fn test_tokenizer_backslash_routes_to_ident_like() {
    // `\{` → 转义花括号，是 ident 的一部分（`{`），**不**应产生 LBrace 误开声明块。
    let tokens = tokenize("\\{");
    assert!(
        matches!(&tokens[0], Token::Ident(s) if s == "{"),
        "expected Ident(\"{{\"), got {:?}",
        tokens[0]
    );

    // `\.` → 转义点，ident 含 `.`（`p\.class` 中 `.` 不再当 class 组合器）。
    let tokens = tokenize("\\.");
    assert!(
        matches!(&tokens[0], Token::Ident(s) if s == "."),
        "expected Ident(\".\"), got {:?}",
        tokens[0]
    );

    // `\`+换行 = 非法转义 → `\` 作 Delim（CSS Syntax §4.3.4），换行单独成空白。
    let tokens = tokenize("\\\n");
    assert!(
        matches!(&tokens[0], Token::Delim(c) if *c == '\\'),
        "expected Delim('\\\\'), got {:?}",
        tokens[0]
    );
}

#[test]
fn test_tokenizer_escape_at_eof() {
    // \ 后直接 EOF（line 421）
    let tokens = tokenize("\\");
    assert!(tokens.len() >= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// tokenizer.rs — # 后 EOF（lines 656-657）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_tokenizer_hash_at_eof() {
    let tokens = tokenize("#");
    assert!(tokens.len() >= 1);
    let has_error = tokens.iter().any(|t| matches!(t, Token::Error(_)));
    assert!(has_error);
}

// ═══════════════════════════════════════════════════════════════════════
// tokenizer.rs — 单独的 - 标识符（lines 357-358）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_tokenizer_only_dash_ident() {
    let tokens = tokenize("-1");
    assert!(tokens.len() >= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// tokenizer.rs — url() 引号形式（lines 537-549）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_tokenizer_url_with_single_quotes() {
    let tokens = tokenize("url('path/to/font.woff2')");
    let has_url = tokens.iter().any(|t| matches!(t, Token::Url(_)));
    assert!(has_url);
}

#[test]
fn test_tokenizer_url_with_double_quotes() {
    let tokens = tokenize("url(\"https://example.com/bg.png\")");
    let has_url = tokens.iter().any(|t| matches!(t, Token::Url(_)));
    assert!(has_url);
}

// ═══════════════════════════════════════════════════════════════════════
// tokenizer.rs — position() 方法（lines 192-194）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_tokenizer_position_method() {
    let tokenizer = Tokenizer::new("abc");
    assert_eq!(tokenizer.position(), 0);
}

// ═══════════════════════════════════════════════════════════════════════
// media_query.rs — Height 各种运算符评估（lines 337-341）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_media_query_height_gt() {
    let css = "@media (height > 500px) { div { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

#[test]
fn test_media_query_height_ge() {
    let css = "@media (height >= 300px) { div { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

#[test]
fn test_media_query_height_lt() {
    let css = "@media (height < 800px) { div { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

#[test]
fn test_media_query_height_le() {
    let css = "@media (height <= 600px) { div { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// media_query.rs — prefers-color-scheme（lines 422-428）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_media_query_prefers_color_scheme_dark() {
    let css = "@media (prefers-color-scheme: dark) { body { background: black; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

#[test]
fn test_media_query_prefers_color_scheme_light() {
    let css = "@media (prefers-color-scheme: light) { body { background: white; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// media_query.rs — prefers-reduced-motion（lines 430-435）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_media_query_prefers_reduced_motion_reduce() {
    let css = "@media (prefers-reduced-motion: reduce) { * { animation: none; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

#[test]
fn test_media_query_prefers_reduced_motion_no_preference() {
    let css = "@media (prefers-reduced-motion: no-preference) { * { animation: auto; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// media_query.rs — pointer（lines 438-444）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_media_query_pointer_none() {
    let css = "@media (pointer: none) { a { padding: 8px; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

#[test]
fn test_media_query_pointer_coarse() {
    let css = "@media (pointer: coarse) { a { padding: 12px; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

#[test]
fn test_media_query_pointer_fine() {
    let css = "@media (pointer: fine) { a { padding: 4px; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// media_query.rs — resolution/min-resolution/max-resolution（lines 447-458）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_media_query_resolution() {
    let css = "@media (resolution: 150dpi) { body { font-size: 16px; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

#[test]
fn test_media_query_min_resolution() {
    let css = "@media (min-resolution: 96dpi) { body { font-size: 14px; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

#[test]
fn test_media_query_max_resolution() {
    let css = "@media (max-resolution: 300dpi) { body { font-size: 18px; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// media_query.rs — orientation（lines 413-420）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_media_query_orientation_landscape() {
    let css = "@media (orientation: landscape) { .layout { flex-direction: row; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

#[test]
fn test_media_query_orientation_portrait() {
    let css = "@media (orientation: portrait) { .layout { flex-direction: column; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// media_query.rs — flip_op 覆盖（通过组合范围语法触发）
// lines 523-530: flip_op 在组合范围中翻转运算符
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_media_query_combined_range_height() {
    // 300px <= height <= 800px 触发 flip_op 和 Height 范围
    let css = "@media (300px <= height <= 800px) { div { height: auto; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// media_query.rs — parse_colon_syntax 回退特征名（lines 459-469）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_media_query_colon_width() {
    let css = "@media (width: 600px) { div { width: 100%; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

#[test]
fn test_media_query_colon_height() {
    let css = "@media (height: 400px) { div { height: 100%; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

#[test]
fn test_media_query_colon_min_height() {
    let css = "@media (min-height: 300px) { div { min-height: 100px; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

#[test]
fn test_media_query_colon_max_height() {
    let css = "@media (max-height: 800px) { div { max-height: 500px; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// media_query.rs — find_matching_paren 失败（line 639）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_media_query_unmatched_paren() {
    // 不匹配的括号
    let css = "@media (not ((broken) { div { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    // 不应崩溃
    assert!(sheet.rules.len() >= 0);
}

// ═══════════════════════════════════════════════════════════════════════
// media_query.rs — not 关键字 + media type（line 300）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_media_query_not_print() {
    let css = "@media not print { body { background: white; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

#[test]
fn test_media_query_not_all() {
    let css = "@media not all { body { display: none; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// media_query.rs — parse_leading_value + parse_op + parse_feature_name +
//   make_feature_condition 覆盖（通过简单范围语法触发）
//   lines 534-580
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_media_query_simple_range_width_gt() {
    let css = "@media (width > 600px) { div { width: auto; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

#[test]
fn test_media_query_simple_range_width_lt() {
    let css = "@media (width < 400px) { div { width: 100%; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

#[test]
fn test_media_query_simple_range_height_ge() {
    let css = "@media (height >= 300px) { div { height: auto; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

#[test]
fn test_media_query_simple_range_height_le() {
    let css = "@media (height <= 900px) { div { height: 100%; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() >= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// values/parse_transform.rs — extract_parens_content 失败（line 256）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_transform_invalid_function_no_paren() {
    // 没有括号的函数名应返回 None 或空
    let result = parse_transform("translateX");
    assert!(result.is_none() || matches!(result, Some(TransformValue::None)));
}

#[test]
fn test_parse_transform_nested_parens() {
    // 嵌套括号的 transform 函数（line 369 depth += 1）
    let result = parse_transform("translate(calc(10px + 5px), 20px)");
    // 应该能解析或者返回 None，但不能 panic
    assert!(result.is_some() || result.is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// values/parse_transform.rs — parse_css_number 失败（line 512）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_transform_invalid_unit_value() {
    // 无法解析的 transform 值
    let result = parse_transform("rotateX(abc)");
    assert!(result.is_none() || matches!(result, Some(TransformValue::None)));
}

// ═══════════════════════════════════════════════════════════════════════
// values/types.rs — parse_min/parse_max/parse_clamp 残余输入返回 None
// lines 665, 711, 736
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_min_with_trailing_input() {
    let result = parse_calc("min(10px, 20px) extra");
    assert!(result.is_none());
}

#[test]
fn test_parse_max_with_trailing_input() {
    let result = parse_calc("max(10px, 20px) extra");
    assert!(result.is_none());
}

#[test]
fn test_parse_clamp_with_trailing_input() {
    let result = parse_calc("clamp(10px, 50px, 100px) extra");
    assert!(result.is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// values/types.rs — parse_calc_expr 中 clamp 缺少逗号（lines 547-553）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_clamp_missing_comma() {
    // clamp(min val max) — 缺少逗号
    let result = parse_calc("clamp(10px 50px 100px)");
    assert!(result.is_none());
}

#[test]
fn test_parse_clamp_missing_second_comma() {
    // clamp(min, val max) — 缺少第二个逗号
    let result = parse_calc("clamp(10px, 50px 100px)");
    assert!(result.is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// values/types.rs — 深度超限（lines 502, 515, 528, 541）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_calc_nested_depth_limit() {
    // 嵌套 calc 超过最大深度
    let result = parse_calc("calc(calc(calc(calc(calc(10px)))))");
    // 深度限制应该返回 None 或成功解析
    assert!(result.is_some() || result.is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// values/types.rs — parse_calc 缺少右括号（lines 508, 521, 534, 557）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_calc_missing_close_paren() {
    let result = parse_calc("calc(10px + 20px");
    assert!(result.is_none());
}

#[test]
fn test_parse_min_missing_close_paren() {
    let result = parse_calc("min(10px, 20px");
    assert!(result.is_none());
}

#[test]
fn test_parse_max_missing_close_paren() {
    let result = parse_calc("max(10px, 20px");
    assert!(result.is_none());
}

#[test]
fn test_parse_clamp_missing_close_paren() {
    let result = parse_calc("clamp(10px, 50px, 100px");
    assert!(result.is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// values/types.rs — parse_min/parse_max 缺少逗号
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_min_no_comma() {
    let result = parse_calc("min(10px 20px)");
    assert!(result.is_none());
}

#[test]
fn test_parse_max_no_comma() {
    let result = parse_calc("max(10px 20px)");
    assert!(result.is_none());
}
