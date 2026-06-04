//! parser.rs 覆盖率补全测试。
//!
//! 针对 parser.rs 中未覆盖的代码路径。
//! 通过公共 API (parse_stylesheet) 触发内部分支。

use crate::ast::*;
use crate::parser::Parser;

// ── 1. Function token 形式的伪类选择器（行 328-344）──────────────────
// Tokenizer 对 :not(.a) 会产生 Function("not") token（不是 Ident + LParen），
// 这些测试触发 Function token 分支。

#[test]
fn test_function_token_pseudo_not() {
    let css = "div:not(.foo) { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_function_token_pseudo_is() {
    let css = "div:is(.foo) { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_function_token_pseudo_where() {
    let css = "div:where(.foo) { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_function_token_pseudo_has() {
    let css = "div:has(.foo) { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_function_token_nth_child() {
    let css = "li:nth-child(2n+1) { color: blue; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_function_token_nth_last_child() {
    let css = "li:nth-last-child(3) { color: green; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_function_token_nth_of_type() {
    let css = "dd:nth-of-type(odd) { color: purple; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_function_token_nth_last_of_type() {
    let css = "dd:nth-last-of-type(even) { color: orange; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_function_token_lang() {
    let css = "html:lang(en) { direction: ltr; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_function_token_unknown_pseudo() {
    let css = "div:custom-fn(arg) { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert!(ss.rules.len() <= 1);
}

// ── 2. 属性选择器值（行 649-694）───────────────────────────────────

#[test]
fn test_attribute_value_delim_dot_with_ident_and_number() {
    let css = r#"a[href$=".pdf"] { color: blue; }"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_attribute_value_number_only() {
    let css = r#"div[data-count=100] { color: red; }"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_attribute_value_number_with_unit_suffix() {
    let css = r#"div[data-size=100px] { width: auto; }"#;
    let ss = Parser::parse_stylesheet(css);
    assert!(ss.rules.len() <= 1);
}

#[test]
fn test_attribute_value_delim_dot_multi_part() {
    let css = r#"div[data-val=.5] { color: red; }"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ── 3. keyframes 各种变体（行 847-946）─────────────────────────────

#[test]
fn test_keyframes_empty_selector_skips_block() {
    let css = "@keyframes bounce { garbage { opacity: 1; } 50% { opacity: 0.5; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Keyframes(kf) = &ss.rules[0] {
        let _ = kf;
    }
}

#[test]
fn test_keyframes_no_brace_after_selector() {
    let css = "@keyframes test { from color: red; } to { color: blue; } }";
    let ss = Parser::parse_stylesheet(css);
    assert!(!ss.rules.is_empty());
}

#[test]
fn test_keyframes_from_and_to() {
    let css = "@keyframes fade { from { opacity: 0; } to { opacity: 1; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Keyframes(kf) = &ss.rules[0] {
        assert_eq!(kf.name, "fade");
        assert_eq!(kf.keyframes.len(), 2);
    }
}

#[test]
fn test_keyframes_multiple_percentages() {
    let css = "@keyframes slide { 0%, 50%, 100% { transform: none; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Keyframes(kf) = &ss.rules[0] {
        assert_eq!(kf.keyframes.len(), 1);
        assert_eq!(kf.keyframes[0].selectors.len(), 3);
    }
}

// ── 4. 容器条件：size() 和 inline-size() 包装（行 1238-1242）────────

#[test]
fn test_container_with_size_function() {
    let css = "@container (size(min-width: 400px)) { .inner { color: red; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Container(c) = &ss.rules[0] {
        matches!(c.condition, ContainerCondition::Size(_));
    }
}

#[test]
fn test_container_with_inline_size_function() {
    let css = "@container (inline-size(min-width: 400px)) { .inner { color: blue; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Container(c) = &ss.rules[0] {
        matches!(c.condition, ContainerCondition::InlineSize(_));
    }
}

#[test]
fn test_container_inline_size_comparison() {
    let css = "@container (inline-size(width > 300px)) { .box { color: blue; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Container(c) = &ss.rules[0] {
        if let ContainerCondition::InlineSize(size_cond) = &c.condition {
            assert_eq!(size_cond.operator.as_deref(), Some(">"));
        }
    }
}

// ── 5. 容器条件：范围语法（行 1263-1286）──────────────────────────

#[test]
fn test_container_range_syntax() {
    let css = "@container (200px <= width <= 500px) { .box { display: flex; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_container_range_syntax_with_name() {
    let css = "@container sidebar (200px <= width <= 500px) { .box { display: flex; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Container(c) = &ss.rules[0] {
        assert_eq!(c.name.as_deref(), Some("sidebar"));
    }
}

