// Auto-generated test file — split from style-system/lib.rs
use super::super::*;

use zero_css_parser::ast::{
    ComplexSelector, CompoundSelector, Declaration, Rule, Selector, StyleRule, SubclassSelector, TypeSelector,
};
use zero_css_parser::values::{ColorValue, DisplayValue, LengthValue, OverflowValue};
use zero_dom::{Document, NodeId};

/// 创建测试 DOM：html > body > div#main > p.text
fn make_test_dom() -> (Document, NodeId, NodeId, NodeId, NodeId) {
    let mut doc = Document::new();
    let root = doc.root();

    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();

    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();

    let div = doc.create_element("div");
    doc.set_attribute(div, "id", "main");
    doc.append_child(body, div).unwrap();

    let p = doc.create_element("p");
    doc.set_attribute(p, "class", "text");
    doc.append_child(div, p).unwrap();

    (doc, html, body, div, p)
}

fn make_tag_selector(tag: &str) -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag(tag.to_string())),
                    subclass_selectors: vec![],
                },
                None,
            )],
        },
    }
}

#[test]
fn test_style_system_new() {
    let sys = StyleSystem::new();
    assert!(sys.custom_properties.is_empty());
}

#[test]
fn test_compute_styles_empty() {
    let (doc, _html, _body, _div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();
    let stylesheets = vec![];
    let styles = sys.compute_styles(&doc, &stylesheets);
    // 应该有 html, body, div, p 四个元素的样式
    assert!(styles.len() >= 4);
}

#[test]
fn test_compute_styles_with_rules() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "color".to_string(),
                value: "red".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div 应该有样式");
    assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255));
}

#[test]
fn test_compute_styles_inheritance() {
    let (doc, _html, _body, _div, p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "color".to_string(),
                value: "blue".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let p_style = styles.get(&p).expect("p 应该有样式");
    // p 应该继承 div 的 color
    assert_eq!(p_style.color, ColorValue::Rgba(0, 0, 255, 255));
}

#[test]
fn test_compute_element_style() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "display".to_string(),
                value: "flex".to_string(),
                important: false,
            }],
        })],
    }];

    let style = sys.compute_element_style(&doc, div, &stylesheets, None);
    assert_eq!(style.display, DisplayValue::Flex);
}

#[test]
fn test_compute_styles_with_class_selector() {
    let (doc, _html, _body, _div, p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let class_sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Class("text".to_string())],
                },
                None,
            )],
        },
    };

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![class_sel],
            declarations: vec![Declaration {
                property: "font-size".to_string(),
                value: "20px".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let p_style = styles.get(&p).expect("p 应该有样式");
    assert_eq!(p_style.font_size, LengthValue::Px(20.0));
}

#[test]
fn test_compute_styles_specificity() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let tag_sel = make_tag_selector("div");
    let id_sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Id("main".to_string())],
                },
                None,
            )],
        },
    };

    // tag 选择器设置 color: red
    // id 选择器设置 color: blue
    // id 选择器特异性更高，应该胜出
    let stylesheets = vec![Stylesheet {
        rules: vec![
            Rule::Style(StyleRule {
                selectors: vec![tag_sel],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            }),
            Rule::Style(StyleRule {
                selectors: vec![id_sel],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "blue".to_string(),
                    important: false,
                }],
            }),
        ],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div 应该有样式");
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255));
}

#[test]
fn test_set_viewport() {
    let mut sys = StyleSystem::new();
    sys.set_viewport(1920.0, 1080.0);
    assert_eq!(sys.viewport_width, Some(1920.0));
    assert_eq!(sys.viewport_height, Some(1080.0));
}

#[test]
fn test_default_style_system() {
    let sys = StyleSystem::default();
    assert!(sys.custom_properties.is_empty());
}

