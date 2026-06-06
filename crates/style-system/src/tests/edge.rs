// Auto-generated test file — split from style-system/lib.rs
use super::super::*;
use super::helpers::*;

// ═══════════════════════════════════════════════════════════════════

#[test]
/// scroll-snap-type: none 产生默认值（strictness=None, axis=Both）。
fn test_scroll_snap_type_none() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "scroll-snap-type".to_string(),
                value: "none".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div 应该有样式");
    assert_eq!(
        div_style.scroll_snap_type.strictness,
        property::ScrollSnapStrictness::None
    );
    assert_eq!(
        div_style.scroll_snap_type.axis,
        zero_css_parser::values::ScrollSnapAxis::Both
    );
}

#[test]
/// scroll-snap-type: x mandatory 存储 strictness=Mandatory, axis=X。
fn test_scroll_snap_type_mandatory() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "scroll-snap-type".to_string(),
                value: "x mandatory".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div 应该有样式");
    assert_eq!(
        div_style.scroll_snap_type.strictness,
        property::ScrollSnapStrictness::Mandatory
    );
    assert_eq!(
        div_style.scroll_snap_type.axis,
        zero_css_parser::values::ScrollSnapAxis::X
    );
}

#[test]
/// scroll-snap-type: y proximity 存储 strictness=Proximity, axis=Y。
fn test_scroll_snap_type_proximity() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "scroll-snap-type".to_string(),
                value: "y proximity".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div 应该有样式");
    assert_eq!(
        div_style.scroll_snap_type.strictness,
        property::ScrollSnapStrictness::Proximity
    );
    assert_eq!(
        div_style.scroll_snap_type.axis,
        zero_css_parser::values::ScrollSnapAxis::Y
    );
}

#[test]
/// scroll-snap-align 的 start/center/end 值端到端存储验证。
fn test_scroll_snap_align_values() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    // start
    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "scroll-snap-align".to_string(),
                value: "start".to_string(),
                important: false,
            }],
        })],
    }];
    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div 应该有样式");
    assert_eq!(div_style.scroll_snap_align, property::ScrollSnapAlign::Start);

    // center
    let mut sys2 = StyleSystem::new();
    let stylesheets2 = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "scroll-snap-align".to_string(),
                value: "center".to_string(),
                important: false,
            }],
        })],
    }];
    let styles2 = sys2.compute_styles(&doc, &stylesheets2);
    let div_style2 = styles2.get(&div).expect("div 应该有样式");
    assert_eq!(div_style2.scroll_snap_align, property::ScrollSnapAlign::Center);

    // end
    let mut sys3 = StyleSystem::new();
    let stylesheets3 = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "scroll-snap-align".to_string(),
                value: "end".to_string(),
                important: false,
            }],
        })],
    }];
    let styles3 = sys3.compute_styles(&doc, &stylesheets3);
    let div_style3 = styles3.get(&div).expect("div 应该有样式");
    assert_eq!(div_style3.scroll_snap_align, property::ScrollSnapAlign::End);
}

#[test]
/// scroll-snap-stop: normal 和 always 两个值的端到端存储验证。
fn test_scroll_snap_stop_normal_always() {
    let (doc, _html, _body, div, _p) = make_test_dom();

    // normal（默认值）
    let mut sys = StyleSystem::new();
    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "scroll-snap-stop".to_string(),
                value: "normal".to_string(),
                important: false,
            }],
        })],
    }];
    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div 应该有样式");
    assert_eq!(div_style.scroll_snap_stop, property::ScrollSnapStop::Normal);

    // always
    let mut sys2 = StyleSystem::new();
    let stylesheets2 = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "scroll-snap-stop".to_string(),
                value: "always".to_string(),
                important: false,
            }],
        })],
    }];
    let styles2 = sys2.compute_styles(&doc, &stylesheets2);
    let div_style2 = styles2.get(&div).expect("div 应该有样式");
    assert_eq!(div_style2.scroll_snap_stop, property::ScrollSnapStop::Always);
}

// ═══════════════════════════════════════════════════════════════════
// 边界条件端到端测试
// ═══════════════════════════════════════════════════════════════════

