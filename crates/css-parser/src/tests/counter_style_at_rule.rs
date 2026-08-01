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

// ── R2394 slice 2：additive-symbols / range 描述符解析 ──────────────────────────

#[test]
/// R2394：`additive-symbols` 裸字形对（CSS §3.1.8）：解析为 (weight, symbol) 对并按 weight 降序。
fn test_parse_counter_style_additive_symbols() {
    let css = "@counter-style a { system: additive; additive-symbols: 6 \\2685, 5 \\2684, 4 \\2683; suffix: \"\"; }";
    let cs = first_counter_style(css);
    assert_eq!(cs.system, CounterSystem::Additive);
    // 降序排序（声明已是降序，验证顺序 + 去反斜杠转义）。
    assert_eq!(
        cs.additive_symbols,
        vec![
            (6, "\u{2685}".to_string()),
            (5, "\u{2684}".to_string()),
            (4, "\u{2683}".to_string()),
        ]
    );
}

#[test]
/// R2394：`additive-symbols` 乱序声明 → 按 weight 降序排序（贪心分解所需）。
fn test_parse_counter_style_additive_symbols_sorted_desc() {
    let css = "@counter-style a { system: additive; additive-symbols: 1 \"a\", 3 \"c\", 2 \"b\"; }";
    let cs = first_counter_style(css);
    assert_eq!(
        cs.additive_symbols.iter().map(|(w, _)| *w).collect::<Vec<_>>(),
        vec![3, 2, 1],
        "应按 weight 降序"
    );
}

#[test]
/// R2394：`additive-symbols` 引号串 + 整数位置互换（`<integer> && <symbol>` 顺序无关）。
fn test_parse_counter_style_additive_symbols_quoted_swap() {
    let css = "@counter-style a { system: additive; additive-symbols: 3 \"a\", \"b\" 2; }";
    let cs = first_counter_style(css);
    assert_eq!(cs.additive_symbols.len(), 2);
    assert_eq!(cs.additive_symbols[0], (3, "a".to_string()));
    assert_eq!(cs.additive_symbols[1], (2, "b".to_string()));
}

#[test]
/// R2394：additive 系统无 additive-symbols → at-rule 无效 → 丢弃。
fn test_parse_counter_style_additive_no_symbols_invalid() {
    let css = "@counter-style bad { system: additive; }";
    let ws = Parser::parse_stylesheet(css);
    assert!(
        !ws.rules.iter().any(|r| matches!(r, Rule::CounterStyle(_))),
        "additive 无 additive-symbols 应丢弃"
    );
}

#[test]
/// R2394：`range` 单区间 `1 5` → Some([(1,5)])。
fn test_parse_counter_style_range_single() {
    let css = "@counter-style a { system: extends upper-roman; range: 1 5; }";
    let cs = first_counter_style(css);
    assert_eq!(cs.range, Some(vec![(1, 5)]));
}

#[test]
/// R2394：`range` 多区间逗号分隔 + `infinite`（lower→MIN，upper→MAX）。
fn test_parse_counter_style_range_multi_infinite() {
    let css = "@counter-style a { system: extends decimal; range: infinite -1, 5 infinite; }";
    let cs = first_counter_style(css);
    assert_eq!(cs.range, Some(vec![(i32::MIN, -1), (5, i32::MAX)]));
}

#[test]
/// R2394：`range: auto` → None（走系统默认）。
fn test_parse_counter_style_range_auto() {
    let css = "@counter-style a { system: extends decimal; range: auto; }";
    let cs = first_counter_style(css);
    assert_eq!(cs.range, None);
}

#[test]
/// R2394：`system: extends <name>` 无需 symbols（extends 继承）→ 规则保留。
fn test_parse_counter_style_extends_no_symbols() {
    let css = "@counter-style ext { system: extends disclosure-closed; }";
    let cs = first_counter_style(css);
    assert_eq!(cs.system, CounterSystem::Extends("disclosure-closed".to_string()));
    assert!(cs.symbols.is_empty(), "extends 不需要 symbols");
}