#[test]
fn test_ua_default_margins_expand_shorthand() {
    let (doc, _html, body, _div, p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let styles = sys.compute_styles(&doc, &[]);
    let body_style = styles.get(&body).expect("body should have UA style");
    let p_style = styles.get(&p).expect("p should have UA style");

    assert_eq!(body_style.margin_top, LengthValue::Px(8.0));
    assert_eq!(body_style.margin_right, LengthValue::Px(8.0));
    assert_eq!(body_style.margin_bottom, LengthValue::Px(8.0));
    assert_eq!(body_style.margin_left, LengthValue::Px(8.0));

    assert_eq!(p_style.margin_top, LengthValue::Px(16.0));
    assert_eq!(p_style.margin_right, LengthValue::Px(0.0));
    assert_eq!(p_style.margin_bottom, LengthValue::Px(16.0));
    assert_eq!(p_style.margin_left, LengthValue::Px(0.0));
}

// ── 简写属性端到端测试 ──

#[test]
fn test_shorthand_margin_in_style_computation() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "margin".to_string(),
                value: "10px 20px".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.margin_top, LengthValue::Px(10.0));
    assert_eq!(div_style.margin_right, LengthValue::Px(20.0));
    assert_eq!(div_style.margin_bottom, LengthValue::Px(10.0));
    assert_eq!(div_style.margin_left, LengthValue::Px(20.0));
}

#[test]
fn test_shorthand_padding_in_style_computation() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "padding".to_string(),
                value: "5px".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.padding_top, LengthValue::Px(5.0));
    assert_eq!(div_style.padding_right, LengthValue::Px(5.0));
    assert_eq!(div_style.padding_bottom, LengthValue::Px(5.0));
    assert_eq!(div_style.padding_left, LengthValue::Px(5.0));
}

#[test]
fn test_shorthand_border_in_style_computation() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "border".to_string(),
                value: "1px solid red".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.border_top_width, LengthValue::Px(1.0));
    assert_eq!(div_style.border_right_width, LengthValue::Px(1.0));
    assert_eq!(div_style.border_bottom_width, LengthValue::Px(1.0));
    assert_eq!(div_style.border_left_width, LengthValue::Px(1.0));
    assert_eq!(div_style.border_top_style, property::BorderStyleValue::Solid);
    assert_eq!(div_style.border_top_color, ColorValue::Rgba(255, 0, 0, 255));
}

#[test]
/// R2356: longhand 关键字大小写不敏感（CSS Syntax §：所有关键字大小写不敏感）。
/// 覆盖 parse.rs 多个此前大小写敏感的 match 解析器：border-style/text-align/white-space/cursor。
fn test_longhand_keyword_case_insensitive() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let declarations = vec![
        Declaration {
            property: "border-top-style".to_string(),
            value: "SOLID".to_string(),
            important: false,
        },
        Declaration {
            property: "text-align".to_string(),
            value: "CENTER".to_string(),
            important: false,
        },
        Declaration {
            property: "white-space".to_string(),
            value: "NOWRAP".to_string(),
            important: false,
        },
        Declaration {
            property: "cursor".to_string(),
            value: "POINTER".to_string(),
            important: false,
        },
        Declaration {
            property: "transform-style".to_string(),
            value: "PRESERVE-3D".to_string(),
            important: false,
        },
        Declaration {
            property: "backface-visibility".to_string(),
            value: "HIDDEN".to_string(),
            important: false,
        },
    ];
    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations,
        })],
    }];
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(
        div_style.border_top_style,
        property::BorderStyleValue::Solid,
        "border-top-style: SOLID 应识别为 Solid"
    );
    assert_eq!(
        div_style.text_align,
        property::TextAlignValue::Center,
        "text-align: CENTER"
    );
    assert_eq!(
        div_style.white_space,
        property::WhiteSpaceValue::Nowrap,
        "white-space: NOWRAP"
    );
    assert_eq!(div_style.cursor, property::CursorValue::Pointer, "cursor: POINTER");
    assert_eq!(
        div_style.transform_style,
        property::TransformStyleValue::Preserve3d,
        "transform-style: PRESERVE-3D"
    );
    assert_eq!(
        div_style.backface_visibility,
        property::BackfaceVisibilityValue::Hidden,
        "backface-visibility: HIDDEN"
    );
}

#[test]
fn test_shorthand_overflow_in_style_computation() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "overflow".to_string(),
                value: "hidden".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.overflow_x, OverflowValue::Hidden);
    assert_eq!(div_style.overflow_y, OverflowValue::Hidden);
}

#[test]
fn test_shorthand_border_radius_in_style_computation() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "border-radius".to_string(),
                value: "5px 10px".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.border_top_left_radius, LengthValue::Px(5.0));
    assert_eq!(div_style.border_top_right_radius, LengthValue::Px(10.0));
    assert_eq!(div_style.border_bottom_right_radius, LengthValue::Px(5.0));
    assert_eq!(div_style.border_bottom_left_radius, LengthValue::Px(10.0));
}

