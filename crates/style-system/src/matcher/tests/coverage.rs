// 覆盖率提升测试 — matcher/mod.rs 中未覆盖的分支
use super::super::*;
use zero_css_parser::ast::{
    AttributeMatcher, AttributeSelector, Combinator, ComplexSelector, CompoundSelector, ContainerCondition,
    ContainerRule, ContainerSizeCondition, NthPattern, PseudoClassSelector, PseudoElementSelector, Selector,
    SubclassSelector, SupportsCondition, TypeSelector,
};
use zero_css_parser::media_query::{
    MediaContext, MediaType, PointerValue, PrefersColorSchemeValue, ReducedMotionValue,
};
use zero_dom::Document;

// 复用 core.rs 的辅助函数
use super::core::make_tag_selector;

/// 辅助：构建 2 部分选择器（带组合器）
fn make_compound_with_combinator(type1: &str, combinator: Combinator, type2: &str) -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag(type1.to_string())),
                        subclass_selectors: vec![],
                    },
                    Some(combinator),
                ),
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag(type2.to_string())),
                        subclass_selectors: vec![],
                    },
                    None,
                ),
            ],
        },
    }
}

/// 辅助：构建带伪类选择器
fn make_pseudo_selector(pc: PseudoClassSelector) -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(pc)],
                },
                None,
            )],
        },
    }
}

/// 辅助：创建 3 层 DOM (parent > child1, child2 > grandchild)
pub(super) fn make_nested_dom() -> (Document, Vec<zero_dom::NodeId>) {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let _ = doc.append_child(root, parent);
    let child1 = doc.create_element("span");
    let _ = doc.append_child(parent, child1);
    let child2 = doc.create_element("p");
    let _ = doc.append_child(parent, child2);
    let grandchild = doc.create_element("em");
    let _ = doc.append_child(child1, grandchild);
    (doc, vec![root, parent, child1, child2, grandchild])
}

/// 辅助：创建 ContainerRule
fn make_container_rule(feature: &str, value: &str, rules: Vec<zero_css_parser::ast::Rule>) -> ContainerRule {
    ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: feature.to_string(),
            range_min: None,
            range_max: None,
            operator: None,
            value: value.to_string(),
        }),
        rules,
    }
}

/// 辅助：创建带范围/操作符的 ContainerRule
fn make_container_rule_advanced(
    feature: &str,
    range_min: Option<&str>,
    range_max: Option<&str>,
    operator: Option<&str>,
    value: &str,
    rules: Vec<zero_css_parser::ast::Rule>,
) -> ContainerRule {
    ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: feature.to_string(),
            range_min: range_min.map(String::from),
            range_max: range_max.map(String::from),
            operator: operator.map(String::from),
            value: value.to_string(),
        }),
        rules,
    }
}

// ── SubsequentSibling 组合器测试 ──

#[test]
fn test_subsequent_sibling_combinator() {
    let (doc, nodes) = make_nested_dom();
    let child2 = nodes[3]; // p

    let sel = make_compound_with_combinator("span", Combinator::SubsequentSibling, "p");
    assert!(
        matches_selector(&doc, child2, &sel),
        "p after span should match span ~ p"
    );
}

#[test]
fn test_subsequent_sibling_no_match() {
    let (doc, nodes) = make_nested_dom();
    let child1 = nodes[2]; // span

    let sel_rev = make_compound_with_combinator("p", Combinator::SubsequentSibling, "span");
    assert!(
        !matches_selector(&doc, child1, &sel_rev),
        "span before p should NOT match p ~ span"
    );
}

// ── PseudoElement 返回 false ──

#[test]
fn test_pseudo_element_never_matches() {
    let (doc, nodes) = make_nested_dom();
    let child1 = nodes[2];
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoElement(PseudoElementSelector::Standard(
                        "before".to_string(),
                    ))],
                },
                None,
            )],
        },
    };
    assert!(
        !matches_selector(&doc, child1, &sel),
        "pseudo-element should never match DOM element"
    );
}

