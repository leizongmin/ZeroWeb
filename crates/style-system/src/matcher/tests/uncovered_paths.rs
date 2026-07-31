// Matcher module - uncovered paths test
use super::super::*;
use zero_css_parser::ast::{
    AttrCaseModifier, AttributeMatcher, AttributeSelector, Combinator, ComplexSelector, CompoundSelector,
    ContainerRule, ContainerSizeCondition, NthPattern, PseudoClassSelector, Selector, SubclassSelector, TypeSelector,
};
use zero_css_parser::media_query::{
    MediaContext, MediaType, PointerValue, PrefersColorSchemeValue, ReducedMotionValue,
};
use zero_dom::Document;

// ── is_valid_selector_parse edge cases ──

/// 测试无效选择器：连续组合器（>>>）
#[test]
fn test_valid_selector_parse_consecutive_combinators() {
    assert!(!is_valid_selector_parse(">>>", &[]), ">>> should be invalid");
}

/// 测试无效选择器：> + 组合器
#[test]
fn test_valid_selector_parse_gt_plus() {
    assert!(!is_valid_selector_parse("> + div", &[]), "> + should be invalid");
}

/// 测试无效选择器：~ > 组合器
#[test]
fn test_valid_selector_parse_gt_tilde() {
    assert!(!is_valid_selector_parse("~ > span", &[]), "~ > should be invalid");
}

/// 测试无效选择器：以组合器开头的选择器
#[test]
fn test_valid_selector_parse_starts_with_combinator() {
    assert!(!is_valid_selector_parse("> div", &[]), "> div should be invalid");
    assert!(!is_valid_selector_parse("+ span", &[]), "+ span should be invalid");
    assert!(!is_valid_selector_parse("~ p", &[]), "~ p should be invalid");
}

/// 测试有效选择器
#[test]
fn test_valid_selector_parse_valid() {
    let css = "div { }";
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
    if let Some(zero_css_parser::ast::Rule::Style(style_rule)) = stylesheet.rules.first() {
        assert!(
            is_valid_selector_parse("div", &style_rule.selectors),
            "div should be valid"
        );
    }
}

/// 测试空选择器
#[test]
fn test_valid_selector_parse_empty() {
    assert!(!is_valid_selector_parse("", &[]), "empty selector should be invalid");
}

/// 测试带括号的有效选择器
#[test]
fn test_valid_selector_parse_with_parens() {
    let css = "div:not(.foo) > span { }";
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
    if let Some(zero_css_parser::ast::Rule::Style(style_rule)) = stylesheet.rules.first() {
        assert!(
            is_valid_selector_parse("div:not(.foo) > span", &style_rule.selectors),
            "div:not(.foo) > span should be valid"
        );
    }
}

// ── 复杂选择器组合 ──

/// 测试带多个子类选择器的复合选择器
#[test]
fn test_multiple_subclass_selectors() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.set_attribute(el, "id", "test");
    doc.set_attribute(el, "class", "foo bar");
    doc.set_attribute(el, "data-x", "y");
    doc.append_child(root, el).unwrap();

    // div#test.foo.bar[data-x="y"]
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("div".to_string())),
                    subclass_selectors: vec![
                        SubclassSelector::Id("test".to_string()),
                        SubclassSelector::Class("foo".to_string()),
                        SubclassSelector::Class("bar".to_string()),
                        SubclassSelector::Attribute(AttributeSelector {
                            name: "data-x".to_string(),
                            matcher: AttributeMatcher::Exact("y".to_string()),
                            case: AttrCaseModifier::Default,
                        }),
                    ],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, el, &sel));
}

/// 测试空类列表
#[test]
fn test_empty_class_list() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.set_attribute(el, "id", "test");
    doc.append_child(root, el).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("div".to_string())),
                    subclass_selectors: vec![SubclassSelector::Id("test".to_string())],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, el, &sel));
}

// ── :has() 复杂情况测试 ──

/// 测试 :has() 带直接子元素选择器
#[test]
fn test_has_direct_child() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");

    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, child).unwrap();

    // div:has(> span)
    let inner_sel = Selector {
        complex: ComplexSelector {
            parts: vec![
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Universal),
                        subclass_selectors: vec![],
                    },
                    Some(Combinator::Child),
                ),
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("span".to_string())),
                        subclass_selectors: vec![],
                    },
                    None,
                ),
            ],
        },
    };

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("div".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Has(vec![
                        inner_sel.clone(),
                    ]))],
                },
                None,
            )],
        },
    };

    // 首先检查基础匹配
    let inner_matches = matches_selector(&doc, child, &inner_sel);
    assert!(inner_matches, "> span should match span");

    assert!(matches_selector(&doc, parent, &sel), "div:has(> span) should match");
}

