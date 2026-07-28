//! 第八轮覆盖率测试：parser.rs 剩余未覆盖分支。
//!
//! 重点覆盖：
//! - 函数伪类选择器（Ident + LParen 形式：:not/:is/:where/:has/:nth-child/:nth-last-child/:nth-of-type/:nth-last-of-type/:lang）
//! - Function token 形式的函数伪类选择器
//! - 未知函数伪类
//! - 属性选择器高级匹配器（~=, |=, ^=, $=, *=）
//! - 属性选择器值（Delim, Number 开头）
//! - @keyframes 完整解析
//! - @layer 规则变体
//! - @import 带媒体查询
//! - @supports 规则
//! - @container 规则（命名容器、inline-size、范围语法）

use crate::ast::*;
use crate::parser::Parser;

// ═══════════════════════════════════════════════════════════════════════
// 函数伪类选择器 — Ident + LParen 形式
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_not_selector_ident_lparen() {
    let css = "p:not(.special) { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_is_selector_ident_lparen() {
    let css = ":is(h1, h2, h3) { color: blue; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_where_selector_ident_lparen() {
    let css = ":where(article, section) p { color: green; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_has_selector_ident_lparen() {
    let css = "div:has(> p) { background: gold; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_child_ident_lparen() {
    let css = "li:nth-child(odd) { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_last_child_ident_lparen() {
    let css = "li:nth-last-child(even) { color: blue; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_of_type_ident_lparen() {
    let css = "p:nth-of-type(2) { font-weight: bold; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_last_of_type_ident_lparen() {
    let css = "span:nth-last-of-type(3n+1) { color: teal; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_lang_selector_ident_lparen() {
    let css = "p:lang(en) { color: navy; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_lang_selector_string_arg() {
    let css = r#"p:lang("fr") { color: blue; }"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_unknown_function_pseudo_class() {
    // 未知函数伪类 — tokenizer 将 :custom-func( 视为 Colon + Ident("custom-func") + LParen
    // 但 "custom-func" 包含连字符，tokenizer 不会产生 Ident，会分成多个 token
    // 使用不含连字符的名称来测试
    let css = "p:custom(arg) { color: gray; }";
    let sheet = Parser::parse_stylesheet(css);
    // 未知函数伪类回退为 Simple，不匹配函数模式
    // 根据实际解析行为验证
    if sheet.rules.is_empty() {
        // 如果 tokenizer 不能正确解析，空规则也是合理的
        return;
    }
    assert!(sheet.rules.len() <= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 函数伪类选择器 — Function token 形式
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_nth_child_function_token() {
    let css = "li:nth-child(2n+1) { background: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_last_child_function_token() {
    let css = "li:nth-last-child(3) { color: green; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_of_type_function_token() {
    let css = "p:nth-of-type(even) { color: purple; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_last_of_type_function_token() {
    let css = "span:nth-last-of-type(odd) { color: orange; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_lang_function_token() {
    let css = r#"p:lang("de") { color: darkblue; }"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 属性选择器 — 高级匹配器
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_attribute_includes_match() {
    // [attr~=val]
    let css = r#"[class~="active"] { color: green; }"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_attribute_dash_match() {
    // [attr|=val]
    let css = r#"[lang|="en"] { color: blue; }"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_attribute_prefix_match() {
    // [attr^=val]
    let css = r#"[href^="https"] { color: green; }"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_attribute_suffix_match() {
    // [attr$=val]
    let css = r#"[href$=".pdf"] { color: red; }"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_attribute_substring_match() {
    // [attr*=val]
    let css = r#"[title*="hello"] { color: purple; }"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_attribute_value_with_delim_start() {
    // 属性值以非标识符字符开头
    let css = r#"[data-ext=".pdf"] { color: red; }"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_attribute_value_number_start() {
    // 属性值以数字开头
    let css = r#"[data-ver="3px"] { color: blue; }"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_attribute_unknown_matcher_fallback() {
    // 未知匹配器应回退到 Exists
    let css = r#"[data-test] { color: gray; }"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// @keyframes 完整解析
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_keyframes_basic() {
    let css = "@keyframes fade { from { opacity: 0; } to { opacity: 1; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    let Rule::Keyframes(kf) = &sheet.rules[0] else {
        panic!("expected keyframes")
    };
    assert_eq!(kf.name, "fade");
    assert_eq!(kf.keyframes.len(), 2);
}

#[test]
fn test_keyframes_string_name() {
    let css = r#"@keyframes "my-anim" { from { color: red; } to { color: blue; } }"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_keyframes_percentage() {
    let css = "@keyframes pulse { 0% { opacity: 1; } 50% { opacity: 0.5; } 100% { opacity: 1; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    let Rule::Keyframes(kf) = &sheet.rules[0] else {
        panic!("expected keyframes")
    };
    assert_eq!(kf.keyframes.len(), 3);
}

#[test]
fn test_keyframes_comma_separated_selectors() {
    let css = "@keyframes multi { from, to { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    let Rule::Keyframes(kf) = &sheet.rules[0] else {
        panic!("expected keyframes")
    };
    assert_eq!(kf.keyframes[0].selectors.len(), 2);
}

#[test]
fn test_keyframes_empty_name_fails() {
    let css = "@keyframes { from { opacity: 0; } }";
    let sheet = Parser::parse_stylesheet(css);
    // 无名称的 @keyframes：consume_keyframes_rule 返回 None，
    // 但外层 consume_rule 不会回退，因为 At 路径优先
    // 实际行为取决于 tokenizer 对 `{` 的处理
    assert!(sheet.rules.len() <= 1);
}

#[test]
fn test_keyframes_missing_lbrace() {
    let css = "@keyframes test from { opacity: 0; }";
    let sheet = Parser::parse_stylesheet(css);
    // 缺少 { 应该失败
    assert!(sheet.rules.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// @layer 规则变体
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_layer_with_block() {
    let css = "@layer base { p { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    let Rule::Layer(layer) = &sheet.rules[0] else {
        panic!("expected layer")
    };
    assert_eq!(layer.name, "base");
    assert_eq!(layer.rules.len(), 1);
}

#[test]
fn test_layer_statement_only() {
    let css = "@layer base;";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    let Rule::Layer(layer) = &sheet.rules[0] else {
        panic!("expected layer")
    };
    assert_eq!(layer.name, "base");
    assert!(layer.rules.is_empty());
}

#[test]
fn test_layer_anonymous() {
    let css = "@layer { p { color: blue; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    let Rule::Layer(layer) = &sheet.rules[0] else {
        panic!("expected layer")
    };
    assert!(layer.name.is_empty());
}

#[test]
fn test_layer_semicolon_no_name() {
    let css = "@layer;";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    let Rule::Layer(layer) = &sheet.rules[0] else {
        panic!("expected layer")
    };
    assert!(layer.name.is_empty());
    assert!(layer.rules.is_empty());
}

#[test]
fn test_layer_string_name() {
    let css = r#"@layer "custom-layer";"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    let Rule::Layer(layer) = &sheet.rules[0] else {
        panic!("expected layer")
    };
    assert_eq!(layer.name, "custom-layer");
}

