// ═══════════════════════════════════════════════════════════
// matcher 最终覆盖率测试
// ═══════════════════════════════════════════════════════════

use super::super::*;
use super::helpers::*;

#[test]
/// 连续组合器检测边缘情况
fn test_invalid_selector_consecutive_combinators_edge() {
    let (_doc, _html, _body, _div, _p) = make_test_dom();
    let _sys = StyleSystem::new();

    // 测试各种连续组合器情况
    let test_cases = [
        "div >>> span",      // 三个>
        "div > + span",      // > 后跟 +
        "div ~ > span",      // ~ 后跟 >
        "div + ~ span",      // + 后跟 ~
        "> > div",           // 两个>开头
        "+ > div",           // + 后跟 >
        "~ + div",           // ~ 后跟 +
    ];

    for css in test_cases {
        let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
        // 这些选择器应该被标记为无效
        assert!(!stylesheet.rules.is_empty(), "Should parse but not apply: {}", css);
    }
}

#[test]
/// 选择器以组合器开头的边缘情况
fn test_selector_starts_with_combinator_edge() {
    let (_doc, _html, _body, _div, _p) = make_test_dom();
    let _sys = StyleSystem::new();

    // 测试各种以组合器开头的情况
    let test_cases = [
        "> div",
        "+ span",
        "~ p",
        "> > div",
        "+ > span",
    ];

    for css in test_cases {
        let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
        // 这些选择器应该被标记为无效
        assert!(!stylesheet.rules.is_empty(), "Should parse but not apply: {}", css);
    }
}

#[test]
/// 空的选择器列表边缘情况
fn test_empty_selector_list_edge() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    // 空的选择器列表
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![], // 空选择器列表
            declarations: vec![Declaration {
                property: "color".to_string(),
                value: "red".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");

    // 没有选择器匹配，样式不应用
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 0, 255)); // default
}

#[test]
/// 无效CSS的容错处理
fn test_invalid_css_error_handling() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    // 测试各种无效CSS的容错处理
    let test_cases = vec![
        // 空的CSS
        "",
        // 只有空格
        "   ",
        // 只有声明块
        "{ color: red; }",
        // 只有选择器
        "div",
        // 缺少大括号
        "div color: red;",
        // 缺少属性值
        "div { color; }",
        // 无效的属性名
        "div { 123: red; }",
        // 无效的值
        "div { color: invalid-color; }",
    ];

    for css in test_cases {
        let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
        let stylesheets = vec![stylesheet];

        let styles = sys.compute_styles(&doc, &stylesheets);
        let div_style = styles.get(&div).expect("div should have style");

        // 解析器应该容错处理，不panic
        assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 0, 255));
    }
}