// ── :nth-last-child 测试 ──

#[test]
fn test_nth_last_child() {
    let (doc, nodes) = make_nested_dom();
    let child1 = nodes[2]; // span — 倒数第2
    let child2 = nodes[3]; // p — 倒数第1

    let sel_last = make_pseudo_selector(PseudoClassSelector::NthLastChild(NthPattern { a: 0, b: 1 }));
    assert!(
        matches_selector(&doc, child2, &sel_last),
        "p is last child (nth-last-child(1))"
    );

    let sel_2nd_last = make_pseudo_selector(PseudoClassSelector::NthLastChild(NthPattern { a: 0, b: 2 }));
    assert!(matches_selector(&doc, child1, &sel_2nd_last), "span is 2nd from last");
}

// ── :nth-last-of-type 测试 ──

#[test]
fn test_nth_last_of_type() {
    let (doc, nodes) = make_nested_dom();
    let child2 = nodes[3]; // p — 唯一的 p

    let sel = make_pseudo_selector(PseudoClassSelector::NthLastOfType(NthPattern { a: 0, b: 1 }));
    assert!(matches_selector(&doc, child2, &sel), "p is nth-last-of-type(1)");
}

// ── :nth-of-type 2n ──

#[test]
fn test_nth_of_type_even() {
    let (doc, nodes) = make_nested_dom();
    let child1 = nodes[2]; // span (1st of type)

    let sel = make_pseudo_selector(PseudoClassSelector::NthOfType(NthPattern { a: 2, b: 0 }));
    assert!(!matches_selector(&doc, child1, &sel), "1st of type is not even");
}

// ── :not() 测试 ──

#[test]
fn test_not_selector() {
    let (doc, nodes) = make_nested_dom();
    let child1 = nodes[2]; // span
    let child2 = nodes[3]; // p

    let sel = make_pseudo_selector(PseudoClassSelector::Not(vec![make_tag_selector("p")]));
    assert!(matches_selector(&doc, child1, &sel), "span is NOT p");
    assert!(!matches_selector(&doc, child2, &sel), "p IS p → :not(p) fails");
}

// ── :is() 测试 ──

#[test]
fn test_is_selector() {
    let (doc, nodes) = make_nested_dom();
    let child1 = nodes[2]; // span

    let sel = make_pseudo_selector(PseudoClassSelector::Is(vec![
        make_tag_selector("span"),
        make_tag_selector("div"),
    ]));
    assert!(matches_selector(&doc, child1, &sel), "span matches :is(span, div)");
}

// ── :where() 测试 ──

#[test]
fn test_where_selector() {
    let (doc, nodes) = make_nested_dom();
    let child2 = nodes[3]; // p

    let sel = make_pseudo_selector(PseudoClassSelector::Where(vec![make_tag_selector("p")]));
    assert!(matches_selector(&doc, child2, &sel), "p matches :where(p)");
}

// ── :lang() 匹配（CSS 2.1 §5.11.4）──