#[test]
fn test_shorthand_margin_with_longhand_override() {
    // margin 简写设置后，后面的 longhand 应该覆盖
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![
                Declaration {
                    property: "margin".to_string(),
                    value: "10px".to_string(),
                    important: false,
                },
                Declaration {
                    property: "margin-top".to_string(),
                    value: "20px".to_string(),
                    important: false,
                },
            ],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    // margin-top 被 longhand 覆盖为 20px
    assert_eq!(div_style.margin_top, LengthValue::Px(20.0));
    // 其他边保持 10px
    assert_eq!(div_style.margin_right, LengthValue::Px(10.0));
}

// ── 媒体查询端到端测试 ──

#[test]
fn test_media_query_applies_when_condition_matches() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();
    sys.set_viewport(1024.0, 768.0); // 宽屏

    // @media (min-width: 600px) { div { color: red; } }
    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::At(zero_css_parser::ast::AtRule {
            name: "media".to_string(),
            prelude: "(min-width: 600px)".to_string(),
            body: zero_css_parser::ast::AtRuleBody::Block(vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            })]),
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255));
}

#[test]
fn test_media_query_skips_when_condition_fails() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();
    sys.set_viewport(400.0, 300.0); // 窄屏

    // @media (min-width: 600px) { div { color: red; } }
    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::At(zero_css_parser::ast::AtRule {
            name: "media".to_string(),
            prelude: "(min-width: 600px)".to_string(),
            body: zero_css_parser::ast::AtRuleBody::Block(vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            })]),
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    // 条件不满足，color 保持默认黑色
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 0, 255));
}

#[test]
fn test_media_print_not_applied_in_screen_mode() {
    // R1981：默认渲染媒体 = Screen，`@media print` 规则不应生效（CSS §7）。
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0); // 默认 Screen（不调 set_media_type）。

    // @media print { div { color: red; } }
    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::At(zero_css_parser::ast::AtRule {
            name: "media".to_string(),
            prelude: "print".to_string(),
            body: zero_css_parser::ast::AtRuleBody::Block(vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            })]),
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    // Screen 模式下 @media print 不应用，color 保持默认黑色。
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 0, 255));
}

#[test]
fn test_media_print_applied_in_print_mode() {
    // R1981：set_media_type(Print) 后，`@media print` 规则生效，`@media screen` 规则失效。
    let (doc, _html, _body, div, p) = make_test_dom();
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    sys.set_media_type(zero_css_parser::media_query::MediaType::Print);

    // @media print { div { color: red; } }  —— Print 模式应生效
    // @media screen { p { color: blue; } }  —— Print 模式应失效（screen != print）
    let stylesheets = vec![Stylesheet {
        rules: vec![
            Rule::At(zero_css_parser::ast::AtRule {
                name: "media".to_string(),
                prelude: "print".to_string(),
                body: zero_css_parser::ast::AtRuleBody::Block(vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    }],
                })]),
            }),
            Rule::At(zero_css_parser::ast::AtRule {
                name: "media".to_string(),
                prelude: "screen".to_string(),
                body: zero_css_parser::ast::AtRuleBody::Block(vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("p")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "blue".to_string(),
                        important: false,
                    }],
                })]),
            }),
        ],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    let p_style = styles.get(&p).expect("p should have style");
    // @media print 生效：div color = red。
    assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255));
    // @media screen 失效：p 未被设为 blue。p 是 div 的子元素，color 继承自 div 的 red
    // （若 @media screen 生效，p 应为 blue；此处为 red 证明 screen 规则未应用）。
    assert_eq!(p_style.color, ColorValue::Rgba(255, 0, 0, 255));
}

#[test]
fn test_media_query_with_regular_rules() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);

    // 正常规则 + @media 规则
    let stylesheets = vec![Stylesheet {
        rules: vec![
            // 基础样式
            Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "blue".to_string(),
                    important: false,
                }],
            }),
            // 响应式样式
            Rule::At(zero_css_parser::ast::AtRule {
                name: "media".to_string(),
                prelude: "(min-width: 600px)".to_string(),
                body: zero_css_parser::ast::AtRuleBody::Block(vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "margin-top".to_string(),
                        value: "20px".to_string(),
                        important: false,
                    }],
                })]),
            }),
        ],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    // 基础样式应用
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255));
    // @media 条件满足，响应式样式也应用
    assert_eq!(div_style.margin_top, LengthValue::Px(20.0));
}