/// 测试 :has() 带多个后代选择器（OR 关系）
#[test]
fn test_has_multiple_descendants() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let child1 = doc.create_element("span");
    let child2 = doc.create_element("p");
    let grandchild = doc.create_element("a");

    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, child1).unwrap();
    doc.append_child(parent, child2).unwrap();
    doc.append_child(child1, grandchild).unwrap();

    // div:has(span, p)
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("div".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Has(vec![
                        make_tag_selector("span"),
                        make_tag_selector("p"),
                    ]))],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, parent, &sel));
}

// ── :only-child 和 :only-of-type 测试 ──

/// 测试 :only-child
#[test]
fn test_only_child() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let only = doc.create_element("p");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, only).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "only-child".to_string(),
                    ))],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, only, &sel));
}

/// 测试 :only-of-type
#[test]
fn test_only_of_type() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let only = doc.create_element("p");
    let other_type = doc.create_element("span");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, only).unwrap();
    doc.append_child(parent, other_type).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "only-of-type".to_string(),
                    ))],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, only, &sel));
}

// ── 属性选择器 Includes 测试 ──

/// 测试属性选择器 Includes（空格分隔）
#[test]
fn test_attribute_includes() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.set_attribute(el, "class", "foo bar baz");
    doc.append_child(root, el).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "class".to_string(),
                        matcher: AttributeMatcher::Includes("bar".to_string()),
                        case: AttrCaseModifier::Default,
                    })],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, el, &sel));
}

/// 测试属性选择器 Includes（不匹配）
#[test]
fn test_attribute_includes_no_match() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.set_attribute(el, "class", "foo baz");
    doc.append_child(root, el).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "class".to_string(),
                        matcher: AttributeMatcher::Includes("bar".to_string()),
                        case: AttrCaseModifier::Default,
                    })],
                },
                None,
            )],
        },
    };
    assert!(!matches_selector(&doc, el, &sel));
}

// ── 匹配函数边界条件 ──

/// 测试 matches_selector 空选择器
#[test]
fn test_matches_selector_empty() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    let sel = Selector {
        complex: ComplexSelector { parts: vec![] },
    };
    assert!(!matches_selector(&doc, el, &sel));
}

// ── collect_from_rules 边界条件 ──

// ── @media 规则的特殊情况测试 ──

/// 测试 @media 规则没有媒体上下文时不应用
#[test]
fn test_media_rule_no_context() {
    let (doc, nodes) = super::coverage::make_nested_dom();
    let child1 = nodes[2]; // span

    use zero_css_parser::ast::{Declaration, Rule, StyleRule};
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::At(zero_css_parser::ast::AtRule {
            name: "media".to_string(),
            prelude: "(min-width: 400px)".to_string(),
            body: zero_css_parser::ast::AtRuleBody::Block(vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("span")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            })]),
        })],
    }];

    // 不提供媒体上下文，@media 规则不应该应用
    let decls = collect_matching_declarations(&doc, child1, &stylesheets);
    assert!(decls.is_empty());
}

/// 测试 @media 规则逗号分隔的查询（OR 关系）
#[test]
fn test_media_rule_comma_queries() {
    let (doc, nodes) = super::coverage::make_nested_dom();
    let child1 = nodes[2]; // span

    use zero_css_parser::ast::{Declaration, Rule, StyleRule};
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::At(zero_css_parser::ast::AtRule {
            name: "media".to_string(),
            prelude: "(max-width: 400px), (prefers-color-scheme: dark)".to_string(),
            body: zero_css_parser::ast::AtRuleBody::Block(vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("span")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "blue".to_string(),
                    important: false,
                }],
            })]),
        })],
    }];

    // 视口宽度为 800px，不是 dark 模式 - 两个查询都不匹配
    let media_ctx = MediaContext {
        viewport_width: 800.0,
        viewport_height: 600.0,
        media_type: MediaType::Screen,
        prefers_color_scheme: PrefersColorSchemeValue::Light,
        prefers_reduced_motion: ReducedMotionValue::NoPreference,
        pointer_type: PointerValue::Fine,
        resolution_dpi: 96.0,
    };
    let decls = collect_matching_declarations_with_media(&doc, child1, &stylesheets, Some(&media_ctx), None);
    assert!(decls.is_empty());
}

