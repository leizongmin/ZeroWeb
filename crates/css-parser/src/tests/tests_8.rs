// CSS 解析器覆盖率提升测试
// 专门针对 values/color.rs、tokenizer.rs 和 parser.rs 的未覆盖路径

use super::*;
use crate::values::*;
use crate::ast::*;
use crate::tokenizer::{Token, Tokenizer, Spanned};
use crate::parser::Parser;

// ═══════════════════════════════════════════════════════════════════════
// values/color.rs 的额外测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 parse_color 无效十六进制颜色长度的更多情况
fn test_parse_color_invalid_hex_lengths() {
    let test_cases = vec![
        "#12345",   // 5 位
        "#1234567", // 7 位
        "#1",       // 1 位
        "#12",      // 2 位
        "#1234",    // 4 位（非标准）
        "#1234567890", // 10 位
    ];

    for input in test_cases {
        let result = parse_color(input);
        assert_eq!(result, None, "无效的十六进制长度应该返回 None: {}", input);
    }
}

#[test]
/// 测试 parse_named_color 的大小写混合
fn test_parse_named_color_mixed_case() {
    let test_cases = vec![
        ("CRIMSON", ColorValue::Rgba(220, 20, 60, 255)),
        ("CurrentColor", ColorValue::CurrentColor),
        ("aLiCeBlUe", ColorValue::Rgba(240, 248, 255, 255)),
        ("DaRkGrEy", ColorValue::Rgba(169, 169, 169, 255)),
    ];

    for (input, expected) in test_cases {
        let result = parse_color(input);
        assert_eq!(result, Some(expected), "大小写混合的命名颜色应该被正确解析: {}", input);
    }
}

#[test]
/// 测试 hwb_to_rgba 的边界情况
fn test_hwb_to_rgba_edge_cases() {
    // W + B > 1.0 的情况
    assert_eq!(hwb_to_rgba(0.0, 0.8, 0.8, 1.0), (255, 255, 255, 255));

    // W + B = 1.0 的情况
    assert_eq!(hwb_to_rgba(120.0, 0.5, 0.5, 1.0), (0, 255, 0, 255));

    // 超过 1.0 的值
    assert_eq!(hwb_to_rgba(240.0, 1.5, 0.5, 1.0), (128, 128, 255, 255));

    // 负值
    assert_eq!(hwb_to_rgba(-30.0, -0.1, 0.2, 1.0), (255, 128, 0, 255));

    // 0 度色相
    assert_eq!(hwb_to_rgba(0.0, 0.0, 0.0, 1.0), (255, 0, 0, 255));

    // 360 度色相
    assert_eq!(hwb_to_rgba(360.0, 0.0, 0.0, 1.0), (255, 0, 0, 255));
}

#[test]
/// 测试 rgb() 函数的越界百分比
fn test_rgb_function_out_of_range() {
    // 超出范围的百分比
    assert_eq!(parse_color("rgba(-10%, 120%, 50%, 2.0)"), None);

    // 测试 parse_color_component 直接处理越界值
    assert_eq!(super::values::color::parse_color_component("150%"), Some(255));
    assert_eq!(super::values::color::parse_color_component("-20%"), Some(0));
    assert_eq!(super::values::color::parse_color_component("300"), Some(255));
    assert_eq!(super::values::color::parse_color_component("-50"), Some(0));

    // 越界 alpha
    assert_eq!(super::values::color::parse_alpha_component("150%"), Some(255));
    assert_eq!(super::values::color::parse_alpha_component("-10%"), Some(0));
    assert_eq!(super::values::color::parse_alpha_component("2.0"), Some(255));
    assert_eq!(super::values::color::parse_alpha_component("-0.5"), Some(0));
}

