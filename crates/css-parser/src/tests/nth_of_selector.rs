//! `:nth-child(an+b of S)` / `:nth-last-child(an+b of S)`（Selectors Level 4）。
//!
//! 背景：修复前 `parse_nth_expression` 一路收集到 `)`，把 `of S` 的选择器列表混入 nth
//! 表达式文本 → `parse_nth_expression_str` 找 'n' 后 b_part = " of .item" 解析失败 →
//! **`of S` 选择器被静默丢弃**（且大括号前的残余 token 可能进一步破坏解析）。本轮在
//! nth-child/nth-last-child 路径上检测 `of` 关键字并解析选择器列表，产出 NthChildOf。

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
fn test_parse_nth_child_of_single_selector() {
    let css = ":nth-child(2n of .item) { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1, ":nth-child(2n of .item) 不应使规则被丢弃");
    match first_pseudo(&sheet.rules[0]) {
        Some(PseudoClassSelector::NthChildOf(p, sels)) => {
            assert_eq!((p.a, p.b), (2, 0), "2n → a=2,b=0");
            assert_eq!(sels.len(), 1, "应解析出 1 个 of 选择器");
        }
        other => panic!("应为 NthChildOf，实际: {:?}", other),
    }
}

#[test]
fn test_parse_nth_child_of_selector_list_and_b() {
    let css = ":nth-child(2n+1 of .a, .b) { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    match first_pseudo(&sheet.rules[0]) {
        Some(PseudoClassSelector::NthChildOf(p, sels)) => {
            assert_eq!((p.a, p.b), (2, 1), "2n+1 → a=2,b=1");
            assert_eq!(sels.len(), 2, "逗号列表应解析出 2 个选择器");
        }
        other => panic!("应为 NthChildOf，实际: {:?}", other),
    }
}

#[test]
fn test_parse_nth_child_of_odd_keyword() {
    let css = ":nth-child(odd of .x) { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    match first_pseudo(&sheet.rules[0]) {
        Some(PseudoClassSelector::NthChildOf(p, sels)) => {
            assert_eq!((p.a, p.b), (2, 1), "odd → a=2,b=1");
            assert_eq!(sels.len(), 1);
        }
        other => panic!("应为 NthChildOf，实际: {:?}", other),
    }
}

#[test]
fn test_parse_nth_last_child_of() {
    let css = ":nth-last-child(1 of .item) { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    match first_pseudo(&sheet.rules[0]) {
        Some(PseudoClassSelector::NthLastChildOf(p, sels)) => {
            assert_eq!((p.a, p.b), (0, 1));
            assert_eq!(sels.len(), 1);
        }
        other => panic!("应为 NthLastChildOf，实际: {:?}", other),
    }
}

#[test]
fn test_parse_nth_child_without_of_unchanged() {
    // 回归：无 of 时仍走 NthChild（既有行为不变）。
    let css = ":nth-child(2n+1) { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    match first_pseudo(&sheet.rules[0]) {
        Some(PseudoClassSelector::NthChild(p)) => assert_eq!((p.a, p.b), (2, 1)),
        other => panic!("无 of 应为 NthChild，实际: {:?}", other),
    }
}

#[test]
fn test_parse_nth_child_of_followed_by_another_rule() {
    // 回归：of S 选择器列表与右括号正确消耗后，不应吞掉紧跟其后的规则。
    let css = ":nth-child(2n of .x) { color: red; }\np { color: blue; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 2, ":nth-child(of S) 消耗完整后应留下第二条规则");
}
