//! `::slotted(<compound-selector>)` 解析与特异性（CSS Shadow §3.2.4）。
//!
//! 背景（45 号 shadow-dom-rendering M2）：修复前 `::slotted` 落未知函数伪元素分支 →
//! 整规则丢弃（shadow-dom reftest 池 ~30 红案共同根因）。本轮新增
//! `PseudoElementSelector::Slotted(Box<CompoundSelector>)` 变体与专解析分支：spec 语法
//! 为单化合物参数；`::slotted` 只允许出现在选择器**最后一个化合物**（缀后组合器非法，
//! slotted-parsing driving）；缀后仅 tree-abiding 伪元素白名单；参数内 `:has()` 合法且
//! 须匹配（slotted-has-001..004，csswg-drafts#10586 形态）。
//! https://drafts.csswg.org/css-shadow-1/#slotted-pseudo

use super::*;

/// 取首个复杂选择器的最后一个化合物（`::slotted` 仅允许在末化合物）。
fn last_compound(rule: &Rule) -> Option<&CompoundSelector> {
    let Rule::Style(style) = rule else {
        return None;
    };
    style
        .selectors
        .first()
        .and_then(|s| s.complex.parts.last().map(|(c, _)| c))
}

/// 末化合物的最后一个子类选择器。
fn last_subclass(rule: &Rule) -> Option<&SubclassSelector> {
    last_compound(rule).and_then(|c| c.subclass_selectors.last())
}

/// 末化合物的 Slotted 参数化合物克隆。
fn slotted_arg(rule: &Rule) -> Option<CompoundSelector> {
    match last_subclass(rule) {
        Some(SubclassSelector::PseudoElement(PseudoElementSelector::Slotted(arg))) => Some((**arg).clone()),
        _ => None,
    }
}

// ---------- 合法形态 ----------

#[test]
fn test_parse_slotted_universal() {
    // 裸 `::slotted(*)` ≡ `*::slotted(*)`——余部空化合物按 universal 语义（匹配侧）。
    let sheet = Parser::parse_stylesheet("::slotted(*) { color: red; }");
    assert_eq!(sheet.rules.len(), 1, "::slotted(*) 不应使规则被丢弃");
    assert!(slotted_arg(&sheet.rules[0]).is_some(), "应为 Slotted 变体");
}

#[test]
fn test_parse_slotted_compound_arg() {
    let sheet = Parser::parse_stylesheet("::slotted(div.foo#bar) { color: red; }");
    assert_eq!(sheet.rules.len(), 1);
    let arg = slotted_arg(&sheet.rules[0]).expect("应为 Slotted 变体");
    assert!(
        matches!(arg.type_selector, Some(TypeSelector::Tag(ref t)) if t == "div"),
        "参数类型选择器应为 div"
    );
    assert_eq!(arg.subclass_selectors.len(), 2, "参数应含 .foo 与 #bar 两个子类");
}

#[test]
fn test_parse_slotted_slot_rest_compound() {
    // `slot.named::slotted(em)`：余部（slot.named）匹配 slot 元素，参数匹配被分配元素。
    let sheet = Parser::parse_stylesheet("slot.named::slotted(em) { color: red; }");
    assert_eq!(sheet.rules.len(), 1);
    let compound = last_compound(&sheet.rules[0]).expect("应有末化合物");
    assert!(
        matches!(compound.type_selector, Some(TypeSelector::Tag(ref t)) if t == "slot"),
        "余部类型选择器应为 slot"
    );
    assert!(slotted_arg(&sheet.rules[0]).is_some());
}

#[test]
fn test_parse_slotted_prefix_chain() {
    // `:host ::slotted(span)`：前缀组合子链（host-slotted 形态）。
    let sheet = Parser::parse_stylesheet(":host ::slotted(span) { color: red; }");
    assert_eq!(sheet.rules.len(), 1);
    let selector = sheet
        .rules
        .first()
        .and_then(|r| match r {
            Rule::Style(s) => s.selectors.first().cloned(),
            _ => None,
        })
        .expect("应有选择器");
    assert_eq!(selector.complex.parts.len(), 2, "应为两化合物（:host 前缀 + slotted）");
}

#[test]
fn test_parse_slotted_has_in_argument() {
    // 参数内 `:has()` 合法（slotted-has-001..004 期望绿＝规则生效）；仅**缀后**形态
    // `::slotted(*):has(span)` 非法（slotted-parsing 唯一 :has 测试，见下）。
    let sheet = Parser::parse_stylesheet("::slotted(:has(.a)) { color: red; }");
    assert_eq!(sheet.rules.len(), 1, "::slotted(:has(.a)) 参数内 :has 应合法");
}