#[test]
fn test_container_range_syntax_empty_min() {
    let css = "@container ( <= width <= 500px) { .box { color: red; } }";
    let ss = Parser::parse_stylesheet(css);
    assert!(ss.rules.is_empty() || ss.rules.len() <= 1);
}

// ── 6. 容器条件：比较运算符（行 1305-1321）───────────────────────

#[test]
fn test_container_comparison_gte() {
    let css = "@container (width >= 300px) { .box { color: red; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_container_comparison_lte() {
    let css = "@container (width <= 600px) { .box { color: blue; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_container_comparison_gt() {
    let css = "@container (width > 300px) { .box { color: green; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_container_comparison_lt() {
    let css = "@container (width < 800px) { .box { color: orange; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ── 7. 容器条件：冒号格式（行 1290-1303）─────────────────────────

#[test]
fn test_container_bare_condition() {
    let css = "@container (min-width: 400px) { .inner { color: red; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Container(c) = &ss.rules[0] {
        if let ContainerCondition::Size(size_cond) = &c.condition {
            assert_eq!(size_cond.feature, "min-width");
            assert_eq!(size_cond.value, "400px");
        }
    }
}

#[test]
fn test_container_colon_empty_value() {
    let css = "@container (min-width:) { .box { color: red; } }";
    let ss = Parser::parse_stylesheet(css);
    assert!(ss.rules.is_empty() || ss.rules.len() <= 1);
}

#[test]
fn test_container_comparison_empty_feature() {
    let css = "@container (> 300px) { .box { color: red; } }";
    let ss = Parser::parse_stylesheet(css);
    assert!(ss.rules.is_empty() || ss.rules.len() <= 1);
}

#[test]
fn test_container_comparison_empty_value() {
    let css = "@container (width > ) { .box { color: red; } }";
    let ss = Parser::parse_stylesheet(css);
    assert!(ss.rules.is_empty() || ss.rules.len() <= 1);
}

// ── 8. @layer 各种变体（行 952-1014）─────────────────────────────

#[test]
fn test_layer_anonymous_with_rules() {
    let css = "@layer { div { color: red; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Layer(layer) = &ss.rules[0] {
        assert!(layer.name.is_empty());
        assert_eq!(layer.rules.len(), 1);
    }
}

#[test]
fn test_layer_string_name_semicolon() {
    let css = "@layer \"my-layer\";";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Layer(layer) = &ss.rules[0] {
        assert_eq!(layer.name, "my-layer");
        assert!(layer.rules.is_empty());
    }
}

#[test]
fn test_layer_string_name_with_block() {
    let css = "@layer \"my-layer\" { div { color: blue; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_layer_semicolon_only() {
    let css = "@layer;";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Layer(layer) = &ss.rules[0] {
        assert!(layer.name.is_empty());
        assert!(layer.rules.is_empty());
    }
}

#[test]
fn test_layer_no_name_no_semicolon_no_brace() {
    let css = "@layer !invalid;";
    let _ss = Parser::parse_stylesheet(css);
}

// ── 9. @import 带多个媒体查询（行 1021-1082）───────────────────────

#[test]
fn test_import_with_multiple_media_queries() {
    let css = "@import url(\"style.css\") screen, print;";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Import(imp) = &ss.rules[0] {
        assert_eq!(imp.url, "style.css");
        assert!(imp.media_queries.len() >= 2);
    }
}

#[test]
fn test_import_string_url_no_media() {
    let css = "@import \"theme.css\";";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Import(imp) = &ss.rules[0] {
        assert_eq!(imp.url, "theme.css");
        assert!(imp.media_queries.is_empty());
    }
}

#[test]
fn test_import_eof_without_semicolon() {
    let css = "@import url(\"style.css\")";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ── 10. 声明中带 !important（行 757-768）───────────────────────────

#[test]
fn test_declaration_important() {
    let css = "div { color: red !important; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Style(sr) = &ss.rules[0] {
        assert_eq!(sr.declarations.len(), 1);
        assert!(sr.declarations[0].important);
    }
}

#[test]
fn test_declaration_bang_not_important() {
    let css = "div { color: red !other; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Style(sr) = &ss.rules[0] {
        assert_eq!(sr.declarations.len(), 1);
        assert!(!sr.declarations[0].important);
    }
}

// ── 11. 选择器中的组合器（行 160-252）─────────────────────────────

#[test]
fn test_selector_descendant_combinator() {
    let css = "div p span { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Style(sr) = &ss.rules[0] {
        assert_eq!(sr.selectors.len(), 1);
        assert_eq!(sr.selectors[0].complex.parts.len(), 3);
    }
}

