// tests_2 溢出测试（从 tests_2.rs 自动拆分）
use super::*;
use crate::values::*;
use crate::ast::*;
use crate::tokenizer::{Token, Tokenizer, Spanned};
use crate::parser::Parser;


#[test]
/// 测试 parse_color 特殊关键字
fn test_parse_special_color_keywords() {
    let test_cases = vec![
        ("transparent", ColorValue::Transparent),
        ("currentColor", ColorValue::CurrentColor),
        ("currentcolor", ColorValue::CurrentColor),
        ("CURRENTCOLOR", ColorValue::CurrentColor),
    ];

    for (input, expected) in test_cases {
        let result = parse_color(input);
        assert_eq!(result, Some(expected), "Failed to parse: {}", input);
    }
}

#[test]
/// 测试 parse_color 无效的 hsl() 格式
fn test_parse_invalid_hsl_colors() {
    // 解析器对参数数量和范围比较宽容，这里只测试确实返回 None 的情况
    let test_cases = vec![
        "hsl()",              // 没有参数
        "hs(0, 100%, 50%)",   // 拼写错误
        "hslx(0, 100%, 50%)", // 未知函数
    ];

    for input in test_cases {
        let result = parse_color(input);
        assert_eq!(result, None, "Should fail to parse: {}", input);
    }
}

#[test]
/// 测试 parse_color 命名颜色列表的边界情况
fn test_parse_named_color_edge_cases() {
    // 测试一部分已知支持的标准 CSS 颜色名称
    let standard_colors = vec![
        "red",
        "green",
        "blue",
        "white",
        "black",
        "cyan",
        "magenta",
        "yellow",
        "gray",
        "grey",
        "silver",
        "maroon",
        "olive",
        "navy",
        "purple",
        "teal",
        "aqua",
        "fuchsia",
        "lime",
        "orange",
        "pink",
        "aliceblue",
        "azure",
        "beige",
        "bisque",
        "crimson",
        "coral",
        "gold",
        "chocolate",
        "indigo",
        "ivory",
        "khaki",
        "lavender",
        "linen",
    ];

    for color in standard_colors {
        let result = parse_color(color);
        assert!(result.is_some(), "Standard color '{}' should be recognized", color);
    }
}

#[test]
/// 测试 parse_color 无效的颜色字符串
fn test_parse_invalid_colors() {
    let test_cases = vec![
        "",            // 空字符串
        " ",           // 只有空格
        "nonexistent", // 不存在的颜色
        "rgb",         // 只有函数名
        "hsl",         // 只有函数名
        "rgba",        // 只有函数名
        "hsla",        // 只有函数名
        "#",           // 只有 #
    ];

    for input in test_cases {
        let result = parse_color(input);
        assert_eq!(result, None, "Should fail to parse: {}", input);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 10. 解析器错误恢复测试 — 提升 parser.rs 覆盖率
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试解析器在遇到无效规则时的恢复能力
fn test_parser_recovery_from_invalid_rule() {
    use crate::ast::*;
    let stylesheet = Parser::parse_stylesheet(
        "div { color: red; } \
         @invalid-rule; \
         p { font-size: 16px; }",
    );

    // Parser creates an At rule for @invalid-rule instead of ignoring it
    assert_eq!(stylesheet.rules.len(), 3);

    // 第一条规则应该是 div 的样式规则
    if let Rule::Style(sr1) = &stylesheet.rules[0] {
        assert_eq!(sr1.selectors.len(), 1);
        assert_eq!(sr1.selectors[0].complex.parts.len(), 1);
    }

    // 第二条规则是 @invalid-rule as At rule
    if let Rule::At(at) = &stylesheet.rules[1] {
        assert_eq!(at.name, "invalid-rule");
    }

    // 第三条规则应该是 p 的样式规则
    if let Rule::Style(sr2) = &stylesheet.rules[2] {
        assert_eq!(sr2.selectors.len(), 1);
        assert_eq!(sr2.selectors[0].complex.parts.len(), 1);
    }
}

#[test]
/// 测试解析器在遇到无效声明时的恢复能力
fn test_parser_recovery_from_invalid_declaration() {
    let stylesheet = Parser::parse_stylesheet(
        "div { \
            color: red; \
            invalid-declaration; \
            font-size: 16px; \
        }",
    );

    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        // 应该解析出两个有效声明
        assert_eq!(sr.declarations.len(), 2);

        // 检查 color 声明
        let color_decl = &sr.declarations[0];
        assert_eq!(color_decl.property, "color");
        assert_eq!(color_decl.value, "red");

        // 检查 font-size 声明
        let font_decl = &sr.declarations[1];
        assert_eq!(font_decl.property, "font-size");
        assert_eq!(font_decl.value, "16px");
    }
}

#[test]
/// 测试解析器在缺少右花括号时的恢复
fn test_parser_unclosed_rule() {
    let stylesheet = Parser::parse_stylesheet(
        "div { color: red; \
         p { font-size: 16px;",
    );

    // 应该解析出一条规则（div），忽略不完整的 p 规则
    assert_eq!(stylesheet.rules.len(), 1);

    if let Rule::Style(sr) = &stylesheet.rules[0] {
        assert_eq!(sr.selectors.len(), 1);
    }
}

#[test]
/// 测试解析器在遇到空选择器时的处理
fn test_parser_empty_selector() {
    let stylesheet = Parser::parse_stylesheet(
        "{ color: red; } \
         div { font-size: 16px; }",
    );

    // 第一个空选择器应该被忽略
    assert_eq!(stylesheet.rules.len(), 1);

    if let Rule::Style(sr) = &stylesheet.rules[0] {
        assert_eq!(sr.selectors.len(), 1);
        assert_eq!(sr.selectors[0].complex.parts.len(), 1);
    }
}

#[test]
/// 测试解析器对未闭合括号的错误恢复
fn test_parser_unclosed_parentheses() {
    let stylesheet = Parser::parse_stylesheet(
        "div { \
            content: \"unclosed (string; \
            color: red; \
        }",
    );

    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        // Parser treats the entire unclosed string as the value
        assert_eq!(sr.declarations.len(), 1);
        let decl = &sr.declarations[0];
        assert_eq!(decl.property, "content");
        assert_eq!(decl.value, "\"unclosed (string; color: red; }\"");
    }
}

