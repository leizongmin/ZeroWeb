//! `:is()`/`:where()`/`:has()` 的 forgiving selector list（Selectors Level 4）。
//!
//! 背景：修复前 `consume_selector_list_for_function` 遇无效选择器（consume_selector 返回
//! None）即 `break`，丢弃其后所有有效选择器，且残余 token 可能不被消耗致规则被破坏。
//! Selectors L4 规定 :is()/:where()/:has() 取 **forgiving selector list**——无效选择器跳过
//! （到下个逗号），保留有效者；:not() 与 nth `of S` 取普通列表（无效即停）。

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
fn test_is_forgiving_keeps_valid_around_invalid() {
    // :unknownpseudo 是未知简单伪类 → consume_compound_selector 返回 None。
    // forgiving 应保留 .a 与 .b，丢弃 :unknownpseudo。
    let css = ":is(.a, :unknownpseudo, .b) { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1, "forgiving 不应使规则被丢弃");
    match first_pseudo(&sheet.rules[0]) {
        Some(PseudoClassSelector::Is(sels)) => assert_eq!(sels.len(), 2, "应保留 2 个有效选择器"),
        other => panic!("应为 Is，实际: {:?}", other),
    }
}

#[test]
fn test_where_forgiving_keeps_valid() {
    let css = ":where(.a, :unknownpseudo, .b) { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    match first_pseudo(&sheet.rules[0]) {
        Some(PseudoClassSelector::Where(sels)) => assert_eq!(sels.len(), 2),
        other => panic!("应为 Where，实际: {:?}", other),
    }
}

#[test]
fn test_has_forgiving_keeps_valid() {
    let css = ":has(.a, :unknownpseudo, .b) { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    match first_pseudo(&sheet.rules[0]) {
        Some(PseudoClassSelector::Has(sels)) => assert_eq!(sels.len(), 2),
        other => panic!("应为 Has，实际: {:?}", other),
    }
}

#[test]
fn test_forgiving_invalid_only_keeps_rule() {
    // 列表全为无效时，forgiving 仍保留规则（伪类匹配空集 = 不匹配任何元素），不破坏后续规则。
    let css = ":is(:unknownpseudo) { color: red; }\np { color: blue; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 2, "forgiving 即使列表全无效也应保留规则与后续规则");
}

#[test]
fn test_forgiving_all_valid_unchanged() {
    // 回归：全有效列表行为不变。
    let css = ":is(.a, .b, .c) { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    match first_pseudo(&sheet.rules[0]) {
        Some(PseudoClassSelector::Is(sels)) => assert_eq!(sels.len(), 3),
        other => panic!("应为 Is，实际: {:?}", other),
    }
}

#[test]
fn test_not_non_forgiving_invalidates_selector() {
    // :not() 非 forgiving（Selectors L4）：列表含无效选择器使整个 :not() 失效，
    // 进而使所在选择器非法 → 规则被丢弃（与 :is/:where/:has 的 forgiving 形成对照）。
    let css = ":not(.a, :unknownpseudo, .b) { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 0, ":not 含无效选择器应使规则被丢弃");
}
