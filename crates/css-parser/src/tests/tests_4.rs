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
    assert_eq!(
        parse_animation_iteration_count("0"),
        Some(AnimationIterationCountValue::Number(0.0)),
        "0 应为合法的非负迭代次数"
    );
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
/// CSS 属性名 ASCII 大小写不敏感（CSS Syntax §5「All CSS keywords are
/// case-insensitive」）：`bACkGRounD` 须归一化为小写 `background`，否则下游
/// apply.rs 按小写名 dispatch 会丢声明（WPT case-sensitive-000）。
/// 自定义属性（`--*`）大小写敏感（CSS Variables §2），须保留原值。
fn test_parse_property_name_case_insensitive() {
    let css = "p { bACkGRounD: gREen; --MyVar: red; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    let style = stylesheet
        .rules
        .iter()
        .find_map(|r| if let Rule::Style(s) = r { Some(s) } else { None })
        .expect("应有 style 规则");
    assert_eq!(style.declarations.len(), 2);
    // 标准属性名归一化为小写
    assert_eq!(
        style.declarations[0].property, "background",
        "bACkGRounD 应归一化为 background（CSS §5 大小写不敏感）"
    );
    // 自定义属性保留原大小写
    assert_eq!(
        style.declarations[1].property, "--MyVar",
        "自定义属性 --MyVar 大小写敏感须保留原值（CSS Variables §2）"
    );
}

#[test]
/// CSS 伪类 / 伪元素名 ASCII 大小写不敏感（CSS Syntax §5）：`:FiRSt-cHIlD` 与
/// `::FiRst-LiNe` 须归一化为小写，否则下游 matcher 按小写名匹配会失配
/// （WPT case-sensitive-003）。
fn test_parse_pseudo_name_case_insensitive() {
    let css = "p:FiRSt-cHIlD { color: green; } span::FiRst-LiNe { color: red; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    // 不 panic + 解析出 2 条规则即可（伪名归一化在 selector 解析内部，归一化后 matcher
    // 能匹配；此处主要守 parse 不因大小写丢规则）。
    assert_eq!(stylesheet.rules.len(), 2, "两条规则都应被解析（伪名大小写不敏感）");
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

// ═══════════════════════════════════════════════════════════════════════
// 30. Tokenizer 边界和错误处理测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 consume_comment - 注释未终止（EOF 在注释中）
fn test_consume_comment_unterminated() {
    let tokenizer = crate::Tokenizer::new("/* comment");
    let tokens: Vec<_> = tokenizer.collect_tokens();
    // 注释 tokenizer 可能直接返回错误 token 而不是产生 whitespace token
    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0], Token::Error(_)));
    if let Token::Error(msg) = &tokens[0] {
        assert_eq!(msg, "Unterminated comment");
    }
}

#[test]
/// 测试 consume_comment - 注释中有换行符
fn test_consume_comment_with_newlines() {
    let tokenizer = crate::Tokenizer::new("/* comment\nwith\nnewlines */");
    let tokens: Vec<_> = tokenizer.collect_tokens();
    // Comment tokenizer returns the comment directly
    assert_eq!(tokens.len(), 1); // just the comment
    assert!(matches!(tokens[0], Token::Comment(_)));
    if let Token::Comment(content) = &tokens[0] {
        assert_eq!(content, " comment\nwith\nnewlines ");
    }
}

#[test]
/// 测试 consume_escape - 各种转义序列（在标识符中）
fn test_consume_escape_sequences() {
    // 测试 Unicode 转义在标识符中
    let tokenizer = crate::Tokenizer::new("a\\41");
    let tokens: Vec<_> = tokenizer.collect_tokens();
    // Simple escape sequence
    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0], Token::Ident(ref s) if s == "aA"));
}

#[test]
/// 测试 consume_escape - 转义后跟换行符
fn test_consume_escape_with_newline() {
    let tokenizer = crate::Tokenizer::new("a\\ \nb");
    let tokens: Vec<_> = tokenizer.collect_tokens();
    // The escape sequence is treated as part of the identifier
    assert_eq!(tokens.len(), 3);
    assert!(matches!(tokens[0], Token::Ident(ref s) if s == "a "));
    assert!(matches!(tokens[1], Token::Whitespace));
    assert!(matches!(tokens[2], Token::Ident(ref s) if s == "b"));
}

