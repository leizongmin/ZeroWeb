// tests_3 溢出测试（从 tests_3.rs 自动拆分）
use super::*;
use crate::values::*;
use crate::ast::*;
use crate::tokenizer::{Token, Tokenizer, Spanned};
use crate::parser::Parser;


#[test]
/// 测试 tokenizer 处理各种字符串引号
fn test_tokenizer_string_quotes() {
    let test_cases = vec![
        r#""hello""#,
        r#"'hello'"#,
        r#""hello 'world'""#,
        r#"''hello''"#,
        r#""""#,             // 空字符串
        r#"''"#,             // 空字符串
        r#""unclosed"#,      // 未闭合的字符串
        r#""hello\nworld""#, // 包含转义字符
        r#""hello\tworld""#,
        r#""hello\""world""#, // 包含引号
    ];

    for css in test_cases {
        let tokenizer = crate::Tokenizer::new(css);
        let tokens: Vec<_> = tokenizer.collect_tokens();
        // 确保字符串被正确解析
        let has_string = tokens.iter().any(|t| matches!(t, Token::String(_)));
        if !css.contains("unclosed") {
            assert!(has_string, "Should parse string: {}", css);
        }
    }
}

#[test]
/// 测试 tokenizer 处理注释中的特殊字符
fn test_tokenizer_comments_with_special_chars() {
    let test_cases = vec![
        "/* comment */",
        "/* multi\nline\ncomment */",
        "/* comment with /* nested */ */",
        "/* comment with symbols: @ # $ % ^ & * () */",
        "/* comment with unicode: © ® ™ */",
        "/**/",                // 空注释
        "/* unclosed comment", // 未闭合的注释
    ];

    for css in test_cases {
        let tokenizer = crate::Tokenizer::new(css);
        let tokens: Vec<_> = tokenizer.collect_tokens();
        // 确保注释被正确处理
        let _ = tokens; // 关键是不 panic
    }
}

#[test]
/// 测试 tokenizer 处理 URL 函数的各种情况
fn test_tokenizer_url_function() {
    let test_cases = vec![
        "url('image.png')",
        "url(\"image.png\")",
        "url(image.png)",
        "url('path with spaces/image.png')",
        "url('http://example.com')",
        "url(data:image/png;base64,...)",
        "url()",                  // 空 URL
        "url(   'image.png'   )", // 带空格
    ];

    for css in test_cases {
        let tokenizer = crate::Tokenizer::new(css);
        let tokens: Vec<_> = tokenizer.collect_tokens();
        // 确保函数被正确解析
        assert!(!tokens.is_empty());
    }
}

#[test]
/// 测试 tokenizer 处理冒号和分号的边界情况
fn test_tokenizer_colon_semicolon() {
    let test_cases = vec![
        ":",
        "::",
        ":::::",
        ";",
        ";;",
        ";:",
        ":;",
        "a:b;c",
        "a::pseudo",
        "a : b ; c",
    ];

    for css in test_cases {
        let tokenizer = crate::Tokenizer::new(css);
        let tokens: Vec<_> = tokenizer.collect_tokens();
        // 确保正确分割
        let _ = tokens; // 关键是不 panic
    }
}

#[test]
/// 测试 tokenizer 处理方括号的各种情况
fn test_tokenizer_brackets() {
    let test_cases = vec![
        "[]",
        "[attr]",
        "[attr=value]",
        "[attr~=value]",
        "[attr|=value]",
        "[attr^=value]",
        "[attr$=value]",
        "[attr*=value]",
        "[||]", // 列选择器
        "[",    // 开方括号
        "]",    // 闭方括号
        "[attr",
        "attr]",
    ];

    for css in test_cases {
        let tokenizer = crate::Tokenizer::new(css);
        let tokens: Vec<_> = tokenizer.collect_tokens();
        // 确保正确处理方括号
        let _ = tokens; // 关键是不 panic
    }
}

#[test]
/// 测试 tokenizer 处理各种边界输入
fn test_tokenizer_boundary_inputs() {
    let test_cases = vec![
        "",           // 空字符串
        " ",          // 单个空格
        "\n",         // 单个换行
        "\t",         // 制表符
        "\r\n",       // CRLF
        "\u{0000}",   // null 字符
        "\u{0007}",   // bell 字符
        "\u{0008}",   // backspace
        "\u{000B}",   // vertical tab
        "\u{000C}",   // form feed
        "\u{001B}",   // escape
        "a\u{0000}b", // 包含 null 字符
        "a\nb\r\nc",  // 混合换行符
    ];

    for css in test_cases {
        let tokenizer = crate::Tokenizer::new(css);
        let tokens: Vec<_> = tokenizer.collect_tokens();
        // 确保不 panic
        let _ = tokens;
    }
}

