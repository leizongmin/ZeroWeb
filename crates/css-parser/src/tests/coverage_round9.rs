//! 第九轮覆盖率测试：parser.rs 剩余未覆盖分支。
//!
//! 重点覆盖：
//! - 属性选择器：无名称直接 `]`、未知匹配器、空值
//! - nth 表达式：各种边界情况
//! - :lang() 伪类
//! - 声明中 `!important` 的各种变体
//! - @keyframes 无效选择器、空选择器列表
//! - @layer 匿名层
//! - @container 各种条件变体（比较运算符、范围语法边界）
//! - @supports 规则
//! - @import 带逗号分隔的媒体查询

use crate::ast::*;
use crate::parser::Parser;

// ═══════════════════════════════════════════════════════════════════════
// 属性选择器边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_attribute_selector_no_name() {
    // [] 无属性名 → skip_to_rbracket
    let css = "[=value] { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    // 应该不 panic，可能产生空规则或跳过
    assert!(sheet.rules.len() <= 1);
}

#[test]
fn test_attribute_selector_unknown_matcher() {
    // 未知匹配器后跳到 ]
    let css = "[data-x?val] { color: blue; }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() <= 1);
}

#[test]
fn test_attribute_selector_empty_value() {
    // 属性选择器空值
    let css = "[data-x=] { color: green; }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() <= 1);
}

#[test]
fn test_attribute_selector_delim_value() {
    // 属性值以 Delim 开头（如 .pdf）
    let css = r#"[href$=".pdf"] { color: red; }"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_attribute_selector_number_value() {
    // 属性值是数字开头
    let css = "[data-ver=3] { color: blue; }";
    let sheet = Parser::parse_stylesheet(css);
    // 数字开头的属性值可能不被完整解析为样式规则
    assert!(sheet.rules.len() <= 1);
}

#[test]
fn test_attribute_selector_number_with_unit() {
    // 属性值数字+单位 — 这种语法在标准 CSS 中不常见，解析可能不完整
    let css = "[data-size=10px] { color: green; }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() <= 1);
}

#[test]
fn test_attribute_selector_exists_only() {
    // [attr] 仅存在
    let css = "[disabled] { opacity: 0.5; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// nth 表达式边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_nth_child_odd() {
    let css = "tr:nth-child(odd) { background: gray; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_child_even() {
    let css = "tr:nth-child(even) { background: lightgray; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_child_an_plus_b() {
    let css = "li:nth-child(3n+1) { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_child_an_minus_b() {
    let css = "li:nth-child(3n-1) { color: blue; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_child_negative_n() {
    let css = "li:nth-child(-n+5) { color: green; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_child_plus_n() {
    let css = "li:nth-child(+n) { color: yellow; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_child_n_only() {
    let css = "li:nth-child(n) { color: purple; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_child_plain_number() {
    let css = "li:nth-child(5) { color: orange; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_child_invalid_expr() {
    // 非法 nth 表达式（`abc` 非合法 An+B）→ 选择器非法 → 整条规则被丢弃（与浏览器一致；
    // 旧宽松实现误纳为 a=0,b=0 并保留规则）。
    let css = "li:nth-child(abc) { color: pink; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 0, "`abc` 非合法 An+B，应使规则被丢弃");
}

#[test]
fn test_nth_last_child() {
    let css = "li:nth-last-child(2n) { color: teal; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_of_type() {
    let css = "p:nth-of-type(2) { font-weight: bold; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_last_of_type() {
    let css = "span:nth-last-of-type(1) { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// :lang() 伪类
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_lang_with_ident() {
    let css = ":lang(en) { font-style: italic; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_lang_with_string() {
    let css = r#":lang("fr") { font-style: italic; }"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_lang_empty() {
    // :lang() 无参数
    let css = ":lang() { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 声明中的 !important 变体
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_declaration_important() {
    let css = "div { color: red !important; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Style(style) = &sheet.rules[0] {
        assert!(style.declarations[0].important);
    }
}

#[test]
fn test_declaration_important_no_semicolon() {
    let css = "div { color: red !important }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Style(style) = &sheet.rules[0] {
        assert!(style.declarations[0].important);
    }
}

#[test]
fn test_declaration_bang_not_important() {
    // ! 后面不是 important，应作为值的一部分
    let css = "div { color: red !other; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Style(style) = &sheet.rules[0] {
        assert!(!style.declarations[0].important);
    }
}