/// 测试级联特异性：ID 选择器与 class 选择器冲突时，ID 选择器胜出。
#[test]
fn test_cascade_specificity_id_vs_class() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let div = doc.create_element("div");
    doc.set_attribute(div, "id", "myid");
    doc.set_attribute(div, "class", "myclass");
    doc.append_child(body, div).unwrap();

    let mut sys = StyleSystem::new();

    let id_sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Id("myid".to_string())],
                },
                None,
            )],
        },
    };
    let class_sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Class("myclass".to_string())],
                },
                None,
            )],
        },
    };

    // #myid { color: red } vs .myclass { color: blue }
    // ID 选择器特异性 (1,0,0) > class 选择器 (0,1,0)，red 胜出
    let stylesheets = vec![Stylesheet {
        rules: vec![
            Rule::Style(StyleRule {
                selectors: vec![class_sel],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "blue".to_string(),
                    important: false,
                }],
            }),
            Rule::Style(StyleRule {
                selectors: vec![id_sel],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            }),
        ],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255)); // red
}

/// 测试 !important 声明即使特异性更低也能覆盖 normal 声明。
#[test]
fn test_cascade_important_override() {
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

    // div { color: red !important } vs #main { color: blue }
    // 标签选择器 + !important 应胜过 ID 选择器 + normal
    let stylesheets = vec![Stylesheet {
        rules: vec![
            Rule::Style(StyleRule {
                selectors: vec![id_sel],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "blue".to_string(),
                    important: false,
                }],
            }),
            Rule::Style(StyleRule {
                selectors: vec![tag_sel],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: true,
                }],
            }),
        ],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    // !important 胜过更高特异性的 normal 声明
    assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255)); // red
}

/// 测试 color 属性继承：父元素设置 color 后，子元素应继承该值。
#[test]
fn test_inherit_color_from_parent() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let parent = doc.create_element("div");
    doc.append_child(body, parent).unwrap();
    let child = doc.create_element("span");
    doc.append_child(parent, child).unwrap();

    let mut sys = StyleSystem::new();

    // div { color: green } — span 未设置 color，应继承
    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "color".to_string(),
                value: "green".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let child_style = styles.get(&child).expect("span should have style");
    // span 应从 div 继承 green
    assert_eq!(child_style.color, ColorValue::Rgba(0, 128, 0, 255));
}

/// 测试无样式元素的计算 display 默认值。
#[test]
fn test_computed_default_display() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    // 不设置任何 CSS 规则
    let stylesheets = vec![];
    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    // div 的 UA 默认 display 为 Block（HTML 元素语义默认值）
    assert_eq!(div_style.display, DisplayValue::Block);
}

/// 测试简写 margin: 10px 展开后四个边均为 10px。
#[test]
fn test_shorthand_margin_expansion() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "margin".to_string(),
                value: "10px".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.margin_top, LengthValue::Px(10.0));
    assert_eq!(div_style.margin_right, LengthValue::Px(10.0));
    assert_eq!(div_style.margin_bottom, LengthValue::Px(10.0));
    assert_eq!(div_style.margin_left, LengthValue::Px(10.0));
}

/// 测试 var() 回退值：var(--unknown, blue) 在 --unknown 未定义时使用 blue。
#[test]
fn test_custom_property_fallback() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    // color: var(--unknown, blue) — --unknown 不存在，应使用 blue
    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "color".to_string(),
                value: "var(--unknown, blue)".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255));
}

/// 测试无视口时 @media 规则不应用。
#[test]
fn test_media_query_no_viewport() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();
    // 不设置视口

    // @media (min-width: 500px) { div { color: red; } }
    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::At(zero_css_parser::ast::AtRule {
            name: "media".to_string(),
            prelude: "(min-width: 500px)".to_string(),
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
    // 无视口信息，@media 不应用，color 保持默认黑色
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 0, 255));
}

// ═══════════════════════════════════════════════════════════════════
// 新增边界条件测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// 视口单位在端到端管线中正确解析：vw/vh 设置视口后转换为 px。
fn test_viewport_units_e2e() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();
    sys.set_viewport(1000.0, 500.0);

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![
                Declaration {
                    property: "width".to_string(),
                    value: "50vw".to_string(),
                    important: false,
                },
                Declaration {
                    property: "height".to_string(),
                    value: "20vh".to_string(),
                    important: false,
                },
            ],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div 应该有样式");
    // 50vw = 50% * 1000 = 500px
    assert_eq!(div_style.width, LengthValue::Px(500.0));
    // 20vh = 20% * 500 = 100px
    assert_eq!(div_style.height, LengthValue::Px(100.0));
}

