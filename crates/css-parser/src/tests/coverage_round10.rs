//! 第十轮覆盖率测试：parser.rs 剩余分支覆盖。
//!
//! 重点：
//! - parse_pseudo_class_function_list（:has() 嵌套选择器）
//! - 缺少 RParen 的路径
//! - 注释在 skip_whitespace 中
//! - consume_rule 中未知 token
//! - @rule EOF 和错误边界
//! - @supports 条件解析失败后继续
//! - @container 名称回退路径

use crate::ast::*;
use crate::parser::Parser;

// ═══════════════════════════════════════════════════════════════════════
// :has() 带复杂嵌套选择器
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_has_with_descendant() {
    let css = "div:has(p span) { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_has_with_class() {
    let css = "div:has(.active) { background: gold; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_has_with_id() {
    let css = "div:has(#main) { background: gold; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_has_with_attribute() {
    let css = "div:has([data-active]) { color: green; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_not_with_multiple_selectors() {
    let css = "p:not(.special, .hidden) { color: blue; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_is_with_complex_selectors() {
    let css = ":is(div > p, section > p) { margin: 0; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_where_with_nested() {
    let css = ":where(article p, section p) { line-height: 1.5; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 注释在样式表中
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_comment_between_rules() {
    let css = "div { color: red; } /* comment */ p { color: blue; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 2);
}

#[test]
fn test_comment_before_selector() {
    let css = "/* header */ div { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_comment_inside_declaration_block() {
    let css = "div { /* comment */ color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Style(style) = &sheet.rules[0] {
        assert!(!style.declarations.is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════════
// consume_rule 的未知 token 路径
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_style_rule_missing_lbrace() {
    // 选择器后没有 { → 返回 None
    let css = "div p";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.is_empty());
}

#[test]
fn test_at_keyword_unknown() {
    // 未知 @规则走通用 consume_at_rule
    let css = "@font-face { font-family: Test; src: url(test.woff); }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::At(at) = &sheet.rules[0] {
        assert_eq!(at.name, "font-face");
    }
}

#[test]
fn test_at_charset() {
    let css = "@charset \"UTF-8\";";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::At(at) = &sheet.rules[0] {
        assert_eq!(at.name, "charset");
    }
}

#[test]
fn test_at_rule_eof_in_prelude() {
    // @media 后直接 EOF
    let css = "@media";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_at_rule_with_body_rules() {
    let css = "@unknown { div { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::At(at) = &sheet.rules[0] {
        if let AtRuleBody::Block(rules) = &at.body {
            assert_eq!(rules.len(), 1);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// @supports 边界
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_supports_with_nested_and() {
    let css = "@supports (display: grid) and ((gap: 10px) or (grid-gap: 10px)) { .box { display: grid; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_supports_not_condition() {
    let css = "@supports not (display: grid) { .fallback { display: flex; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// @container 名称回退路径
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_container_ident_not_followed_by_paren() {
    // ident 后面不是 ( → 回退，整个 ident 被视为条件的一部分
    // 这种情况比较特殊，container 条件文本以 ident 开头
    let css = "@container sidebar { .box { width: 100%; } }";
    let sheet = Parser::parse_stylesheet(css);
    // sidebar 后面没有 (，应该回退
    assert!(sheet.rules.len() <= 1);
}

#[test]
fn test_container_no_lbrace() {
    // 条件解析后没有 { → 返回 None
    let css = "@container (min-width: 400px)";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// @layer 边界
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_layer_no_lbrace_no_semicolon() {
    // @layer name 后面没有 { 也没有 ;
    let css = "@layer base";
    let sheet = Parser::parse_stylesheet(css);
    // 可能无法产生有效规则
    assert!(sheet.rules.len() <= 1);
}

#[test]
fn test_layer_with_unrecognized_after_name() {
    let css = "@layer base )";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() <= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// @keyframes 边界
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_keyframes_string_name() {
    let css = "@keyframes \"my-anim\" { from { opacity: 0; } to { opacity: 1; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_keyframes_no_name() {
    // @keyframes 后没有名称 → 畸形 at-rule，整条（含 {...} 块）应被丢弃（CSS Syntax L3
    // consume_an_at_rule：at-rule 须消费全部 extent，body 不泄漏成顶层规则）。R2140 at-rule
    // fallback 使 consume_keyframes_rule 返回 None 时消耗残余 → 0 规则。
    let css = "@keyframes { from { opacity: 0; } to { opacity: 1; } }";
    let sheet = Parser::parse_stylesheet(css);
    // 不应 panic；spec-correct：畸形 @keyframes 整条丢弃
    assert!(
        sheet.rules.is_empty(),
        "malformed @keyframes (no name) should be dropped entirely"
    );
}

#[test]
fn test_keyframes_no_lbrace_after_name() {
    let css = "@keyframes fade";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// 选择器中的伪元素
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_pseudo_element_with_class() {
    let css = "div::before.active { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_pseudo_element_after_class() {
    let css = ".item::after { content: ''; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 声明中缺失分号
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_multiple_declarations_no_trailing_semicolon() {
    let css = "div { color: red; background: blue }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Style(style) = &sheet.rules[0] {
        assert_eq!(style.declarations.len(), 2);
    }
}

#[test]
fn test_declaration_with_function_value() {
    let css = "div { color: rgb(255, 0, 0); }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Style(style) = &sheet.rules[0] {
        assert_eq!(style.declarations[0].property, "color");
        assert!(style.declarations[0].value.contains("rgb"));
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 属性选择器各种匹配器
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_attribute_dash_match() {
    // [lang|=en]
    let css = "[lang|=en] { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_attribute_prefix_match() {
    // [href^=https]
    let css = r#"[href^="https"] { color: green; }"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_attribute_suffix_match() {
    // [href$=.pdf]
    let css = r#"[href$=".pdf"] { color: blue; }"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_attribute_substring_match() {
    // [title*=hello]
    let css = r#"[title*="hello"] { color: purple; }"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 通用 @规则 带复杂 prelude
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_at_media_with_complex_query() {
    let css = "@media screen and (max-width: 600px) and (min-width: 300px) { div { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::At(at) = &sheet.rules[0] {
        assert!(at.prelude.contains("screen"));
        assert!(at.prelude.contains("600px"));
    }
}

// ═══════════════════════════════════════════════════════════════════════
// @import 无 EOF 处理
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_import_eof_without_semicolon() {
    let css = r#"@import "style.css""#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Import(imp) = &sheet.rules[0] {
        assert_eq!(imp.url, "style.css");
    }
}
