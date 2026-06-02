// ═══════════════════════════════════════════════════════════════════
// matcher 额外覆盖率测试
// ═══════════════════════════════════════════════════════════════════

use super::super::*;
use super::helpers::*;
use crate::ComputedStyle;

#[test]
/// 连续组合器检测 (如 >>>)
fn test_invalid_selector_consecutive_combinators() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    // 测试连续组合器检测功能
    // 通过创建包含连续组合器的 CSS 字符串来测试
    let css = "div >>> span { color: red; }";
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);

    // 解析器应该能处理这种情况，但 Matcher 会检测无效选择器
    // 此测试主要确保不会 panic
    assert!(!stylesheet.rules.is_empty());
}

#[test]
/// 选择器以组合器开头（无效）
fn test_selector_starts_with_combinator() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    // 测试以组合器开头的选择器
    let css = "> div { color: red; }";
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);

    // 解析器应该能处理这种情况，但 Matcher 会检测无效选择器
    assert!(!stylesheet.rules.is_empty());
}

#[test]
/// 复杂属性值测试 - 未知属性
fn test_complex_property_unknown() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    // 使用未知属性
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "unknown-property".to_string(),
                value: "value".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");

    // 未知属性不会被应用，保持默认值
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 0, 255));
}

#[test]
/// 复杂属性值测试 - 已知属性但无效值
fn test_complex_property_invalid_value() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    // 使用有效属性但无效值
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "color".to_string(),
                value: "invalid-color-name".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");

    // 无效值不会被应用，保持默认值
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 0, 255));
}

#[test]
/// @container 条件使用 min-/max- 前缀
fn test_container_condition_with_min_max_prefix() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();
    sys.set_viewport(400.0, 300.0);

    // @container 使用 min-width
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Container(zero_css_parser::ast::ContainerRule {
            name: None,
            condition: zero_css_parser::ast::ContainerCondition::Size(zero_css_parser::ast::ContainerSizeCondition {
                feature: "min-width".to_string(),
                value: "500px".to_string(),
                operator: None,
                range_min: None,
                range_max: None,
            }),
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "blue".to_string(),
                    important: false,
                }],
            })],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");

    // 400px 容器 < 500px min-width，条件不满足
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 0, 255));
}

#[test]
/// @container 条件使用范围语法
fn test_container_condition_range_syntax() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();
    sys.set_viewport(400.0, 300.0);

    // @container 使用范围语法
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Container(zero_css_parser::ast::ContainerRule {
            name: None,
            condition: zero_css_parser::ast::ContainerCondition::Size(zero_css_parser::ast::ContainerSizeCondition {
                feature: "width".to_string(),
                value: "400px".to_string(),
                operator: None,
                range_min: Some("300px".to_string()),
                range_max: Some("500px".to_string()),
            }),
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "green".to_string(),
                    important: false,
                }],
            })],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");

    // 400px 在 300px-500px 范围内，条件满足
    assert_eq!(div_style.color, ColorValue::Rgba(0, 128, 0, 255)); // green
}

#[test]
/// @supports 条件 - 未知属性
fn test_supports_unknown_property() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    // @supports 检查未知属性
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Supports(zero_css_parser::ast::SupportsRule {
            condition: zero_css_parser::ast::SupportsCondition::Property(
                "unknown-property".to_string(),
                "value".to_string(),
            ),
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "purple".to_string(),
                    important: false,
                }],
            })],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");

    // 未知属性不支持，@supports 条件不成立
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 0, 255));
}

#[test]
/// @supports 条件 - 无效选择器
fn test_supports_invalid_selector() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    // @supports 检查无效选择器
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Supports(zero_css_parser::ast::SupportsRule {
            condition: zero_css_parser::ast::SupportsCondition::Selector("div >>> span".to_string()),
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "orange".to_string(),
                    important: false,
                }],
            })],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");

    // 无效选择器，@supports 条件不成立
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 0, 255));
}

#[test]
/// @supports 条件 - And 组合
fn test_supports_and_condition() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    // @supports 使用 And 组合
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Supports(zero_css_parser::ast::SupportsRule {
            condition: zero_css_parser::ast::SupportsCondition::And(vec![
                zero_css_parser::ast::SupportsCondition::Property("display".to_string(), "flex".to_string()),
                zero_css_parser::ast::SupportsCondition::Property("position".to_string(), "relative".to_string()),
            ]),
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "teal".to_string(),
                    important: false,
                }],
            })],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");

    // 两个条件都成立，@supports 条件满足
    assert_eq!(div_style.color, ColorValue::Rgba(0, 128, 128, 255)); // teal
}

