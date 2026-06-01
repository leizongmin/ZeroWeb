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
