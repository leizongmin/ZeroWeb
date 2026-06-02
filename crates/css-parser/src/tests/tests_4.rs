use super::*;

// ═══════════════════════════════════════════════════════════════════════
// 1. Tokenizer 测试
// ═══════════════════════════════════════════════════════════════════════

// ── 新增边界测试 ──

#[test]
/// 测试解析空媒体查询列表不 panic。
fn test_parse_empty_media_query() {
    let css = "@media {} .a { color: red; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    // 不应 panic，至少应有一条规则
    assert!(!stylesheet.rules.is_empty(), "空 @media 后的规则应被解析");
}

#[test]
/// 测试解析带多个伪类选择器 :not(:first-child)。
fn test_parse_not_pseudo_class_nested() {
    let css = ".item:not(:first-child) { margin-top: 10px; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1, "应解析出 1 条规则");
}

#[test]
/// 测试解析 CSS 变量声明。
fn test_parse_custom_property_declaration() {
    let css = ":root { --main-bg: #ffffff; --spacing: 16px; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1, "应解析出 1 条规则");
}

#[test]
/// 测试解析 @supports 规则。
fn test_parse_supports_rule() {
    let css = "@supports (display: grid) { .container { display: grid; } }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty(), "@supports 应被解析为规则");
}

#[test]
/// 测试解析多个动画名称逗号分隔。
fn test_parse_animation_multiple_names() {
    let css = ".box { animation-name: fadeIn, slideUp; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1, "应解析出 1 条规则");
}

// ═══════════════════════════════════════════════════════════════════════
// 28. Animation / Column / ObjectFit 解析测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 parse_column_count 正常值和 auto。
fn test_parse_column_count() {
    assert_eq!(parse_column_count("auto"), Some(ColumnCountValue::Auto));
    assert_eq!(parse_column_count("3"), Some(ColumnCountValue::Number(3)));
    assert_eq!(parse_column_count("1"), Some(ColumnCountValue::Number(1)));
    assert_eq!(parse_column_count("0"), None, "0 应返回 None");
    assert_eq!(parse_column_count("-1"), None, "负数应返回 None");
    assert_eq!(parse_column_count(""), None, "空字符串应返回 None");
    assert_eq!(parse_column_count("  2  "), Some(ColumnCountValue::Number(2)));
    assert_eq!(parse_column_count("abc"), None, "非数字应返回 None");
}

#[test]
/// 测试 parse_column_width 正常值和 auto。
fn test_parse_column_width() {
    assert_eq!(parse_column_width("auto"), Some(ColumnWidthValue::Auto));
    assert_eq!(
        parse_column_width("200px"),
        Some(ColumnWidthValue::Length(LengthValue::Px(200.0)))
    );
    assert_eq!(
        parse_column_width("10em"),
        Some(ColumnWidthValue::Length(LengthValue::Em(10.0)))
    );
    assert_eq!(parse_column_width(""), None);
    assert_eq!(parse_column_width("  auto  "), Some(ColumnWidthValue::Auto));
}

#[test]
/// 测试 parse_object_fit 所有关键字。
fn test_parse_object_fit() {
    assert_eq!(parse_object_fit("fill"), Some(ObjectFitValue::Fill));
    assert_eq!(parse_object_fit("contain"), Some(ObjectFitValue::Contain));
    assert_eq!(parse_object_fit("cover"), Some(ObjectFitValue::Cover));
    assert_eq!(parse_object_fit("none"), Some(ObjectFitValue::None));
    assert_eq!(parse_object_fit("scale-down"), Some(ObjectFitValue::ScaleDown));
    assert_eq!(parse_object_fit("FILL"), Some(ObjectFitValue::Fill), "大小写不敏感");
    assert_eq!(parse_object_fit("  Cover  "), Some(ObjectFitValue::Cover), "前后空白");
    assert_eq!(parse_object_fit("invalid"), None);
    assert_eq!(parse_object_fit(""), None);
}

#[test]
/// 测试 parse_animation_name none 和自定义名称。
fn test_parse_animation_name() {
    assert_eq!(parse_animation_name("none"), Some(AnimationNameValue::None));
    assert_eq!(
        parse_animation_name("fadeIn"),
        Some(AnimationNameValue::Custom("fadeIn".to_string()))
    );
    assert_eq!(
        parse_animation_name("slide-in"),
        Some(AnimationNameValue::Custom("slide-in".to_string()))
    );
    assert_eq!(
        parse_animation_name("_private"),
        Some(AnimationNameValue::Custom("_private".to_string()))
    );
    assert_eq!(parse_animation_name("1invalid"), None, "数字开头应返回 None");
    assert_eq!(parse_animation_name(""), None, "空字符串应返回 None");
    assert_eq!(parse_animation_name("has space"), None, "含空格应返回 None");
    assert_eq!(
        parse_animation_name("NONE"),
        Some(AnimationNameValue::None),
        "大小写不敏感"
    );
}