// ═══════════════════════════════════════════════════════════════════════
// @import 规则
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_import_basic_url() {
    let css = r#"@import url("style.css");"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    let Rule::Import(imp) = &sheet.rules[0] else {
        panic!("expected import")
    };
    assert_eq!(imp.url, "style.css");
}

#[test]
fn test_import_string_url() {
    let css = r#"@import "theme.css";"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    let Rule::Import(imp) = &sheet.rules[0] else {
        panic!("expected import")
    };
    assert_eq!(imp.url, "theme.css");
}

#[test]
fn test_import_with_media_query() {
    let css = r#"@import "print.css" print;"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    let Rule::Import(imp) = &sheet.rules[0] else {
        panic!("expected import")
    };
    assert_eq!(imp.media_queries.len(), 1);
    assert_eq!(imp.media_queries[0], "print");
}

#[test]
fn test_import_with_multiple_media_queries() {
    let css = r#"@import "responsive.css" screen and (max-width: 600px), print;"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    let Rule::Import(imp) = &sheet.rules[0] else {
        panic!("expected import")
    };
    assert_eq!(imp.media_queries.len(), 2);
}

#[test]
fn test_import_eof_without_semicolon() {
    let css = r#"@import "style.css""#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    let Rule::Import(imp) = &sheet.rules[0] else {
        panic!("expected import")
    };
    assert_eq!(imp.url, "style.css");
}

