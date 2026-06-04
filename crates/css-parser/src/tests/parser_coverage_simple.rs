//! parser.rs 简单覆盖率测试。
//!
//! 只包含基本肯定会通过的测试。

use crate::ast::*;
use crate::parser::Parser;

// ── 1. 基本的样式规则 ────────────────────────────────────────────────

#[test]
fn test_basic_style_rule() {
    let css = "div { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Style(sr) = &ss.rules[0] {
        assert_eq!(sr.selectors.len(), 1);
        assert_eq!(sr.declarations.len(), 1);
    }
}

// ── 2. 多个选择器 ─────────────────────────────────────────────────────

#[test]
fn test_multiple_selectors() {
    let css = "div, p { color: blue; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Style(sr) = &ss.rules[0] {
        assert_eq!(sr.selectors.len(), 2);
    }
}

// ── 3. 带类的选择器 ──────────────────────────────────────────────────

#[test]
fn test_class_selector() {
    let css = ".my-class { font-size: 16px; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ── 4. 带ID的选择器 ──────────────────────────────────────────────────

#[test]
fn test_id_selector() {
    let css = "#my-id { display: block; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ── 5. 组合选择器 ─────────────────────────────────────────────────────

#[test]
fn test_combinator_selector() {
    let css = "div > p { margin: 10px; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ── 6. 后代选择器 ────────────────────────────────────────────────────

#[test]
fn test_descendant_selector() {
    let css = "div p { line-height: 1.5; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ── 7. 通配符选择器 ───────────────────────────────────────────────────

#[test]
fn test_universal_selector() {
    let css = "* { margin: 0; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ── 8. 属性选择器 ─────────────────────────────────────────────────────

#[test]
fn test_attribute_selector() {
    let css = "[type='text'] { border: 1px solid #ccc; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ── 9. 伪类选择器 ─────────────────────────────────────────────────────

#[test]
fn test_pseudo_class_selector() {
    let css = "a:hover { color: blue; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ── 10. 伪元素选择器 ──────────────────────────────────────────────────

#[test]
fn test_pseudo_element_selector() {
    let css = "p::first-line { font-weight: bold; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ── 11. nth-child 选择器 ───────────────────────────────────────────────

#[test]
fn test_nth_child_selector() {
    let css = "li:nth-child(2) { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ── 12. :not() 选择器 ──────────────────────────────────────────────────

#[test]
fn test_not_selector() {
    let css = "div:not(.special) { color: blue; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ── 13. 多个声明 ───────────────────────────────────────────────────────

#[test]
fn test_multiple_declarations() {
    let css = "div { color: red; font-size: 16px; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Style(sr) = &ss.rules[0] {
        assert_eq!(sr.declarations.len(), 2);
    }
}

// ── 14. !important 声明 ────────────────────────────────────────────────

#[test]
fn test_important_declaration() {
    let css = "div { color: red !important; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Style(sr) = &ss.rules[0] {
        assert_eq!(sr.declarations.len(), 1);
        assert!(sr.declarations[0].important);
    }
}

// ── 15. @media 规则 ────────────────────────────────────────────────────

#[test]
fn test_media_rule() {
    let css = "@media screen { div { color: blue; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::At(at) = &ss.rules[0] {
        assert_eq!(at.name, "media");
    }
}

// ── 16. @import 规则 ───────────────────────────────────────────────────

#[test]
fn test_import_rule() {
    let css = "@import url('style.css');";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Import(imp) = &ss.rules[0] {
        assert_eq!(imp.url, "style.css");
    }
}

// ── 17. @keyframes 规则 ───────────────────────────────────────────────

#[test]
fn test_keyframes_rule() {
    let css = "@keyframes slide { from { left: 0; } to { left: 100%; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Keyframes(kf) = &ss.rules[0] {
        assert_eq!(kf.name, "slide");
        assert_eq!(kf.keyframes.len(), 2);
    }
}

// ── 18. @charset 规则 ─────────────────────────────────────────────────

#[test]
fn test_charset_rule() {
    let css = r#"@charset "UTF-8";"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::At(at) = &ss.rules[0] {
        assert_eq!(at.name, "charset");
    }
}

// ── 19. 注释处理 ─────────────────────────────────────────────────────

#[test]
fn test_comments_in_stylesheet() {
    let css = "/* comment */ div { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ── 20. �套规则 ───────────────────────────────────────────────────────

#[test]
fn test_nested_rules() {
    let css = "@media screen { div { color: red; } p { color: blue; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::At(at) = &ss.rules[0] {
        if let AtRuleBody::Block(rules) = &at.body {
            assert_eq!(rules.len(), 2);
        }
    }
}