#[test]
fn test_media_query_no_viewport_skips() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();
    // 不设置视口

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::At(zero_css_parser::ast::AtRule {
            name: "media".to_string(),
            prelude: "(min-width: 600px)".to_string(),
            body: zero_css_parser::ast::AtRuleBody::Block(vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            })]),
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    // 没有视口信息，@media 不应用
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 0, 255));
}

// ── @supports 端到端测试 ──

#[test]
fn test_supports_applies_when_condition_met() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Supports(zero_css_parser::ast::SupportsRule {
            condition: zero_css_parser::ast::SupportsCondition::Property("display".to_string(), "grid".to_string()),
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "display".to_string(),
                    value: "grid".to_string(),
                    important: false,
                }],
            })],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.display, DisplayValue::Grid);
}

#[test]
fn test_supports_skips_when_condition_not_met() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Supports(zero_css_parser::ast::SupportsRule {
            condition: zero_css_parser::ast::SupportsCondition::Property(
                "display".to_string(),
                "unknown-value".to_string(),
            ),
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
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 0, 255));
}

#[test]
fn test_supports_not_condition() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Supports(zero_css_parser::ast::SupportsRule {
            condition: zero_css_parser::ast::SupportsCondition::Not(Box::new(
                zero_css_parser::ast::SupportsCondition::Property("display".to_string(), "grid".to_string()),
            )),
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
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 0, 255));
}

#[test]
fn test_supports_and_condition() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Supports(zero_css_parser::ast::SupportsRule {
            condition: zero_css_parser::ast::SupportsCondition::And(vec![
                zero_css_parser::ast::SupportsCondition::Property("display".to_string(), "flex".to_string()),
                zero_css_parser::ast::SupportsCondition::Property("color".to_string(), "blue".to_string()),
            ]),
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
    assert_eq!(div_style.color, ColorValue::Rgba(0, 128, 0, 255));
}

#[test]
fn test_supports_or_condition() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Supports(zero_css_parser::ast::SupportsRule {
            condition: zero_css_parser::ast::SupportsCondition::Or(vec![
                zero_css_parser::ast::SupportsCondition::Property("display".to_string(), "unknown".to_string()),
                zero_css_parser::ast::SupportsCondition::Property("display".to_string(), "flex".to_string()),
            ]),
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
    assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255));
}

#[test]
fn test_supports_with_regular_rules() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![
            Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "blue".to_string(),
                    important: false,
                }],
            }),
            Rule::Supports(zero_css_parser::ast::SupportsRule {
                condition: zero_css_parser::ast::SupportsCondition::Property("display".to_string(), "grid".to_string()),
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "margin-top".to_string(),
                        value: "20px".to_string(),
                        important: false,
                    }],
                })],
            }),
        ],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255));
    assert_eq!(div_style.margin_top, LengthValue::Px(20.0));
}

// ── @supports selector() 端到端测试 ──