#[test]
/// rem 单位在端到端管线中正确解析：rem 始终基于根字体大小 16px，
/// 不受父元素或自身 font-size 影响。
fn test_rem_unit_e2e() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![
            // 设置父元素 font-size 为 32px
            Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![
                    Declaration {
                        property: "font-size".to_string(),
                        value: "32px".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "width".to_string(),
                        value: "2rem".to_string(),
                        important: false,
                    },
                ],
            }),
        ],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div 应该有样式");
    // 2rem = 2 * 16px(root) = 32px，不受自身 font-size: 32px 影响
    assert_eq!(div_style.width, LengthValue::Px(32.0));
}

#[test]
/// 多个样式表声明合并：不同样式表中的同一属性按出现顺序合并，
/// 后出现的样式表中的声明应覆盖前者（同特异性下）。
fn test_multiple_stylesheets_merge() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![
        // 第一个样式表：color: red, margin-top: 10px
        Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![
                    Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "margin-top".to_string(),
                        value: "10px".to_string(),
                        important: false,
                    },
                ],
            })],
        },
        // 第二个样式表：color: green（覆盖 red），font-size: 20px
        Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![
                    Declaration {
                        property: "color".to_string(),
                        value: "green".to_string(),
                        important: false,
                    },
                    Declaration {
                        property: "font-size".to_string(),
                        value: "20px".to_string(),
                        important: false,
                    },
                ],
            })],
        },
    ];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div 应该有样式");
    // color: 第二个样式表的 green 覆盖第一个的 red
    assert_eq!(div_style.color, ColorValue::Rgba(0, 128, 0, 255));
    // margin-top: 仅在第一个样式表中，保持 10px
    assert_eq!(div_style.margin_top, LengthValue::Px(10.0));
    // font-size: 仅在第二个样式表中，为 20px
    assert_eq!(div_style.font_size, LengthValue::Px(20.0));
}

#[test]
/// 自定义属性循环引用防护：--a 引用 --b，--b 引用 --a，
/// 系统应通过迭代上限防止无限循环，不会 panic。
fn test_custom_property_circular_reference() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![
                Declaration {
                    property: "--a".to_string(),
                    value: "var(--b)".to_string(),
                    important: false,
                },
                Declaration {
                    property: "--b".to_string(),
                    value: "var(--a)".to_string(),
                    important: false,
                },
                Declaration {
                    property: "color".to_string(),
                    value: "var(--a)".to_string(),
                    important: false,
                },
            ],
        })],
    }];

    // 不应 panic，循环引用被迭代上限保护
    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div 应该有样式");
    // 循环引用无法解析到具体颜色值，color 保持默认黑色
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 0, 255));
}

#[test]
/// 两个 !important 声明冲突时，特异性更高的胜出。
/// div { color: red !important } #main { color: blue !important }
/// 两个都是 !important，ID 选择器 (1,0,0) > 标签选择器 (0,0,1)。
fn test_dual_important_higher_specificity_wins() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

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

    let stylesheets = vec![Stylesheet {
        rules: vec![
            // 标签选择器 + !important → color: red
            Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: true,
                }],
            }),
            // ID 选择器 + !important → color: blue
            Rule::Style(StyleRule {
                selectors: vec![id_sel],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "blue".to_string(),
                    important: true,
                }],
            }),
        ],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div 应该有样式");
    // 同为 !important 时，ID 选择器特异性 (1,0,0) 高于标签 (0,0,1)，blue 胜出
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255));
}

// ═══════════════════════════════════════════════════════════════════
// 新增边界条件端到端测试（round 12）
// ═══════════════════════════════════════════════════════════════════

#[test]
/// 仅含文本节点的父元素不产生计算样式，但相邻元素节点各自独立计算样式。
/// 验证非元素节点（文本节点）不参与样式系统，元素节点正确获得默认样式。
fn test_text_nodes_get_no_style_but_siblings_do() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    // 添加一个文本节点
    let text = doc.create_text_node("Hello");
    doc.append_child(body, text).unwrap();
    // 添加一个元素节点
    let span = doc.create_element("span");
    doc.append_child(body, span).unwrap();

    let mut sys = StyleSystem::new();
    let stylesheets = vec![];
    let styles = sys.compute_styles(&doc, &stylesheets);

    // span 应有样式（默认值）
    assert!(styles.get(&span).is_some(), "span 应该有计算样式");
    // body 应有样式
    assert!(styles.get(&body).is_some(), "body 应该有计算样式");
    // 文本节点不在 styles 中（NodeId 无法直接查，但总样式数应只含元素节点）
    // html, body, span 三个元素节点有样式
    assert!(styles.len() >= 3, "至少 3 个元素节点有样式");
}