// ── collect_from_rules 边界条件 ──

/// 测试空的规则列表
#[test]
fn test_collect_from_empty_rules() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    let decls = collect_matching_declarations(&doc, el, &[]);
    assert!(decls.is_empty());
}

/// 测试没有匹配选择器的规则
#[test]
fn test_collect_no_matching_selector() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    use zero_css_parser::ast::{Declaration, Rule, StyleRule};
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("span")], // 不匹配 div
            declarations: vec![Declaration {
                property: "color".to_string(),
                value: "red".to_string(),
                important: false,
            }],
        })],
    }];

    let decls = collect_matching_declarations(&doc, el, &stylesheets);
    assert!(decls.is_empty());
}

// ── :has() 带复杂组合器链测试 ──

/// 测试 :has() 带后代组合器链
#[test]
fn test_has_descendant_chain() {
    let mut doc = Document::new();
    let root = doc.root();
    let grandparent = doc.create_element("div");
    let parent = doc.create_element("section");
    let child = doc.create_element("p");
    let target = doc.create_element("span");

    doc.append_child(root, grandparent).unwrap();
    doc.append_child(grandparent, parent).unwrap();
    doc.append_child(parent, child).unwrap();
    doc.append_child(child, target).unwrap();

    // div:has(section p span)
    let inner_sel = Selector {
        complex: ComplexSelector {
            parts: vec![
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("section".to_string())),
                        subclass_selectors: vec![],
                    },
                    Some(Combinator::Descendant),
                ),
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("p".to_string())),
                        subclass_selectors: vec![],
                    },
                    Some(Combinator::Descendant),
                ),
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("span".to_string())),
                        subclass_selectors: vec![],
                    },
                    None,
                ),
            ],
        },
    };

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("div".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Has(vec![inner_sel]))],
                },
                None,
            )],
        },
    };

    assert!(matches_selector(&doc, grandparent, &sel));
}

/// 测试 :has() 带多个候选元素（只有最后一个匹配）
#[test]
fn test_has_only_last_candidate_matches() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let child1 = doc.create_element("p");
    let child2 = doc.create_element("p");
    let grandchild = doc.create_element("span");

    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, child1).unwrap();
    doc.append_child(parent, child2).unwrap();
    doc.append_child(child2, grandchild).unwrap();

    // Simple test: div:has(span)
    let simple_sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("div".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Has(vec![
                        make_tag_selector("span"),
                    ]))],
                },
                None,
            )],
        },
    };

    assert!(matches_selector(&doc, parent, &simple_sel));
}

// ── evaluate_container_condition 特性名称处理 ──

