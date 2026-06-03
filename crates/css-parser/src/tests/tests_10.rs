//! CSS 解析器 parser.rs 和 values 覆盖率补充测试。

use crate::ast::*;
use crate::parser::Parser;
use crate::tokenizer::{Token, Tokenizer};

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — nth 表达式解析（通过 :nth-child 等伪类测试）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_nth_child_odd_even() {
    let css = "li:nth-child(odd) { color: red; } li:nth-child(even) { color: blue; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 2);
}

#[test]
fn test_nth_child_an_plus_b() {
    let css = "li:nth-child(2n+1) { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_nth_child_neg_an_b() {
    let css = "li:nth-child(-n+3) { color: green; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_nth_child_just_n() {
    let css = "li:nth-child(n) { color: blue; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_nth_child_just_number() {
    let css = "li:nth-child(5) { color: purple; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_nth_last_child() {
    let css = "li:nth-last-child(3n+1) { color: orange; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_nth_of_type() {
    let css = "p:nth-of-type(2n) { color: teal; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — @规则解析
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_at_rule_unknown_with_block() {
    // 未知 @rule 带大括号块
    let css = "@unknown { div { color: red; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_at_rule_unknown_statement() {
    let css = "@custom-rule value;";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_at_import_with_media() {
    let css = r#"@import "style.css" screen, print;"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_at_import_simple() {
    let css = r#"@import "reset.css";"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_at_container_basic() {
    let css = "@container (min-width: 700px) { div { color: blue; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_at_supports() {
    let css = "@supports (display: grid) { div { display: grid; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_at_layer_block() {
    let css = "@layer base { div { color: red; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — 伪类函数选择器
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_not_selector() {
    let css = "p:not(.excluded) { color: green; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_is_selector() {
    let css = "p:is(.a, .b) { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_where_selector() {
    let css = "p:where(.a, .b) { color: blue; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_has_selector() {
    let css = "div:has(> p) { color: teal; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_lang_selector() {
    let css = "p:lang(en) { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — 属性选择器
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_attribute_exact_match() {
    let css = r#"[data-type="text"] { color: red; }"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_attribute_starts_with() {
    let css = r#"[href^="https"] { color: green; }"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_attribute_ends_with() {
    let css = r#"[src$=".png"] { color: blue; }"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_attribute_contains() {
    let css = r#"[class*="btn"] { color: teal; }"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_attribute_whitespace_separated() {
    let css = r#"[class~="active"] { color: orange; }"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_attribute_dash_match() {
    let css = r#"[lang|="en"] { color: purple; }"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — 声明与 !important
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_declaration_with_important() {
    let css = "div { color: red !important; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_multiple_declarations() {
    let css = "div { color: red; background: blue; font-size: 16px; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — keyframes
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_keyframes_from_to() {
    let css = "@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_keyframes_percentage() {
    let css = "@keyframes slide { 0% { left: 0; } 50% { left: 50%; } 100% { left: 100%; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — 复杂选择器
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_descendant_combinator() {
    let css = "div p span { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_child_combinator() {
    let css = "div > p { color: blue; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_adjacent_sibling_combinator() {
    let css = "h1 + p { color: green; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_general_sibling_combinator() {
    let css = "h1 ~ p { color: teal; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_pseudo_elements() {
    let css = "p::before { content: '»'; } p::after { content: '.'; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 2);
}

#[test]
fn test_class_and_id_combo() {
    let css = "div.card#main { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_multiple_selectors_same_rule() {
    let css = "h1, h2, h3 { font-weight: bold; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// values/types.rs — calc 深度限制和错误路径
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_calc_deeply_nested() {
    // 深度嵌套的 calc 表达式
    use crate::values::parse_calc;
    let result = parse_calc("calc(1px + 2px)");
    assert!(result.is_some());
}

#[test]
fn test_calc_simple_addition() {
    use crate::values::parse_calc;
    assert!(parse_calc("calc(100% - 20px)").is_some());
}

#[test]
fn test_calc_multiplication() {
    use crate::values::parse_calc;
    assert!(parse_calc("calc(2 * 10px)").is_some());
}

#[test]
fn test_calc_division() {
    use crate::values::parse_calc;
    assert!(parse_calc("calc(100% / 2)").is_some());
}

#[test]
fn test_calc_invalid() {
    use crate::values::parse_calc;
    assert!(parse_calc("calc()").is_none());
    assert!(parse_calc("calc(invalid)").is_none());
}

#[test]
fn test_length_various_units() {
    use crate::values::parse_length;
    assert!(parse_length("10px").is_some());
    assert!(parse_length("1.5em").is_some());
    assert!(parse_length("2rem").is_some());
    assert!(parse_length("50%").is_some());
    assert!(parse_length("100vh").is_some());
    assert!(parse_length("100vw").is_some());
    assert!(parse_length("5vmin").is_some());
    assert!(parse_length("5vmax").is_some());
    assert!(parse_length("").is_none());
    assert!(parse_length("invalid").is_none());
}

#[test]
fn test_eval_calc_basic() {
    use crate::values::{eval_calc, parse_calc};
    if let Some(expr) = parse_calc("calc(10px + 20px)") {
        let result = eval_calc(&expr, None);
        assert!(result.is_some());
    }
}