#[test]
/// 测试 consume_escape - 无效的转义序列
fn test_consume_escape_invalid() {
    // R2132：`\` 起始走 ident-like 路径。`\000000` = 6 hex 全 0 → codepoint 0 →
    // REPLACEMENT CHAR（consume_escape §4.3.7），产 Ident("\u{FFFD}")，不再落 Error。
    let tokenizer = crate::Tokenizer::new("\\000000");
    let tokens: Vec<_> = tokenizer.collect_tokens();
    assert!(matches!(tokens[0], Token::Ident(_)));
}

#[test]
/// 测试 consume_number - 小数点开头
fn test_consume_number_leading_dot() {
    let tokenizer = crate::Tokenizer::new(".5px");
    let tokens: Vec<_> = tokenizer.collect_tokens();
    assert!(matches!(tokens[0], Token::Dimension(0.5, ref s) if s == "px"));
}

#[test]
/// 测试 consume_number - 科学计数法
fn test_consume_number_scientific_notation() {
    let tokenizer = crate::Tokenizer::new("1.5e+3px");
    let tokens: Vec<_> = tokenizer.collect_tokens();
    assert!(matches!(tokens[0], Token::Dimension(1500.0, ref s) if s == "px"));
}

#[test]
/// 测试 consume_number - 负数
fn test_consume_number_negative() {
    let tokenizer = crate::Tokenizer::new("-5px");
    let tokens: Vec<_> = tokenizer.collect_tokens();
    assert!(matches!(tokens[0], Token::Dimension(-5.0, ref s) if s == "px"));
}

#[test]
/// 测试 consume_string - 未终止的字符串
fn test_consume_string_unterminated() {
    let tokenizer = crate::Tokenizer::new("\"unterminated");
    let tokens: Vec<_> = tokenizer.collect_tokens();
    // 未终止的字符串，应该包含原始内容
    assert!(matches!(tokens[0], Token::String(ref s) if s == "unterminated"));
}

