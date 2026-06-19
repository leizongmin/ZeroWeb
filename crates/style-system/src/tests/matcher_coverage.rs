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
/// R359 诊断：border-bottom-width longhand 在完整级联（选择器匹配 + 级联 + apply）中是否应用。
/// 直接 apply_property_value 已证 longhand 工作（coverage_round3）；此测试验证完整级联。
fn test_border_bottom_width_longhand_full_cascade() {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let test_div = doc.create_element("div");
    doc.set_attribute(test_div, "id", "test");
    doc.append_child(body, test_div).unwrap();

    let id_selector = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Id("test".to_string())],
                },
                None,
            )],
        },
    };
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![id_selector],
            declarations: vec![
                Declaration {
                    property: "border-bottom-style".to_string(),
                    value: "solid".to_string(),
                    important: false,
                },
                Declaration {
                    property: "border-bottom-width".to_string(),
                    value: "96px".to_string(),
                    important: false,
                },
            ],
        })],
    }];

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &stylesheets);
    let s = styles.get(&test_div).expect("#test should have computed style");
    assert_eq!(
        s.border_bottom_width,
        LengthValue::Px(96.0),
        "border-bottom-width:96px longhand must apply via full cascade"
    );
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