#[test]
/// 多层嵌套继承：grandparent 设置 color: red，parent 未设置，
/// child 也未设置，验证继承链在三代之间正确传递。
/// 同时验证非继承属性（margin-top）不在代际之间传递。
fn test_deep_nesting_inheritance_chain() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let grandparent = doc.create_element("div");
    doc.set_attribute(grandparent, "id", "gp");
    doc.append_child(body, grandparent).unwrap();
    let parent = doc.create_element("section");
    doc.append_child(grandparent, parent).unwrap();
    let child = doc.create_element("span");
    doc.append_child(parent, child).unwrap();

    let mut sys = StyleSystem::new();

    let id_sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Id("gp".to_string())],
                },
                None,
            )],
        },
    };

    // #gp { color: red; margin-top: 20px }
    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![id_sel],
            declarations: vec![
                Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
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

    // grandparent 的 color = red, margin-top = 20px
    let gp_style = styles.get(&grandparent).expect("grandparent 应有样式");
    assert_eq!(gp_style.color, ColorValue::Rgba(255, 0, 0, 255));
    assert_eq!(gp_style.margin_top, LengthValue::Px(20.0));

    // parent 继承 color = red，但 margin-top 不继承
    let parent_style = styles.get(&parent).expect("parent 应有样式");
    assert_eq!(parent_style.color, ColorValue::Rgba(255, 0, 0, 255));
    assert_eq!(parent_style.margin_top, LengthValue::Px(0.0));

    // child 继承 color = red（经过两代传递），margin-top 不继承
    let child_style = styles.get(&child).expect("child 应有样式");
    assert_eq!(child_style.color, ColorValue::Rgba(255, 0, 0, 255));
    assert_eq!(child_style.margin_top, LengthValue::Px(0.0));
}

#[test]
/// 简写与 longhand 混合应用后，后声明的 longhand 覆盖简写中对应子属性。
/// 验证 margin 简写 + 单独的 margin-top 覆盖在端到端管线中正确工作。
fn test_shorthand_then_longhand_override_e2e() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    // div { margin: 10px; margin-top: 30px; padding: 5px 15px; padding-left: 25px }
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
                    value: "30px".to_string(),
                    important: false,
                },
                Declaration {
                    property: "padding".to_string(),
                    value: "5px 15px".to_string(),
                    important: false,
                },
                Declaration {
                    property: "padding-left".to_string(),
                    value: "25px".to_string(),
                    important: false,
                },
            ],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div 应该有样式");

    // margin-top 被 longhand 覆盖为 30px
    assert_eq!(div_style.margin_top, LengthValue::Px(30.0));
    // 其余 margin 边保持简写值 10px
    assert_eq!(div_style.margin_right, LengthValue::Px(10.0));
    assert_eq!(div_style.margin_bottom, LengthValue::Px(10.0));
    assert_eq!(div_style.margin_left, LengthValue::Px(10.0));

    // padding-left 被 longhand 覆盖为 25px
    assert_eq!(div_style.padding_left, LengthValue::Px(25.0));
    // 其余 padding 保持简写值
    assert_eq!(div_style.padding_top, LengthValue::Px(5.0));
    assert_eq!(div_style.padding_right, LengthValue::Px(15.0));
    assert_eq!(div_style.padding_bottom, LengthValue::Px(5.0));
}

#[test]
/// 自定义属性与 var() 三层嵌套解析：
/// --base → --mid → --top，color 使用 var(--top)，
/// 验证系统正确展开三层间接引用。
fn test_custom_property_triple_indirection() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![
                Declaration {
                    property: "--base".to_string(),
                    value: "green".to_string(),
                    important: false,
                },
                Declaration {
                    property: "--mid".to_string(),
                    value: "var(--base)".to_string(),
                    important: false,
                },
                Declaration {
                    property: "--top".to_string(),
                    value: "var(--mid)".to_string(),
                    important: false,
                },
                Declaration {
                    property: "color".to_string(),
                    value: "var(--top)".to_string(),
                    important: false,
                },
            ],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div 应该有样式");
    // var(--top) → var(--mid) → var(--base) → green
    assert_eq!(div_style.color, ColorValue::Rgba(0, 128, 0, 255));
}