#[test]
/// 测试 hsl() 函数的越界值
fn test_hsl_function_out_of_range() {
    // 超出 360 的色相
    assert_eq!(parse_color("hsl(720, 50%, 50%)"), Some(ColorValue::Hsla(0.0, 50.0, 50.0, 1.0)));
    assert_eq!(parse_color("hsl(-30, 50%, 50%)"), Some(ColorValue::Hsla(330.0, 50.0, 50.0, 1.0)));

    // 超出 0-100% 的饱和度
    assert_eq!(parse_color("hsl(120, 150%, 50%)"), Some(ColorValue::Hsla(120.0, 100.0, 50.0, 1.0)));
    assert_eq!(parse_color("hsl(120, -20%, 50%)"), Some(ColorValue::Hsla(120.0, 0.0, 50.0, 1.0)));

    // 超出 0-100% 的亮度
    assert_eq!(parse_color("hsl(120, 50%, 150%)"), Some(ColorValue::Hsla(120.0, 50.0, 100.0, 1.0)));
    assert_eq!(parse_color("hsl(120, 50%, -20%)"), Some(ColorValue::Hsla(120.0, 50.0, 0.0, 1.0)));

    // hsla 超值
    assert_eq!(parse_color("hsla(480, 200%, 200%, 2.0)"), Some(ColorValue::Hsla(120.0, 100.0, 100.0, 1.0)));
}

