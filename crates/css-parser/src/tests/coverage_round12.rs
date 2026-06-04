//! 覆盖率补充第 12 轮：针对 parser.rs 中未覆盖路径的测试。
//!
//! 主要覆盖：
//! - @supports 规则解析
//! - 容器查询 size() / inline-size() 条件
//! - 容器查询范围语法
//! - 伪类选择器与组合选择器的综合场景

use super::*;

// ═══════════════════════════════════════════════════════════════════════
// 1. @supports 规则解析
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_supports_rule_basic() {
    let css = r#"@supports (display: grid) {
    .container { display: grid; }
}"#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    let has_supports = stylesheet.rules.iter().any(|r| matches!(r, Rule::Supports(_)));
    assert!(has_supports, "应解析出 @supports 规则");
}

#[test]
fn test_parse_supports_rule_not() {
    let css = r#"@supports not (display: grid) {
    .fallback { display: block; }
}"#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    let has_supports = stylesheet.rules.iter().any(|r| matches!(r, Rule::Supports(_)));
    assert!(has_supports, "应解析出 @supports not 规则");
}

#[test]
fn test_parse_supports_rule_and() {
    let css = r#"@supports (display: flex) and (appearance: none) {
    .widget { display: flex; }
}"#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    let has_supports = stylesheet.rules.iter().any(|r| matches!(r, Rule::Supports(_)));
    assert!(has_supports, "应解析出 @supports and 规则");
}

// ═══════════════════════════════════════════════════════════════════════
// 2. 容器查询 size() 和 inline-size() 条件
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_container_size_condition() {
    // size 作为容器名，后面是条件
    let css = r#"@container size (min-width: 400px) {
    .card { flex-direction: column; }
}"#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    let has_container = stylesheet.rules.iter().any(|r| matches!(r, Rule::Container(_)));
    assert!(has_container, "应解析出 @container 规则");
}

#[test]
fn test_parse_container_inline_size_condition() {
    // inline-size 作为容器名
    let css = r#"@container inline-size (min-width: 600px) {
    .sidebar { width: 300px; }
}"#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    let has_container = stylesheet.rules.iter().any(|r| matches!(r, Rule::Container(_)));
    assert!(has_container, "应解析出 @container 规则");
}

#[test]
fn test_parse_container_bare_condition() {
    // 裸条件（无 size()/inline-size() 包装）
    let css = r#"@container (min-width: 400px) {
    .card { flex-direction: column; }
}"#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    let has_container = stylesheet.rules.iter().any(|r| matches!(r, Rule::Container(_)));
    assert!(has_container, "应解析出裸条件的 @container 规则");
}

#[test]
fn test_parse_container_range_syntax() {
    // 范围语法：200px <= width <= 500px
    let css = r#"@container (200px <= width <= 500px) {
    .responsive { display: block; }
}"#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    // 不应 panic
    let _ = stylesheet;
}

#[test]
fn test_parse_container_width_greater_than() {
    let css = r#"@container (width > 300px) {
    .layout { grid-template-columns: 1fr 1fr; }
}"#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    let _ = stylesheet;
}

// ═══════════════════════════════════════════════════════════════════════
// 3. 复杂伪类选择器综合场景
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_nth_child_even() {
    let css = "tr:nth-child(even) { background: #f0f0f0; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
fn test_parse_nth_child_with_compound_selector() {
    let css = "li:nth-child(2n+1):hover { color: blue; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
fn test_parse_has_with_descendant_combinator() {
    let css = ".card:has(.title) { padding: 20px; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
fn test_parse_has_with_child_combinator() {
    let css = ".list:has(> .active) { border-color: blue; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
fn test_parse_lang_pseudo() {
    let css = ":lang(zh-CN) { font-family: serif; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
fn test_parse_where_with_complex_selectors() {
    let css = ":where(h1, h2, h3, h4, h5, h6) { margin: 0; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
fn test_parse_is_with_class_selectors() {
    let css = ":is(.primary, .secondary) { font-weight: bold; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 4. 属性选择器综合场景
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_attribute_with_value() {
    let css = r#"[data-version="1.0"] { color: green; }"#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
fn test_parse_attribute_starts_with() {
    let css = r#"[href^="https"] { color: green; }"#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
fn test_parse_attribute_ends_with() {
    let css = r#"[href$=".pdf"] { icon: pdf; }"#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
fn test_parse_attribute_contains() {
    let css = r#"[class*="btn"] { cursor: pointer; }"#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}