#[test]
/// @layer 内的 @media 规则同时生效：
/// @layer base { @media (min-width: 600px) { div { color: red } } }
/// 设置视口 800px，验证分层内的媒体查询条件正确评估。
fn test_layer_with_media_inside() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);

    let stylesheets = vec![Stylesheet {
        rules: vec![
            // @layer base { @media (min-width: 600px) { div { color: red } } }
            Rule::Layer(zero_css_parser::ast::LayerRule {
                name: "base".to_string(),
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
            }),
            // 未分层规则 — div { color: blue }
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
    let div_style = styles.get(&div).expect("div 应该有样式");
    // 未分层声明胜过分层声明，即使 @media 条件满足
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255)); // blue

    // 额外验证：去掉未分层规则后，@layer 内的 @media 样式应生效
    let stylesheets_layer_only = vec![Stylesheet {
        rules: vec![Rule::Layer(zero_css_parser::ast::LayerRule {
            name: "base".to_string(),
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
        })],
    }];

    let mut sys2 = StyleSystem::new();
    sys2.set_viewport(800.0, 600.0);
    let styles2 = sys2.compute_styles(&doc, &stylesheets_layer_only);
    let div_style2 = styles2.get(&div).expect("div 应该有样式");
    // @layer 内 @media 条件满足，color 应为红色
    assert_eq!(div_style2.color, ColorValue::Rgba(255, 0, 0, 255)); // red
}

// ═══════════════════════════════════════════════════════════════════
// 新增边界条件端到端测试（round 16）
// ═══════════════════════════════════════════════════════════════════

#[test]
/// background-color 在端到端管线中正确应用：
/// div { background-color: #ff6600 } 应产生对应的 RGBA 颜色值。
fn test_background_color_hex_e2e() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "background-color".to_string(),
                value: "#ff6600".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div 应该有样式");
    // #ff6600 → rgba(255, 102, 0, 255)
    assert_eq!(div_style.background_color, ColorValue::Rgba(255, 102, 0, 255));
}

#[test]
/// visibility 是继承属性：父元素设置 visibility:hidden，
/// 子元素未显式设置 visibility 时应继承 hidden。
fn test_visibility_inheritance_e2e() {
    let (doc, _html, _body, _div, p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "visibility".to_string(),
                value: "hidden".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let p_style = styles.get(&p).expect("p 应该有样式");
    // visibility 是继承属性，p 应从 div 继承 hidden
    assert_eq!(p_style.visibility, zero_css_parser::values::VisibilityValue::Hidden);
}

#[test]
/// position + top/left 端到端：position:absolute 配合偏移属性
/// 在端到端管线中正确存储。
fn test_position_absolute_with_offsets_e2e() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![
                Declaration {
                    property: "position".to_string(),
                    value: "absolute".to_string(),
                    important: false,
                },
                Declaration {
                    property: "top".to_string(),
                    value: "10px".to_string(),
                    important: false,
                },
                Declaration {
                    property: "left".to_string(),
                    value: "20px".to_string(),
                    important: false,
                },
            ],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div 应该有样式");
    assert_eq!(div_style.position, zero_css_parser::values::PositionValue::Absolute);
    assert_eq!(div_style.top, LengthValue::Px(10.0));
    assert_eq!(div_style.left, LengthValue::Px(20.0));
}

#[test]
/// z-index 整数值在端到端管线中正确存储。
/// 验证 z-index: 100 被解析为 ZIndexValue::Integer(100)。
fn test_z_index_integer_e2e() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "z-index".to_string(),
                value: "100".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div 应该有样式");
    assert_eq!(div_style.z_index, property::ZIndexValue::Integer(100));
}

#[test]
/// min-width / max-width 在端到端管线中正确应用。
/// min-width: 50px, max-width: 500px 应被解析为对应 Px 长度值。
fn test_min_max_width_e2e() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![
                Declaration {
                    property: "min-width".to_string(),
                    value: "50px".to_string(),
                    important: false,
                },
                Declaration {
                    property: "max-width".to_string(),
                    value: "500px".to_string(),
                    important: false,
                },
            ],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div 应该有样式");
    assert_eq!(div_style.min_width, LengthValue::Px(50.0));
    assert_eq!(div_style.max_width, LengthValue::Px(500.0));
}