#[test]
/// 测试 consume_string - 字符串中的转义引号
fn test_consume_string_escaped_quotes() {
    let tokenizer = crate::Tokenizer::new(r#""escaped \" quotes""#);
    let tokens: Vec<_> = tokenizer.collect_tokens();
    // 转义引号保留为字面引号字符："escaped \" quotes" → `escaped " quotes`
    assert!(matches!(tokens[0], Token::String(ref s) if s == r#"escaped " quotes"#));
}

#[test]
/// 测试 consume_string - 字符串中的换行
fn test_consume_string_with_newline() {
    let tokenizer = crate::Tokenizer::new("\"hello\nworld\"");
    let tokens: Vec<_> = tokenizer.collect_tokens();
    // 换行符结束字符串
    assert!(matches!(tokens[0], Token::String(ref s) if s == "hello"));
}

#[test]
/// 测试 consume_url - 无引号的 URL 中有非法字符
fn test_consume_url_invalid_char() {
    let tokenizer = crate::Tokenizer::new("url(invalid(char)");
    let tokens: Vec<_> = tokenizer.collect_tokens();
    // 应该产生错误
    assert!(matches!(tokens[0], Token::Error(_)));
}

#[test]
/// 测试 consume_url - EOF 在 URL 中
fn test_consume_url_eof() {
    let tokenizer = crate::Tokenizer::new("url(");
    let tokens: Vec<_> = tokenizer.collect_tokens();
    // 应该产生空的 URL
    assert!(matches!(tokens[0], Token::Url(ref s) if s == ""));
}

#[test]
/// 测试 consume_ident_like - 函数 vs 标识符
fn test_consume_ident_like_function_vs_ident() {
    let tokenizer = crate::Tokenizer::new("func ident func()");
    let tokens: Vec<_> = tokenizer.collect_tokens();
    // The third token should be the function, but there are more tokens
    assert!(matches!(tokens[4], Token::Function(ref s) if s == "func"));
}

#[test]
/// 测试 consume_ident - 以数字开头的标识符（通过转义）
fn test_consume_ident_start_with_digit_escape() {
    // R2132：`\` 起始路由到 ident-like。`\31 ident` = `\31`(=0x31='1'，消耗一个空白终止)
    // + `ident` → ident "1ident"（通过转义以数字开头）。driving：escapes-002 谱系。
    // 注：raw 字符串单反斜杠（`r#"\31 ident"#`），非 `\\`（两个反斜杠）。
    let tokenizer = crate::Tokenizer::new(r#"\31 ident"#);
    let tokens: Vec<_> = tokenizer.collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0], Token::Ident(ref s) if s == "1ident"));
}

#[test]
/// 测试 consume_ident - 连字符处理
fn test_consume_ident_hyphen_handling() {
    let tokenizer = crate::Tokenizer::new("a- -b");
    let tokens: Vec<_> = tokenizer.collect_tokens();
    // There's a whitespace token between the two identifiers
    assert_eq!(tokens.len(), 3);
    assert!(matches!(tokens[0], Token::Ident(ref s) if s == "a-"));
    assert!(matches!(tokens[1], Token::Whitespace));
    assert!(matches!(tokens[2], Token::Ident(ref s) if s == "-b"));
}

#[test]
/// 测试 consume_number_and_suffix - 百分比
fn test_consume_number_and_suffix_percentage() {
    let tokenizer = crate::Tokenizer::new("50%");
    let tokens: Vec<_> = tokenizer.collect_tokens();
    assert!(matches!(tokens[0], Token::Percentage(50.0)));
}

#[test]
/// 测试 consume_number_and_suffix - 带单位
fn test_consume_number_and_suffix_dimension() {
    let tokenizer = crate::Tokenizer::new("10.5rem");
    let tokens: Vec<_> = tokenizer.collect_tokens();
    assert!(matches!(tokens[0], Token::Dimension(10.5, ref s) if s == "rem"));
}

#[test]
/// 测试 consume_whitespace - 各种空白字符
fn test_consume_whitespace_various() {
    // R2132：raw 串 ` \t\n\r\f `（`\t`/`\n`/`\r`/`\f` 为字面 反斜杠+字母）。
    // `\` 起始路由 ident-like：`\t`(='t')、`\n`(='n')、`\r`(='r')、`\f`(hex 0x0f=换页，
    // 消耗尾随空格作终止) → 合并成 ident "tnr\u{f}"，不再产生 spurious Error。
    let tokenizer = crate::Tokenizer::new(r" \t\n\r\f ");
    let tokens: Vec<_> = tokenizer.collect_tokens();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::Whitespace));
    assert!(matches!(tokens[1], Token::Ident(ref s) if s == "tnr\u{f}"));
}

#[test]
/// 测试 peek_at - 偏移查看
fn test_peek_at_offset() {
    // "abc" 被当作一个标识符 token
    let tokenizer = crate::Tokenizer::new("abc");
    let tokens = tokenizer.collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0], crate::Token::Ident(ref s) if s == "abc"));
}

// ═══════════════════════════════════════════════════════════════════════
// 31. Parser 边界和错误恢复测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试解析不完整的属性选择器 [attr]
fn test_parse_incomplete_attribute_selector() {
    let css = "[attr";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    // 不完整的属性选择器可能产生空规则或错误恢复规则
    // 关键是不 panic
    let _ = stylesheet.rules;
}

#[test]
/// 测试解析带嵌套的 @keyframes
fn test_parse_keyframes_with_nested_rules() {
    let css = r#"
        @keyframes slide {
            0% { top: 0; left: 0; }
            50% { top: 50px; left: 50px; }
            100% { top: 100px; left: 100px; }
        }
    "#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Keyframes(keyframes) = &stylesheet.rules[0] {
        assert_eq!(keyframes.keyframes.len(), 3);
    }
}

#[test]
/// 测试解析 @keyframes 使用百分比
fn test_parse_keyframes_with_percentages() {
    let css = r#"
        @keyframes pulse {
            0% { transform: scale(1); }
            50% { transform: scale(1.1); }
            100% { transform: scale(1); }
        }
    "#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Keyframes(keyframes) = &stylesheet.rules[0] {
        assert_eq!(keyframes.keyframes.len(), 3);
    }
}

#[test]
/// 测试解析 @layer 匿名层
fn test_parse_layer_anonymous() {
    let css = "@layer { .a { color: red; } }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Layer(layer) = &stylesheet.rules[0] {
        assert_eq!(layer.name, "");
        assert_eq!(layer.rules.len(), 1);
    }
}