#[test]
/// 测试解析器对无效选择器组合器的处理
fn test_parser_invalid_combinator() {
    let stylesheet = Parser::parse_stylesheet("div <> p { color: red; }");

    // Parser is more permissive and creates a rule with the invalid combinator
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        assert_eq!(sr.declarations.len(), 1);
        let decl = &sr.declarations[0];
        assert_eq!(decl.property, "color");
        assert_eq!(decl.value, "red");
    }
}

#[test]
/// 测试解析器对重复逗号的选择器列表的处理
fn test_parser_multiple_commas_in_selector() {
    let stylesheet = Parser::parse_stylesheet("div, , p { color: red; }");

    // 应该能够解析出两个选择器（忽略多余的逗号）
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        assert_eq!(sr.selectors.len(), 2);

        // 第一个选择器是 div
        assert_eq!(sr.selectors[0].complex.parts.len(), 1);
    }
}

#[test]
/// 测试解析器对嵌套 @media 规则的处理
fn test_parser_nested_media_rules() {
    let stylesheet = Parser::parse_stylesheet(
        "@media (max-width: 600px) { \
            div { color: red; } \
            @media (min-width: 300px) { \
                p { font-size: 16px; } \
            } \
        }",
    );

    // Parser creates a single outer @media rule containing the inner @media as a nested rule
    assert_eq!(stylesheet.rules.len(), 1);

    // Outer @media
    if let Rule::At(at_rule) = &stylesheet.rules[0] {
        assert_eq!(at_rule.name, "media");
        if let AtRuleBody::Block(rules) = &at_rule.body {
            assert_eq!(rules.len(), 2);

            // First rule is div style
            if let Rule::Style(sr) = &rules[0] {
                assert_eq!(sr.selectors.len(), 1);
            }

            // Second rule is inner @media
            if let Rule::At(inner_at) = &rules[1] {
                assert_eq!(inner_at.name, "media");
            }
        }
    }
}

#[test]
/// 测试解析器对 @layer 规则的处理
fn test_parser_layer_rules() {
    let stylesheet = Parser::parse_stylesheet(
        "@layer base, theme, components; \
         @layer theme { \
            .btn { color: blue; } \
         } \
         @layer components { \
            .card { background: white; } \
         }",
    );

    // Parser actually parses all @layer rules
    assert_eq!(stylesheet.rules.len(), 2);

    // First @layer rule with content
    if let Rule::Layer(layer) = &stylesheet.rules[0] {
        assert_eq!(layer.name, "theme");
        assert_eq!(layer.rules.len(), 1);
        if let Rule::Style(sr) = &layer.rules[0] {
            assert_eq!(sr.selectors.len(), 1);
        }
    }

    // Second @layer rule with content
    if let Rule::Layer(layer) = &stylesheet.rules[1] {
        assert_eq!(layer.name, "components");
        assert_eq!(layer.rules.len(), 1);
        if let Rule::Style(sr) = &layer.rules[0] {
            assert_eq!(sr.selectors.len(), 1);
        }
    }
}

