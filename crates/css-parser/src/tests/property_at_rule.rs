//! `@property` at-rule 解析测试（CSS Properties and Values API Level 1）。

use crate::ast::{PropertyRule, Rule};
use crate::parser::Parser;

/// 从样式表首规则提取 `PropertyRule`，否则 panic。
fn first_property(css: &str) -> PropertyRule {
    let ws = Parser::parse_stylesheet(css);
    assert_eq!(ws.rules.len(), 1, "应仅解析出一条规则，实际 {}", ws.rules.len());
    match ws.rules.first() {
        Some(Rule::Property(pr)) => pr.clone(),
        other => panic!("期望 Rule::Property，得到 {other:?}"),
    }
}

#[test]
/// 基本形式：`@property --foo { syntax: "<color>"; inherits: false; initial-value: #c0ffee; }`
fn test_parse_property_basic() {
    let css = r#"@property --foo {
        syntax: "<color>";
        inherits: false;
        initial-value: #c0ffee;
    }"#;
    let pr = first_property(css);
    assert_eq!(pr.name, "--foo");
    assert_eq!(pr.syntax, "\"<color>\"");
    assert!(!pr.inherits, "inherits:false 应解析为 false");
    assert_eq!(pr.initial_value.as_deref(), Some("#c0ffee"));
}

#[test]
/// `inherits: true` 与不同 syntax / initial-value。
fn test_parse_property_inherits_true() {
    let css = r#"@property --gap { syntax: "<length>"; inherits: true; initial-value: 10px; }"#;
    let pr = first_property(css);
    assert_eq!(pr.name, "--gap");
    assert_eq!(pr.syntax, "\"<length>\"");
    assert!(pr.inherits, "inherits:true 应解析为 true");
    assert_eq!(pr.initial_value.as_deref(), Some("10px"));
}

#[test]
/// 缺省 initial-value（仅 syntax:"*" 时合法）：initial_value 应为 None。
fn test_parse_property_missing_initial_value() {
    let css = r#"@property --any { syntax: "*"; inherits: false; }"#;
    let pr = first_property(css);
    assert_eq!(pr.name, "--any");
    assert_eq!(pr.syntax, "\"*\"");
    assert!(pr.initial_value.is_none(), "缺省 initial-value 应为 None");
}

#[test]
/// 描述符顺序无关：syntax/inherits/initial-value 任意顺序均正确提取。
fn test_parse_property_descriptor_order() {
    let css = r#"@property --x { initial-value: green; syntax: "<color>"; inherits: true; }"#;
    let pr = first_property(css);
    assert_eq!(pr.syntax, "\"<color>\"");
    assert!(pr.inherits);
    assert_eq!(pr.initial_value.as_deref(), Some("green"));
}

#[test]
/// 名称非 `--` 起始 → 规则非法，应被丢弃（不产 Rule::Property）。
/// `@property foo { ... }` 的 prelude `foo` 不是自定义属性名。
fn test_parse_property_invalid_name_dropped() {
    let css = "@property foo { syntax: \"*\"; inherits: false; }";
    let ws = Parser::parse_stylesheet(css);
    assert!(
        !ws.rules.iter().any(|r| matches!(r, Rule::Property(_))),
        "非 `--` 名称的 @property 应被丢弃，实际规则: {:?}",
        ws.rules
    );
}

#[test]
/// `@property` 后随真实样式规则不应被吞（at-rule extent 必须完全消费）。
/// driving: 防止畸形恢复吞块（R2204 CDO/CDC 同族 bug）。
fn test_parse_property_followed_by_rule_not_swallowed() {
    let css = r#"
        @property --x { syntax: "<color>"; inherits: false; initial-value: green; }
        div { color: var(--x); }
    "#;
    let ws = Parser::parse_stylesheet(css);
    assert!(
        ws.rules.iter().any(|r| matches!(r, Rule::Property(_))),
        "@property 规则应在"
    );
    let has_div = ws.rules.iter().any(|r| match r {
        Rule::Style(sr) => sr.declarations.iter().any(|d| d.value.contains("var(--x)")),
        _ => false,
    });
    assert!(has_div, "@property 后随的 div 规则不应被吞");
}

#[test]
/// 语句形式 `@property --x;`（无块）→ 名称解析后无 `{`，返回 None，规则被丢弃。
fn test_parse_property_statement_form_dropped() {
    let css = "@property --x;";
    let ws = Parser::parse_stylesheet(css);
    assert!(
        !ws.rules.iter().any(|r| matches!(r, Rule::Property(_))),
        "无块的 @property 语句形式应被丢弃"
    );
}