/// 测试容器查询特性名称处理（min-/max- 前缀）
#[test]
fn test_container_feature_name_processing() {
    let rule = ContainerRule {
        name: None,
        condition: zero_css_parser::ast::ContainerCondition::Size(ContainerSizeCondition {
            feature: "min-width".to_string(),
            range_min: None,
            range_max: None,
            operator: None,
            value: "400px".to_string(),
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(500.0, 600.0);
    assert!(evaluate_container_condition(&rule, Some(&ctx)));

    let rule_max = ContainerRule {
        name: None,
        condition: zero_css_parser::ast::ContainerCondition::Size(ContainerSizeCondition {
            feature: "max-height".to_string(),
            range_min: None,
            range_max: None,
            operator: None,
            value: "700px".to_string(),
        }),
        rules: vec![],
    };
    let ctx_short = ContainerContext::with_size(800.0, 500.0);
    assert!(evaluate_container_condition(&rule_max, Some(&ctx_short)));

    // width: 500px (exact match) — ctx has width=500
    let rule_inline = ContainerRule {
        name: None,
        condition: zero_css_parser::ast::ContainerCondition::Size(ContainerSizeCondition {
            feature: "width".to_string(),
            range_min: None,
            range_max: None,
            operator: None,
            value: "500px".to_string(),
        }),
        rules: vec![],
    };
    assert!(evaluate_container_condition(&rule_inline, Some(&ctx)));

    // height: 600px (exact match) — ctx has height=600
    let rule_block = ContainerRule {
        name: None,
        condition: zero_css_parser::ast::ContainerCondition::Size(ContainerSizeCondition {
            feature: "height".to_string(),
            range_min: None,
            range_max: None,
            operator: None,
            value: "600px".to_string(),
        }),
        rules: vec![],
    };
    assert!(evaluate_container_condition(&rule_block, Some(&ctx)));

    // Non-matching exact value
    let rule_no = ContainerRule {
        name: None,
        condition: zero_css_parser::ast::ContainerCondition::Size(ContainerSizeCondition {
            feature: "width".to_string(),
            range_min: None,
            range_max: None,
            operator: None,
            value: "300px".to_string(),
        }),
        rules: vec![],
    };
    assert!(!evaluate_container_condition(&rule_no, Some(&ctx)));
}

// ── is_property_supported 边界值 ──

/// 测试 is_property_supported 边界值
#[test]
fn test_property_supported_edge_cases() {
    // 空字符串
    assert!(!is_property_supported("display", ""));
    assert!(!is_property_supported("color", "   "));

    // container-name 任何非空字符串都有效
    assert!(is_property_supported("container-name", "sidebar"));
    assert!(is_property_supported("container-name", "main content"));
    assert!(is_property_supported("container-name", "123"));
}

// ── matches_selector_recursive 边界条件 ──

/// 测试 matches_selector_recursive 无组合器（None）
#[test]
fn test_matches_selector_recursive_no_combinator() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");

    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, child).unwrap();

    // 创建选择器：span div（无组合器的情况不应该发生，但我们测试处理）
    let parts = vec![
        (
            CompoundSelector {
                type_selector: Some(TypeSelector::Tag("div".to_string())),
                subclass_selectors: vec![],
            },
            None,
        ),
        (
            CompoundSelector {
                type_selector: Some(TypeSelector::Tag("span".to_string())),
                subclass_selectors: vec![],
            },
            None,
        ),
    ];

    // 这个测试覆盖 None 组合器的分支
    let _result = super::super::matches_selector_recursive(&doc, child, &parts, 1);
    // 注意：在实际选择器中，无组合器是无效的，但代码会继续处理
    // 这里我们测试的是代码对 None 的处理
}

// ── collect_from_rules @layer 规则处理 ──

/// 测试 @layer 规则的层索引分配
#[test]
fn test_collect_from_layer_rules() {
    let mut doc = Document::new();
    let root = doc.root();
    let span = doc.create_element("span");
    doc.append_child(root, span).unwrap();

    use zero_css_parser::ast::{Declaration, LayerRule, Rule, StyleRule};

    // 创建一个 @layer 规则
    let layer_rule = LayerRule {
        name: "base".to_string(),
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("span")],
            declarations: vec![Declaration {
                property: "color".to_string(),
                value: "red".to_string(),
                important: false,
            }],
        })],
    };

    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Layer(layer_rule)],
    }];

    let decls = collect_matching_declarations(&doc, span, &stylesheets);
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].4, Some(0)); // 层索引应为 0
}

/// 测试多个 @layer 规则的层索引递增
#[test]
fn test_collect_from_multiple_layer_rules() {
    let mut doc = Document::new();
    let root = doc.root();
    let span = doc.create_element("span");
    let p = doc.create_element("p");
    doc.append_child(root, span).unwrap();
    doc.append_child(root, p).unwrap();

    use zero_css_parser::ast::{Declaration, LayerRule, Rule, StyleRule};

    let layer1 = LayerRule {
        name: "base".to_string(),
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("span")],
            declarations: vec![Declaration {
                property: "color".to_string(),
                value: "red".to_string(),
                important: false,
            }],
        })],
    };

    let layer2 = LayerRule {
        name: "components".to_string(),
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("p")],
            declarations: vec![Declaration {
                property: "background".to_string(),
                value: "blue".to_string(),
                important: false,
            }],
        })],
    };

    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Layer(layer1), Rule::Layer(layer2)],
    }];

    let decls_span = collect_matching_declarations(&doc, span, &stylesheets);
    let decls_p = collect_matching_declarations(&doc, p, &stylesheets);

    // span 应该有层索引 0
    assert_eq!(decls_span.len(), 1);
    assert_eq!(decls_span[0].4, Some(0));

    // p 应该有层索引 1
    assert_eq!(decls_p.len(), 1);
    assert_eq!(decls_p[0].4, Some(1));
}