#[test]
fn test_lang_matches_basic_inherit_boundary_caseinsensitive() {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let _ = doc.append_child(root, parent);
    // parent 设 lang="en-US"，验证 :lang(en) 通过连字符前缀边界匹配（en == "en" 前缀 + "-"）。
    doc.set_attribute(parent, "lang", "en-US");
    let child = doc.create_element("span");
    let _ = doc.append_child(parent, child);
    // 兄弟 sibling 无 lang 祖先链（parent 是其祖先，故继承 en-US）。
    let sibling = doc.create_element("p");
    let _ = doc.append_child(parent, sibling);

    let sel_en = make_pseudo_selector(PseudoClassSelector::Lang("en".to_string()));
    // child 继承 parent 的 en-US → :lang(en) 匹配（连字符前缀边界）。
    assert!(
        matches_selector(&doc, child, &sel_en),
        ":lang(en) should match inherited lang=en-US"
    );
    // 大小写不敏感：:lang(EN) 也匹配。
    let sel_en_upper = make_pseudo_selector(PseudoClassSelector::Lang("EN".to_string()));
    assert!(
        matches_selector(&doc, child, &sel_en_upper),
        ":lang(EN) case-insensitive match"
    );
    // 精确匹配另一 range：:lang(en-US) 匹配。
    let sel_en_us = make_pseudo_selector(PseudoClassSelector::Lang("en-US".to_string()));
    assert!(matches_selector(&doc, child, &sel_en_us), ":lang(en-US) exact match");

    // 不应匹配：:lang(fr) 与 en-US 无关。
    let sel_fr = make_pseudo_selector(PseudoClassSelector::Lang("fr".to_string()));
    assert!(
        !matches_selector(&doc, child, &sel_fr),
        ":lang(fr) should not match en-US"
    );

    // 连字符边界：:lang(eng) 不匹配 en-US（必须 "eng-" 前缀，"en-US" 不以 "eng-" 开头）。
    let sel_eng = make_pseudo_selector(PseudoClassSelector::Lang("eng".to_string()));
    assert!(
        !matches_selector(&doc, child, &sel_eng),
        ":lang(eng) must not match en-US (hyphen boundary)"
    );

    // 无 lang 祖先的元素不匹配。
    let orphan = doc.create_element("em");
    let _ = doc.append_child(root, orphan);
    assert!(
        !matches_selector(&doc, orphan, &sel_en),
        ":lang(en) should not match element with no lang ancestor"
    );
}

// ── :has() 内部 NextSibling/SubsequentSibling 组合器 ──

#[test]
fn test_has_next_sibling() {
    let (doc, nodes) = make_nested_dom();
    let parent = nodes[1]; // div
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("div".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Has(vec![
                        Selector {
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
                                            type_selector: Some(TypeSelector::Tag("p".to_string())),
                                            subclass_selectors: vec![],
                                        },
                                        None,
                                    ),
                                ],
                            },
                        },
                    ]))],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, parent, &sel), "div:has(span + p) should match");
}

#[test]
fn test_has_subsequent_sibling() {
    let (doc, nodes) = make_nested_dom();
    let parent = nodes[1]; // div
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("div".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Has(vec![
                        Selector {
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
                        },
                    ]))],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, parent, &sel), "div:has(span ~ p) should match");
}

// ── matches_nth_pattern 负 a 值 ──

#[test]
fn test_nth_pattern_negative_a() {
    let pattern = NthPattern { a: -1, b: 3 };
    assert!(matches_nth_pattern(3, &pattern), "index=3 matches -n+3");
    assert!(matches_nth_pattern(1, &pattern), "index=1 matches -n+3");
    assert!(!matches_nth_pattern(4, &pattern), "index=4 does NOT match -n+3");
}

// ── ContainerContext 测试 ──

#[test]
fn test_container_context_default() {
    let ctx = ContainerContext::default();
    assert!(ctx.container_width.is_none());
    assert!(ctx.container_height.is_none());
}

#[test]
fn test_container_context_with_size() {
    let ctx = ContainerContext::with_size(800.0, 600.0);
    assert_eq!(ctx.container_width, Some(800.0));
    assert_eq!(ctx.container_height, Some(600.0));
}

// ── evaluate_container_condition 范围语法 ──

#[test]
fn test_container_range_syntax() {
    let rule = make_container_rule_advanced("width", Some("200px"), Some("500px"), None, "", vec![]);
    let ctx = ContainerContext::with_size(300.0, 600.0);
    assert!(
        evaluate_container_condition(&rule, Some(&ctx)),
        "300px is within [200px, 500px]"
    );

    let ctx_outside = ContainerContext::with_size(600.0, 600.0);
    assert!(
        !evaluate_container_condition(&rule, Some(&ctx_outside)),
        "600px is outside [200px, 500px]"
    );
}

