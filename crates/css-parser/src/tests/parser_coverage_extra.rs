//! 针对 parser.rs 中未覆盖路径的补充测试。

use super::*;
use crate::parser::Parser;

// ═══════════════════════════════════════════════════════════════════════
// 1. @container inline-size() 条件路径
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试解析 @container inline-size() 条件
fn test_container_inline_size_condition() {
    let css = "@container (inline-size > 300px) { .nav { display: flex; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Container(container) = &ss.rules[0] {
        assert!(container.name.is_none());
    }
}

#[test]
/// 测试解析 @container size() 条件
fn test_container_size_function_condition() {
    let css = "@container (size(min-width: 400px)) { .box { width: 100%; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
/// 测试解析 @container inline-size() 带名称
fn test_container_inline_size_with_name() {
    let css = "@container sidebar (inline-size > 200px) { .nav { flex-direction: row; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Container(container) = &ss.rules[0] {
        assert_eq!(container.name, Some("sidebar".to_string()));
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 2. 属性选择器值 — Number+Ident 路径和 Delim 路径
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试解析属性选择器中数字值（[attr=1]）
fn test_attribute_selector_numeric_value() {
    let css = "[data-index=1] { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
/// 测试解析属性选择器中数字+单位值（[data-size=100px]）
fn test_attribute_selector_number_with_unit() {
    let css = "[data-size=100px] { width: 100%; }";
    let ss = Parser::parse_stylesheet(css);
    // 100px 被 tokenizer 作为 Dimension token 处理，可能不被属性选择器匹配
    // 关键是不 panic
    let _ = ss.rules;
}

#[test]
/// 测试解析属性选择器中 . 开头的值（[href=.pdf]）
fn test_attribute_selector_delim_dot_value() {
    let css = r#"[href=".pdf"] { color: red; }"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
/// 测试解析属性选择器存在形式 [attr]
fn test_attribute_selector_exists_only() {
    let css = "[disabled] { opacity: 0.5; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Style(style) = &ss.rules[0] {
        if let Some(sel) = style.selectors.first() {
            if let Some(part) = sel.complex.parts.first() {
                if let Some(SubclassSelector::Attribute(attr)) = part.0.subclass_selectors.first() {
                    assert!(matches!(attr.matcher, AttributeMatcher::Exists));
                }
            }
        }
    }
}

#[test]
/// 测试解析属性选择器中未知匹配器
fn test_attribute_selector_unknown_matcher() {
    let css = "[attr??value] { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    // 未知匹配器不应该 panic
    let _ = ss.rules;
}

// ═══════════════════════════════════════════════════════════════════════
// 3. nth 表达式边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试解析 :nth-child(n) — a=1, b=0
fn test_nth_child_bare_n() {
    let css = "li:nth-child(n) { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
/// 测试解析 :nth-child(-n+3) — a=-1, b=3
fn test_nth_child_negative_n() {
    let css = "li:nth-child(-n+3) { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
/// 测试解析 :nth-child(+n+1) — a=1, b=1
fn test_nth_child_plus_n() {
    let css = "li:nth-child(+n+1) { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
/// 测试解析 :nth-child(0n+5) — a=0, b=5
fn test_nth_child_zero_a() {
    let css = "li:nth-child(0n+5) { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
/// 测试解析 :nth-child(3n-2) — a=3, b=-2
fn test_nth_child_negative_b() {
    let css = "li:nth-child(3n-2) { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
/// 测试解析 :nth-last-of-type() 伪类
fn test_nth_last_of_type() {
    let css = "p:nth-last-of-type(2n+1) { color: blue; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
/// 测试解析 :nth-child(odd) 和 :nth-child(even)
fn test_nth_child_odd_even() {
    let css = "tr:nth-child(odd) { background: #f0f0f0; } tr:nth-child(even) { background: #fff; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════
// 4. @keyframes 字符串名称和边界
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试解析 @keyframes 带字符串名称
fn test_keyframes_string_name() {
    let css = r#"@keyframes "my-animation" { from { opacity: 0; } to { opacity: 1; } }"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Keyframes(kf) = &ss.rules[0] {
        assert_eq!(kf.name, "my-animation");
    }
}

#[test]
/// 测试解析 @keyframes 混合选择器（from, to, 百分比）
fn test_keyframes_mixed_selectors() {
    let css = "@keyframes mixed { from { top: 0; } 25% { top: 25px; } to { top: 100px; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Keyframes(kf) = &ss.rules[0] {
        assert_eq!(kf.keyframes.len(), 3);
    }
}

#[test]
/// 测试解析 @keyframes 逗号分隔的选择器
fn test_keyframes_comma_separated_selectors() {
    let css = "@keyframes bounce { 0%, 100% { transform: scale(1); } 50% { transform: scale(1.5); } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Keyframes(kf) = &ss.rules[0] {
        assert_eq!(kf.keyframes.len(), 2);
        assert_eq!(kf.keyframes[0].selectors.len(), 2);
    }
}

#[test]
/// 测试解析空的 @keyframes
fn test_keyframes_empty_body() {
    let css = "@keyframes empty { }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Keyframes(kf) = &ss.rules[0] {
        assert_eq!(kf.name, "empty");
        assert!(kf.keyframes.is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 5. @layer 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试解析 @layer 匿名空层
fn test_layer_anonymous_empty() {
    let css = "@layer { }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Layer(layer) = &ss.rules[0] {
        assert_eq!(layer.name, "");
        assert!(layer.rules.is_empty());
    }
}

#[test]
/// 测试解析 @layer 仅分号
fn test_layer_semicolon_only() {
    let css = "@layer ;";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Layer(layer) = &ss.rules[0] {
        assert_eq!(layer.name, "");
        assert!(layer.rules.is_empty());
    }
}

#[test]
/// 测试解析 @layer 带字符串名称
fn test_layer_string_name() {
    let css = r#"@layer "my layer" { .a { color: red; } }"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Layer(layer) = &ss.rules[0] {
        assert_eq!(layer.name, "my layer");
    }
}

#[test]
/// 测试解析 @layer 带名称后分号
fn test_layer_named_semicolon() {
    let css = "@layer base;";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Layer(layer) = &ss.rules[0] {
        assert_eq!(layer.name, "base");
        assert!(layer.rules.is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 6. @import 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试解析 @import 带字符串 URL
fn test_import_string_url() {
    let css = r#"@import "styles.css";"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Import(import) = &ss.rules[0] {
        assert_eq!(import.url, "styles.css");
    }
}

#[test]
/// 测试解析 @import 带空媒体查询
fn test_import_no_media_query() {
    let css = r#"@import url("reset.css");"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Import(import) = &ss.rules[0] {
        assert_eq!(import.url, "reset.css");
        assert!(import.media_queries.is_empty());
    }
}

#[test]
/// 测试解析 @import 带单个媒体查询
fn test_import_single_media_query() {
    let css = r#"@import "print.css" print;"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Import(import) = &ss.rules[0] {
        assert_eq!(import.media_queries.len(), 1);
        assert_eq!(import.media_queries[0], "print");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 7. @supports 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试解析 @supports 带 not 条件
fn test_supports_not_condition() {
    let css = "@supports not (display: grid) { .fallback { display: block; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
/// 测试解析 @supports 带多条件
fn test_supports_and_condition() {
    let css = "@supports (display: grid) and (gap: 10px) { .grid { display: grid; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 8. 伪类 Function token 路径
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试解析 Function token 形式的伪类（:nth-child 通过 Function token）
fn test_function_token_nth_child() {
    let css = "li:nth-child(2n+1) { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
/// 测试解析 :not() Function token 形式
fn test_function_token_not() {
    let css = "div:not(.hidden) { display: block; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
/// 测试解析 :is() Function token 形式
fn test_function_token_is() {
    let css = ":is(h1, h2, h3) { font-weight: bold; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
/// 测试解析 :where() Function token 形式
fn test_function_token_where() {
    let css = ":where(.card, .panel) { border: 1px solid; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
/// 测试解析 :has() Function token 形式
fn test_function_token_has() {
    let css = ".card:has(.title) { padding: 20px; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
/// 测试解析 :lang() Function token 形式
fn test_function_token_lang() {
    let css = ":lang(en) { quotes: '«' '»'; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
/// 测试解析 :lang() 带字符串参数
fn test_lang_string_argument() {
    let css = r#":lang("zh-CN") { font-family: "Microsoft YaHei"; }"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 9. 伪元素选择器
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试解析 ::before 伪元素
fn test_pseudo_element_before() {
    let css = "div::before { content: ''; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
/// 测试解析 ::after 伪元素
fn test_pseudo_element_after() {
    let css = "div::after { content: '!'; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
/// 测试解析带伪元素和伪类组合
fn test_pseudo_element_with_pseudo_class() {
    let css = "div:hover::before { content: 'hover'; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 10. @container 条件边界
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试解析 @container width < 条件
fn test_container_less_than_condition() {
    let css = "@container (width < 300px) { .box { flex-direction: column; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
/// 测试解析 @container width >= 条件
fn test_container_greater_equal_condition() {
    let css = "@container (width >= 500px) { .box { flex-direction: row; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
/// 测试解析 @container width <= 条件
fn test_container_less_equal_condition() {
    let css = "@container (width <= 800px) { .box { width: 100%; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
/// 测试解析 @container 裸条件（无 size/inline-size 包装）
fn test_container_bare_condition() {
    let css = "@container (min-width: 400px) { .card { width: 50%; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
/// 测试解析 @container 空条件
fn test_container_empty_condition() {
    let css = "@container () { .box { color: red; } }";
    let ss = Parser::parse_stylesheet(css);
    // 空条件可能解析失败，但不应 panic
    let _ = ss.rules;
}

#[test]
/// 测试解析 @container 名称后不是括号（回退路径）
fn test_container_name_not_followed_by_paren() {
    let css = "@container name { .box { color: red; } }";
    let ss = Parser::parse_stylesheet(css);
    // 名称后不是括号，应该回退处理
    let _ = ss.rules;
}

// ═══════════════════════════════════════════════════════════════════════
// 11. 通用选择器 + 子类选择器组合
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试解析通用选择器 *
fn test_universal_selector() {
    let css = "* { margin: 0; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Style(style) = &ss.rules[0] {
        if let Some(sel) = style.selectors.first() {
            if let Some(part) = sel.complex.parts.first() {
                assert!(matches!(part.0.type_selector, Some(TypeSelector::Universal)));
            }
        }
    }
}

#[test]
/// 测试解析通用选择器 + ID 组合
fn test_universal_with_id() {
    let css = "*#main { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
/// 测试解析通用选择器 + 类组合
fn test_universal_with_class() {
    let css = "*.active { color: green; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 12. @at-rule 边界情况（consume_at_rule）
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试解析带花括号体的 @media 规则
fn test_at_rule_with_block() {
    let css = "@media screen { .a { color: red; } }";
    let ss = Parser::parse_stylesheet(css);
    assert!(!ss.rules.is_empty());
}

#[test]
/// 测试解析以分号结尾的 @charset
fn test_at_rule_with_semicolon() {
    let css = "@charset \"UTF-8\";";
    let ss = Parser::parse_stylesheet(css);
    assert!(!ss.rules.is_empty());
}

#[test]
/// 测试解析 @at-rule 到达 EOF
fn test_at_rule_eof() {
    let css = "@custom-rule value";
    let ss = Parser::parse_stylesheet(css);
    assert!(!ss.rules.is_empty());
    if let Rule::At(at) = &ss.rules[0] {
        assert_eq!(at.name, "custom-rule");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 13. 组合器选择器路径
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试后代组合器（空格分隔）
fn test_descendant_combinator() {
    let css = "div p { color: blue; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Style(style) = &ss.rules[0] {
        if let Some(sel) = style.selectors.first() {
            assert_eq!(sel.complex.parts.len(), 2);
            assert!(matches!(sel.complex.parts[0].1, Some(Combinator::Descendant)));
        }
    }
}

#[test]
/// 测试子组合器（>）
fn test_child_combinator() {
    let css = "div > p { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Style(style) = &ss.rules[0] {
        if let Some(sel) = style.selectors.first() {
            assert_eq!(sel.complex.parts.len(), 2);
            assert!(matches!(sel.complex.parts[0].1, Some(Combinator::Child)));
        }
    }
}

#[test]
/// 测试相邻兄弟组合器（+）
fn test_next_sibling_combinator() {
    let css = "h1 + p { margin-top: 0; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Style(style) = &ss.rules[0] {
        if let Some(sel) = style.selectors.first() {
            assert!(matches!(sel.complex.parts[0].1, Some(Combinator::NextSibling)));
        }
    }
}

#[test]
/// 测试后续兄弟组合器（~）
fn test_subsequent_sibling_combinator() {
    let css = "h1 ~ p { color: gray; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Style(style) = &ss.rules[0] {
        if let Some(sel) = style.selectors.first() {
            assert!(matches!(sel.complex.parts[0].1, Some(Combinator::SubsequentSibling)));
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 14. 声明块边界
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试解析空声明
fn test_empty_declaration() {
    let css = "div { ; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Style(style) = &ss.rules[0] {
        assert!(style.declarations.is_empty());
    }
}

#[test]
/// 测试解析声明缺少冒号
fn test_declaration_missing_colon() {
    let css = "div { color }";
    let ss = Parser::parse_stylesheet(css);
    // 缺少冒号的声明应该被跳过
    assert!(!ss.rules.is_empty());
}

#[test]
/// 测试解析声明中 ! 后面不是 important
fn test_declaration_exclamation_not_important() {
    let css = "div { color: red!blue; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
    if let Rule::Style(style) = &ss.rules[0] {
        assert_eq!(style.declarations.len(), 1);
        assert!(!style.declarations[0].important);
    }
}

#[test]
/// R2127：deferred-whitespace 保留值内转义产生的空白（不 trim 掉），使非法值被
/// apply 拒绝→cascade 丢弃。driving：escapes-014/015/016（`red\9`→`red\t`）。
fn test_declaration_value_preserves_escaped_whitespace() {
    // `color: red\9` → 值应为 `red\t`（tab 保留），非 `red`（被 trim 剥掉会误判合法）。
    let css = "p { color: red\\9; }";
    let ss = Parser::parse_stylesheet(css);
    if let Rule::Style(style) = &ss.rules[0] {
        assert_eq!(style.declarations[0].value, "red\t");
    }
    // 对照：普通值无转义，首尾空白 token 不入值（deferred-ws）。
    let css2 = "p { color:   red   ; }";
    let ss2 = Parser::parse_stylesheet(css2);
    if let Rule::Style(style) = &ss2.rules[0] {
        assert_eq!(style.declarations[0].value, "red");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 15. 前导组合器（如 :has(> .child)）
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试解析 :has(> .child) 前导组合器
fn test_has_leading_child_combinator() {
    let css = ".card:has(> .title) { padding: 20px; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
/// 测试解析 :has(+ .sibling) 前导相邻兄弟
fn test_has_leading_next_sibling() {
    let css = ".card:has(+ .next) { margin: 0; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
/// 测试解析 :has(~ .sibling) 前导后续兄弟
fn test_has_leading_subsequent_sibling() {
    let css = ".card:has(~ .after) { padding: 10px; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}