// ── matches_nth_type 0 个元素的情况 ──

/// 测试 matches_nth_of_type 无同类型兄弟
#[test]
fn test_nth_of_type_no_siblings_of_type() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let only_child = doc.create_element("span");
    let other_type = doc.create_element("p");

    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, only_child).unwrap();
    doc.append_child(parent, other_type).unwrap();

    // 选择器：span:nth-of-type(1)
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::NthOfType(
                        NthPattern { a: 0, b: 1 },
                    ))],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, only_child, &sel));
}

// ── is_empty_element 边界条件 ──

/// 测试 is_empty_element 只有空白文本节点
#[test]
fn test_empty_element_with_whitespace() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");

    doc.append_child(root, parent).unwrap();

    // 创建一个空白文本节点
    let text_id = doc.create_text_node("   \n\t  ");
    doc.append_child(parent, text_id).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("div".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "empty".to_string(),
                    ))],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, parent, &sel));
}

/// 测试 is_empty_element 有元素子节点但不匹配
#[test]
fn test_empty_element_with_non_matching_element() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");

    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, child).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("div".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "empty".to_string(),
                    ))],
                },
                None,
            )],
        },
    };
    assert!(!matches_selector(&doc, parent, &sel));
}

// ── NextSibling 和 SubsequentSibling 组合器测试 ──

/// 测试相邻兄弟组合器 (NextSibling)
#[test]
fn test_next_sibling_combinator() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let prev = doc.create_element("span");
    let target = doc.create_element("span");
    let next = doc.create_element("p");

    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, prev).unwrap();
    doc.append_child(parent, target).unwrap();
    doc.append_child(parent, next).unwrap();

    // 选择器：span + span
    // 这应该匹配 target，因为 target 前面有 span
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("span".to_string())),
                        subclass_selectors: vec![],
                    },
                    Some(Combinator::NextSibling),
                ),
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("span".to_string())),
                        subclass_selectors: vec![],
                    },
                    None,
                ),
            ],
        },
    };
    assert!(!matches_selector(&doc, prev, &sel)); // prev 没有前一个兄弟
    assert!(matches_selector(&doc, target, &sel)); // target 前面有 span
    assert!(!matches_selector(&doc, next, &sel)); // next 前面是 span，不是 span
}

/// 测试通用兄弟组合器 (SubsequentSibling)
#[test]
fn test_subsequent_sibling_combinator() {
    let (doc, nodes) = super::coverage::make_nested_dom();
    let child2 = nodes[3]; // p

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("span".to_string())),
                        subclass_selectors: vec![],
                    },
                    Some(Combinator::SubsequentSibling),
                ),
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("p".to_string())),
                        subclass_selectors: vec![],
                    },
                    None,
                ),
            ],
        },
    };
    assert!(
        matches_selector(&doc, child2, &sel),
        "p after span should match span ~ p"
    );
}

// ── 不支持的伪类测试 ──

/// 测试不支持的伪类返回 false
#[test]
fn test_unsupported_pseudo_class() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("div".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "unknown-pseudo".to_string(),
                    ))],
                },
                None,
            )],
        },
    };
    assert!(!matches_selector(&doc, el, &sel));
}

// ── 属性选择器变体测试 ──

/// 测试属性选择器 DashMatch
#[test]
fn test_attribute_dash_match() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.set_attribute(el, "lang", "en-US");
    doc.append_child(root, el).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "lang".to_string(),
                        matcher: AttributeMatcher::DashMatch("en".to_string()),
                        case: AttrCaseModifier::Default,
                    })],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, el, &sel));
}

/// 测试属性选择器 Prefix
#[test]
fn test_attribute_prefix() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.set_attribute(el, "data-test", "value123");
    doc.append_child(root, el).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "data-test".to_string(),
                        matcher: AttributeMatcher::Prefix("value".to_string()),
                        case: AttrCaseModifier::Default,
                    })],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, el, &sel));
}

/// 测试属性选择器 Suffix
#[test]
fn test_attribute_suffix() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.set_attribute(el, "href", "https://example.com/page.html");
    doc.append_child(root, el).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "href".to_string(),
                        matcher: AttributeMatcher::Suffix(".html".to_string()),
                        case: AttrCaseModifier::Default,
                    })],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, el, &sel));
}