#[test]
fn test_container_no_context() {
    let rule = make_container_rule("width", "400px", vec![]);
    assert!(!evaluate_container_condition(&rule, None), "no context → false");
}

// ── evaluate_container_condition 操作符语法 ──

#[test]
fn test_container_operator_gt() {
    let rule = make_container_rule_advanced("width", None, None, Some(">"), "300px", vec![]);
    let ctx = ContainerContext::with_size(500.0, 600.0);
    assert!(evaluate_container_condition(&rule, Some(&ctx)), "500px > 300px");
}

#[test]
fn test_container_operator_lt() {
    let rule = make_container_rule_advanced("height", None, None, Some("<"), "500px", vec![]);
    let ctx = ContainerContext::with_size(800.0, 400.0);
    assert!(evaluate_container_condition(&rule, Some(&ctx)), "400px < 500px");
}

#[test]
fn test_container_operator_gte() {
    let rule = make_container_rule_advanced("width", None, None, Some(">="), "400px", vec![]);
    let ctx = ContainerContext::with_size(400.0, 600.0);
    assert!(evaluate_container_condition(&rule, Some(&ctx)), "400px >= 400px");
}

#[test]
fn test_container_operator_lte() {
    let rule = make_container_rule_advanced("width", None, None, Some("<="), "800px", vec![]);
    let ctx = ContainerContext::with_size(600.0, 600.0);
    assert!(evaluate_container_condition(&rule, Some(&ctx)), "600px <= 800px");
}

// ── evaluate_container_condition 冒号语法 ──

#[test]
fn test_container_colon_min_width() {
    let rule = make_container_rule("min-width", "400px", vec![]);
    let ctx = ContainerContext::with_size(500.0, 600.0);
    assert!(
        evaluate_container_condition(&rule, Some(&ctx)),
        "500px >= min-width 400px"
    );
}

#[test]
fn test_container_colon_max_height() {
    let rule = make_container_rule("max-height", "600px", vec![]);
    let ctx = ContainerContext::with_size(800.0, 500.0);
    assert!(
        evaluate_container_condition(&rule, Some(&ctx)),
        "500px <= max-height 600px"
    );
}

#[test]
fn test_container_colon_exact_width() {
    let rule = make_container_rule("width", "800px", vec![]);
    let ctx = ContainerContext::with_size(800.0, 600.0);
    assert!(evaluate_container_condition(&rule, Some(&ctx)), "width exactly 800px");
}

// ── @supports 条件评估 ──

#[test]
fn test_supports_and() {
    let cond = SupportsCondition::And(vec![
        SupportsCondition::Property("display".to_string(), "flex".to_string()),
        SupportsCondition::Property("position".to_string(), "absolute".to_string()),
    ]);
    assert!(evaluate_supports_condition(&cond), "both supported → AND = true");
}

#[test]
fn test_supports_or() {
    let cond = SupportsCondition::Or(vec![
        SupportsCondition::Property("display".to_string(), "invalid-value".to_string()),
        SupportsCondition::Property("color".to_string(), "red".to_string()),
    ]);
    assert!(evaluate_supports_condition(&cond), "one supported → OR = true");
}

#[test]
fn test_supports_not() {
    let cond = SupportsCondition::Not(Box::new(SupportsCondition::Property(
        "display".to_string(),
        "invalid-value".to_string(),
    )));
    assert!(evaluate_supports_condition(&cond), "NOT unsupported = true");
}

#[test]
fn test_supports_selector_valid() {
    let cond = SupportsCondition::Selector("div".to_string());
    assert!(evaluate_supports_condition(&cond), "valid selector → true");
}

#[test]
fn test_supports_selector_consecutive_combinators() {
    let cond = SupportsCondition::Selector(">>>".to_string());
    assert!(!evaluate_supports_condition(&cond), ">>> is invalid");
}