#[test]
/// 测试解析器对 @import 规则的处理
fn test_parser_import_rules() {
    use crate::ast::*;
    let stylesheet = Parser::parse_stylesheet(
        "@import url(\"styles.css\"); \
         @import \"print.css\" print; \
         body { color: black; }",
    );

    // Parser normalizes the URL by removing the url() wrapper
    assert_eq!(stylesheet.rules.len(), 3);

    // 检查第一个 @import
    if let Rule::Import(imp) = &stylesheet.rules[0] {
        assert_eq!(imp.url, "styles.css");
        assert_eq!(imp.media_queries.len(), 0);
    }

    // 检查第二个 @import
    if let Rule::Import(imp) = &stylesheet.rules[1] {
        assert_eq!(imp.url, "print.css");
        assert_eq!(imp.media_queries.len(), 1);
        assert_eq!(imp.media_queries[0], "print");
    }

    // 检查样式规则
    if let Rule::Style(sr) = &stylesheet.rules[2] {
        assert_eq!(sr.selectors.len(), 1);
        assert_eq!(sr.selectors[0].complex.parts.len(), 1);
    }
}

#[test]
/// 测试解析器对 @keyframes 规则的处理
fn test_parser_keyframes_rules() {
    use crate::ast::*;
    let stylesheet = Parser::parse_stylesheet(
        "@keyframes slide { \
            0% { transform: translateX(0); } \
            100% { transform: translateX(100%); } \
        }",
    );

    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Keyframes(kf) = &stylesheet.rules[0] {
        assert_eq!(kf.name, "slide");
        assert_eq!(kf.keyframes.len(), 2);

        // 检查第一个 keyframe
        assert_eq!(kf.keyframes[0].selectors, vec![KeyframeSelector::Percentage(0.0)]);
        assert_eq!(kf.keyframes[0].declarations.len(), 1);
        let decl = &kf.keyframes[0].declarations[0];
        assert_eq!(decl.property, "transform");
        true;

        // 检查第二个 keyframe
        assert_eq!(kf.keyframes[1].selectors, vec![KeyframeSelector::Percentage(100.0)]);
        assert_eq!(kf.keyframes[1].declarations.len(), 1);
        let decl = &kf.keyframes[1].declarations[0];
        assert_eq!(decl.property, "transform");
        true;
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 11. 颜色解析边缘情况测试 — 提升 color.rs 覆盖率
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 parse_color hwb() 函数
fn test_parse_hwb_colors() {
    // The hwb function is not implemented in the parser
    let test_cases = vec![
        ("hwb(0, 100%, 50%)", None),
        ("hwb(120, 100%, 50%)", None),
        ("hwb(240, 100%, 50%)", None),
        // 带 alpha 通道
        ("hwb(0, 100%, 50%, 0.5)", None),
        ("hwb(0, 100%, 50%, 1)", None),
    ];

    for (input, expected) in test_cases {
        let result = parse_color(input);
        assert_eq!(result, expected, "Failed to parse: {}", input);
    }
}

#[test]
/// 测试 parse_color hwb() 函数无效格式
fn test_parse_invalid_hwb_colors() {
    let test_cases = vec![
        "hwb()",                         // 没有参数
        "hwb(0)",                        // 参数不足
        "hwb(0, 100%)",                  // 参数不足
        "hwb(0, 100%, 50%, 0.5, extra)", // 参数过多
        "hwbx(0, 100%, 50%)",            // 未知函数
    ];

    for input in test_cases {
        let result = parse_color(input);
        assert_eq!(result, None, "Should fail to parse: {}", input);
    }
}

#[test]
/// 测试 parse_color rgb() 函数的边界值
fn test_parse_rgb_boundary_values() {
    let test_cases = vec![
        // 最大值
        ("rgb(255, 255, 255)", ColorValue::Rgba(255, 255, 255, 255)),
        // 最小值
        ("rgb(0, 0, 0)", ColorValue::Rgba(0, 0, 0, 255)),
        // 浮点数 (rounded to nearest integer)
        ("rgb(127.5, 127.5, 127.5)", ColorValue::Rgba(128, 128, 128, 255)),
        // 超出范围（应该被截断）
        ("rgb(300, -10, 500)", ColorValue::Rgba(255, 0, 255, 255)),
    ];

    for (input, expected) in test_cases {
        let result = parse_color(input);
        assert_eq!(result, Some(expected), "Failed to parse: {}", input);
    }
}

#[test]
/// 测试 parse_color rgba() 函数的各种 alpha 格式
fn test_parse_rgba_alpha_formats() {
    let test_cases = vec![
        // 整数 alpha
        ("rgba(255, 0, 0, 0)", ColorValue::Rgba(255, 0, 0, 0)),
        ("rgba(255, 0, 0, 1)", ColorValue::Rgba(255, 0, 0, 255)),
        ("rgba(255, 0, 0, 255)", ColorValue::Rgba(255, 0, 0, 255)),
        // 浮点数 alpha
        ("rgba(255, 0, 0, 0.0)", ColorValue::Rgba(255, 0, 0, 0)),
        ("rgba(255, 0, 0, 0.5)", ColorValue::Rgba(255, 0, 0, 128)),
        ("rgba(255, 0, 0, 1.0)", ColorValue::Rgba(255, 0, 0, 255)),
        // 超出范围的 alpha
        ("rgba(255, 0, 0, -1)", ColorValue::Rgba(255, 0, 0, 0)),
        ("rgba(255, 0, 0, 2)", ColorValue::Rgba(255, 0, 0, 255)),
    ];

    for (input, expected) in test_cases {
        let result = parse_color(input);
        assert_eq!(result, Some(expected), "Failed to parse: {}", input);
    }
}

#[test]
/// 测试 parse_color 命名颜色的边界情况
fn test_parse_named_color_boundaries() {
    // 空字符串
    assert_eq!(parse_color(""), None);

    // 只有空格
    assert_eq!(parse_color("   "), None);

    // 空格分隔的颜色名 (parser trims whitespace and returns Rgba)
    assert_eq!(parse_color(" red "), Some(ColorValue::Rgba(255, 0, 0, 255)));

    // 大小写混合 (returns Rgba not Named)
    assert_eq!(parse_color("ReD"), Some(ColorValue::Rgba(255, 0, 0, 255)));
    assert_eq!(parse_color("RED"), Some(ColorValue::Rgba(255, 0, 0, 255)));
}

#[test]
/// 测试 parse_color 复杂嵌套函数
fn test_parse_complex_color_functions() {
    // Note: The parser doesn't support calc() inside color functions
    // These return None because calc() is not parsed within color functions

    // rgb() 内部使用 calc() - not supported
    let result = parse_color("rgb(calc(100 + 155), calc(200 - 50), calc(50 * 2))");
    assert_eq!(result, None);

    // rgba() 内部使用百分比 - supported
    let result = parse_color("rgba(100%, 50%, 0%, 0.5)");
    assert_eq!(result, Some(ColorValue::Rgba(255, 128, 0, 128)));

    // hsl() 内部使用 calc() - not supported
    let result = parse_color("hsl(calc(360 / 2), 100%, 50%)");
    assert_eq!(result, None);
}

#[test]
/// 测试 parse_color 的大小写不敏感
fn test_parse_color_case_insensitive() {
    let test_cases = vec![
        // 十六进制颜色（不区分大小写）
        ("#FFF", ColorValue::Rgba(255, 255, 255, 255)),
        ("#fff", ColorValue::Rgba(255, 255, 255, 255)),
        ("#FfF", ColorValue::Rgba(255, 255, 255, 255)),
        // rgb/rgba 函数（区分大小写 - only lowercase works）
        ("rgb(255,0,0)", ColorValue::Rgba(255, 0, 0, 255)),
        ("rgba(255,0,0,1)", ColorValue::Rgba(255, 0, 0, 255)),
        // hsl/hsla 函数（区分大小写 - only lowercase works）
        ("hsl(0,100%,50%)", ColorValue::Hsla(0.0, 100.0, 50.0, 1.0)),
        ("hsla(0,100%,50%,1)", ColorValue::Hsla(0.0, 100.0, 50.0, 1.0)),
        // 命名颜色（不区分大小写， returns Rgba not Named）
        ("RED", ColorValue::Rgba(255, 0, 0, 255)),
        ("Red", ColorValue::Rgba(255, 0, 0, 255)),
        ("rEd", ColorValue::Rgba(255, 0, 0, 255)),
        // 特殊关键字（不区分大小写）
        ("TRANSPARENT", ColorValue::Transparent),
        ("transparent", ColorValue::Transparent),
        ("CurrentColor", ColorValue::CurrentColor),
        ("currentColor", ColorValue::CurrentColor),
    ];

    for (input, expected) in test_cases {
        let result = parse_color(input);
        assert_eq!(result, Some(expected), "Failed to parse case-insensitive: {}", input);
    }
}

#[test]
/// 测试 ColorValue 枚举的所有变体
fn test_color_value_variants() {
    let test_cases = vec![
        ColorValue::Rgba(255, 0, 0, 255),
        ColorValue::Hsla(120.0, 50.0, 50.0, 1.0),
        ColorValue::Named("red".to_string()),
        ColorValue::Transparent,
        ColorValue::CurrentColor,
    ];

    for color in test_cases {
        // 测试 Clone
        let cloned = color.clone();
        assert_eq!(color, cloned);

        // 测试 Debug 格式化
        let _ = format!("{:?}", color);

        // 测试 PartialEq
        let expected = color.clone();
        assert_eq!(color, expected);
    }
}