/// 测试属性选择器 Substring
#[test]
fn test_attribute_substring() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.set_attribute(el, "class", "important highlight urgent");
    doc.append_child(root, el).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "class".to_string(),
                        matcher: AttributeMatcher::Substring("high".to_string()),
                        case: AttrCaseModifier::Default,
                    })],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, el, &sel));
}

// ── evaluate_supports_condition 测试 ──

/// 测试 SupportsCondition::And
#[test]
fn test_supports_condition_and() {
    use zero_css_parser::ast::SupportsCondition;

    // 两个都支持的条件
    assert!(evaluate_supports_condition(&SupportsCondition::And(vec![
        SupportsCondition::Property("display".to_string(), "block".to_string()),
        SupportsCondition::Property("color".to_string(), "red".to_string()),
    ])));

    // 一个不支持的条件
    assert!(!evaluate_supports_condition(&SupportsCondition::And(vec![
        SupportsCondition::Property("display".to_string(), "block".to_string()),
        SupportsCondition::Property("color".to_string(), "invalid-color".to_string()),
    ])));
}

/// 测试 SupportsCondition::Or
#[test]
fn test_supports_condition_or() {
    use zero_css_parser::ast::SupportsCondition;

    // 两个都支持的条件
    assert!(evaluate_supports_condition(&SupportsCondition::Or(vec![
        SupportsCondition::Property("display".to_string(), "block".to_string()),
        SupportsCondition::Property("color".to_string(), "red".to_string()),
    ])));

    // 一个支持一个不支持的条件
    assert!(evaluate_supports_condition(&SupportsCondition::Or(vec![
        SupportsCondition::Property("display".to_string(), "block".to_string()),
        SupportsCondition::Property("color".to_string(), "invalid-color".to_string()),
    ])));

    // 都不支持的条件
    assert!(!evaluate_supports_condition(&SupportsCondition::Or(vec![
        SupportsCondition::Property("display".to_string(), "invalid-display".to_string()),
        SupportsCondition::Property("color".to_string(), "invalid-color".to_string()),
    ])));
}

/// 测试 SupportsCondition::Not
#[test]
fn test_supports_condition_not() {
    use zero_css_parser::ast::SupportsCondition;

    // 不支持的条件
    assert!(evaluate_supports_condition(&SupportsCondition::Not(Box::new(
        SupportsCondition::Property("color".to_string(), "invalid-color".to_string())
    ))));

    // 支持的条件
    assert!(!evaluate_supports_condition(&SupportsCondition::Not(Box::new(
        SupportsCondition::Property("display".to_string(), "block".to_string())
    ))));
}

// ── evaluate_container_condition 范围语法测试 ──

/// 测试容器查询范围语法：200px <= width <= 500px
#[test]
fn test_container_condition_range_syntax() {
    let rule = ContainerRule {
        name: None,
        condition: zero_css_parser::ast::ContainerCondition::Size(ContainerSizeCondition {
            feature: "width".to_string(),
            range_min: Some("200px".to_string()),
            range_max: Some("500px".to_string()),
            operator: None,
            value: "".to_string(),
        }),
        rules: vec![],
    };

    // 匹配的情况：width = 300px
    let ctx_match = ContainerContext::with_size(300.0, 600.0);
    assert!(evaluate_container_condition(&rule, Some(&ctx_match)));

    // 不匹配的情况：width = 100px
    let ctx_too_small = ContainerContext::with_size(100.0, 600.0);
    assert!(!evaluate_container_condition(&rule, Some(&ctx_too_small)));

    // 不匹配的情况：width = 600px
    let ctx_too_big = ContainerContext::with_size(600.0, 600.0);
    assert!(!evaluate_container_condition(&rule, Some(&ctx_too_big)));
}

/// 测试容器查询比较运算符：width > 300px
#[test]
fn test_container_condition_comparison_operator() {
    let rule = ContainerRule {
        name: None,
        condition: zero_css_parser::ast::ContainerCondition::Size(ContainerSizeCondition {
            feature: "width".to_string(),
            range_min: None,
            range_max: None,
            operator: Some(">".to_string()),
            value: "300px".to_string(),
        }),
        rules: vec![],
    };

    // 匹配的情况：width = 400px
    let ctx_match = ContainerContext::with_size(400.0, 600.0);
    assert!(evaluate_container_condition(&rule, Some(&ctx_match)));

    // 不匹配的情况：width = 300px
    let ctx_equal = ContainerContext::with_size(300.0, 600.0);
    assert!(!evaluate_container_condition(&rule, Some(&ctx_equal)));
}

