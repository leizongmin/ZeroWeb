//! `:dir(ltr|rtl)` 函数伪类解析（CSS Selectors Level 4 §14）。
//!
//! 背景：修复前 `dir` 虽在 `is_known_function_pseudo_class` 名单（规则不被早丢弃），但无
//! 专门 parse 分支 → 落 `_ => PseudoClassSelector::Simple("dir")`，**参数与右括号未被消耗**
//! → 残余 `ltr)` 破坏选择器解析，整条规则实际不可用。本轮新增 `parse_dir` 消耗参数 + `)`，
//! 产出 `PseudoClassSelector::Dir("ltr")`。

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
fn test_parse_dir_ltr_preserves_rule_and_variant() {
    let css = ":dir(ltr) { color: green; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1, ":dir(ltr) 不应使规则被丢弃");
    match first_pseudo(&sheet.rules[0]) {
        Some(PseudoClassSelector::Dir(d)) => assert_eq!(d, "ltr"),
        other => panic!("应为 Dir(\"ltr\")，实际: {:?}", other),
    }
}

#[test]
fn test_parse_dir_rtl_preserves_rule() {
    let css = "div:dir(rtl) { text-align: right; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1, "div:dir(rtl) 不应使规则被丢弃");
    match first_pseudo(&sheet.rules[0]) {
        Some(PseudoClassSelector::Dir(d)) => assert_eq!(d, "rtl"),
        other => panic!("应为 Dir(\"rtl\")，实际: {:?}", other),
    }
}

#[test]
fn test_parse_dir_uppercase_normalized() {
    // 伪类名与参数均 ASCII 大小写不敏感（CSS Syntax §5），归一化为小写。
    let css = ":DIR(LTR) { color: green; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1, ":DIR(LTR) 大小写不应影响解析");
    match first_pseudo(&sheet.rules[0]) {
        Some(PseudoClassSelector::Dir(d)) => assert_eq!(d, "ltr"),
        other => panic!("大写参数应归一化为小写 ltr，实际: {:?}", other),
    }
}

#[test]
fn test_parse_dir_followed_by_another_rule() {
    // 回归：:dir(ltr) 后紧跟另一条规则，前者参数/右括号正确消耗后不应吞掉后者。
    let css = ":dir(ltr) { color: green; }\np { color: blue; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 2, ":dir(ltr) 消耗完整后应留下第二条规则");
}