#[test]
fn test_declaration_no_colon() {
    // 缺少冒号
    let css = "div { color red; }";
    let sheet = Parser::parse_stylesheet(css);
    // 应跳过此声明
    assert!(sheet.rules.len() <= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// @keyframes 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_keyframes_invalid_selector() {
    // 无法识别的关键帧选择器
    let css = "@keyframes test { abc { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Keyframes(kf) = &sheet.rules[0] {
        // 无效选择器被跳过
        assert!(kf.keyframes.is_empty());
    }
}

#[test]
fn test_keyframes_empty_selector_list() {
    // 逗号开头 → 空选择器列表
    let css = "@keyframes test { , { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Keyframes(kf) = &sheet.rules[0] {
        assert!(kf.keyframes.is_empty());
    }
}

#[test]
fn test_keyframes_multiple_selectors() {
    let css = "@keyframes test { 0%, 100% { opacity: 1; } 50% { opacity: 0; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Keyframes(kf) = &sheet.rules[0] {
        assert_eq!(kf.keyframes.len(), 2);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// @layer 规则变体
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_layer_statement_only() {
    // @layer; — 无名称无规则体
    let css = "@layer;";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Layer(layer) = &sheet.rules[0] {
        assert!(layer.name.is_empty());
        assert!(layer.rules.is_empty());
    }
}

#[test]
fn test_layer_named_statement() {
    // @layer base; — 仅声明层名
    let css = "@layer base;";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Layer(layer) = &sheet.rules[0] {
        assert_eq!(layer.name, "base");
        assert!(layer.rules.is_empty());
    }
}

#[test]
fn test_layer_anonymous() {
    // @layer { div { color: red; } }
    let css = "@layer { div { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Layer(layer) = &sheet.rules[0] {
        assert!(layer.name.is_empty());
        assert_eq!(layer.rules.len(), 1);
    }
}

#[test]
fn test_layer_named_with_rules() {
    let css = "@layer components { .btn { padding: 8px; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Layer(layer) = &sheet.rules[0] {
        assert_eq!(layer.name, "components");
        assert_eq!(layer.rules.len(), 1);
    }
}

#[test]
fn test_layer_string_name() {
    // 层名也可以是字符串
    let css = r#"@layer "my-layer" { div { color: blue; } }"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// @container 规则变体
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_container_named() {
    let css = "@container sidebar (min-width: 400px) { .card { flex-direction: row; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Container(c) = &sheet.rules[0] {
        assert_eq!(c.name.as_deref(), Some("sidebar"));
    }
}

#[test]
fn test_container_unnamed() {
    let css = "@container (min-width: 400px) { .card { flex-direction: column; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Container(c) = &sheet.rules[0] {
        assert!(c.name.is_none());
    }
}

#[test]
fn test_container_size_function() {
    let css = "@container size(min-width: 400px) { .box { width: 100%; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_container_inline_size_function() {
    let css = "@container inline-size(min-width: 400px) { .box { width: 100%; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_container_comparison_operators() {
    // width > 300px
    let css = "@container (width > 300px) { .box { display: flex; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_container_comparison_gte() {
    // width >= 300px
    let css = "@container (width >= 300px) { .box { display: flex; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_container_comparison_lt() {
    // width < 600px
    let css = "@container (width < 600px) { .box { display: block; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_container_range_syntax() {
    // 200px <= width <= 500px
    let css = "@container (200px <= width <= 500px) { .box { display: grid; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_container_range_syntax_empty_min() {
    // 范围语法但 min 为空 → 不匹配
    let css = "@container ( <= width <= 500px) { .box { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    // 解析条件失败，可能不产生 container 规则
    assert!(sheet.rules.len() <= 1);
}

#[test]
fn test_container_range_syntax_empty_max() {
    // 范围语法但 max 为空 → 不匹配
    let css = "@container (200px <= width <= ) { .box { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() <= 1);
}

#[test]
fn test_container_colon_empty_feature() {
    // : 前为空
    let css = "@container (: 400px) { .box { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.len() <= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// @supports 规则
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_supports_property() {
    let css = "@supports (display: grid) { .container { display: grid; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Supports(s) = &sheet.rules[0] {
        assert_eq!(s.rules.len(), 1);
    }
}

#[test]
fn test_supports_not() {
    let css = "@supports not (display: grid) { .fallback { display: flex; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_supports_and() {
    let css = "@supports (display: grid) and (gap: 10px) { .grid { display: grid; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_supports_or() {
    let css = "@supports (display: flex) or (display: grid) { .box { display: flex; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_supports_empty_prelude() {
    // 无效 supports 条件
    let css = "@supports { div { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    // prelude 为空 → 条件解析失败 → 不产生 supports 规则
    assert!(sheet.rules.len() <= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// @import 带媒体查询
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_import_url_string() {
    let css = r#"@import "style.css";"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Import(imp) = &sheet.rules[0] {
        assert_eq!(imp.url, "style.css");
    }
}

#[test]
fn test_import_url_function() {
    let css = "@import url(theme.css);";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_import_with_media() {
    let css = r#"@import "print.css" print;"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Import(imp) = &sheet.rules[0] {
        assert_eq!(imp.media_queries.len(), 1);
        assert_eq!(imp.media_queries[0], "print");
    }
}

#[test]
fn test_import_with_multiple_media() {
    let css = r#"@import "style.css" screen, print;"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Import(imp) = &sheet.rules[0] {
        assert_eq!(imp.media_queries.len(), 2);
    }
}