#[test]
/// 测试 parse_animation_duration 秒和毫秒。
fn test_parse_animation_duration() {
    assert_eq!(
        parse_animation_duration("1s"),
        Some(AnimationDurationValue::Time(1.0, TimeUnit::S))
    );
    assert_eq!(
        parse_animation_duration("500ms"),
        Some(AnimationDurationValue::Time(500.0, TimeUnit::Ms))
    );
    assert_eq!(
        parse_animation_duration("0.5s"),
        Some(AnimationDurationValue::Time(0.5, TimeUnit::S))
    );
    assert_eq!(
        parse_animation_duration("0s"),
        Some(AnimationDurationValue::Time(0.0, TimeUnit::S))
    );
    assert_eq!(parse_animation_duration("-1s"), None, "负数应返回 None");
    assert_eq!(parse_animation_duration(""), None);
    assert_eq!(parse_animation_duration("100"), None, "无单位应返回 None");
}

#[test]
/// 测试 parse_animation_iteration_count infinite 和数值。
fn test_parse_animation_iteration_count() {
    assert_eq!(
        parse_animation_iteration_count("infinite"),
        Some(AnimationIterationCountValue::Infinite)
    );
    assert_eq!(
        parse_animation_iteration_count("3"),
        Some(AnimationIterationCountValue::Number(3.0))
    );
    assert_eq!(
        parse_animation_iteration_count("2.5"),
        Some(AnimationIterationCountValue::Number(2.5))
    );
    assert_eq!(
        parse_animation_iteration_count("1"),
        Some(AnimationIterationCountValue::Number(1.0))
    );
    assert_eq!(parse_animation_iteration_count("0"), None, "0 应返回 None");
    assert_eq!(parse_animation_iteration_count("-1"), None, "负数应返回 None");
    assert_eq!(
        parse_animation_iteration_count("INFINITE"),
        Some(AnimationIterationCountValue::Infinite)
    );
    assert_eq!(parse_animation_iteration_count("abc"), None);
}

// ═══════════════════════════════════════════════════════════════════════
// 29. parse_stylesheet 全路径覆盖测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试解析空样式表。
fn test_parse_stylesheet_empty() {
    let stylesheet = crate::Parser::parse_stylesheet("");
    assert!(stylesheet.rules.is_empty());
}

#[test]
/// 测试解析仅包含空白和注释的样式表。
fn test_parse_stylesheet_only_whitespace() {
    let css = "  \n\t  /* comment */  \n  ";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert!(stylesheet.rules.is_empty());
}

#[test]
/// 测试解析多个样式规则。
fn test_parse_stylesheet_multiple_rules() {
    let css = "a { color: red; } b { color: blue; } c { color: green; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 3);
}

#[test]
/// 测试解析 @import 规则。
fn test_parse_stylesheet_import() {
    let css = r#"@import url("style.css");"#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Import(import) => assert_eq!(import.url, "style.css"),
        _ => panic!("Expected Import rule"),
    }
}

#[test]
/// 测试解析 @layer 规则。
fn test_parse_stylesheet_layer() {
    let css = "@layer base { .a { color: red; } }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    assert!(matches!(&stylesheet.rules[0], Rule::Layer(_)));
}

#[test]
/// 测试解析 @keyframes 规则。
fn test_parse_stylesheet_keyframes() {
    let css = "@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    assert!(matches!(&stylesheet.rules[0], Rule::Keyframes(_)));
}

#[test]
/// 测试解析 @supports 规则。
fn test_parse_stylesheet_supports() {
    let css = "@supports (display: grid) { .container { display: grid; } }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    assert!(matches!(&stylesheet.rules[0], Rule::Supports(_)));
}

#[test]
/// 测试解析 @container 规则。
fn test_parse_stylesheet_container() {
    let css = "@container (min-width: 700px) { .card { flex-direction: row; } }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    assert!(matches!(&stylesheet.rules[0], Rule::Container(_)));
}

#[test]
/// 测试解析 @media 规则。
fn test_parse_stylesheet_media() {
    let css = "@media screen and (min-width: 768px) { .a { color: red; } }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    assert!(matches!(&stylesheet.rules[0], Rule::At(_)));
}