#[test]
/// 测试 tokenizer 的 line_column_from_offset 函数
fn test_line_column_from_offset() {
    let source = "div\nspan";
    let test_cases = vec![
        (0, (1, 1)), // d
        (3, (1, 4)), // v
        (4, (2, 1)), // s
    ];

    for (offset, expected) in test_cases {
        let result = crate::tokenizer::line_column_from_offset(source, offset);
        assert_eq!(
            result, expected,
            "offset {}: expected {:?}, got {:?}",
            offset, expected, result
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 35. Media query 边界测试（覆盖 media_query.rs 的 uncovered 路径）
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 parse_media_query 输入为空字符串或空白字符
fn test_parse_empty_media_query() {
    let test_cases = vec!["", " ", "   ", "\n", "\t", "\r\n", "  \n  \t  \r\n  "];

    for input in test_cases {
        let result = crate::media_query::parse_media_query(input);
        assert_eq!(result, None, "Empty input should return None: {:?}", input);
    }
}

#[test]
/// 测试 parse_media_query 只有媒体类型
fn test_parse_media_query_type_only() {
    let test_cases = vec![
        ("screen", crate::media_query::MediaType::Screen),
        ("print", crate::media_query::MediaType::Print),
        ("all", crate::media_query::MediaType::All),
    ];

    for (input, expected_type) in test_cases {
        let result = crate::media_query::parse_media_query(input);
        assert!(result.is_some(), "Should parse type: {:?}", input);
        let queries = result.unwrap();
        assert_eq!(queries.len(), 1);
        let q = &queries[0];
        assert!(q.conditions.is_empty());
        assert_eq!(q.media_type, Some(expected_type));
    }
}

#[test]
/// 测试 parse_media_query 带有 not 和 only 前缀
fn test_parse_media_query_not_only_prefix() {
    let test_cases = vec![
        "not screen",
        "not print",
        "not all",
        "only screen",
        "only print",
        "only all",
        "not screen and (max-width: 600px)",
        "only print and (orientation: portrait)",
    ];

    for input in test_cases {
        let result = crate::media_query::parse_media_query(input);
        assert!(result.is_some(), "Should parse with prefix: {:?}", input);
        let queries = result.unwrap();
        assert!(!queries.is_empty());
    }
}

#[test]
/// 测试 parse_media_query 布尔特性
fn test_parse_media_query_boolean_features() {
    let test_cases = vec![
        "(hover)",
        "(hover) and (color)",
        "(color)",
        "(hover) or (color)",
        "not (hover)",
        "not (color)",
        "screen and (hover)",
        "print and (color)",
    ];

    for input in test_cases {
        let result = crate::media_query::parse_media_query(input);
        assert!(result.is_some(), "Should parse boolean feature: {:?}", input);
        let queries = result.unwrap();
        assert!(!queries.is_empty());
    }
}

#[test]
/// 测试 parse_media_query range 语法（Level 4）
fn test_parse_media_query_range_syntax_level4() {
    let test_cases = vec![
        // 宽度范围
        "(width > 600px)",
        "(width >= 600px)",
        "(width < 1000px)",
        "(width <= 1000px)",
        // 高度范围
        "(height > 400px)",
        "(height >= 400px)",
        "(height < 800px)",
        "(height <= 800px)",
        // 组合范围
        "(600px <= width <= 1000px)",
        "(400px <= height <= 800px)",
        "(800px < width < 1200px)",
        // 带空格
        "(  width  >  600px  )",
        "( 600px  <=  width  <=  1000px  )",
    ];

    for input in test_cases {
        let result = crate::media_query::parse_media_query(input);
        assert!(result.is_some(), "Should parse range syntax: {:?}", input);
        let queries = result.unwrap();
        assert!(!queries.is_empty());
    }
}

#[test]
/// 测试 parse_media_query 无效的语法
fn test_parse_media_query_invalid_syntax() {
    let test_cases = vec![
        "(",                           // 不闭合的括号
        ")",                           // 只有闭合括号
        "screen and",                  // 不完整的条件
        "and (max-width: 600px)",      // 缺少第一个条件
        "(max-width: 600px) and",      // 缺少第二个条件
        "screen (max-width: 600px)",   // 缺少 and
        "(max-width 600px)",           // 缺少冒号
        "screen and max-width: 600px", // 缺少括号
        "not and (max-width: 600px)",  // 无效的 not 用法
    ];

    for input in test_cases {
        let result = crate::media_query::parse_media_query(input);
        // 无效语法可能返回 None，但不应该 panic
        let _ = result;
    }
}

#[test]
/// 测试 MediaContext 的构造函数
fn test_media_context_constructors() {
    // 测试默认构造函数
    let ctx1 = crate::media_query::MediaContext::new(800.0, 600.0);
    assert_eq!(ctx1.viewport_width, 800.0);
    assert_eq!(ctx1.viewport_height, 600.0);
    assert_eq!(ctx1.media_type, crate::media_query::MediaType::Screen);

    // 测试带类型的构造函数
    let ctx2 = crate::media_query::MediaContext::with_type(1024.0, 768.0, crate::media_query::MediaType::Print);
    assert_eq!(ctx2.viewport_width, 1024.0);
    assert_eq!(ctx2.viewport_height, 768.0);
    assert_eq!(ctx2.media_type, crate::media_query::MediaType::Print);

    // 测试默认值
    assert_eq!(
        ctx1.prefers_color_scheme,
        crate::media_query::PrefersColorSchemeValue::Light
    );
    assert_eq!(
        ctx1.prefers_reduced_motion,
        crate::media_query::ReducedMotionValue::NoPreference
    );
    assert_eq!(ctx1.pointer_type, crate::media_query::PointerValue::Coarse);
    assert_eq!(ctx1.resolution_dpi, 96.0);
}

#[test]
/// 测试 MediaType 的相等性比较
fn test_media_type_equality() {
    let test_cases = vec![
        (
            crate::media_query::MediaType::Screen,
            crate::media_query::MediaType::Screen,
            true,
        ),
        (
            crate::media_query::MediaType::Print,
            crate::media_query::MediaType::Print,
            true,
        ),
        (
            crate::media_query::MediaType::All,
            crate::media_query::MediaType::All,
            true,
        ),
        (
            crate::media_query::MediaType::Screen,
            crate::media_query::MediaType::Print,
            false,
        ),
        (
            crate::media_query::MediaType::Print,
            crate::media_query::MediaType::All,
            false,
        ),
        (
            crate::media_query::MediaType::All,
            crate::media_query::MediaType::Screen,
            false,
        ),
    ];

    for (val1, val2, expected_equal) in test_cases {
        assert_eq!(
            val1 == val2,
            expected_equal,
            "{:?} == {:?} should be {}",
            val1,
            val2,
            expected_equal
        );
    }
}

#[test]
/// 测试 OrientationValue 的相等性比较
fn test_orientation_value_equality() {
    let test_cases = vec![
        (
            crate::media_query::OrientationValue::Portrait,
            crate::media_query::OrientationValue::Portrait,
            true,
        ),
        (
            crate::media_query::OrientationValue::Landscape,
            crate::media_query::OrientationValue::Landscape,
            true,
        ),
        (
            crate::media_query::OrientationValue::Portrait,
            crate::media_query::OrientationValue::Landscape,
            false,
        ),
        (
            crate::media_query::OrientationValue::Landscape,
            crate::media_query::OrientationValue::Portrait,
            false,
        ),
    ];

    for (val1, val2, expected_equal) in test_cases {
        assert_eq!(
            val1 == val2,
            expected_equal,
            "{:?} == {:?} should be {}",
            val1,
            val2,
            expected_equal
        );
    }
}

#[test]
/// 测试 MediaContext 的 Clone
fn test_media_context_clone() {
    let original = crate::media_query::MediaContext::with_type(1920.0, 1080.0, crate::media_query::MediaType::Screen);
    let cloned = original.clone();

    // 验证克隆后的值是否相同
    assert_eq!(original.viewport_width, cloned.viewport_width);
    assert_eq!(original.viewport_height, cloned.viewport_height);
    assert_eq!(original.media_type, cloned.media_type);
    assert_eq!(original.prefers_color_scheme, cloned.prefers_color_scheme);
    assert_eq!(original.prefers_reduced_motion, cloned.prefers_reduced_motion);
    assert_eq!(original.pointer_type, cloned.pointer_type);
    assert_eq!(original.resolution_dpi, cloned.resolution_dpi);
}