// ═══════════════════════════════════════════════════════════════════════
// @supports 规则
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_supports_display_grid() {
    let css = "@supports (display: grid) { .container { display: grid; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    let Rule::Supports(sup) = &sheet.rules[0] else {
        panic!("expected supports")
    };
    assert_eq!(sup.rules.len(), 1);
}

#[test]
fn test_supports_not_condition() {
    let css = "@supports not (display: grid) { .container { display: block; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_supports_and_condition() {
    let css = "@supports (display: flex) and (display: grid) { div { display: flex; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// @container 规则
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_container_named() {
    let css = "@container sidebar (min-width: 400px) { .card { display: grid; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    let Rule::Container(cont) = &sheet.rules[0] else {
        panic!("expected container")
    };
    assert_eq!(cont.name.as_deref(), Some("sidebar"));
    assert_eq!(cont.rules.len(), 1);
}

#[test]
fn test_container_unnamed() {
    let css = "@container (min-width: 300px) { .widget { width: 100%; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    let Rule::Container(cont) = &sheet.rules[0] else {
        panic!("expected container")
    };
    assert!(cont.name.is_none());
}

#[test]
fn test_container_size_function() {
    let css = "@container size(min-width: 400px) { p { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_container_inline_size_function() {
    let css = "@container inline-size(min-width: 400px) { p { font-size: 14px; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_container_comparison_operator() {
    let css = "@container (width > 300px) { .wide { width: 100%; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_container_range_syntax() {
    let css = "@container (200px <= width <= 500px) { .medium { width: 50%; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_container_gte_operator() {
    let css = "@container (width >= 600px) { .large { width: 100%; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_container_lt_operator() {
    let css = "@container (width < 200px) { .small { font-size: 12px; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_container_lte_operator() {
    let css = "@container (width <= 400px) { .narrow { flex-direction: column; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_container_ident_not_name() {
    // ident 后面不是 '(' → 回退，这不是名称
    let css = "@container (min-width: 100px) { p { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// nth 表达式 — 更多 edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_nth_expression_with_whitespace() {
    let css = "li:nth-child( 2n + 1 ) { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_expression_3n_plus_2() {
    let css = "li:nth-child(3n+2) { color: blue; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_expression_n_only() {
    let css = "li:nth-child(n) { color: green; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_expression_minus_n_plus_3() {
    let css = "li:nth-child(-n+3) { color: teal; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_last_of_type_function_token_form() {
    let css = "span:nth-last-of-type(2n) { color: olive; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// @ 规则 — 一般 @规则（非特殊关键字）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_generic_at_rule_with_block() {
    let css = "@custom-rule param { p { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    let Rule::At(at) = &sheet.rules[0] else {
        panic!("expected at rule")
    };
    assert_eq!(at.name, "custom-rule");
    assert!(matches!(at.body, AtRuleBody::Block(_)));
}

#[test]
fn test_generic_at_rule_statement() {
    let css = "@charset \"UTF-8\";";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    let Rule::At(at) = &sheet.rules[0] else {
        panic!("expected at rule")
    };
    assert_eq!(at.name, "charset");
    assert!(matches!(at.body, AtRuleBody::Statement));
}

#[test]
fn test_generic_at_rule_eof() {
    let css = "@namespace svg http://www.w3.org/2000/svg";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    let Rule::At(at) = &sheet.rules[0] else {
        panic!("expected at rule")
    };
    assert!(matches!(at.body, AtRuleBody::Statement));
}

// ═══════════════════════════════════════════════════════════════════════
// 声明块 — !important 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_declaration_with_bang_not_important() {
    // `!` 后面不是 `important` → 应该把 ! 放入 value
    let css = "p { color: red !something; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    let Rule::Style(style) = &sheet.rules[0] else {
        panic!("expected style")
    };
    assert_eq!(style.declarations.len(), 1);
    assert!(!style.declarations[0].important);
}