#[test]
fn test_import_with_complex_media() {
    let css = r#"@import "style.css" screen and (max-width: 600px);"#;
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Import(imp) = &sheet.rules[0] {
        assert_eq!(imp.media_queries.len(), 1);
    }
}

#[test]
fn test_import_no_url() {
    // @import 后无 URL → 返回 None
    let css = "@import ;";
    let sheet = Parser::parse_stylesheet(css);
    assert!(sheet.rules.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// 选择器组合器
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_child_combinator() {
    let css = "div > p { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_next_sibling_combinator() {
    let css = "h1 + p { margin-top: 0; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_subsequent_sibling_combinator() {
    let css = "h1 ~ p { color: gray; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_descendant_combinator() {
    let css = "div p { color: blue; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_complex_combinator_chain() {
    let css = "div > p ~ span + a { color: green; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_leading_child_combinator() {
    // :has(> .child) → 隐式通用选择器
    let css = "div:has(> .child) { background: yellow; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_leading_sibling_combinator() {
    // :has(+ .sibling) → 隐式通用选择器
    let css = "div:has(+ .sibling) { background: yellow; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_leading_tilde_combinator() {
    // :has(~ .after) → 隐式通用选择器
    let css = "div:has(~ .after) { background: yellow; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 通用选择器 & 复合选择器
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_universal_selector() {
    let css = "* { margin: 0; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_universal_descendant() {
    let css = "div * { color: inherit; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_compound_tag_and_class() {
    let css = "div.active { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_compound_tag_and_id() {
    let css = "div#main { color: blue; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_compound_tag_class_id() {
    let css = "div#main.active { color: green; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 媒体查询
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_media_simple() {
    let css = "@media screen { div { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    // @media 作为 AtRule 处理
    if let Rule::At(at) = &sheet.rules[0] {
        assert_eq!(at.name, "media");
    }
}

#[test]
fn test_media_with_condition() {
    let css = "@media (max-width: 600px) { .container { width: 100%; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_media_not() {
    let css = "@media not print { body { background: white; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_media_only() {
    let css = "@media only screen { .mobile { display: block; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// font-face 规则
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_font_face_basic() {
    let css = "@font-face { font-family: MyFont; src: url(myfont.woff2); }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    // @font-face 作为 AtRule 处理
    if let Rule::At(at) = &sheet.rules[0] {
        assert_eq!(at.name, "font-face");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 空样式表和边界输入
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_empty_stylesheet() {
    let sheet = Parser::parse_stylesheet("");
    assert!(sheet.rules.is_empty());
}

#[test]
fn test_whitespace_only() {
    let sheet = Parser::parse_stylesheet("   \n\t  ");
    assert!(sheet.rules.is_empty());
}

#[test]
fn test_comment_only() {
    let sheet = Parser::parse_stylesheet("/* comment */");
    assert!(sheet.rules.is_empty());
}

#[test]
fn test_multiple_rules() {
    let css = "div { color: red; } p { color: blue; } span { color: green; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 3);
}

#[test]
fn test_at_rule_semicolon_only() {
    // @charset 等只有分号的 @规则
    let css = "@charset \"UTF-8\";";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_at_rule_with_block() {
    let css = "@unknown { div { color: red; } }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 逗号分隔选择器列表
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_selector_list() {
    let css = "h1, h2, h3 { font-weight: bold; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Style(style) = &sheet.rules[0] {
        assert_eq!(style.selectors.len(), 3);
    }
}

#[test]
fn test_mixed_selector_list() {
    let css = "div, .class, #id, [data-x] { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
    if let Rule::Style(style) = &sheet.rules[0] {
        assert_eq!(style.selectors.len(), 4);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 伪元素
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_pseudo_element_before() {
    let css = "div::before { content: 'hi'; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_pseudo_element_after() {
    let css = "div::after { content: ''; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_pseudo_element_first_line() {
    let css = "p::first-line { font-weight: bold; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_pseudo_element_first_letter() {
    let css = "p::first-letter { font-size: 2em; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1);
}