#[test]
/// 测试 parse_font_style 的各种 oblique 形式
fn test_parse_font_style_oblique_angles() {
    let test_cases = vec![
        ("oblique", Some(FontStyleValue::Oblique(None))),
        ("oblique 10deg", Some(FontStyleValue::Oblique(Some(10.0)))),
        ("oblique -15deg", Some(FontStyleValue::Oblique(Some(-15.0)))),
        ("oblique(5deg)", Some(FontStyleValue::Oblique(Some(5.0)))),
        ("oblique( 10deg )", Some(FontStyleValue::Oblique(Some(10.0)))),
        ("oblique(-20deg)", Some(FontStyleValue::Oblique(Some(-20.0)))),
    ];

    for (input, expected) in test_cases {
        let result = parse_font_style(input);
        assert_eq!(result, expected, "oblique 角度应该被正确解析: {}", input);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// tokenizer.rs 的额外测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 consume_escape 的各种边界情况
fn test_consume_escape_edge_cases() {
    let test_cases = vec![
        ("\\41", 'A'),                    // 有效十六进制
        ("\\4", '\u{FFFD}'),              // 不完整的十六进制
        ("\\x41", 'A'),                   // 两位十六进制
        ("\\x41A", 'A'),                  // 两位十六进制 + 后续字符
        ("\\0", '\u{FFFD}'),              // 无效的十六进制
        ("\\D800", '\u{FFFD}'),           // 无效的 UTF-16 代理对
        ("\\110000", '\u{FFFD}'),         // 超出 Unicode 范围
        ("\\20 ", ' '),                   // 带后续空白
        ("\\\n", None as Option<char>),  // 换行不能转义
        ("\\\r", None as Option<char>),  // 回车不能转义
        ("\\\x0C", None as Option<char>), // 换页不能转义
        ("", None as Option<char>),       // 空输入（不应发生）
    ];

    for (input, expected) in test_cases {
        let mut tokenizer = Tokenizer::new(input);
        if input.starts_with('\\') {
            let result = tokenizer.consume_escape();
            assert_eq!(result, expected, "转义序列解析失败: \\{}", &input[1..]);
        }
    }
}

#[test]
/// 测试 consume_url 的特殊字符情况
fn test_consume_url_special_chars() {
    let test_cases = vec![
        "url('http://example.com/test?query=value#fragment')",
        "url(\"https://example.com/path with spaces/file.png\")",
        "url(data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=)",
        "url('image.png')",
        "url(image.png)",
        "url()", // 空 URL
        "url(   )", // 只有空格
    ];

    for css in test_cases {
        let tokenizer = Tokenizer::new(css);
        let tokens: Vec<_> = tokenizer.collect_tokens();
        // 确保不 panic
        assert!(!tokens.is_empty(), "URL 解析不应为空: {}", css);
    }
}

#[test]
/// 测试数字解析的各种情况
fn test_number_parsing_edge_cases() {
    let test_cases = vec![
        "1e5",      // 科学计数法
        "1.5e-3",   // 科学计数法带负指数
        ".5",       // 无整数部分
        "0.5",      // 标准小数
        "+1.5",     // 正数
        "-2.5",     // 负数
        "1e+5",     // 科学计数法带正指数
        "1E5",      // 大写 E
        "1.0e0",    // 科学计数法归零
        ".5e-3",    // 小数开头
        "1.",       // 小数点后无数字
        "1.e5",     // 小数点后无数字但有指数
    ];

    for css in test_cases {
        let tokenizer = Tokenizer::new(css);
        let tokens: Vec<_> = tokenizer.collect_tokens();
        // 确保有数字 token
        let has_number = tokens.iter().any(|t| matches!(t, Token::Number(_)));
        assert!(has_number, "应该解析为数字: {}", css);
    }
}

#[test]
/// 测试未闭合的注释
fn test_unterminated_comments() {
    let test_cases = vec![
        "/* unclosed",
        "/* unclosed comment",
        "/* multi\nline\nunclosed",
        "/* /* nested /* still unclosed",
    ];

    for css in test_cases {
        let tokenizer = Tokenizer::new(css);
        let tokens: Vec<_> = tokenizer.collect_tokens();
        // 确保不 panic
        assert!(!tokens.is_empty(), "未闭合注释解析不应为空: {}", css);
    }
}

#[test]
/// 测试 line_column_from_offset 的各种换行符
fn test_line_column_from_offset_newline_types() {
    let test_cases = vec![
        ("hello\nworld", vec![(5, (1, 6)), (6, (2, 1))]),      // LF
        ("hello\rworld", vec![(5, (1, 6)), (6, (2, 1))]),      // CR
        ("hello\r\nworld", vec![(5, (1, 6)), (7, (2, 1))]),   // CRLF
        ("hello\n\rworld", vec![(5, (1, 6)), (6, (2, 1)), (7, (3, 1))]), // LF + CR
        ("line1\nline2\r\nline3", vec![(6, (1, 7)), (12, (2, 1)), (19, (3, 1))]),
        ("", vec![(0, (1, 1))]),                              // 空字符串
    ];

    for (source, offsets) in test_cases {
        for (offset, expected) in offsets {
            let result = line_column_from_offset(source, offset);
            assert_eq!(
                result, expected,
                "换行符处理错误 - offset {}: expected {:?}, got {:?}",
                offset, expected, result
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs 的额外测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试错误恢复：缺少闭合花括号
fn test_parser_error_recovery_missing_brace() {
    let test_cases = vec![
        ".test { color: red; } .other { font-size: 16px; ",
        ".test { background: blue; display: block",
        "@media screen { .test { color: red; }",
        "@supports (display: flex) { .test { color: blue; }",
    ];

    for css in test_cases {
        let stylesheet = crate::Parser::parse_stylesheet(css);
        // 确保解析能继续，不 panic
        assert!(!stylesheet.rules.is_empty() || css.contains("error"), "错误恢复应该能解析一些规则: {}", css);
    }
}

#[test]
/// 测试错误恢复：未闭合字符串
fn test_parser_error_recovery_unclosed_string() {
    let test_cases = vec![
        ".test { content: 'unclosed string; color: red; }",
        ".test { content: \"unclosed; font-size: 16px;",
        "@import 'unclosed-url.css';",
    ];

    for css in test_cases {
        let stylesheet = crate::Parser::parse_stylesheet(css);
        // 确保不 panic
        assert!(!stylesheet.rules.is_empty() || css.contains("error"), "未闭合字符串应该能继续解析: {}", css);
    }
}

#[test]
/// 测试 @layer 匿名层
fn test_parse_layer_anonymous() {
    let css = "@layer { div { color: red; } }";
    let stylesheet = crate::Parser::parse_stylesheet(css);

    // 查找 @layer 规则
    let layer_rules = stylesheet.rules.iter().filter(|r| matches!(r, Rule::Layer(_))).collect::<Vec<_>>();
    assert!(!layer_rules.is_empty(), "应该解析出 @layer 规则");

    if let Rule::Layer(layer) = &layer_rules[0] {
        assert!(layer.name.is_none(), "匿名层应该没有名称");
        assert!(!layer.rules.is_empty(), "匿名层应该包含规则");
    }
}

#[test]
/// 测试 @layer 声明（只有名称）
fn test_parse_layer_declaration_only() {
    let css = "@layer components;";
    let stylesheet = crate::Parser::parse_stylesheet(css);

    // 查找 @layer 规则
    let layer_rules = stylesheet.rules.iter().filter(|r| matches!(r, Rule::Layer(_))).collect::<Vec<_>>();
    assert!(!layer_rules.is_empty(), "应该解析出 @layer 规则");

    if let Rule::Layer(layer) = &layer_rules[0] {
        assert_eq!(&layer.name, "components", "层名称应该是 'components'");
        assert!(layer.rules.is_empty(), "只有声明的层应该是空的");
    }
}

#[test]
/// 测试 @import 带媒体查询
fn test_parse_import_with_media_queries() {
    let test_cases = vec![
        r#"@import url("test.css") screen and (min-width: 500px);"#,
        r#"@import "style.css" print and (orientation: portrait);"#,
        r#"@import "mobile.css" (max-width: 768px);"#,
        r#"@import "print.css" screen, print;"#,
        r#"@import "all.css" screen and (color), screen and (monochrome);"#,
    ];

    for css in test_cases {
        let stylesheet = crate::Parser::parse_stylesheet(css);

        // 查找 @import 规则
        let import_rules = stylesheet.rules.iter().filter(|r| matches!(r, Rule::Import(_))).collect::<Vec<_>>();
        assert!(!import_rules.is_empty(), "应该解析出 @import 规则: {}", css);

        if let Rule::Import(import) = &import_rules[0] {
            assert!(!import.url.is_empty(), "URL 不应为空");
            assert!(!import.media_queries.is_empty(), "媒体查询不为空");
        }
    }
}

#[test]
/// 测试 @container 带名称
fn test_parse_container_with_name() {
    let test_cases = vec![
        r#"@container sidebar (min-width: 400px) { .test { color: red; } }"#,
        r#"@component-card (orientation: portrait) { .item { float: left; } }"#,
        r#"@container panel (inline-size > 200px) { button { width: 100%; } }"#,
    ];

    for css in test_cases {
        let stylesheet = crate::Parser::parse_stylesheet(css);

        // 查找 @container 规则
        let container_rules = stylesheet.rules.iter().filter(|r| matches!(r, Rule::Container(_))).collect::<Vec<_>>();
        assert!(!container_rules.is_empty(), "应该解析出 @container 规则: {}", css);

        if let Rule::Container(container) = &container_rules[0] {
            assert!(!container.name.is_empty(), "容器名称不为空");
            assert!(matches!(container.condition, Some(_)), "容器条件应该存在");
            assert!(!container.rules.is_empty(), "容器规则不为空");
        }
    }
}

#[test]
/// 测试 @keyframes 无效选择器
fn test_parse_keyframes_invalid_selectors() {
    let test_cases = vec![
        r#"@keyframes 1% { from { opacity: 0; } to { opacity: 1; } }"#, // 数字开头
        r#"@keyframes test { invalid-selector { color: red; } }"#, // 无效选择器
        r#"@keyframes test { 50% {} }"#, // 只有百分比没有块
    ];

    for css in test_cases {
        let stylesheet = crate::Parser::parse_stylesheet(css);

        // 查找 @keyframes 规则
        let keyframe_rules = stylesheet.rules.iter().filter(|r| matches!(r, Rule::Keyframes(_))).collect::<Vec<_>>();
        if !keyframe_rules.is_empty() {
            if let Rule::Keyframes(keyframes) = &keyframe_rules[0] {
                // 即使选择器无效，也应该能解析出一些规则
                assert!(!keyframes.name.is_empty(), "keyframes 名称应该存在");
            }
        }
    }
}

#[test]
/// 测试复杂的 @layer 层级
fn test_parse_layer_hierarchy() {
    let css = r#"
        @layer base, components, utilities;
        @layer components {
            .button { background: blue; }
        }
        @layer utilities {
            .text-center { text-align: center; }
        }
        @layer base {
            body { margin: 0; }
        }
    "#;

    let stylesheet = crate::Parser::parse_stylesheet(css);

    // 查找所有 @layer 规则
    let layer_rules = stylesheet.rules.iter().filter(|r| matches!(r, Rule::Layer(_))).collect::<Vec<_>>();
    assert_eq!(layer_rules.len(), 3, "应该有 3 个 @layer 规则");

    // 检查是否有包含规则的层
    for layer in layer_rules {
        if let Rule::Layer(layer_rule) = layer {
            if layer_rule.name == "components" {
                assert!(!layer_rule.rules.is_empty(), "components 层应该包含规则");
            }
        }
    }
}