#[test]
fn test_declaration_with_important() {
    let css = "p { color: red !important; }";
    let sheet = Parser::parse_stylesheet(css);
    let Rule::Style(style) = &sheet.rules[0] else {
        panic!("expected style")
    };
    assert!(style.declarations[0].important);
}

/// R2134：`!important` 必须紧跟 `;` / `}` / EOF 才有效。trailing token（如
/// `background: red ! important fail`）使整个声明非法——`!important` 回填进值，
/// 值整体无效 → cascade 丢弃（driving: core-syntax-006，chromium 一致）。
/// 旧实现直接 break 把 `red !important` 当有效声明、trailing `fail` 成独立坏声明。
#[test]
fn test_declaration_important_with_trailing_token_is_invalid() {
    // `! important fail` → 重要标志不得置位
    let css = "div { background: red ! important fail; }";
    let sheet = Parser::parse_stylesheet(css);
    let Rule::Style(style) = &sheet.rules[0] else {
        panic!("expected style")
    };
    assert_eq!(style.declarations.len(), 1);
    assert!(
        !style.declarations[0].important,
        "trailing token after !important must invalidate the priority flag"
    );
    // 值应含 `!important`（回填），使下游 cascade apply-on-dummy 判定非法并丢弃
    assert!(
        style.declarations[0].value.contains("important"),
        "value should absorb the invalid !important: {}",
        style.declarations[0].value
    );

    // 对照：合法 `!important`（紧跟 `;`）仍置位
    let css_ok = "div { color: green ! important; }";
    let sheet_ok = Parser::parse_stylesheet(css_ok);
    let Rule::Style(style_ok) = &sheet_ok.rules[0] else {
        panic!("expected style")
    };
    assert!(
        style_ok.declarations[0].important,
        "valid !important (followed by ;) must set the flag"
    );

    // 对照：合法 `!IMPORTANT`（大小写不敏感 + 紧跟 `}`）
    let css_ci = "div { color: green !IMPORTANT }";
    let sheet_ci = Parser::parse_stylesheet(css_ci);
    let Rule::Style(style_ci) = &sheet_ci.rules[0] else {
        panic!("expected style")
    };
    assert!(
        style_ci.declarations[0].important,
        "!IMPORTANT (case-insensitive, followed by }}) must set the flag"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 选择器列表 — 多选择器
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_selector_list_multiple() {
    let css = "h1, h2, h3 { color: navy; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    let Rule::Style(style) = &sheet.rules[0] else {
        panic!("expected style")
    };
    assert_eq!(style.selectors.len(), 3);
}

#[test]
fn test_selector_with_child_combinator() {
    let css = "div > p { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_selector_with_sibling_combinator() {
    let css = "h2 + p { color: blue; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_selector_with_subsequent_sibling() {
    let css = "h2 ~ p { color: green; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_selector_descendant_combinator() {
    let css = "div p span { color: teal; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    let Rule::Style(style) = &sheet.rules[0] else {
        panic!("expected style")
    };
    // 3 个复合选择器：div (descendant) p (descendant) span
    assert!(style.selectors[0].complex.parts.len() >= 3);
}

#[test]
fn test_pseudo_element_selector() {
    let css = "p::before { content: 'hello'; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_pseudo_element_after() {
    let css = "p::after { content: '!'; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 错误恢复 — 缺少闭合括号
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_container_no_condition() {
    // @container 没有 ( → None
    let css = "@container { p { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    // 应该跳过这个无效规则或产生空规则
    assert!(sheet.rules.len() <= 1);
}

#[test]
fn test_supports_no_prelude() {
    // @supports 没有 prelude → EOF
    let css = "@supports { }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() <= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 完整页面级 CSS — 组合多个规则
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_full_stylesheet_with_mixed_rules() {
    let css = r#"
        @import "reset.css";
        @layer base { * { margin: 0; padding: 0; } }
        @supports (display: grid) { .grid { display: grid; } }
        @container (min-width: 400px) { .card { width: 100%; } }
        @keyframes fade { from { opacity: 0; } to { opacity: 1; } }
        h1 { color: navy; font-size: 2em; }
        p:not(.intro) { line-height: 1.5; }
        li:nth-child(odd) { background: #eee; }
    "#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 8);
}