/// 测试 selector() 基本用法：有效的选择器应返回 true。
#[test]
fn test_supports_selector_basic() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    // @supports selector(div > .class) { div { color: red; } }
    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Supports(zero_css_parser::ast::SupportsRule {
            condition: zero_css_parser::ast::SupportsCondition::Selector("div > .class".to_string()),
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
    assert_eq!(
        div_style.color,
        ColorValue::Rgba(255, 0, 0, 255),
        "selector(div > .class) 应该评估为 true，颜色应为红色"
    );
}

/// 测试 selector() 复杂伪类：有效的 :is() 选择器应返回 true。
#[test]
fn test_supports_selector_complex() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    // @supports selector(:is(div, span)) { div { color: green; } }
    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Supports(zero_css_parser::ast::SupportsRule {
            condition: zero_css_parser::ast::SupportsCondition::Selector(":is(div, span)".to_string()),
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
    assert_eq!(
        div_style.color,
        ColorValue::Rgba(0, 128, 0, 255),
        "selector(:is(div, span)) 应该评估为 true，颜色应为绿色"
    );
}

/// 测试 selector() 无效选择器应返回 false。
#[test]
fn test_supports_selector_invalid() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    // @supports selector(>>>invalid) { div { color: red; } }
    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Supports(zero_css_parser::ast::SupportsRule {
            condition: zero_css_parser::ast::SupportsCondition::Selector(">>>invalid".to_string()),
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
    assert_eq!(
        div_style.color,
        ColorValue::Rgba(0, 0, 0, 255),
        "selector(>>>invalid) 应该评估为 false，不应应用红色"
    );
}

/// 测试 selector() 在完整规则中的端到端应用。
#[test]
fn test_supports_selector_in_rule() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    // @supports selector(p) { div { color: red; } }
    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Supports(zero_css_parser::ast::SupportsRule {
            condition: zero_css_parser::ast::SupportsCondition::Selector("p".to_string()),
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
    assert_eq!(
        div_style.color,
        ColorValue::Rgba(255, 0, 0, 255),
        "selector(p) 应该评估为 true，div 颜色应为红色"
    );
}

// ── Grid 属性端到端测试 ──

#[test]
fn test_grid_template_columns_end_to_end() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![
                Declaration {
                    property: "display".to_string(),
                    value: "grid".to_string(),
                    important: false,
                },
                Declaration {
                    property: "grid-template-columns".to_string(),
                    value: "100px 1fr auto".to_string(),
                    important: false,
                },
            ],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.display, DisplayValue::Grid);
    assert_eq!(div_style.grid_template_columns, Some("100px 1fr auto".to_string()));
}

#[test]
fn test_grid_template_rows_end_to_end() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![
                Declaration {
                    property: "display".to_string(),
                    value: "grid".to_string(),
                    important: false,
                },
                Declaration {
                    property: "grid-template-rows".to_string(),
                    value: "50px 1fr".to_string(),
                    important: false,
                },
            ],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.display, DisplayValue::Grid);
    assert_eq!(div_style.grid_template_rows, Some("50px 1fr".to_string()));
}

#[test]
fn test_grid_auto_flow_end_to_end() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![
                Declaration {
                    property: "display".to_string(),
                    value: "grid".to_string(),
                    important: false,
                },
                Declaration {
                    property: "grid-auto-flow".to_string(),
                    value: "column dense".to_string(),
                    important: false,
                },
            ],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.display, DisplayValue::Grid);
    assert_eq!(div_style.grid_auto_flow, property::GridAutoFlowValue::ColumnDense);
}

#[test]
fn test_grid_combined_properties_end_to_end() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![
                Declaration {
                    property: "display".to_string(),
                    value: "grid".to_string(),
                    important: false,
                },
                Declaration {
                    property: "grid-template-columns".to_string(),
                    value: "1fr 1fr".to_string(),
                    important: false,
                },
                Declaration {
                    property: "grid-template-rows".to_string(),
                    value: "auto".to_string(),
                    important: false,
                },
                Declaration {
                    property: "grid-auto-flow".to_string(),
                    value: "row".to_string(),
                    important: false,
                },
                Declaration {
                    property: "gap".to_string(),
                    value: "10px".to_string(),
                    important: false,
                },
            ],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.display, DisplayValue::Grid);
    assert_eq!(div_style.grid_template_columns, Some("1fr 1fr".to_string()));
    assert_eq!(div_style.grid_template_rows, Some("auto".to_string()));
    assert_eq!(div_style.grid_auto_flow, property::GridAutoFlowValue::Row);
    assert_eq!(div_style.gap, LengthValue::Px(10.0));
}

#[test]
fn test_grid_unset_uses_initial() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![
                Declaration {
                    property: "display".to_string(),
                    value: "grid".to_string(),
                    important: false,
                },
                Declaration {
                    property: "grid-auto-flow".to_string(),
                    value: "unset".to_string(),
                    important: false,
                },
            ],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    // grid-auto-flow is not inherited, unset = initial = Row
    assert_eq!(div_style.grid_auto_flow, property::GridAutoFlowValue::Row);
}

#[test]
fn test_grid_default_values_no_css() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![];
    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.grid_template_columns, None);
    assert_eq!(div_style.grid_template_rows, None);
    assert_eq!(div_style.grid_auto_flow, property::GridAutoFlowValue::Row);
    assert_eq!(div_style.grid_column_start, property::GridLineValue::Auto);
    assert_eq!(div_style.grid_column_end, property::GridLineValue::Auto);
    assert_eq!(div_style.grid_row_start, property::GridLineValue::Auto);
    assert_eq!(div_style.grid_row_end, property::GridLineValue::Auto);
    assert_eq!(div_style.grid_auto_rows, None);
    assert_eq!(div_style.grid_auto_columns, None);
}
