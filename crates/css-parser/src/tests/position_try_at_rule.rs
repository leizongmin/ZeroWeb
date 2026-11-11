//! `@position-try` at-rule 解析测试（css-anchor-position-1 §position-try-fallback）。
//! driving: wptfyi-css at-position-try-parse / at-position-try-cssom。

use crate::ast::{PositionTryRule, Rule};
use crate::parser::Parser;

/// 从样式表提取全部 `PositionTryRule`。
fn position_try_rules(css: &str) -> Vec<PositionTryRule> {
    let ws = Parser::parse_stylesheet(css);
    ws.rules
        .into_iter()
        .filter_map(|r| match r {
            Rule::PositionTry(p) => Some(p),
            _ => None,
        })
        .collect()
}

#[test]
/// 基本形式：`@position-try --pf { top: 1px; left: 2px; }`——块名 + 声明块。
fn test_parse_position_try_basic() {
    let rules = position_try_rules("@position-try --pf { top: 1px; left: 2px; }");
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].name, "--pf");
    assert_eq!(rules[0].declarations.len(), 2);
    assert_eq!(rules[0].declarations[0].property, "top");
    assert_eq!(rules[0].declarations[0].value, "1px");
    assert_eq!(rules[0].declarations[1].property, "left");
    assert_eq!(rules[0].declarations[1].value, "2px");
}

#[test]
/// 空声明块：`@position-try --empty { }`——无声明，规则保留。
fn test_parse_position_try_empty_block() {
    let rules = position_try_rules("@position-try --empty { }");
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].name, "--empty");
    assert!(rules[0].declarations.is_empty());
}

#[test]
/// 转义块名：tokenizer 解码 `--\{` → `--{`（css-syntax-3 consume escaped code point）。
fn test_parse_position_try_escaped_name() {
    let rules = position_try_rules("@position-try --\\{ { inset: 0; }");
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].name, "--{");
}

#[test]
/// 无效 prelude（块名非 `--` custom ident）：整条 extent 丢弃，声明块不得泄漏为
/// 顶层样式规则（链式 None → skip_malformed_qualified_rule 回归位）。
fn test_parse_position_try_invalid_prelude_discarded() {
    for css in [
        "@position-try broken { top: 1px; }",
        "@position-try 5 { top: 1px; }",
        // 语句形式（无块）：同样整体丢弃。
        "@position-try --pf;",
    ] {
        let ws = Parser::parse_stylesheet(css);
        assert!(ws.rules.is_empty(), "`{css}` 应无任何规则，实际 {:?}", ws.rules);
    }
}

#[test]
/// `!important` 声明位解析保真（白名单裁剪归 shim 描述符面）。
fn test_parse_position_try_important_declaration() {
    let rules = position_try_rules("@position-try --pf { top: 1px !important; }");
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].declarations.len(), 1);
    assert!(rules[0].declarations[0].important);
}

#[test]
/// 与样式规则混排：解析顺序保持 [Style, PositionTry, Style]。
fn test_parse_position_try_mixed_with_style_rules() {
    let css = "h1 { color: red; } @position-try --p { top: 1px; } div { margin: 0; }";
    let ws = Parser::parse_stylesheet(css);
    assert_eq!(ws.rules.len(), 3, "实际 {:?}", ws.rules);
    assert!(matches!(ws.rules[0], Rule::Style(_)));
    assert!(matches!(ws.rules[1], Rule::PositionTry(_)));
    assert!(matches!(ws.rules[2], Rule::Style(_)));
    if let Rule::PositionTry(p) = &ws.rules[1] {
        assert_eq!(p.name, "--p");
    }
}