#[test]
/// 测试解析 @layer 仅声明
fn test_parse_layer_declaration_only() {
    let css = "@layer components;";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Layer(layer) = &stylesheet.rules[0] {
        assert_eq!(layer.name, "components");
        assert_eq!(layer.rules.len(), 0);
    }
}

#[test]
/// 测试解析 @import 带多个媒体查询
fn test_parse_import_multiple_media_queries() {
    let css = r#"@import "style.css" screen, print and (orientation: landscape);"#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert!(!stylesheet.rules.is_empty());
    if let Rule::Import(import) = &stylesheet.rules[0] {
        assert_eq!(import.media_queries.len(), 2);
    }
}

#[test]
/// 测试解析 @supports 嵌套规则
fn test_parse_supports_nested_rules() {
    let css = r#"
        @supports (display: grid) {
            .container {
                display: grid;
                grid-template-columns: repeat(2, 1fr);
            }
            @supports (place-items: center) {
                .container { place-items: center; }
            }
        }
    "#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Supports(supports) = &stylesheet.rules[0] {
        assert_eq!(supports.rules.len(), 2);
    }
}

#[test]
/// 测试解析 @container 带名称和复杂条件
fn test_parse_container_with_name() {
    let css = r#"
        @container sidebar (min-width: 200px) {
            .card { width: 100%; }
        }
    "#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Container(container) = &stylesheet.rules[0] {
        assert_eq!(container.name, Some("sidebar".to_string()));
    }
}

#[test]
/// 测试解析 @container 带范围条件
fn test_parse_container_range_condition() {
    let css = r#"
        @container (200px <= width <= 500px) {
            .card { flex-direction: column; }
        }
    "#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
/// 测试解析 :nth-child() 表达式
fn test_parse_nth_child_expressions() {
    let css = r#"
        li:nth-child(2n+1) { /* odd */ }
        li:nth-child(2n) { /* even */ }
        li:nth-child(3n+2) { /* every 3rd, starting from 2 */ }
        li:nth-child(-n+3) { /* first 3 */ }
    "#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 4);
}

#[test]
/// 测试解析 :nth-of-type() 表达式
fn test_parse_nth_of_type_expressions() {
    let css = r#"
        p:nth-of-type(odd) { /* odd paragraphs */ }
        p:nth-of-type(even) { /* even paragraphs */ }
        p:nth-of-type(5n) { /* every 5th paragraph */ }
    "#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 3);
}

#[test]
/// 测试解析 :lang() 函数
fn test_parse_lang_function() {
    let css = r#"
        :lang(en) { /* English */ }
        :lang("zh-CN") { /* Chinese */ }
        :lang(de-DE) { /* German */ }
    "#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 3);
}

#[test]
/// 测试解析多重伪类组合
fn test_parse_pseudo_class_combinations() {
    let css = r#"
        .item:hover:active { /* hover and active */ }
        .item:first-child:last-child { /* first and last */ }
        :is(.class1, .class2):not(.disabled) { /* is and not */ }
    "#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 3);
}

#[test]
/// 测试解析复杂的嵌套选择器
fn test_parse_complex_nested_selectors() {
    let css = r#"
        div > p + span ~ a::before { /* complex chain */ }
        .container .item .sub-item { /* nested descendant */ }
        section > h1 + p, article > h2 + p { /* selector list */ }
        ul li:nth-child(odd):not([hidden]) { /* complex with pseudo and attribute */ }
    "#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 4);
}

#[test]
/// 测试解析带 !important 的多个声明
fn test_parse_multiple_declarations_with_important() {
    let css = r#"
        .box {
            color: red !important;
            background: white !important;
            border: 1px solid #ccc;
            margin: 0 !important;
        }
    "#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(style) = &stylesheet.rules[0] {
        assert_eq!(style.declarations.len(), 4);
        assert_eq!(style.declarations.iter().filter(|d| d.important).count(), 3);
    }
}

#[test]
/// 测试解析注释后的规则
fn test_parse_rules_after_comment() {
    let css = r#"
        /* comment */
        .a { color: red; }
        /* another comment */
        .b { color: blue; }
    "#;
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 2);
}

#[test]
/// 测试解析混合空白字符
fn test_parse_mixed_whitespace() {
    let css = r"a  \t\n  \r\f  { color: red; }";
    let stylesheet = crate::Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
}
