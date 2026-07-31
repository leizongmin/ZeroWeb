//! CSS Selectors Level 4 §14 `:lang()` —— 逗号分隔语言列表 + 通配符。
//!
//! 背景（修复前）：`parse_lang` 仅读取单个 ident/string 后消耗 `)`。`:lang(en, fr)` 读 `en`
//! 后 peek 到 `,`（非 `)`）→ `)` 未消耗 → 残余 `,` `)` 破坏选择器解析、**整条规则被丢**
//! （R2204/R2208 同族「parse 残余 token 吞规则」bug）。L4 规定 `:lang()` 取逗号分隔的语言
//! 范围列表，每项可为 ident 或 string，并支持 BCP 47 通配符 `*`（裸 `*` 匹配任意语言；
//! `*-CA` 子标签通配）。

use super::*;

/// 取首个复杂选择器首个伪类（跳过非伪类 subclass）。
fn first_pseudo(rule: &Rule) -> Option<&PseudoClassSelector> {
    let Rule::Style(style) = rule else {
        return None;
    };
    style.selectors.first().and_then(|s| {
        s.complex.parts.first().and_then(|(compound, _)| {
            compound.subclass_selectors.iter().find_map(|sub| {
                if let SubclassSelector::PseudoClass(pc) = sub {
                    Some(pc)
                } else {
                    None
                }
            })
        })
    })
}

#[test]
fn test_parse_lang_single_still_works() {
    let sheet = Parser::parse_stylesheet(r#"p:lang(en) { color: blue; }"#);
    assert_eq!(sheet.rules.len(), 1, "单语言基线不变");
    match first_pseudo(&sheet.rules[0]) {
        Some(PseudoClassSelector::Lang(list)) => assert_eq!(list, &["en".to_string()]),
        other => panic!("期望 Lang([\"en\"])，实际: {:?}", other),
    }
}

#[test]
fn test_parse_lang_comma_list_not_dropped() {
    // 修复前：`:lang(en, fr)` 读 `en` 后遇 `,`，`)` 未消耗 → 规则被丢（rules.len()==0）。
    let sheet = Parser::parse_stylesheet(r#"p:lang(en, fr) { color: blue; }"#);
    assert_eq!(sheet.rules.len(), 1, ":lang(en, fr) 不应使规则被丢弃");
    match first_pseudo(&sheet.rules[0]) {
        Some(PseudoClassSelector::Lang(list)) => {
            assert_eq!(list, &["en".to_string(), "fr".to_string()], "应解析为语言列表");
        }
        other => panic!("期望 Lang([\"en\",\"fr\"])，实际: {:?}", other),
    }
}

#[test]
fn test_parse_lang_three_item_list_with_string() {
    // 混合 ident 与 string、多项。
    let sheet = Parser::parse_stylesheet(r#"p:lang(de-DE, "de-AT", de-CH) { color: blue; }"#);
    assert_eq!(sheet.rules.len(), 1);
    match first_pseudo(&sheet.rules[0]) {
        Some(PseudoClassSelector::Lang(list)) => {
            assert_eq!(list, &["de-DE".to_string(), "de-AT".to_string(), "de-CH".to_string(),]);
        }
        other => panic!("期望三项列表，实际: {:?}", other),
    }
}

#[test]
fn test_parse_lang_wildcard_star() {
    let sheet = Parser::parse_stylesheet(r#"p:lang(*) { color: blue; }"#);
    assert_eq!(sheet.rules.len(), 1, ":lang(*) 应解析");
    match first_pseudo(&sheet.rules[0]) {
        Some(PseudoClassSelector::Lang(list)) => assert_eq!(list, &["*".to_string()]),
        other => panic!("期望 Lang([\"*\"])，实际: {:?}", other),
    }
}

#[test]
fn test_parse_lang_subtag_wildcard() {
    let sheet = Parser::parse_stylesheet(r#"p:lang(*-CA) { color: blue; }"#);
    assert_eq!(sheet.rules.len(), 1, ":lang(*-CA) 应解析");
    match first_pseudo(&sheet.rules[0]) {
        Some(PseudoClassSelector::Lang(list)) => assert_eq!(list, &["*-CA".to_string()]),
        other => panic!("期望 Lang([\"*-CA\"])，实际: {:?}", other),
    }
}