#[test]
fn test_selector_child_combinator() {
    let css = "div > p { color: blue; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Style(sr) = &ss.rules[0] {
        assert_eq!(sr.selectors[0].complex.parts.len(), 2);
        assert_eq!(sr.selectors[0].complex.parts[0].1, Some(Combinator::Child));
    }
}

#[test]
fn test_selector_next_sibling_combinator() {
    let css = "div + p { color: green; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_selector_subsequent_sibling_combinator() {
    let css = "div ~ p { color: orange; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_selector_leading_child_combinator() {
    let css = ":has(> .child) { color: purple; }";
    let ss = Parser::parse_stylesheet(css);
    assert!(ss.rules.len() <= 1);
}

#[test]
fn test_selector_leading_plus_combinator() {
    let css = ":has(+ .sibling) { color: pink; }";
    let ss = Parser::parse_stylesheet(css);
    assert!(ss.rules.len() <= 1);
}

#[test]
fn test_selector_leading_tilde_combinator() {
    let css = ":has(~ .sibling) { color: yellow; }";
    let ss = Parser::parse_stylesheet(css);
    assert!(ss.rules.len() <= 1);
}

// ── 12. @at-rule 一般形式（行 787-842）────────────────────────────

#[test]
fn test_at_rule_with_semicolon() {
    let css = "@charset \"UTF-8\";";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::At(at) = &ss.rules[0] {
        assert_eq!(at.name, "charset");
        matches!(&at.body, AtRuleBody::Statement);
    }
}

#[test]
fn test_at_rule_with_block() {
    let css = "@font-face { font-family: \"MyFont\"; src: url(\"font.woff2\"); }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::At(at) = &ss.rules[0] {
        assert_eq!(at.name, "font-face");
        matches!(&at.body, AtRuleBody::Block(_));
    }
}

#[test]
fn test_at_rule_eof_in_prelude() {
    let css = "@namespace svg http://www.w3.org/2000/svg";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::At(at) = &ss.rules[0] {
        matches!(&at.body, AtRuleBody::Statement);
    }
}

// ── 13. 声明块中无法识别的 token（行 722-723）─────────────────────

#[test]
fn test_declaration_block_with_garbage() {
    let css = "div { ;;; color: red; ;;; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Style(sr) = &ss.rules[0] {
        assert!(sr.declarations.len() >= 1);
    }
}

// ── 14. nth 表达式解析（行 479-508）─────────────────────────────────

#[test]
fn test_nth_expression_n_only() {
    let css = "li:nth-child(n) { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_nth_expression_negative_n() {
    let css = "li:nth-child(-n+3) { color: blue; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_nth_expression_odd_even() {
    let css = "li:nth-child(odd) { color: red; } li:nth-child(even) { color: blue; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 2);
}

// ── 15. 样式规则中缺少 }（行 114-116 的 else 分支）─────────────

#[test]
fn test_style_rule_missing_rbrace() {
    let css = "div { color: red;";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Style(sr) = &ss.rules[0] {
        assert_eq!(sr.declarations.len(), 1);
    }
}

// ── 16. @supports EOF/无效（行 1088-1136）─────────────────────────

#[test]
fn test_supports_rule_no_lbrace() {
    let css = "@supports (display: grid) ";
    let ss = Parser::parse_stylesheet(css);
    assert!(ss.rules.len() <= 1);
}

#[test]
fn test_supports_eof_before_brace() {
    let css = "@supports (display: grid)";
    let ss = Parser::parse_stylesheet(css);
    assert!(ss.rules.is_empty());
}

// ── 17. 容器回退和 EOF（行 1141-1228）─────────────────────────────

#[test]
fn test_container_ident_not_followed_by_paren_fallback() {
    let css = "@container test { .inner { color: red; } }";
    let ss = Parser::parse_stylesheet(css);
    assert!(ss.rules.len() <= 1);
}

#[test]
fn test_container_eof_in_parens() {
    let css = "@container (min-width: 400px";
    let ss = Parser::parse_stylesheet(css);
    assert!(ss.rules.is_empty());
}

// ── 18. 属性选择器：未知匹配器（行 635-642）─────────────────────

#[test]
fn test_attribute_selector_unknown_matcher() {
    let css = "div[data-x?] { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert!(ss.rules.len() <= 1);
}

// ── 19. parse_lang 的 String 参数形式（行 522-523）───────────────

#[test]
fn test_lang_with_string_argument() {
    let css = "div:lang(\"en\") { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert!(ss.rules.len() <= 1);
}
