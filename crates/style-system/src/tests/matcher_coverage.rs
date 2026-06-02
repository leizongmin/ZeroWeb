// ═══════════════════════════════════════════════════════════════════
// matcher 覆盖率测试
// ═══════════════════════════════════════════════════════════════════

use super::super::*;
use super::helpers::*;

#[test]
/// div 选择器匹配并应用样式
fn test_tag_selector_matches_and_applies() {
    let (doc, _html, _body, div, _p) = make_test_dom();

    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "color".to_string(),
                value: "red".to_string(),
                important: false,
            }],
        })],
    }];

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");

    // div 选择器匹配，color: red 应用
    assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255));
}

#[test]
/// @container 条件使用无效长度值应返回 false
fn test_container_condition_invalid_length() {
    let (doc, _html, _body, div, _p) = make_test_dom();

    let mut sys = StyleSystem::new();
    sys.set_viewport(400.0, 300.0);

    // @container 使用无效长度值
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Container(zero_css_parser::ast::ContainerRule {
            name: None,
            condition: zero_css_parser::ast::ContainerCondition::Size(zero_css_parser::ast::ContainerSizeCondition {
                feature: "width".to_string(),
                value: "invalid-length".to_string(), // 无效长度
                operator: None,
                range_min: None,
                range_max: None,
            }),
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            })],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");

    // 无效长度应导致条件评估失败，不应用样式
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 0, 255));
}

#[test]
/// @import 规则在 collect_from_rules 中被跳过
fn test_import_rules_skipped() {
    let (doc, _html, _body, div, _p) = make_test_dom();

    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![
            Rule::Import(zero_css_parser::ast::ImportRule {
                url: "url('styles.css')".to_string(),
                media_queries: vec![],
            }),
            Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            }),
        ],
    }];

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");

    // @import 规则被跳过，只有 div 规则被应用
    assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255));
}