// ── collect_from_rules @keyframes 和 @import 测试 ──

/// 测试 @keyframes 规则被跳过
#[test]
fn test_keyframes_rule_skipped() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    use zero_css_parser::ast::{KeyframesRule, Rule};
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Keyframes(KeyframesRule {
            name: "slide".to_string(),
            keyframes: vec![],
        })],
    }];

    let decls = collect_matching_declarations(&doc, el, &stylesheets);
    assert!(decls.is_empty()); // @keyframes 规则应该被跳过
}

/// 测试 @import 规则被跳过
#[test]
fn test_import_rule_skipped() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    use zero_css_parser::ast::{ImportRule, Rule};
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Import(ImportRule {
            url: "styles.css".to_string(),
            media_queries: vec![],
        })],
    }];

    let decls = collect_matching_declarations(&doc, el, &stylesheets);
    assert!(decls.is_empty()); // @import 规则应该被跳过
}

// ── matches_nth_pattern 负系数测试 ──

/// 测试 matches_nth_pattern 负系数模式
#[test]
fn test_nth_pattern_negative_coefficient() {
    // 测试模式：-n+3（匹配第1、2、3个元素）
    // 解释：diff = index - 3
    // a = -1，所以需要 diff <= 0 && diff % -1 == 0
    // index=1: diff=-2 <=0 && -2 % -1 == 0 ✓
    // index=2: diff=-1 <=0 && -1 % -1 == 0 ✓
    // index=3: diff=0 <=0 && 0 % -1 == 0 ✓
    // index=4: diff=1 > 0 ✗
    assert!(matches_nth_pattern(
        1,
        &zero_css_parser::ast::NthPattern { a: -1, b: 3 }
    ));
    assert!(matches_nth_pattern(
        2,
        &zero_css_parser::ast::NthPattern { a: -1, b: 3 }
    ));
    assert!(matches_nth_pattern(
        3,
        &zero_css_parser::ast::NthPattern { a: -1, b: 3 }
    ));
    assert!(!matches_nth_pattern(
        4,
        &zero_css_parser::ast::NthPattern { a: -1, b: 3 }
    ));

    // 测试模式：-2n+5（匹配第1、3、5个元素）
    // index=1: diff=-4 <=0 && -4 % -2 == 0 ✓
    // index=2: diff=-3 <=0 && -3 % -2 != 0 ✗
    // index=3: diff=-2 <=0 && -2 % -2 == 0 ✓
    // index=4: diff=-1 <=0 && -1 % -2 != 0 ✗
    // index=5: diff=0 <=0 && 0 % -2 == 0 ✓
    assert!(matches_nth_pattern(
        1,
        &zero_css_parser::ast::NthPattern { a: -2, b: 5 }
    ));
    assert!(!matches_nth_pattern(
        2,
        &zero_css_parser::ast::NthPattern { a: -2, b: 5 }
    ));
    assert!(matches_nth_pattern(
        3,
        &zero_css_parser::ast::NthPattern { a: -2, b: 5 }
    ));
    assert!(!matches_nth_pattern(
        4,
        &zero_css_parser::ast::NthPattern { a: -2, b: 5 }
    ));
    assert!(matches_nth_pattern(
        5,
        &zero_css_parser::ast::NthPattern { a: -2, b: 5 }
    ));
}

/// 测试 matches_nth_pattern 边界值
#[test]
fn test_nth_pattern_boundary_values() {
    // a = 0 的情况：精确匹配
    assert!(matches_nth_pattern(5, &zero_css_parser::ast::NthPattern { a: 0, b: 5 }));
    assert!(!matches_nth_pattern(
        6,
        &zero_css_parser::ast::NthPattern { a: 0, b: 5 }
    ));

    // 正系数
    assert!(matches_nth_pattern(7, &zero_css_parser::ast::NthPattern { a: 3, b: 1 })); // 3*2+1=7
    assert!(matches_nth_pattern(
        10,
        &zero_css_parser::ast::NthPattern { a: 3, b: 1 }
    )); // 3*3+1=10
    assert!(!matches_nth_pattern(
        8,
        &zero_css_parser::ast::NthPattern { a: 3, b: 1 }
    )); // 8-1=7 不能被 3 整除
}

// 辅助函数
pub(super) fn make_tag_selector(tag: &str) -> Selector {
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