#[test]
/// 测试解析通用 @-规则（非特殊规则）。
fn test_parse_stylesheet_generic_at() {
    let css = "@charset \"UTF-8\";";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

#[test]
/// 测试解析 ID 选择器。
fn test_parse_stylesheet_id_selector() {
    let css = "#main { color: red; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
/// 测试解析类选择器。
fn test_parse_stylesheet_class_selector() {
    let css = ".container { padding: 10px; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
/// 测试解析通用选择器。
fn test_parse_stylesheet_universal_selector() {
    let css = "* { margin: 0; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
/// 测试解析属性选择器。
fn test_parse_stylesheet_attribute_selector() {
    let css = "[data-type=\"card\"] { display: block; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
/// 测试解析伪元素选择器。
fn test_parse_stylesheet_pseudo_element() {
    let css = "::before { content: ''; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
/// 测试解析伪类选择器 :hover。
fn test_parse_stylesheet_pseudo_class_hover() {
    let css = "a:hover { text-decoration: underline; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
/// 测试解析子代组合器 >。
fn test_parse_stylesheet_child_combinator() {
    let css = "div > p { color: blue; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
/// 测试解析相邻兄弟组合器 +。
fn test_parse_stylesheet_next_sibling_combinator() {
    let css = "h1 + p { margin-top: 0; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
/// 测试解析一般兄弟组合器 ~。
fn test_parse_stylesheet_subsequent_sibling_combinator() {
    let css = "h1 ~ p { color: gray; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
/// 测试解析后代组合器（空白）。
fn test_parse_stylesheet_descendant_combinator() {
    let css = "div p { color: green; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
/// 测试解析多选择器（逗号分隔）。
fn test_parse_stylesheet_selector_list() {
    let css = "h1, h2, h3 { font-weight: bold; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Style(style) => assert_eq!(style.selectors.len(), 3),
        _ => panic!("Expected Style rule"),
    }
}

#[test]
/// 测试解析 :not() 伪类。
fn test_parse_stylesheet_not_pseudo() {
    let css = ".item:not(:first-child) { margin-top: 10px; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
/// 测试解析 :is() 伪类。
fn test_parse_stylesheet_is_pseudo() {
    let css = ":is(h1, h2, h3) { font-weight: bold; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
/// 测试解析 :where() 伪类。
fn test_parse_stylesheet_where_pseudo() {
    let css = ":where(.card, .panel) { border: 1px solid; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
/// 测试解析 :has() 伪类。
fn test_parse_stylesheet_has_pseudo() {
    let css = ".card:has(.title) { padding: 20px; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
/// 测试解析 :nth-child() 伪类。
fn test_parse_stylesheet_nth_child() {
    let css = "li:nth-child(2n+1) { background: #f0f0f0; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
/// 测试解析 :nth-last-child() 伪类。
fn test_parse_stylesheet_nth_last_child() {
    let css = "li:nth-last-child(3) { color: red; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
/// 测试解析 :nth-of-type() 伪类。
fn test_parse_stylesheet_nth_of_type() {
    let css = "p:nth-of-type(odd) { color: blue; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
/// 测试解析包含无效 token 的规则被跳过。
fn test_parse_stylesheet_invalid_tokens() {
    let css = "!!! { } .valid { color: red; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    // 至少 .valid 规则应被解析
    assert!(!stylesheet.rules.is_empty());
}

#[test]
/// 测试解析声明中的 !important。
fn test_parse_stylesheet_important() {
    let css = ".box { color: red !important; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Style(style) => {
            assert_eq!(style.declarations.len(), 1);
            assert!(style.declarations[0].important);
        }
        _ => panic!("Expected Style rule"),
    }
}

#[test]
/// 测试解析自定义属性声明。
fn test_parse_stylesheet_custom_property() {
    let css = ":root { --main-color: #3498db; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Style(style) => {
            assert_eq!(style.declarations.len(), 1);
            assert_eq!(style.declarations[0].property, "--main-color");
        }
        _ => panic!("Expected Style rule"),
    }
}

#[test]
/// 测试解析 @import 带 media query。
fn test_parse_stylesheet_import_with_media() {
    let css = r#"@import url("dark.css") screen and (prefers-color-scheme: dark);"#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
}

#[test]
/// 测试解析嵌套属性选择器 [attr=val]。
fn test_parse_stylesheet_attribute_exact_match() {
    let css = r#"input[type="text"] { border: 1px solid; }"#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
/// 测试解析属性选择器 [attr~=val]。
fn test_parse_stylesheet_attribute_space_match() {
    let css = r#"[class~="active"] { font-weight: bold; }"#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
/// 测试解析属性选择器 [attr|=val]。
fn test_parse_stylesheet_attribute_dash_match() {
    let css = r#"[lang|="en"] { quotes: "«" "»"; }"#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
/// 测试解析属性选择器 [attr^=val]。
fn test_parse_stylesheet_attribute_prefix_match() {
    let css = r#"[href^="https"] { color: green; }"#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
/// 测试解析属性选择器 [attr$=val]。
fn test_parse_stylesheet_attribute_suffix_match() {
    let css = r#"[href$=".pdf"] { color: red; }"#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
/// 测试解析属性选择器 [attr*=val]。
fn test_parse_stylesheet_attribute_substring_match() {
    let css = r#"[title*="example"] { cursor: help; }"#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
/// 测试解析多重声明。
fn test_parse_stylesheet_multiple_declarations() {
    let css = ".box { color: red; background: white; padding: 10px; margin: 5px; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Style(style) => assert!(style.declarations.len() >= 3),
        _ => panic!("Expected Style rule"),
    }
}

#[test]
/// 测试解析混合规则类型。
fn test_parse_stylesheet_mixed_rules() {
    let css = r#"
        @import url("base.css");
        .container { display: flex; }
        @media (min-width: 768px) { .container { flex-direction: row; } }
        @keyframes fade { from { opacity: 0; } to { opacity: 1; } }
        #app { background: #fff; }
    "#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert!(stylesheet.rules.len() >= 4, "Should parse at least 4 rules");
}