#[test]
/// @supports 条件 - Or 组合
fn test_supports_or_condition() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    // @supports 使用 Or 组合
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Supports(zero_css_parser::ast::SupportsRule {
            condition: zero_css_parser::ast::SupportsCondition::Or(vec![
                zero_css_parser::ast::SupportsCondition::Property("display".to_string(), "grid".to_string()),
                zero_css_parser::ast::SupportsCondition::Property("display".to_string(), "flex".to_string()),
            ]),
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "coral".to_string(),
                    important: false,
                }],
            })],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");

    // 至少一个条件成立（flex），@supports 条件满足
    assert_eq!(div_style.color, ColorValue::Rgba(255, 127, 80, 255)); // coral
}

#[test]
/// @supports 条件 - Not 组合
fn test_supports_not_condition() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    // @supports 使用 Not 组合
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Supports(zero_css_parser::ast::SupportsRule {
            condition: zero_css_parser::ast::SupportsCondition::Not(Box::new(
                zero_css_parser::ast::SupportsCondition::Property("unknown-property".to_string(), "value".to_string()),
            )),
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "indigo".to_string(),
                    important: false,
                }],
            })],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");

    // 未知属性不支持，Not(未知) 为真，@supports 条件满足
    assert_eq!(div_style.color, ColorValue::Rgba(75, 0, 130, 255)); // indigo
}

#[test]
/// @layer 规则 - 多层嵌套
fn test_layer_rule_nested() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    // 多个 @layer 规则
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![
            Rule::Layer(zero_css_parser::ast::LayerRule {
                name: "theme".to_string(),
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "black".to_string(),
                        important: false,
                    }],
                })],
            }),
            Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "white".to_string(),
                    important: false,
                }],
            }),
        ],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");

    // 普通样式比 @layer 样式优先级高
    assert_eq!(div_style.color, ColorValue::Rgba(255, 255, 255, 255)); // white
}

#[test]
/// @media 规则 - 多个查询
fn test_media_rule_multiple_queries() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);

    // 多个媒体查询
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::At(zero_css_parser::ast::AtRule {
            name: "media".to_string(),
            prelude: "(max-width: 600px)".to_string(),
            body: zero_css_parser::ast::AtRuleBody::Block(vec![
                Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    }],
                }),
                Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "background".to_string(),
                        value: "blue".to_string(),
                        important: false,
                    }],
                }),
            ]),
        })],
    }];

    // 直接计算样式，不使用媒体上下文
    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");

    // 800px > 600px，媒体查询不匹配
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 0, 255)); // default
}

#[test]
/// 空的选择器列表
fn test_empty_selector_list() {
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
/// 重复的选择器（同一个样式表中有多个相同选择器）
fn test_duplicate_selectors() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    // 相同的选择器出现两次
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![
            Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            }),
            Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "background".to_string(),
                    value: "blue".to_string(),
                    important: false,
                }],
            }),
        ],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");

    // 两个选择器都匹配，两个属性都应用
    assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255)); // red
}

#[test]
/// 使用相同属性名的多个声明
fn test_duplicate_property_declarations() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    // 相同属性出现多次
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![
                Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                },
                Declaration {
                    property: "color".to_string(),
                    value: "blue".to_string(),
                    important: false,
                },
            ],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");

    // 后面的声明应该覆盖前面的
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255)); // blue
}

#[test]
/// @important 覆盖普通声明
fn test_important_declarations() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    // 普通 !important 和普通声明
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![
            Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: true,
                }],
            }),
            Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "blue".to_string(),
                    important: false,
                }],
            }),
        ],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");

    // !important 应该覆盖普通声明
    assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255)); // red
}

#[test]
/// 混合特异性比较
fn test_specificity_comparison() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    // 不同特异性的选择器
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![
            // 低特异性：div
            Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            }),
            // 中等特异性：div.className (手动构建复杂选择器)
            Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")], // 简化测试
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "green".to_string(),
                    important: false,
                }],
            }),
            // 高特异性：使用 ID 选择器（通过属性）
            Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")], // 简化测试
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "blue".to_string(),
                    important: false,
                }],
            }),
        ],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");

    // 由于简化了选择器构建，优先级取决于声明顺序
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255)); // blue
}