#[test]
fn test_supports_selector_starts_with_combinator() {
    let cond = SupportsCondition::Selector("> div".to_string());
    assert!(
        !evaluate_supports_condition(&cond),
        "selector starting with > is invalid"
    );
}

// ── is_property_supported 更多属性 ──

#[test]
fn test_property_supported_scroll_snap() {
    assert!(is_property_supported("scroll-snap-type", "x mandatory"));
    assert!(is_property_supported("scroll-snap-align", "start"));
    assert!(is_property_supported("scroll-snap-stop", "always"));
}

#[test]
fn test_property_supported_container() {
    assert!(is_property_supported("container-type", "inline-size"));
    assert!(is_property_supported("container-name", "sidebar"));
}

#[test]
fn test_property_supported_transform() {
    assert!(is_property_supported("transform", "rotate(45deg)"));
}

#[test]
fn test_property_supported_unknown() {
    assert!(
        !is_property_supported("unknown-property", "value"),
        "unknown property not supported"
    );
}

// ── collect_from_rules @supports 规则集成 ──

#[test]
fn test_collect_with_supports_rule() {
    let (doc, nodes) = make_nested_dom();
    let child1 = nodes[2]; // span

    use zero_css_parser::ast::{Declaration, Rule, StyleRule};
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Supports(zero_css_parser::ast::SupportsRule {
            condition: SupportsCondition::Property("display".to_string(), "block".to_string()),
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("span")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            })],
        })],
    }];

    let decls = collect_matching_declarations(&doc, child1, &stylesheets);
    assert_eq!(decls.len(), 1, "should match span inside @supports(display:block)");
    assert_eq!(decls[0].0, "color");
}

// ── collect_from_rules @container 规则集成 ──

#[test]
fn test_collect_with_container_rule() {
    let (doc, nodes) = make_nested_dom();
    let child1 = nodes[2]; // span

    use zero_css_parser::ast::{Declaration, Rule, StyleRule};
    let container_rule = make_container_rule(
        "min-width",
        "400px",
        vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("span")],
            declarations: vec![Declaration {
                property: "color".to_string(),
                value: "blue".to_string(),
                important: false,
            }],
        })],
    );
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Container(container_rule)],
    }];

    let ctx = ContainerContext::with_size(500.0, 600.0);
    let decls = collect_matching_declarations_with_media(&doc, child1, &stylesheets, None, Some(&ctx));
    assert_eq!(
        decls.len(),
        1,
        "should match span inside @container(min-width:400px) when width=500"
    );
}

#[test]
fn test_collect_container_no_context() {
    let (doc, nodes) = make_nested_dom();
    let child1 = nodes[2]; // span

    use zero_css_parser::ast::{Declaration, Rule, StyleRule};
    let container_rule = make_container_rule(
        "width",
        "400px",
        vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("span")],
            declarations: vec![Declaration {
                property: "color".to_string(),
                value: "blue".to_string(),
                important: false,
            }],
        })],
    );
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Container(container_rule)],
    }];

    let decls = collect_matching_declarations_with_media(&doc, child1, &stylesheets, None, None);
    assert!(decls.is_empty(), "no container context → @container not applied");
}

// ── @media 规则在 collect_from_rules 中的集成 ──

#[test]
fn test_collect_with_media_rule_matching() {
    let (doc, nodes) = make_nested_dom();
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
                    value: "green".to_string(),
                    important: false,
                }],
            })]),
        })],
    }];

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
    assert_eq!(decls.len(), 1, "media query should match when viewport=800px");
}

