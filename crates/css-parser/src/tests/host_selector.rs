//! `:host` / `:host(...)` / `:host-context(...)` 解析与特异性（CSS Scoping §3/§4）。
//!
//! 背景（45 号 shadow-dom-rendering M1）：修复前 `:host` 落 `Simple("host")`（无参形态
//! 侥幸可用但无参数语义），`:host(...)` 走未知函数伪类分支 → 参数与右括号未消耗、整条
//! 规则被丢弃（shadow-dom reftest 池 B 组红案根因之一）。本轮新增 `Host(Option<Vec<Selector>>)` /
//! `HostContext(Vec<Selector>)` 变体与专解析分支：spec 语法 `:host(<compound-selector>)`
//! 单化合物限定（多参数/组合器链 → 整规则丢弃，同未知函数语义）。

use super::*;

/// 取首个复杂选择器的首个伪类（subclass）。
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
fn test_parse_host_bare() {
    let sheet = Parser::parse_stylesheet(":host { color: green; }");
    assert_eq!(sheet.rules.len(), 1, ":host 不应使规则被丢弃");
    match first_pseudo(&sheet.rules[0]) {
        Some(PseudoClassSelector::Host(None)) => {}
        other => panic!("应为 Host(None)，实际: {:?}", other),
    }
}

#[test]
fn test_parse_host_with_single_compound_arg() {
    let sheet = Parser::parse_stylesheet(":host(.foo) { color: green; }");
    assert_eq!(sheet.rules.len(), 1, ":host(.foo) 不应使规则被丢弃");
    match first_pseudo(&sheet.rules[0]) {
        Some(PseudoClassSelector::Host(Some(args))) => {
            assert_eq!(args.len(), 1, "单化合物参数");
        }
        other => panic!("应为 Host(Some(..))，实际: {:?}", other),
    }
}

#[test]
fn test_parse_host_context_with_arg() {
    let sheet = Parser::parse_stylesheet(":host-context(.theme) { color: green; }");
    assert_eq!(sheet.rules.len(), 1, ":host-context(.theme) 不应使规则被丢弃");
    match first_pseudo(&sheet.rules[0]) {
        Some(PseudoClassSelector::HostContext(args)) => assert_eq!(args.len(), 1),
        other => panic!("应为 HostContext(..)，实际: {:?}", other),
    }
}

#[test]
fn test_parse_host_multi_arg_rule_dropped() {
    // spec 语法 `:host(<compound-selector>)` 仅单化合物：列表参数非法 → 整规则丢弃
    //（同未知函数伪类的既有语义）。
    let sheet = Parser::parse_stylesheet(":host(.a, .b) { color: green; }");
    assert!(
        sheet.rules.is_empty(),
        "多参数 :host 应使规则被丢弃，实际: {:?}",
        sheet.rules.len()
    );
}

#[test]
fn test_parse_host_descendant_arg_rule_dropped() {
    // 组合器链参数（.a .b）非化合物 → 丢弃。
    let sheet = Parser::parse_stylesheet(":host(.a .b) { color: green; }");
    assert!(sheet.rules.is_empty(), "组合器链参数 :host 应使规则被丢弃");
}

#[test]
fn test_host_in_compound_position() {
    // `:host` 可与类/标签同化合物（`.foo:host` 形态）——解析不排位次，匹配侧只认末化合物。
    let sheet = Parser::parse_stylesheet("div:host { color: green; }");
    assert_eq!(sheet.rules.len(), 1);
    match first_pseudo(&sheet.rules[0]) {
        Some(PseudoClassSelector::Host(None)) => {}
        other => panic!("div:host 应含 Host(None)，实际: {:?}", other),
    }
}

#[test]
fn test_specificity_host_variants() {
    use crate::selector::specificity;
    let bare = Parser::parse_stylesheet(":host { color: green; }");
    assert_eq!(
        specificity(&match first_pseudo_rule_selector(&bare.rules[0]) {
            Some(s) => s,
            None => panic!("缺选择器"),
        }),
        (0, 1, 0),
        ":host = 伪类基 (0,1,0)"
    );
    let with_arg = Parser::parse_stylesheet(":host(#x) { color: green; }");
    assert_eq!(
        specificity(&match first_pseudo_rule_selector(&with_arg.rules[0]) {
            Some(s) => s,
            None => panic!("缺选择器"),
        }),
        (1, 1, 0),
        ":host(#x) = (0,1,0) + 参数最大 (1,0,0)"
    );
    let ctx = Parser::parse_stylesheet(":host-context(.a) { color: green; }");
    assert_eq!(
        specificity(&match first_pseudo_rule_selector(&ctx.rules[0]) {
            Some(s) => s,
            None => panic!("缺选择器"),
        }),
        (0, 2, 0),
        ":host-context(.a) = (0,1,0) + (0,1,0)"
    );
}

/// 取规则首个选择器的克隆（specificity 入参）。
fn first_pseudo_rule_selector(rule: &Rule) -> Option<Selector> {
    let Rule::Style(style) = rule else {
        return None;
    };
    style.selectors.first().cloned()
}
