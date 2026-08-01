//! `@counter-style` at-rule 解析测试（CSS Counter Styles 3 §3）。driving: R2392。

use crate::ast::{CounterStyleRule, CounterSystem, Rule};
use crate::parser::Parser;

/// 从样式表首规则提取 `CounterStyleRule`，否则 panic。
fn first_counter_style(css: &str) -> CounterStyleRule {
    let ws = Parser::parse_stylesheet(css);
    assert_eq!(ws.rules.len(), 1, "应仅解析出一条规则，实际 {}", ws.rules.len());
    match ws.rules.first() {
        Some(Rule::CounterStyle(cs)) => cs.clone(),
        other => panic!("期望 Rule::CounterStyle，得到 {other:?}"),
    }
}

#[test]
/// 基本形式：`@counter-style triangles { system: cyclic; symbols: ‣ ․ ▪; }`
fn test_parse_counter_style_cyclic() {
    let css = "@counter-style triangles { system: cyclic; symbols: \"‣\" \"․\" \"▪\"; }";
    let cs = first_counter_style(css);
    assert_eq!(cs.name, "triangles");
    assert_eq!(cs.system, CounterSystem::Cyclic);
    assert_eq!(cs.symbols, vec!["‣".to_string(), "․".to_string(), "▪".to_string()]);
    // suffix 缺省 = ". "。
    assert_eq!(cs.suffix, ". ");
    assert_eq!(cs.prefix, "");
    assert_eq!(cs.fallback, "decimal");
}

#[test]
/// `system: fixed <N>` 解析首符号值。
fn test_parse_counter_style_fixed_with_value() {
    let css = "@counter-style fixed-example { system: fixed 5; symbols: 'v' 'w' 'x'; }";
    let cs = first_counter_style(css);
    assert_eq!(cs.system, CounterSystem::Fixed(Some(5)));
    assert_eq!(cs.symbols, vec!["v".to_string(), "w".to_string(), "x".to_string()]);
}

#[test]
/// `system: fixed`（无首符号值）→ Fixed(None)。
fn test_parse_counter_style_fixed_no_value() {
    let css = "@counter-style f { system: fixed; symbols: 'a'; }";
    let cs = first_counter_style(css);
    assert_eq!(cs.system, CounterSystem::Fixed(None));
}

#[test]
/// symbolic / alphabetic / numeric 系统。
fn test_parse_counter_style_systems() {
    for (kw, expected) in [
        ("symbolic", CounterSystem::Symbolic),
        ("alphabetic", CounterSystem::Alphabetic),
        ("numeric", CounterSystem::Numeric),
        ("additive", CounterSystem::Additive),
    ] {
        let css = format!("@counter-style s {{ system: {kw}; symbols: \"a\" \"b\"; }}");
        let cs = first_counter_style(&css);
        assert_eq!(cs.system, expected, "system: {kw}");
    }
}

#[test]
/// `system: extends <name>`。
fn test_parse_counter_style_extends() {
    let css = "@counter-style ext { system: extends decimal; symbols: \"x\"; }";
    let cs = first_counter_style(css);
    assert_eq!(cs.system, CounterSystem::Extends("decimal".to_string()));
}

#[test]
/// `system` 缺省 → symbolic（CSS §3.1.4 默认）。
fn test_parse_counter_style_default_system() {
    let css = "@counter-style dflt { symbols: \"a\" \"b\"; }";
    let cs = first_counter_style(css);
    assert_eq!(cs.system, CounterSystem::Symbolic, "缺省 system 应为 symbolic");
}

#[test]
/// `suffix` / `prefix` / `fallback` 描述符（去引号）。
fn test_parse_counter_style_descriptors() {
    let css = "@counter-style boxed { system: cyclic; symbols: \"a\"; suffix: \") \"; prefix: \"(\"; fallback: upper-roman; }";
    let cs = first_counter_style(css);
    assert_eq!(cs.suffix, ") ");
    assert_eq!(cs.prefix, "(");
    assert_eq!(cs.fallback, "upper-roman");
}

#[test]
/// 裸字形（无引号）symbols：按空白切分。
fn test_parse_counter_style_bare_symbols() {
    let css = "@counter-style dots { system: cyclic; symbols: ● ○ ■; }";
    let cs = first_counter_style(css);
    assert_eq!(cs.symbols, vec!["●".to_string(), "○".to_string(), "■".to_string()]);
}

#[test]
/// 无 symbols（且非 extends）→ at-rule 无效 → 丢弃（不产出 Rule::CounterStyle）。
fn test_parse_counter_style_no_symbols_invalid() {
    let css = "@counter-style bad { system: cyclic; }";
    let ws = Parser::parse_stylesheet(css);
    assert!(
        !ws.rules.iter().any(|r| matches!(r, Rule::CounterStyle(_))),
        "无 symbols 的 cyclic 应被丢弃"
    );
}

#[test]
/// 无名 / 非 `{` → None（畸形恢复，不产出规则，不泄漏 body）。
fn test_parse_counter_style_malformed_no_body() {
    let css = "@counter-style; .a { color: red; }";
    let ws = Parser::parse_stylesheet(css);
    // @counter-style; 丢弃，后续 style rule 正常解析（不泄漏）。
    assert!(
        !ws.rules.iter().any(|r| matches!(r, Rule::CounterStyle(_))),
        "无名 @counter-style 应丢弃"
    );
    assert!(ws.rules.iter().any(|r| matches!(r, Rule::Style(_))), "后续规则不应被吞");
}