#[test]
fn test_parse_slotted_tree_abiding_suffix() {
    // 缀后 tree-abiding 伪元素（CSS Shadow §3.2.4 白名单抽样）。
    for css in [
        "::slotted(*)::before { content: 'x'; }",
        "::slotted(*)::after { content: 'x'; }",
        "::slotted(*)::marker { content: 'x'; }",
    ] {
        let sheet = Parser::parse_stylesheet(css);
        assert_eq!(sheet.rules.len(), 1, "{css} 不应使规则被丢弃");
    }
}

// ---------- 非法形态（整规则丢弃） ----------

#[test]
fn test_parse_slotted_bare_invalid() {
    // 裸 `::slotted`（无参数）非法——须经专用臂合法化，不加 known 表。
    let sheet = Parser::parse_stylesheet("::slotted { color: red; }");
    assert!(sheet.rules.is_empty(), "裸 ::slotted 应使规则被丢弃");
}

#[test]
fn test_parse_slotted_empty_arg_invalid() {
    let sheet = Parser::parse_stylesheet("::slotted() { color: red; }");
    assert!(sheet.rules.is_empty(), "空参数应使规则被丢弃");
}

#[test]
fn test_parse_slotted_combinator_arg_invalid() {
    // 参数为单化合物：组合器链（.a .b）非法。
    let sheet = Parser::parse_stylesheet("::slotted(.a .b) { color: red; }");
    assert!(sheet.rules.is_empty(), "组合器链参数应使规则被丢弃");
}

#[test]
fn test_parse_slotted_suffix_combinator_invalid() {
    // Slotted 之后缀组合器非法（slotted-parsing `::slotted(*) span` driving）。
    for css in [
        "::slotted(*) span { color: red; }",
        "::slotted(*) > span { color: red; }",
    ] {
        let sheet = Parser::parse_stylesheet(css);
        assert!(sheet.rules.is_empty(), "{css} 应使规则被丢弃");
    }
}

#[test]
fn test_parse_slotted_trailing_subclass_invalid() {
    // Slotted 之后同化合物内再缀子类非法（slotted-parsing 逐案：`.class`/`#id`/
    // 属性/伪类）。
    for css in [
        "::slotted(*).class { color: red; }",
        "::slotted(*)#id { color: red; }",
        "::slotted(*)[attr] { color: red; }",
        "::slotted(*):hover { color: red; }",
    ] {
        let sheet = Parser::parse_stylesheet(css);
        assert!(sheet.rules.is_empty(), "{css} 应使规则被丢弃");
    }
}

#[test]
fn test_parse_slotted_has_suffix_invalid() {
    // 缀后 `:has()` 非法（slotted-parsing.html 唯一 :has 测试）——与参数内合法形态
    //（test_parse_slotted_has_in_argument）对拍。
    let sheet = Parser::parse_stylesheet("::slotted(*):has(span) { color: red; }");
    assert!(sheet.rules.is_empty(), "::slotted(*):has(span) 应使规则被丢弃");
}

#[test]
fn test_parse_slotted_non_tree_abiding_suffix_invalid() {
    // 缀后非 tree-abiding 伪元素/二次 slotted 非法。
    for css in [
        "::slotted(*)::first-line { color: red; }",
        "::slotted(*)::selection { color: red; }",
        "::slotted(*)::slotted(*) { color: red; }",
        "::slotted(*)::unknown-thing { color: red; }",
    ] {
        let sheet = Parser::parse_stylesheet(css);
        assert!(sheet.rules.is_empty(), "{css} 应使规则被丢弃");
    }
}

// ---------- specificity ----------

#[test]
fn test_specificity_slotted_variants() {
    use crate::selector::specificity;
    let cases: [(&str, (u32, u32, u32), &str); 4] = [
        (
            "::slotted(*) { color: red; }",
            (0, 0, 1),
            "::slotted(*) = 伪元素基 (0,0,1)",
        ),
        (
            "::slotted(#s) { color: red; }",
            (1, 0, 1),
            "::slotted(#s) = (0,0,1) + 参数 (1,0,0)",
        ),
        (
            "::slotted(div.x) { color: red; }",
            (0, 1, 2),
            "::slotted(div.x) = (0,0,1) + (0,1,1)",
        ),
        (
            "::slotted(*)::before { color: red; }",
            (0, 0, 2),
            "缀后伪元素再计 (0,0,1)",
        ),
    ];
    for (css, expect, msg) in cases {
        let sheet = Parser::parse_stylesheet(css);
        let selector = match sheet.rules.first() {
            Some(Rule::Style(s)) => s.selectors.first().cloned(),
            _ => panic!("{css} 应解析出规则"),
        };
        assert_eq!(specificity(&selector.expect("应有选择器")), expect, "{msg}");
    }
}
