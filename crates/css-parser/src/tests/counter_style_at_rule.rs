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
/// R3734：`system` descriptor 必须完整匹配 grammar，不能忽略尾部或非法参数。
fn test_parse_counter_style_system_rejects_extra_tokens() {
    for system in ["cyclic extra", "fixed bogus", "fixed 5 extra", "extends decimal extra"] {
        let css = format!("@counter-style bad {{ system: {system}; symbols: \"a\"; }}");
        let ws = Parser::parse_stylesheet(&css);
        assert!(
            !ws.rules.iter().any(|r| matches!(r, Rule::CounterStyle(_))),
            "system: {system} 应整体无效"
        );
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
/// R3738：`@counter-style` rule name 必须排除 `none`、CSS-wide keywords 与保留内置名。
fn test_parse_counter_style_name_rejects_reserved_names() {
    for name in [
        "foo",
        "lower-alpha",
        "cjk-decimal",
        "japanese-informal",
        "ethiopic-numeric",
    ] {
        let css = format!("@counter-style {name} {{ system: symbolic; symbols: \"X\" \"Y\"; }}");
        let cs = first_counter_style(&css);
        assert_eq!(cs.name, name);
    }

    for name in [
        "none",
        "initial",
        "inherit",
        "unset",
        "decimal",
        "disc",
        "square",
        "circle",
        "disclosure-open",
        "disclosure-closed",
    ] {
        let css = format!("@counter-style {name} {{ system: symbolic; symbols: \"X\" \"Y\"; }}");
        let ws = Parser::parse_stylesheet(&css);
        assert!(
            !ws.rules.iter().any(|r| matches!(r, Rule::CounterStyle(_))),
            "@counter-style name {name} 应整体无效"
        );
    }
}

#[test]
/// R3738：可引用的 `<counter-style-name>` 必须是单个非 `none` / CSS-wide ident。
fn test_parse_counter_style_references_reject_invalid_names() {
    for fallback in ["none", "decimal cjk-decimal", "\"*\""] {
        let css = format!(
            "@counter-style fallback-ref {{ system: fixed; symbols: A B; fallback: {fallback}; fallback: lower-roman; }}"
        );
        let cs = first_counter_style(&css);
        assert_eq!(
            cs.fallback, "lower-roman",
            "invalid fallback {fallback} must not mask a later valid fallback"
        );
    }

    let ws = Parser::parse_stylesheet("@counter-style bad { system: extends none; symbols: \"x\"; }");
    assert!(
        !ws.rules.iter().any(|r| matches!(r, Rule::CounterStyle(_))),
        "system: extends none 应整体无效"
    );

    let ws =
        Parser::parse_stylesheet("@counter-style bad { system: extends lower-roman upper-roman; symbols: \"x\"; }");
    assert!(
        !ws.rules.iter().any(|r| matches!(r, Rule::CounterStyle(_))),
        "system: extends 多个名称应整体无效"
    );
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
/// R3737：`prefix`/`suffix` descriptor 必须是单个 `<symbol>`，非法值应被忽略。
fn test_parse_counter_style_prefix_suffix_reject_invalid_symbols() {
    let css = r##"@counter-style a {
        system: extends decimal;
        prefix: "#";
        prefix: *;
        prefix: 0;
        prefix: '$' '$';
        suffix: ',';
        suffix: *;
        suffix: 0;
        suffix: '$' '$';
    }"##;
    let cs = first_counter_style(css);
    assert_eq!(
        cs.prefix, "#",
        "invalid prefix descriptors should not override the first valid one"
    );
    assert_eq!(
        cs.suffix, ",",
        "invalid suffix descriptors should not override the first valid one"
    );
}

#[test]
/// R3737：非法 `prefix`/`suffix` descriptor 在合法值之前出现时也必须被忽略。
fn test_parse_counter_style_prefix_suffix_invalid_before_valid() {
    let css = r##"@counter-style a {
        system: extends decimal;
        prefix: *;
        prefix: "#";
        suffix: 0;
        suffix: ",";
    }"##;
    let cs = first_counter_style(css);
    assert_eq!(cs.prefix, "#");
    assert_eq!(cs.suffix, ",");
}

#[test]
/// R3737：`prefix`/`suffix` 可接受单个 `<image>` symbol，保留原 descriptor 字面量。
fn test_parse_counter_style_prefix_suffix_image_symbols() {
    let css = r#"@counter-style image-affix {
        system: cyclic;
        symbols: "a";
        prefix: url("https://example.com/foo.png");
        suffix: linear-gradient(yellow, blue);
    }"#;
    let cs = first_counter_style(css);
    assert_eq!(cs.prefix, "url(https://example.com/foo.png)");
    assert_eq!(cs.suffix, "linear-gradient(yellow, blue)");
}

