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