#[test]
fn test_collect_with_media_rule_not_matching() {
    let (doc, nodes) = make_nested_dom();
    let child1 = nodes[2]; // span

    use zero_css_parser::ast::{Declaration, Rule, StyleRule};
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::At(zero_css_parser::ast::AtRule {
            name: "media".to_string(),
            prelude: "(min-width: 1200px)".to_string(),
            body: zero_css_parser::ast::AtRuleBody::Block(vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("span")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "green".to_string(),
                    important: false,
                }],
            })]),
        })],
    }];

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
    assert!(
        decls.is_empty(),
        "media query should NOT match when viewport=800px < 1200px"
    );
}

// ── 属性选择器特殊匹配 ──

#[test]
fn test_attribute_dash_match() {
    let (mut doc, nodes) = make_nested_dom();
    let child1 = nodes[2]; // span
    doc.set_attribute(child1, "lang", "en-US");

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "lang".to_string(),
                        matcher: AttributeMatcher::DashMatch("en".to_string()),
                    })],
                },
                None,
            )],
        },
    };
    assert!(
        matches_selector(&doc, child1, &sel),
        "en-US starts with en- → dash-match"
    );
}

#[test]
fn test_attribute_exact_match_case_insensitive_html() {
    // CSS-Selectors §6.3：HTML 文档中属性值选择器对 ASCII 大小写不敏感。
    // WPT attribute-value-selector-007 assert：`[lang="es"]` 应匹配 `lang="ES"`。
    let (mut doc, nodes) = make_nested_dom();
    let child1 = nodes[2]; // span
    doc.set_attribute(child1, "lang", "ES");

    let mk = |matcher: AttributeMatcher| Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "lang".to_string(),
                        matcher,
                    })],
                },
                None,
            )],
        },
    };
    assert!(
        matches_selector(&doc, child1, &mk(AttributeMatcher::Exact("es".to_string()))),
        "[lang=\"es\"] should match lang=\"ES\" (ASCII case-insensitive in HTML)"
    );
    assert!(
        matches_selector(&doc, child1, &mk(AttributeMatcher::DashMatch("es".to_string()))),
        "[lang|=\"es\"] should match lang=\"ES\" (case-insensitive dash-match)"
    );
    // 大小写不敏感但仍要求精确匹配（不含前缀语义）：es-MX 不匹配 [lang=\"es\"] Exact
    doc.set_attribute(child1, "lang", "es-MX");
    assert!(
        !matches_selector(&doc, child1, &mk(AttributeMatcher::Exact("es".to_string()))),
        "[lang=\"es\"] Exact must not match lang=\"es-MX\" (exact, no prefix)"
    );
}

#[test]
fn test_attribute_prefix_match() {
    let (mut doc, nodes) = make_nested_dom();
    let child1 = nodes[2];
    doc.set_attribute(child1, "data-x", "hello-world");

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "data-x".to_string(),
                        matcher: AttributeMatcher::Prefix("hello".to_string()),
                    })],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, child1, &sel), "hello-world starts with hello");
}

#[test]
fn test_attribute_suffix_match() {
    let (mut doc, nodes) = make_nested_dom();
    let child1 = nodes[2];
    doc.set_attribute(child1, "href", "page.html");

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "href".to_string(),
                        matcher: AttributeMatcher::Suffix(".html".to_string()),
                    })],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, child1, &sel), "page.html ends with .html");
}

#[test]
fn test_attribute_substring_match() {
    let (mut doc, nodes) = make_nested_dom();
    let child1 = nodes[2];
    doc.set_attribute(child1, "title", "hello world");

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "title".to_string(),
                        matcher: AttributeMatcher::Substring("lo wo".to_string()),
                    })],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, child1, &sel), "hello world contains 'lo wo'");
}

// ── matches_nth_child 没有父节点 ──

#[test]
fn test_nth_child_no_parent() {
    let mut doc = Document::new();
    let orphan = doc.create_element("div");
    let sel = make_pseudo_selector(PseudoClassSelector::NthChild(NthPattern { a: 0, b: 1 }));
    assert!(
        !matches_selector(&doc, orphan, &sel),
        "orphan has no parent → nth-child fails"
    );
}
