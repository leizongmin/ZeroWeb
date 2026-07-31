//! Selectors Level 4 属性选择器大小写修饰符 `[attr=val i]` / `[attr=val s]`。
//!
//! 背景：修复前 `consume_attribute_selector` 取值后仅当紧跟 `]` 才消耗它；`[type="text" i]`
//! 取值后 peek 到 `i`（非 `]`）→ `]` 未消耗 → 残余 `i` `]` 破坏选择器解析 → **整条规则被丢**。
//! Selectors L4 规定取值后可选空白 + `i`/`s` 修饰符 + 可选空白再 `]`。本轮修复 parser 至少
//! 消耗修饰符（保留规则），匹配层尊重 `i`（大小写不敏感）。

use super::*;

/// 取属性选择器（首个复杂选择器的首个 subclass）。
fn first_attr(rule: &Rule) -> Option<&AttributeSelector> {
    let Rule::Style(style) = rule else {
        return None;
    };
    style.selectors.first().and_then(|s| {
        s.complex.parts.first().and_then(|(compound, _)| {
            compound.subclass_selectors.iter().find_map(|sub| {
                if let SubclassSelector::Attribute(a) = sub {
                    Some(a)
                } else {
                    None
                }
            })
        })
    })
}

#[test]
fn test_attr_case_insensitive_flag_does_not_drop_rule() {
    let css = r#"[type="text" i] { color: red; }"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1, "[attr=val i] 不应使规则被丢弃");
    let attr = first_attr(&sheet.rules[0]).expect("应有属性选择器");
    assert_eq!(attr.name, "type");
    match &attr.matcher {
        AttributeMatcher::Exact(v) => assert_eq!(v, "text"),
        other => panic!("应为 Exact 匹配，实际: {:?}", other),
    }
    assert_eq!(
        attr.case,
        AttrCaseModifier::Insensitive,
        "`i` 修饰符应解析为 Insensitive"
    );
}

#[test]
fn test_attr_case_sensitive_flag_s() {
    let css = r#"[lang|="en" s] { color: red; }"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1, "[attr|=val s] 不应使规则被丢弃");
    let attr = first_attr(&sheet.rules[0]).expect("应有属性选择器");
    assert!(matches!(&attr.matcher, AttributeMatcher::DashMatch(_)));
    assert_eq!(
        attr.case,
        AttrCaseModifier::Sensitive,
        "`s` 修饰符应解析为 Sensitive（强制大小写敏感）"
    );
}

#[test]
fn test_attr_case_modifier_variants() {
    // 缺省修饰符 → Default
    let sheet = Parser::parse_stylesheet(r#"[a="x"] {}"#);
    assert_eq!(first_attr(&sheet.rules[0]).unwrap().case, AttrCaseModifier::Default);
    // `i`/`I` → Insensitive
    for css in [r#"[a="x" i] {}"#, r#"[a="x" I] {}"#] {
        let sheet = Parser::parse_stylesheet(css);
        assert_eq!(
            first_attr(&sheet.rules[0]).unwrap().case,
            AttrCaseModifier::Insensitive,
            "{css} 应为 Insensitive"
        );
    }
    // `s`/`S` → Sensitive
    for css in [r#"[a="x" s] {}"#, r#"[a="x" S] {}"#] {
        let sheet = Parser::parse_stylesheet(css);
        assert_eq!(
            first_attr(&sheet.rules[0]).unwrap().case,
            AttrCaseModifier::Sensitive,
            "{css} 应为 Sensitive"
        );
    }
}

#[test]
fn test_attr_unquoted_value_with_flag() {
    let css = r#"[type=text i] { color: red; }"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1, "无引号值 + 修饰符也应解析");
}

#[test]
fn test_attr_no_flag_unchanged() {
    let css = r#"[type="text"] { color: red; }"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1, "无修饰符基线不变");
    assert_eq!(first_attr(&sheet.rules[0]).unwrap().name, "type");
}