#[test]
/// 裸字形（无引号）symbols：按空白切分。
fn test_parse_counter_style_bare_symbols() {
    let css = "@counter-style dots { system: cyclic; symbols: ● ○ ■; }";
    let cs = first_counter_style(css);
    assert_eq!(cs.symbols, vec!["●".to_string(), "○".to_string(), "■".to_string()]);
}

#[test]
/// R3735：`symbols` descriptor 的 string symbol 可包含空白，裸数字不是合法 symbol token。
fn test_parse_counter_style_symbols_token_boundaries() {
    let css = "@counter-style spaced { system: fixed; symbols: \"a b\" 'c d' e; }";
    let cs = first_counter_style(css);
    assert_eq!(cs.symbols, vec!["a b".to_string(), "c d".to_string(), "e".to_string()]);

    let ws = Parser::parse_stylesheet("@counter-style bad { system: fixed; symbols: 0 1 2; }");
    assert!(
        !ws.rules.iter().any(|r| matches!(r, Rule::CounterStyle(_))),
        "bare number tokens are not valid symbols"
    );

    let ws = Parser::parse_stylesheet("@counter-style bad { system: alphabetic; symbols: ⓐ inherit; }");
    assert!(
        !ws.rules.iter().any(|r| matches!(r, Rule::CounterStyle(_))),
        "CSS-wide keywords are not valid custom-ident symbols"
    );

    let ws = Parser::parse_stylesheet("@counter-style bad { system: alphabetic; symbols: default \"X\"; }");
    assert!(
        !ws.rules.iter().any(|r| matches!(r, Rule::CounterStyle(_))),
        "default is not a valid custom-ident symbol"
    );
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
/// R2394：`additive-symbols` 合法声明按 weight 严格递减。
fn test_parse_counter_style_additive_symbols_descending_weights() {
    let css = "@counter-style a { system: additive; additive-symbols: 3 \"c\", 2 \"b\", 1 \"a\"; }";
    let cs = first_counter_style(css);
    assert_eq!(
        cs.additive_symbols.iter().map(|(w, _)| *w).collect::<Vec<_>>(),
        vec![3, 2, 1],
        "应保留合法的严格递减 weight 列表"
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
/// R3732：`additive-symbols` 中任一逗号项非法时，descriptor 无效，additive 规则整体丢弃。
fn test_parse_counter_style_additive_symbols_invalid_pair_rejected() {
    let css = "@counter-style bad { system: additive; additive-symbols: 3 \"c\", bogus; }";
    let ws = Parser::parse_stylesheet(css);
    assert!(
        !ws.rules.iter().any(|r| matches!(r, Rule::CounterStyle(_))),
        "additive-symbols 含非法 pair 时不应静默保留合法前缀"
    );
}

#[test]
/// R3736：`additive-symbols` weight 必须非负且按严格递减顺序声明。
fn test_parse_counter_style_additive_symbols_rejects_negative_and_non_decreasing_weights() {
    for additive_symbols in ["-1 \"X\"", "1 \"I\", 5 \"V\"", "1 \"X\", 1 \"Y\"", "2 C C, 1 B, 0 A"] {
        let css = format!("@counter-style bad {{ system: additive; additive-symbols: {additive_symbols}; }}");
        let ws = Parser::parse_stylesheet(&css);
        assert!(
            !ws.rules.iter().any(|r| matches!(r, Rule::CounterStyle(_))),
            "additive-symbols: {additive_symbols} 应整体无效"
        );
    }
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
/// R3733：`range` 每个逗号项必须恰好两个边界，不能忽略尾部 token。
fn test_parse_counter_style_range_rejects_extra_bound() {
    let css = "@counter-style a { system: extends decimal; range: 1 5 9; }";
    let cs = first_counter_style(css);
    assert_eq!(cs.range, None, "range: 1 5 9 应无效并走默认 range");
}

#[test]
/// R3739：`range` 的 lower 必须小于等于 upper。
fn test_parse_counter_style_range_rejects_reversed_bounds() {
    let css = "@counter-style a { system: extends decimal; range: 0 -1; }";
    let cs = first_counter_style(css);
    assert_eq!(cs.range, None, "range: 0 -1 应无效并走默认 range");
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
